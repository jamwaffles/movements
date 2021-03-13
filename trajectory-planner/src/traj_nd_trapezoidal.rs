use nalgebra::Vector3;

/// Second order polynomial, x(t) paper equation `(1)`.
pub fn second_order(
    t: f32,
    initial_pos: Vector3<f32>,
    initial_vel: Vector3<f32>,
    accel: Vector3<f32>,
) -> Vector3<f32> {
    initial_pos + initial_vel * t + 0.5 * accel * t.powi(2)
}

#[derive(Debug, Clone, Copy)]
pub struct Phase {
    pub duration: f32,
    pub distance: Vector3<f32>,
}

impl Phase {
    pub fn new(
        start_velocity: Vector3<f32>,
        end_velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
    ) -> Self {
        // FIXME: This shouldn't be `max()` but it'll do for now
        let time = (end_velocity - start_velocity)
            .component_div(&acceleration)
            .max();
        let distance = second_order(time, Vector3::zeros(), start_velocity, acceleration);

        Self {
            distance,
            duration: time,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub acceleration: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct TrapezoidalLineSegment {
    start_phase: Phase,
    cruise_phase: Phase,
    end_phase: Phase,
    limits: Limits,
    start: Point,
    end: Point,

    /// Maximum reachable velocity for this segment,.
    ///
    /// This may be clamped lower than `limits.velocity` if velocity limit cannot be reached in
    /// time.
    max_velocity: Vector3<f32>,

    duration: f32,
    start_accel: Vector3<f32>,
    end_accel: Vector3<f32>,
}

impl TrapezoidalLineSegment {
    pub fn new(limits: Limits, start: Point, end: Point) -> Self {
        let Limits {
            acceleration: max_acc,
            velocity: max_vel,
        } = limits;

        let Point {
            position: start_pos,
            velocity: start_vel,
        } = start;

        let Point {
            position: end_pos,
            velocity: end_vel,
        } = end;

        // Point at which velocity = 0 when decelerating from initial velocity (paper: Xstop)
        let full_stop_position = {
            // FIXME: Should probably not be max() here
            let time_to_stop = start_vel.component_div(&max_acc).max();

            log::trace!("time_to_stop {}", time_to_stop);

            let displacement = 0.5 * start_vel * time_to_stop;

            displacement
        };

        let start_accel_direction = (end_pos - start_pos).map(f32::signum);
        let end_accel_direction = Vector3::repeat(-1.0);

        let start_accel_direction = if start_vel > limits.velocity {
            start_accel_direction * -1.0
        } else {
            start_accel_direction
        };

        let start_accel = max_acc.component_mul(&start_accel_direction);
        let end_accel = max_acc.component_mul(&end_accel_direction);

        let start_phase = Phase::new(start_vel, max_vel, start_accel);
        let end_phase = Phase::new(max_vel, end_vel, end_accel);

        let cruise_time = (end_pos - (start_pos + start_phase.distance + end_phase.distance))
            .component_div(&max_vel)
            // FIXME: Shouldn't be using max() here, but it'll do for now
            .max();

        let cruise_phase = Phase {
            duration: cruise_time,
            // Cruise: No velocity change, acceleration of zero
            distance: second_order(cruise_time, max_vel, max_vel, Vector3::zeros()),
        };

        // Trajectory is too short for a cruise phase, denoted by a negative cruise duration. The
        // accel/decel ramps need to be shortened to create a wedge shaped profile.
        let (start_phase, cruise_phase, end_phase, max_velocity) = if cruise_phase.duration < 0.0 {
            let clamped_max_vel = (max_acc.component_mul(&(end_pos - start_pos))
                + 0.5 * start_vel.component_mul(&start_vel))
            // In lieu of a `.component_sqrt()` method...
            .map(|axis| axis.sqrt());

            // Recompute start/end phases with new clamped velocity
            let start_phase = Phase::new(start_vel, clamped_max_vel, start_accel);
            let end_phase = Phase::new(clamped_max_vel, end_vel, end_accel);

            // Wedge profile - no cruise
            let cruise_phase = Phase {
                duration: 0.0,
                distance: Vector3::zeros(),
            };

            (start_phase, cruise_phase, end_phase, clamped_max_vel)
        } else {
            (start_phase, cruise_phase, end_phase, limits.velocity)
        };

        Self {
            start_phase,
            cruise_phase,
            end_phase,
            start,
            end,
            limits,
            max_velocity,
            start_accel,
            end_accel,
            duration: start_phase.duration + cruise_phase.duration + end_phase.duration,
        }
    }

    pub fn position(&self, time: f32) -> Option<(Point, Vector3<f32>)> {
        // Acceleration phase
        if 0.0 <= time && time < self.start_phase.duration {
            let position = second_order(
                time,
                self.start.position,
                self.start.velocity,
                self.start_accel,
            );

            let velocity = self.start.velocity + self.start_accel * time;

            return Some((Point { position, velocity }, self.start_accel));
        }

        // Subtract start duration if we're in cruise/end phase
        let time = time - self.start_phase.duration;

        // Cruise phase
        if time < self.cruise_phase.duration {
            let initial_pos = self.start.position + self.start_phase.distance;

            let position = second_order(time, initial_pos, self.max_velocity, Vector3::zeros());

            return Some((
                Point {
                    position,
                    velocity: self.max_velocity,
                },
                Vector3::zeros(),
            ));
        }

        // Subtract cruise duration (we already subtracted start duration above)
        let time = time - self.cruise_phase.duration;

        // Decel phase
        if time <= self.end_phase.duration {
            // Position at end of cruise phase
            let initial_pos = second_order(
                self.cruise_phase.duration,
                self.start.position + self.start_phase.distance,
                self.max_velocity,
                Vector3::zeros(),
            );

            let position = second_order(time, initial_pos, self.max_velocity, self.end_accel);

            // Max velocity minus a given value as we're decelerating
            let velocity = self.max_velocity + self.end_accel * time;

            return Some((Point { position, velocity }, self.end_accel));
        }

        // Past end of segment
        None
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn set_velocity_limit(&mut self, velocity: Vector3<f32>) {
        *self = Self::new(
            Limits {
                velocity,
                ..self.limits
            },
            self.start,
            self.end,
        );
    }

    pub fn set_acceleration_limit(&mut self, acceleration: Vector3<f32>) {
        *self = Self::new(
            Limits {
                acceleration,
                ..self.limits
            },
            self.start,
            self.end,
        );
    }

    pub fn set_start_velocity(&mut self, velocity: Vector3<f32>) {
        *self = Self::new(
            self.limits,
            Point {
                velocity,
                ..self.start
            },
            self.end,
        );
    }

    pub fn set_end_velocity(&mut self, velocity: Vector3<f32>) {
        *self = Self::new(
            self.limits,
            self.start,
            Point {
                velocity,
                ..self.end
            },
        );
    }
}

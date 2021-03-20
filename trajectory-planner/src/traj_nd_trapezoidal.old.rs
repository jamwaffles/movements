use nalgebra::Vector3;

/// Second order polynomial, x(t) paper equation `(1)`.
pub fn second_order(
    t: Vector3<f32>,
    initial_pos: Vector3<f32>,
    initial_vel: Vector3<f32>,
    accel: Vector3<f32>,
) -> Vector3<f32> {
    initial_pos + initial_vel.component_mul(&t) + 0.5 * accel.component_mul(&t.component_mul(&t))
}

/// Second order polynomial, single axis version, x(t) paper equation `(1)`.
fn second_order_single_axis(t: f32, initial_pos: f32, initial_vel: f32, accel: f32) -> f32 {
    initial_pos + initial_vel * t + 0.5 * accel * t.powi(2)
}

#[derive(Debug, Clone, Copy)]
pub struct Phase {
    // Duration of each axis in this phase.
    pub duration: Vector3<f32>,
    pub distance: Vector3<f32>,
    start_velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
}

impl Phase {
    pub fn new(
        start_velocity: Vector3<f32>,
        end_velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
    ) -> Self {
        let duration = (end_velocity - start_velocity).component_div(&acceleration);

        let distance = second_order(duration, Vector3::zeros(), start_velocity, acceleration);

        Self {
            distance,
            duration,
            start_velocity,
            acceleration,
        }
    }

    fn change_duration(&mut self, duration: Vector3<f32>) {
        let distance = second_order(
            duration,
            Vector3::zeros(),
            self.start_velocity,
            self.acceleration,
        );

        *self = Self {
            duration,
            distance,
            ..*self
        };
    }

    fn cruise(duration: Vector3<f32>, start_velocity: Vector3<f32>) -> Self {
        let acceleration = Vector3::zeros();
        let distance = second_order(duration, Vector3::zeros(), start_velocity, acceleration);

        Self {
            duration,
            start_velocity,
            distance,
            acceleration,
        }
    }

    // fn zero() -> Self {
    //     Self {
    //         duration: Vector3::zeros(),
    //         distance: Vector3::zeros(),
    //     }
    // }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

impl Point {
    fn zero() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
        }
    }
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

    max_duration: f32,
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

        // // Point at which velocity = 0 when decelerating from initial velocity (paper: Xstop)
        // let full_stop_position = {
        //     // FIXME: Should probably not be max() here
        //     let time_to_stop = start_vel.component_div(&max_acc).max();

        //     log::trace!("time_to_stop {}", time_to_stop);

        //     let displacement = 0.5 * start_vel * time_to_stop;

        //     displacement
        // };

        let mut start_accel_direction = (end_pos - start_pos).map(f32::signum);
        let end_accel_direction = Vector3::repeat(-1.0);

        // Swap acceleration direction if start velocity is higher than velocity limit.
        start_accel_direction
            .iter_mut()
            .enumerate()
            .for_each(|(idx, axis)| {
                if start_vel[idx] > limits.velocity[idx] {
                    *axis *= -1.0
                }
            });

        let start_accel = max_acc.component_mul(&start_accel_direction);
        let end_accel = max_acc.component_mul(&end_accel_direction);

        let start_phase = Phase::new(start_vel, max_vel, start_accel);
        let end_phase = Phase::new(max_vel, end_vel, end_accel);

        let cruise_time = (end_pos - (start_pos + start_phase.distance + end_phase.distance))
            .component_div(&max_vel);

        // Cruise phase with default max velocity
        let mut cruise_phase = Phase::cruise(cruise_time, max_vel);

        let mut max_vel = max_vel;
        // Whether we need to recompute start/end phases after clamped profile
        let mut should_clamp = false;

        cruise_phase
            .duration
            .iter_mut()
            .enumerate()
            .for_each(|(idx, axis_cruise_duration)| {
                if *axis_cruise_duration < 0.0 {
                    should_clamp = true;

                    let max_acc = max_acc[idx];
                    let end_pos = end_pos[idx];
                    let start_pos = start_pos[idx];
                    let start_vel = start_vel[idx];

                    let clamped_max_vel =
                        (max_acc * (end_pos - start_pos) + 0.5 * start_vel.powi(2)).sqrt();

                    max_vel[idx] = clamped_max_vel;

                    // Wedge profile - no cruise
                    *axis_cruise_duration = 0.0;

                    //
                } else {
                    //
                };
            });

        // Recompute start/end phases with new clamped velocity
        let (start_phase, end_phase) = if should_clamp {
            (
                Phase::new(start_vel, max_vel, start_accel),
                Phase::new(max_vel, end_vel, end_accel),
            )
        } else {
            (start_phase, end_phase)
        };

        let mut self_ = Self {
            start_phase,
            cruise_phase,
            end_phase,
            start,
            end,
            limits,
            max_velocity: max_vel,
            start_accel,
            end_accel,
            max_duration: (start_phase.duration + cruise_phase.duration + end_phase.duration).max(),
        };

        self_.sync_multi();

        self_
    }

    // Synchronise multiple axes.
    fn sync_multi(&mut self) {
        let Self {
            start_phase,
            cruise_phase,
            end_phase,
            max_duration,
            max_velocity,
            start,
            ..
        } = *self;

        // Paper variable `T`, longest duration among all axes.
        let max_duration = Vector3::repeat(max_duration);

        // Paper variable `t3`. Durations of each axis.
        let total_duration = start_phase.duration + cruise_phase.duration + end_phase.duration;

        // Paper variable `delta t2`. Cruise durations of each axis.
        let cruise_duration = cruise_phase.duration;

        // Paper variable `A`, unsure of purpose.
        let a = max_duration - (total_duration - cruise_duration);

        let delta = -(a / 2.0)
            + (a.component_mul(&a) / 4.0
                + (max_duration - total_duration)
                    .component_mul(&max_velocity.abs().component_div(&self.limits.acceleration)))
            .map(|axis| axis.sqrt());

        log::info!("Delta {:?}, max {:?}", delta, max_duration);

        // Adjust switching point times
        let t1 = self.start_phase.duration - delta;
        let t2 = max_duration - (self.end_phase.duration - delta);
        let t3 = max_duration;

        // Convert switching points back into phase durations
        let t2 = t2 - t1;
        let t3 = t3 - t2;

        log::debug!("T1 {:?} -> {:?}", self.start_phase.duration, t1);

        self.start_phase.change_duration(t1);
        self.cruise_phase.change_duration(t2);
        self.end_phase.change_duration(t3);

        self.max_velocity = self.cruise_phase.start_velocity;
    }

    /// Get position, velocity and acceleration of a single axis at `time`.
    fn pos(&self, time: f32, idx: usize) -> Option<(f32, f32, f32)> {
        // Acceleration phase
        if 0.0 <= time && time < self.start_phase.duration[idx] {
            let position = second_order_single_axis(
                time,
                self.start.position[idx],
                self.start.velocity[idx],
                self.start_accel[idx],
            );

            let velocity = self.start.velocity[idx] + self.start_accel[idx] * time;

            return Some((position, velocity, self.start_accel[idx]));
        }

        // Subtract start duration if we're in cruise/end phase
        let time = time - self.start_phase.duration[idx];

        // Cruise phase
        if time < self.cruise_phase.duration[idx] {
            let initial_pos = self.start.position[idx] + self.start_phase.distance[idx];

            let position = second_order_single_axis(time, initial_pos, self.max_velocity[idx], 0.0);

            return Some((position, self.max_velocity[idx], 0.0));
        }

        // Subtract cruise duration (we already subtracted start duration above)
        let time = time - self.cruise_phase.duration[idx];

        // Decel phase
        if time <= self.end_phase.duration[idx] {
            // Position at end of cruise phase
            let initial_pos = second_order_single_axis(
                self.cruise_phase.duration[idx],
                self.start.position[idx] + self.start_phase.distance[idx],
                self.max_velocity[idx],
                0.0,
            );

            let position = second_order_single_axis(
                time,
                initial_pos,
                self.max_velocity[idx],
                self.end_accel[idx],
            );

            // Max velocity minus a given value as we're decelerating
            let velocity = self.max_velocity[idx] + self.end_accel[idx] * time;

            return Some((position, velocity, self.end_accel[idx]));
        }

        // Past end of segment for this axis
        None
    }

    pub fn position(&self, time: f32) -> Option<(Point, Vector3<f32>)> {
        let mut point = Point::zero();
        let mut accel = Vector3::zeros();

        for i in 0..self.max_velocity.len() {
            let (pos, vel, acc) = self.pos(time, i).unwrap_or((0.0, 0.0, 0.0));

            point.position[i] = pos;
            point.velocity[i] = vel;
            accel[i] = acc;
        }

        Some((point, accel))
    }

    /// Get durations for all DOF
    pub fn duration(&self) -> Vector3<f32> {
        self.start_phase.duration + self.cruise_phase.duration + self.end_phase.duration
    }

    /// The time taken for the slowest DOF to complete its move.
    pub fn max_duration(&self) -> f32 {
        self.duration().max()
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

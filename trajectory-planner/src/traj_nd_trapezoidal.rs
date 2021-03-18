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
    limits: Limits,
    start: Point,
    end: Point,

    start_accel: Vector3<f32>,
    end_accel: Vector3<f32>,
    max_velocity: Vector3<f32>,

    /// Time at end of accel phase.
    t1: Vector3<f32>,

    /// Time at end of cruise phase
    t2: Vector3<f32>,

    /// Aka total duration, or time at end of decel phase.
    t3: Vector3<f32>,

    /// Accel phase duration
    delta_t1: Vector3<f32>,

    /// Cruise phase duration
    delta_t2: Vector3<f32>,

    /// Decel phase duration
    delta_t3: Vector3<f32>,

    /// Distance change from accel phase
    delta_x1: Vector3<f32>,

    /// Distance change from accel phase
    delta_x2: Vector3<f32>,
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

        // (3) Calculate phase durations
        let delta_t1 = (max_vel - start_vel).component_div(&max_acc);
        // (4) TODO: Non-zero final velocity
        let delta_t3 = max_vel.component_div(&max_acc);

        // (3)
        let delta_x1 = second_order(delta_t1, Vector3::zeros(), start_vel, max_acc);
        // (4)
        let delta_x3 = second_order(delta_t3, Vector3::zeros(), max_vel, max_acc);

        // (5)
        let delta_t2 = (end_pos - (start_pos + delta_x1 + delta_x3)).component_div(&max_vel);

        let delta_x2 = second_order(delta_t2, Vector3::zeros(), max_vel, Vector3::zeros());

        log::debug!("{} {} {}", delta_t1, delta_t2, delta_t3);

        // TODO: Clamp velocity

        let t1 = delta_t1;
        let t2 = delta_t1 + delta_t2;
        let t3 = delta_t1 + delta_t2 + delta_t3;

        let mut self_ = Self {
            start,
            end,
            limits,
            delta_t1,
            delta_t2,
            delta_t3,
            t1,
            t2,
            t3,

            delta_x1,
            delta_x2,

            // TODO: Flip/clamp these properly
            start_accel: max_acc,
            end_accel: -max_acc,
            max_velocity: max_vel,
        };

        self_.sync_multi();

        self_
    }

    // Synchronise multiple axes.
    fn sync_multi(&mut self) {
        let Self { .. } = *self;
    }

    /// Get position, velocity and acceleration of a single axis at `time`.
    fn pos(&self, time: f32, idx: usize) -> Option<(f32, f32, f32)> {
        // Acceleration phase
        if 0.0 <= time && time < self.delta_t1[idx] {
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
        let time = time - self.delta_t1[idx];

        // Cruise phase
        if time < self.delta_t2[idx] {
            let initial_pos = self.start.position[idx] + self.delta_x1[idx];

            let position = second_order_single_axis(time, initial_pos, self.max_velocity[idx], 0.0);

            return Some((position, self.max_velocity[idx], 0.0));
        }

        // Subtract cruise duration (we already subtracted start duration above)
        let time = time - self.delta_t2[idx];

        // Decel phase
        if time <= self.delta_t3[idx] {
            // Position at end of cruise phase
            let initial_pos = self.delta_x1[idx] + self.delta_x2[idx];

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
        self.delta_t1 + self.delta_t2 + self.delta_t3
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

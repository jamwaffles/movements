use std::ops;

#[derive(Debug, Copy, Clone, Default)]
pub struct Vertex {
    pub position: f32,
    pub velocity: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Limits {
    pub acceleration: f32,
    pub velocity: f32,
}

/// Second order polynomial.
///
/// - t:
fn p_2(t: f32, initial_position: f32, initial_velocity: f32, acceleration: f32) -> f32 {
    initial_position + (initial_velocity * t) + (0.5 * acceleration * t.powi(2))
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub delta_t1: f32,
    pub delta_t2: f32,
    pub delta_t3: f32,

    pub range_t1: ops::Range<f32>,
    pub range_t2: ops::Range<f32>,
    pub range_t3: ops::RangeInclusive<f32>,

    pub delta_x1: f32,
    pub delta_x2: f32,
    pub delta_x3: f32,

    pub start: Vertex,
    pub end: Vertex,

    pub acceleration: f32,
    pub deceleration: f32,
    pub cruise_velocity: f32,

    pub x_stop: f32,
}

impl Segment {
    pub fn new(start: Vertex, end: Vertex, limits: &Limits) -> Self {
        // If positions are equal, don't do anything
        if (end.position - start.position).abs() <= f32::EPSILON {
            return Self::zero();
        }

        // Position reached when decelerating from start velocity to full stop
        let x_stop = {
            let final_velocity = 0.0f32;

            (final_velocity.powi(2) - start.velocity.powi(2)) / 2.0 * limits.acceleration
        };

        // Sign of cruising (general direction)
        let mut sign = (end.position - start.position).signum();

        // Acceleration sign
        // let accel_t1 = limits.acceleration * sign;
        let accel_t1 = limits.acceleration
            * if start.velocity <= limits.velocity {
                // Below velocity limit - accelerate
                sign
            } else {
                // Too fast for current limit - decelerate. This will result in a
                // double-deceleration profile.
                -sign
            };

        // Deceleration
        let accel_t3 = -limits.acceleration * sign;

        // Maximum cruise velocity
        let mut cruise_velocity = limits.velocity * sign;

        // First phase accel/decel time
        let mut delta_t1 = f32::abs((cruise_velocity - start.velocity) / accel_t1);

        // First phase displacement
        let mut delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

        // Third phase decel time
        let mut delta_t3 = f32::abs(cruise_velocity / accel_t3);

        // Third phase displacement
        let mut delta_x3 = p_2(delta_t3, 0.0, cruise_velocity, accel_t3);

        let mut delta_t2 =
            (end.position - (start.position + delta_x1 + delta_x3)) / cruise_velocity;

        // Not enough space/time to create a trapezoidal profile. We'll reduce the maximum velocity
        // and recalculate everything to form a "wedge" shaped profile.
        if delta_t2 < 0.0 {
            // New limit for cruise velocity
            cruise_velocity = f32::sqrt(
                accel_t1 * (end.position - start.position) + (0.5 * start.velocity.powi(2)),
            ) * sign;

            delta_t2 = 0.0;

            // First phase accel/decel time
            delta_t1 = f32::abs((cruise_velocity - start.velocity) / accel_t1);

            // First phase displacement
            delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

            // Third phase decel time
            delta_t3 = f32::abs(cruise_velocity / accel_t3);

            // Third phase displacement
            delta_x3 = p_2(delta_t3, 0.0, cruise_velocity, accel_t3);
        }

        // Cruise displacement (will be 0 if a wedge shaped profile is formed)
        let delta_x2 = cruise_velocity * delta_t2;

        // Time at end of first phase
        let t1 = delta_t1;

        // Time at end of cruise phase
        let t2 = delta_t1 + delta_t2;

        // Total segment time
        let t3 = delta_t1 + delta_t2 + delta_t3;

        let range_t1 = 0.0..t1;
        let range_t2 = t1..t2;
        // NOTE: End-inclusive to return correct value for t = t3
        let range_t3 = t2..=t3;

        Self {
            delta_x1,
            delta_x2,
            delta_x3,

            delta_t1,
            delta_t2,
            delta_t3,

            range_t1,
            range_t2,
            range_t3,

            start,
            end,

            acceleration: accel_t1,
            deceleration: accel_t3,
            cruise_velocity,

            x_stop,
        }
    }

    fn zero() -> Self {
        Self {
            delta_x1: 0.0,
            delta_x2: 0.0,
            delta_x3: 0.0,

            delta_t1: 0.0,
            delta_t2: 0.0,
            delta_t3: 0.0,

            range_t1: 0.0..0.0,
            range_t2: 0.0..0.0,
            range_t3: 0.0..=0.0,

            start: Vertex::default(),
            end: Vertex::default(),

            acceleration: 0.0,
            deceleration: 0.0,
            cruise_velocity: 0.0,

            x_stop: 0.0,
        }
    }

    pub fn displacement(&self) -> f32 {
        self.delta_x1 + self.delta_x2 + self.delta_x3
    }

    pub fn duration(&self) -> f32 {
        *self.range_t3.end()
    }

    pub fn position(&self, t: f32) -> f32 {
        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if self.range_t1.contains(&t) {
            p_2(
                t,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            )
        }
        // Cruise phase t2
        else if self.range_t2.contains(&t) {
            // Position at end of t1
            let x1 = p_2(
                self.range_t1.end,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            );

            p_2(t - self.range_t1.end, x1, self.cruise_velocity, 0.0)
        }
        // Deceleration t3
        else if self.range_t3.contains(&t) {
            // Position at end of t1
            let x1 = p_2(
                self.range_t1.end,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            );

            // Position at end of cruise phase
            let x2 = x1 + (self.cruise_velocity * self.delta_t2);

            p_2(
                t - self.range_t2.end,
                x2,
                self.cruise_velocity,
                self.deceleration,
            )
        } else {
            unreachable!()
        }
    }

    pub fn velocity(&self, t: f32) -> f32 {
        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if self.range_t1.contains(&t) {
            self.start.velocity + self.acceleration * t
        }
        // Cruise phase t2
        else if self.range_t2.contains(&t) {
            self.cruise_velocity
        }
        // Deceleration t3
        else if self.range_t3.contains(&t) {
            self.cruise_velocity + self.deceleration * (t - self.range_t2.end)
        } else {
            unreachable!()
        }
    }

    pub fn acceleration(&self, t: f32) -> f32 {
        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if self.range_t1.contains(&t) {
            self.acceleration
        }
        // Cruise phase t2
        else if self.range_t2.contains(&t) {
            0.0
        }
        // Deceleration t3
        else if self.range_t3.contains(&t) {
            self.deceleration
        } else {
            unreachable!()
        }
    }
}

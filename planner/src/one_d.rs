use std::ops;

#[derive(Debug, Copy, Clone)]
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
    delta_t1: f32,
    delta_t2: f32,
    delta_t3: f32,

    t1: f32,
    t2: f32,
    t3: f32,

    range_t1: ops::Range<f32>,
    range_t2: ops::Range<f32>,
    range_t3: ops::RangeInclusive<f32>,

    delta_x1: f32,
    delta_x2: f32,
    delta_x3: f32,

    start: Vertex,
    end: Vertex,

    acceleration: f32,
    deceleration: f32,
    cruise_velocity: f32,
}

impl Segment {
    pub fn new(start: Vertex, end: Vertex, limits: &Limits) -> Self {
        // Position reached when decelerating from start velocity to full stop
        let x_stop = {
            let final_velocity = 0.0f32;

            (final_velocity.powi(2) - start.velocity.powi(2)) / 2.0 * limits.acceleration
        };

        // Sign of cruising (general direction)
        let sign = (end.position - x_stop).signum();

        // Acceleration sign
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
        let accel_t3 = -limits.acceleration;

        // Maximum cruise velocity
        let mut cruise_velocity = limits.velocity;

        // First phase accel/decel time
        let mut delta_t1 = (cruise_velocity - start.velocity) / accel_t1;

        // First phase displacement
        let mut delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

        // Third phase decel time
        let mut delta_t3 = cruise_velocity / accel_t3.abs();

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
            );

            delta_t2 = 0.0;

            // First phase accel/decel time
            delta_t1 = (cruise_velocity - start.velocity) / accel_t1;

            // First phase displacement
            delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

            // Third phase decel time
            delta_t3 = cruise_velocity / accel_t3.abs();

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

            t1,
            t2,
            t3,

            range_t1,
            range_t2,
            range_t3,

            start,
            end,

            acceleration: accel_t1,
            deceleration: accel_t3,
            cruise_velocity,
        }
    }

    pub fn duration(&self) -> f32 {
        self.t3
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
                self.t1,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            );

            p_2(t - self.t1, x1, self.cruise_velocity, 0.0)
        }
        // Deceleration t3
        else if self.range_t3.contains(&t) {
            // Position at end of t1
            let x1 = p_2(
                self.t1,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            );

            // Position at end of cruise phase
            let x2 = x1 + (self.cruise_velocity * self.delta_t2);

            p_2(t - self.t2, x2, self.cruise_velocity, self.deceleration)
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
            self.cruise_velocity + self.deceleration * (t - self.t2)
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
//! Like [`crate::one_d::Segment`], but with jerk limits.

use crate::one_d::{self, p_2, Vertex};
use std::ops::{self, Range, RangeInclusive};

#[derive(Debug, Copy, Clone)]
pub struct Limits {
    pub acceleration: f32,
    pub velocity: f32,
    pub jerk: f32,
}

impl Limits {
    fn into_one_d(self) -> one_d::Limits {
        one_d::Limits {
            acceleration: self.acceleration,
            velocity: self.velocity,
        }
    }

    fn invert_acceleration(self) -> Self {
        Self {
            acceleration: -self.acceleration,
            ..self
        }
    }
}

/// Third order polynomial.
pub fn p_3(
    t: f32,
    initial_position: f32,
    initial_velocity: f32,
    initial_acceleration: f32,
    jerk: f32,
) -> f32 {
    one_d::p_2(t, initial_position, initial_velocity, initial_acceleration)
        + ((1.0 / 6.0) * jerk) * t.powi(3)
}

fn zero_cruise(start_velocity: f32, end_velocity: f32, limits: &Limits) -> one_d::Segment {
    one_d::Segment::new(
        Vertex {
            position: start_velocity,
            velocity: 0.0,
        },
        Vertex {
            position: end_velocity,
            velocity: 0.0,
        },
        &one_d::Limits {
            velocity: limits.acceleration,
            acceleration: limits.jerk,
        },
    )
}

#[derive(Debug, Clone)]
pub struct Segment {
    accel_zero_cruise: one_d::Segment,
    decel_zero_cruise: one_d::Segment,
    duration: f32,
    delta_t4: f32,

    accel: Range<f32>,
    cruise: Range<f32>,
    decel: RangeInclusive<f32>,
    limits: Limits,

    start: Vertex,
    end: Vertex,

    x1: Vertex,
    x2: Vertex,
    x3: Vertex,
    x4: Vertex,
    x5: Vertex,
    x6: Vertex,
}

impl Segment {
    pub fn new(start: Vertex, end: Vertex, limits: &Limits) -> Self {
        let trapezoidal = one_d::Segment::new(
            start,
            end,
            &one_d::Limits {
                velocity: limits.velocity,
                acceleration: limits.acceleration,
            },
        );

        // Sign of cruising (general direction)
        let sign = (end.position - trapezoidal.x_stop).signum();

        let accel_zero_cruise = zero_cruise(start.velocity, limits.velocity, &limits);
        let decel_zero_cruise = zero_cruise(limits.velocity, 0.0, &limits);

        dbg!(&accel_zero_cruise, &decel_zero_cruise);

        // NOTE: displacement() here is change in velocity
        // Displacement during acceleration phase
        let delta_acc = 0.5 * accel_zero_cruise.displacement() * accel_zero_cruise.duration();
        let delta_dec = 0.5 * decel_zero_cruise.displacement() * decel_zero_cruise.duration();

        // Cruise phase between accel/decel phases. If result is negative, there is no cruise
        // and we must decrease accel/decel.
        let delta_t4 = (end.position - (start.position + delta_acc + delta_dec)) / limits.velocity;

        // dbg!(delta_t4);

        // TODO: Negative cruise duration
        let delta_t4 = delta_t4.max(0.0);

        let duration = accel_zero_cruise.duration() + delta_t4 + decel_zero_cruise.duration();

        let accel = 0.0..accel_zero_cruise.duration();
        let cruise = accel.end..(accel.end + delta_t4);
        let decel = cruise.end..=(cruise.end + decel_zero_cruise.duration());

        // dbg!(&accel, &cruise, &decel);

        let x1 = Vertex {
            velocity: accel_zero_cruise.position(accel_zero_cruise.range_t1.end),
            position: p_3(
                accel_zero_cruise.range_t1.end,
                start.position,
                start.velocity,
                0.0,
                limits.jerk,
            ),
        };

        let x2 = Vertex {
            velocity: accel_zero_cruise.position(accel_zero_cruise.range_t2.end),
            position: p_3(
                accel_zero_cruise.range_t2.end - accel_zero_cruise.range_t1.end,
                x1.position,
                x1.velocity,
                accel_zero_cruise.cruise_velocity,
                0.0,
            ),
        };

        let x3 = Vertex {
            velocity: accel_zero_cruise.position(*accel_zero_cruise.range_t3.end()),
            position: p_3(
                accel_zero_cruise.range_t3.end() - accel_zero_cruise.range_t2.end,
                x2.position,
                x2.velocity,
                accel_zero_cruise.cruise_velocity,
                -limits.jerk,
            ),
        };

        let x4 = Vertex {
            velocity: accel_zero_cruise.position(*accel_zero_cruise.range_t3.end()),
            position: p_3(cruise.end - accel.end, x3.position, x3.velocity, 0.0, 0.0),
        };

        let x5 = Vertex {
            velocity: decel_zero_cruise.position(decel_zero_cruise.range_t1.end),
            position: p_3(
                decel_zero_cruise.range_t1.end,
                x4.position,
                x4.velocity,
                0.0,
                -limits.jerk,
            ),
        };

        let x6 = Vertex {
            velocity: decel_zero_cruise.position(decel_zero_cruise.range_t2.end),
            position: p_3(
                decel_zero_cruise.range_t2.end - decel_zero_cruise.range_t1.end,
                x5.position,
                x5.velocity,
                decel_zero_cruise.cruise_velocity,
                0.0,
            ),
        };

        Self {
            accel_zero_cruise,
            decel_zero_cruise,
            duration,
            delta_t4,
            accel,
            cruise,
            decel,
            limits: limits.clone(),
            start,
            end,
            x1,
            x2,
            x3,
            x4,
            x5,
            x6,
        }
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn position(&self, t: f32) -> f32 {
        if self.accel.contains(&t) {
            self.position_accel(t)
        } else if self.cruise.contains(&t) {
            p_3(
                t - self.accel.end,
                self.x3.position,
                self.x3.velocity,
                0.0,
                0.0,
            )
        } else if self.decel.contains(&t) {
            self.position_decel(t)
        } else {
            // unreachable!("{}", t)
            0.0
        }
    }

    /// Compute position for the acceleration phase (t0 to t3).
    fn position_accel(&self, t: f32) -> f32 {
        let accel = &self.accel_zero_cruise;

        if accel.range_t1.contains(&t) {
            p_3(
                t,
                self.start.position,
                self.start.velocity,
                0.0,
                self.limits.jerk,
            )
        } else if accel.range_t2.contains(&t) {
            p_2(
                t - accel.range_t1.end,
                self.x1.position,
                self.x1.velocity,
                accel.cruise_velocity,
            )
        } else if accel.range_t3.contains(&t) {
            p_3(
                t - accel.range_t2.end,
                self.x2.position,
                self.x2.velocity,
                accel.cruise_velocity,
                -self.limits.jerk,
            )
        } else {
            // unreachable!()
            0.0
        }
    }

    /// Compute position for the deceleration phase (t4 to t7).
    fn position_decel(&self, t: f32) -> f32 {
        let decel = &self.decel_zero_cruise;

        // Make all times relative to start of decel phase
        let t = t - self.cruise.end;

        // Deceleration ramp up, max jerk
        if decel.range_t1.contains(&t) {
            p_3(
                t,
                self.x4.position,
                self.x4.velocity,
                0.0,
                -self.limits.jerk,
            )
        }
        // Constant deceleration
        else if decel.range_t2.contains(&t) {
            p_2(
                t - decel.range_t1.end,
                self.x5.position,
                self.x5.velocity,
                decel.cruise_velocity,
            )
        }
        // Deceleration ramp to 0, minimum jerk
        else if decel.range_t3.contains(&t) {
            p_3(
                t - decel.range_t2.end,
                self.x6.position,
                self.x6.velocity,
                decel.cruise_velocity,
                self.limits.jerk,
            )
        } else {
            // unreachable!()
            0.0
        }
    }

    pub fn velocity(&self, t: f32) -> f32 {
        if self.accel.contains(&t) {
            self.accel_zero_cruise.position(t)
        } else if self.cruise.contains(&t) {
            self.limits.velocity
        } else if self.decel.contains(&t) {
            self.decel_zero_cruise.position(t - self.decel.start())
        } else {
            // unreachable!("{}", t)
            0.0
        }
    }

    pub fn acceleration(&self, t: f32) -> f32 {
        if self.accel.contains(&t) {
            self.accel_zero_cruise.velocity(t)
        } else if self.cruise.contains(&t) {
            0.0
        } else if self.decel.contains(&t) {
            self.decel_zero_cruise.velocity(t - self.decel.start())
        } else {
            // unreachable!("{}", t)
            0.0
        }
    }

    // TODO: Correct signs
    pub fn jerk(&self, t: f32) -> f32 {
        if self.accel.contains(&t) {
            self.accel_zero_cruise.acceleration(t)
        } else if self.cruise.contains(&t) {
            0.0
        } else if self.decel.contains(&t) {
            self.decel_zero_cruise.acceleration(t - self.decel.start())
        } else {
            // unreachable!("{}", t)
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jerk() {
        let start = Vertex {
            position: 0.0,
            velocity: 0.0,
        };
        let end = Vertex {
            position: 10.0,
            velocity: 0.0,
        };
        let limits = Limits {
            velocity: 5.0,
            acceleration: 10.0,
            jerk: 10.0,
        };

        let segment = Segment::new(start, end, &limits);

        //
    }
}

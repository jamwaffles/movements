//! Like [`crate::one_d::Segment`], but with jerk limits.

use std::ops::{self, Range, RangeInclusive};

use crate::one_d::{self, Vertex};

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

/// Second order polynomial.
fn p_2(t: f32, initial_velocity: f32, initial_acceleration: f32, jerk: f32) -> f32 {
    initial_velocity + (initial_acceleration * t) + (0.5 * jerk * t.powi(2))
}

// /// Third order polynomial.
// fn p_3(t: f32, initial_position: f32, initial_velocity: f32, acceleration: f32, jerk: f32) -> f32 {
//     p_2(t, initial_position, initial_velocity, acceleration) + (1.0 / 6.0) * jerk * t.powi(3)
// }

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

        let accel_zero_cruise = zero_cruise(0.0, limits.velocity, &limits);
        let decel_zero_cruise = zero_cruise(limits.velocity, 0.0, &limits);

        dbg!(&accel_zero_cruise, &decel_zero_cruise);

        // Cruise phase between accel/decel phases. If result is negative, there is no cruise
        // and we must decrease accel/decel.
        let delta_t4 = (end.position
            - (start.position
                + accel_zero_cruise.displacement()
                + decel_zero_cruise.displacement()))
            / limits.velocity;

        dbg!(delta_t4);

        // TODO: Negative cruise duration
        let delta_t4 = delta_t4.max(0.0);

        let duration =
            accel_zero_cruise.duration() + delta_t4.max(0.0) + decel_zero_cruise.duration();

        let accel = 0.0..accel_zero_cruise.duration();
        let cruise = accel.end..(accel.end + delta_t4);
        let decel = cruise.end..=(cruise.end + decel_zero_cruise.duration());

        dbg!(&accel, &cruise, &decel);

        Self {
            accel_zero_cruise,
            decel_zero_cruise,
            duration,
            delta_t4,
            accel,
            cruise,
            decel,
            limits: limits.clone(),
        }
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn position(&self, t: f32) -> f32 {
        0.0
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

    pub fn jerk(&self, t: f32) -> f32 {
        0.0
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

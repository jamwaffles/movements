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

        dbg!(delta_t4);

        // TODO: Negative cruise duration
        let delta_t4 = delta_t4.max(0.0);

        let duration = accel_zero_cruise.duration() + delta_t4 + decel_zero_cruise.duration();

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
            start,
            end,
        }
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    fn position_accel(&self, t: f32) -> f32 {
        let accel = &self.accel_zero_cruise;

        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if accel.range_t1.contains(&t) {
            p_3(
                t,
                self.start.position,
                self.start.velocity,
                0.0,
                self.limits.jerk,
            )
        }
        // Cruise phase t2
        else if accel.range_t2.contains(&t) {
            // Position at end of acceleration ramp
            let x1 = p_3(
                accel.range_t1.end,
                self.start.position,
                self.start.velocity,
                0.0,
                self.limits.jerk,
            );

            // Velocity at end of acceleration ramp
            let v1 = accel.position(accel.range_t1.end);

            let t = t - accel.range_t1.end;

            p_2(t, x1, v1, accel.cruise_velocity)
        }
        // Deceleration t3
        else if accel.range_t3.contains(&t) {
            // Velocity at end of constant acceleration
            let v2 = accel.position(accel.range_t2.end);

            // Position at end of constant acceleration
            let x2 = {
                // Position at end of acceleration ramp
                let x1 = p_3(
                    accel.range_t1.end,
                    self.start.position,
                    self.start.velocity,
                    0.0,
                    self.limits.jerk,
                );

                // Velocity at end of acceleration ramp
                let v1 = accel.position(accel.range_t1.end);

                p_2(
                    accel.range_t2.end - accel.range_t1.end,
                    x1,
                    v1,
                    accel.cruise_velocity,
                )
            };

            let t = t - accel.range_t2.end;

            p_3(t, x2, v2, accel.cruise_velocity, -self.limits.jerk)
        } else {
            // unreachable!()
            0.0
        }
    }

    fn position_decel(&self, t: f32) -> f32 {
        let decel = &self.decel_zero_cruise;
        let accel = &self.accel_zero_cruise;

        let cruise_end_vel = accel.position(*accel.range_t3.end());
        let cruise_end_pos = {
            // Velocity at end of velocity ramp
            let v1 = accel.position(*accel.range_t3.end());

            // Position at end of velocity ramp
            let x1 = {
                // Velocity at end of constant acceleration
                let v2 = accel.position(accel.range_t2.end);

                // Position at end of constant acceleration
                let x2 = {
                    // Position at end of acceleration ramp
                    let x1 = p_3(
                        accel.range_t1.end,
                        self.start.position,
                        self.start.velocity,
                        0.0,
                        self.limits.jerk,
                    );

                    // Velocity at end of acceleration ramp
                    let v1 = accel.position(accel.range_t1.end);

                    p_2(
                        accel.range_t2.end - accel.range_t1.end,
                        x1,
                        v1,
                        accel.cruise_velocity,
                    )
                };

                let t = accel.range_t3.end() - accel.range_t2.end;

                p_3(t, x2, v2, accel.cruise_velocity, -self.limits.jerk)
            };

            p_3(self.cruise.end - self.accel.end, x1, v1, 0.0, 0.0)
        };

        // Make all times relative to start of decel phase
        let t = t - self.cruise.end;

        // Deceleration ramp up, max jerk
        if decel.range_t1.contains(&t) {
            p_3(t, cruise_end_pos, cruise_end_vel, 0.0, -self.limits.jerk)
        }
        // Constant deceleration
        else if decel.range_t2.contains(&t) {
            let prev_end_vel = decel.position(decel.range_t1.end);
            let prev_end_pos = {
                p_3(
                    decel.range_t1.end,
                    cruise_end_pos,
                    cruise_end_vel,
                    0.0,
                    -self.limits.jerk,
                )
            };

            p_2(
                t - decel.range_t1.end,
                prev_end_pos,
                prev_end_vel,
                -accel.cruise_velocity,
            )
        }
        // Deceleration ramp to 0, minimum jerk
        else if decel.range_t3.contains(&t) {
            let prev_end_vel = decel.position(decel.range_t2.end);
            let prev_end_pos = {
                let prev_end_vel = decel.position(decel.range_t1.end);
                let prev_end_pos = {
                    p_3(
                        decel.range_t1.end,
                        cruise_end_pos,
                        cruise_end_vel,
                        0.0,
                        -self.limits.jerk,
                    )
                };

                p_2(
                    decel.range_t2.end - decel.range_t1.end,
                    prev_end_pos,
                    prev_end_vel,
                    -accel.cruise_velocity,
                )
            };

            let t = t - decel.range_t2.end;

            p_3(
                t,
                prev_end_pos,
                prev_end_vel,
                -accel.cruise_velocity,
                self.limits.jerk,
            )
        } else {
            // unreachable!()
            0.0
        }
    }

    pub fn position(&self, t: f32) -> f32 {
        if self.accel.contains(&t) {
            self.position_accel(t)
        } else if self.cruise.contains(&t) {
            let accel = &self.accel_zero_cruise;

            // Velocity at end of velocity ramp
            let v1 = accel.position(*accel.range_t3.end());

            // Position at end of velocity ramp
            let x1 = {
                // Velocity at end of constant acceleration
                let v2 = accel.position(accel.range_t2.end);

                // Position at end of constant acceleration
                let x2 = {
                    // Position at end of acceleration ramp
                    let x1 = p_3(
                        accel.range_t1.end,
                        self.start.position,
                        self.start.velocity,
                        0.0,
                        self.limits.jerk,
                    );

                    // Velocity at end of acceleration ramp
                    let v1 = accel.position(accel.range_t1.end);

                    p_2(
                        accel.range_t2.end - accel.range_t1.end,
                        x1,
                        v1,
                        accel.cruise_velocity,
                    )
                };

                let t = accel.range_t3.end() - accel.range_t2.end;

                p_3(t, x2, v2, accel.cruise_velocity, -self.limits.jerk)
            };

            p_3(t - self.accel.end, x1, v1, 0.0, 0.0)
        } else if self.decel.contains(&t) {
            self.position_decel(t)
        } else {
            // unreachable!("{}", t)
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

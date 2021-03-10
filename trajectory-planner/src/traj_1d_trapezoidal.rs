use std::fmt::Display;

/// Second order polynomial, x(t) paper equation `(1)`.
fn second_order(t: f32, initial_pos: f32, initial_vel: f32, accel: f32) -> f32 {
    // TODO: Benchmark FMA
    initial_pos + initial_vel * t + 0.5 * accel * t.powi(2)
}

#[derive(Debug, Clone, Copy)]
pub struct Phase {
    pub duration: f32,
    pub distance: f32,
}

impl Phase {
    fn new(start_velocity: f32, end_velocity: f32, acceleration: f32) -> Self {
        let time = (end_velocity - start_velocity) / acceleration;
        let distance = second_order(time, 0.0, start_velocity, acceleration);

        Self {
            distance,
            duration: time,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub acceleration: f32,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub position: f32,
    pub velocity: f32,
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P: {:0.4}, V: {:0.4}", self.position, self.velocity)
    }
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
    max_velocity: f32,

    duration: f32,
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

        let direction = (end_pos - start_pos).signum();

        let start_phase = Phase::new(start_vel, max_vel, max_acc);
        let end_phase = Phase::new(max_vel, end_vel, -max_acc);

        let cruise_time =
            (end_pos - (start_pos + start_phase.distance + end_phase.distance)) / max_vel;

        let cruise_phase = Phase {
            duration: cruise_time,
            // Cruise: No velocity change, acceleration of zero
            distance: second_order(cruise_time, max_vel, max_vel, 0.0),
        };

        // Trajectory is too short for a cruise phase, denoted by a negative cruise duration. The
        // accel/decel ramps need to be shortened to create a wedge shaped profile.
        let (start_phase, cruise_phase, end_phase, max_velocity) = if cruise_phase.duration < 0.0 {
            let clamped_max_vel =
                (max_acc * (end_pos - start_pos) + 0.5 * start_vel.powi(2)).sqrt();

            // Recompute start/end phases with new clamped velocity
            let start_phase = Phase::new(start_vel, clamped_max_vel, max_acc);
            let end_phase = Phase::new(clamped_max_vel, end_vel, -max_acc);

            // Wedge profile - no cruise
            let cruise_phase = Phase {
                duration: 0.0,
                distance: 0.0,
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
            duration: start_phase.duration + cruise_phase.duration + end_phase.duration,
        }
    }

    pub fn position(&self, time: f32) -> Option<(Point, f32)> {
        // Acceleration phase
        if 0.0 <= time && time < self.start_phase.duration {
            let position = second_order(
                time,
                self.start.position,
                self.start.velocity,
                self.limits.acceleration,
            );

            let velocity = self.start.velocity + self.limits.acceleration * time;

            return Some((Point { position, velocity }, self.limits.acceleration));
        }

        // Subtract start duration if we're in cruise/end phase
        let time = time - self.start_phase.duration;

        // Cruise phase
        if time < self.cruise_phase.duration {
            let initial_pos = self.start.position + self.start_phase.distance;

            let position = second_order(time, initial_pos, self.max_velocity, 0.0);

            return Some((
                Point {
                    position,
                    velocity: self.max_velocity,
                },
                0.0,
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
                0.0,
            );

            let position = second_order(
                time,
                initial_pos,
                self.max_velocity,
                -self.limits.acceleration,
            );

            // Max velocity minus a given value as we're decelerating
            let velocity = self.max_velocity - self.limits.acceleration * time;

            return Some((Point { position, velocity }, -self.limits.acceleration));
        }

        // Past end of segment
        None
    }

    /// Get trapezoidal line segment's duration.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn set_velocity_limit(&mut self, limit: f32) {
        self.limits.velocity = limit;
    }

    pub fn set_acceleration_limit(&mut self, limit: f32) {
        self.limits.acceleration = limit;
    }

    pub fn set_start_velocity(&mut self, velocity: f32) {
        self.start.velocity = velocity;
    }

    pub fn set_end_velocity(&mut self, velocity: f32) {
        self.start.velocity = velocity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traj() {
        let limits = Limits {
            velocity: 2.0,
            acceleration: 10.0,
        };

        let start = Point {
            position: 0.0,
            velocity: 0.0,
        };
        let end = Point {
            position: 1.0,
            velocity: 0.0,
        };

        let traj = TrapezoidalLineSegment::new(limits, start, end);

        for ms in (0..=1000).step_by(10) {
            let t = ms as f32 / 1000.0;

            let pos = traj.position(t);

            println!(
                "{:04} -> {}",
                ms,
                pos.map(|(p, _accel)| p.to_string())
                    .unwrap_or_else(String::new)
            );
        }
    }
}

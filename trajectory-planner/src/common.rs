use std::fmt::Display;

/// Second order polynomial, x(t) paper equation `(1)`.
pub fn second_order(t: f32, initial_pos: f32, initial_vel: f32, accel: f32) -> f32 {
    // TODO: Benchmark FMA
    initial_pos + initial_vel * t + 0.5 * accel * t.powi(2)
}

#[derive(Debug, Clone, Copy)]
pub struct Phase {
    pub duration: f32,
    pub distance: f32,
}

impl Phase {
    pub fn new(start_velocity: f32, end_velocity: f32, acceleration: f32) -> Self {
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

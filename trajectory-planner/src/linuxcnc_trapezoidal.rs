/// Trajectory generation based loosely on old LinuxCNC trajectory planner information presented
/// [here](http://wiki.linuxcnc.org/cgi-bin/wiki.pl?Trapezoidal_Velocity_Profile_Trajectory_Planner).
use nalgebra::Vector3;

#[derive(Debug, Clone, Copy, Default)]
pub struct Segment {
    /// Start position
    pub start: Vector3<f32>,
    /// End position
    pub end: Vector3<f32>,
    /// Velocity for each joint in this segment (capped at global limits).
    ///
    /// TODO: Method to create a segment from points and a scalar feedrate.
    pub velocity: Vector3<f32>,
}

impl Segment {
    fn new(start: Vector3<f32>, end: Vector3<f32>, feed: f32) -> Self {
        let delta = end - start;

        let delta = delta.normalize();

        let velocity = feed * delta;

        Self {
            start,
            end,
            velocity,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub acceleration: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Linear(Segment),
    Blend,
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    limits: Limits,
    queue: Vec<Move>,
}

type Vec3 = Vector3<f32>;

impl Trajectory {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            queue: Vec::new(),
        }
    }

    pub fn add_linear_segment(&mut self) {}

    pub fn position(&self, time: f32) -> u8 {
        todo!()
    }

    /// Get durations for all DOF
    pub fn duration(&self) -> Vector3<f32> {
        todo!()
    }

    /// The time taken for the slowest DOF to complete its move.
    pub fn max_duration(&self) -> f32 {
        todo!()
    }

    pub fn set_velocity_limit(&mut self, velocity: Vector3<f32>) {
        todo!()
    }

    pub fn set_acceleration_limit(&mut self, acceleration: Vector3<f32>) {
        todo!()
    }

    pub fn set_start_velocity(&mut self, velocity: Vector3<f32>) {
        todo!()
    }

    pub fn set_end_velocity(&mut self, velocity: Vector3<f32>) {
        todo!()
    }
}

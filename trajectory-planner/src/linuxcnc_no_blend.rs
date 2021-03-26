/// Trajectory generation based loosely on old LinuxCNC trajectory planner information presented
/// [here](http://wiki.linuxcnc.org/cgi-bin/wiki.pl?Trapezoidal_Velocity_Profile_Trajectory_Planner).

#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub position: f32,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub acceleration: f32,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Blend {
    duration: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Linear(LinearSegment),
    Blend(Blend),
}

#[derive(Debug, Clone, Copy)]
pub struct LinearSegment {
    start: f32,
    end: f32,
    velocity: f32,
    duration: f32,
    start_time: f32,
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    limits: Limits,
    queue: Vec<LinearSegment>,
    points: Vec<Point>,
}

impl Trajectory {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            queue: Vec::new(),
            points: Vec::new(),
        }
    }

    /// Feed rate to transition to when this point is reached.
    pub fn add_stuff(&mut self, point: f32, feed: f32) {
        self.points.push(Point {
            position: point,
            velocity: feed,
        });

        self.recompute()
    }

    fn recompute(&mut self) {
        let mut time_offset = 0.0;

        let segments = self
            .points
            .windows(2)
            .map(|points| {
                if let [p1, p2] = points {
                    let x_delta = p2.position - p1.position;

                    let duration = x_delta.abs() / p1.velocity;

                    let start_time = time_offset;

                    time_offset += duration;

                    log::debug!(
                        "delta {:?}, duration {:?}, start_time {:?}",
                        x_delta,
                        duration,
                        start_time
                    );

                    LinearSegment {
                        start: p1.position,
                        end: p2.position,
                        duration,
                        velocity: p1.velocity * x_delta.signum(),
                        start_time,
                    }
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();

        self.queue = segments;
    }

    /// Position, velocity and acceleration for a given time
    pub fn position(&self, time: f32) -> Option<(f32, f32, f32)> {
        self.queue
            .iter()
            .find(|segment| segment.start_time + segment.duration > time)
            .map(|segment| {
                let delta_t = time - segment.start_time;

                let pos = segment.start + (segment.velocity * delta_t);

                let acc = 0.0;

                let vel = segment.velocity;

                (pos, vel, acc)
            })
    }

    pub fn duration(&self) -> f32 {
        self.queue
            .last()
            .map(|segment| segment.start_time + segment.duration)
            .unwrap_or(0.0)
    }

    pub fn set_acceleration_limit(&mut self, acceleration: f32) {
        self.limits.acceleration = acceleration;
    }

    pub fn set_velocity_limit(&mut self, velocity: f32) {
        self.limits.velocity = velocity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check() {
        let mut traj = Trajectory::new(Limits {
            velocity: 1.0,
            acceleration: 1.0,
        });

        traj.add_stuff(1.0, 1.0);
        traj.add_stuff(2.0, 0.5);
        traj.add_stuff(3.0, 2.0);
        traj.add_stuff(1.0, 2.0);
        traj.add_stuff(5.0, 0.5);

        dbg!(traj);
    }
}

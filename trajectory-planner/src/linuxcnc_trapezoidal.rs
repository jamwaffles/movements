use std::io::LineWriter;

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
    /// Start position
    pub start: f32,
    pub duration: f32,
    pub start_time: f32,
    pub acceleration: f32,
    pub start_velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Linear(LinearSegment),
    Blend(Blend),
}

impl Move {
    pub fn start_time(&self) -> f32 {
        match self {
            Move::Linear(segment) => segment.start_time,
            Move::Blend(segment) => segment.start_time,
        }
    }

    fn end_time(&self) -> f32 {
        self.start_time() + self.duration()
    }

    fn duration(&self) -> f32 {
        match self {
            Move::Linear(segment) => segment.duration,
            Move::Blend(segment) => segment.duration,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinearSegment {
    pub start: f32,
    pub end: f32,
    pub velocity: f32,
    pub duration: f32,
    pub start_time: f32,
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    limits: Limits,
    pub queue: Vec<Move>,
    pub points: Vec<Point>,
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

    fn compute_blend(
        &self,
        s1: LinearSegment,
        s2: LinearSegment,
    ) -> (LinearSegment, Blend, LinearSegment) {
        let blend_accel = (s2.velocity - s1.velocity).signum() * self.limits.acceleration;

        let mut blend_duration = (s2.velocity - s1.velocity).abs() / self.limits.acceleration;

        // log::debug!(
        //     "s1 {:0.3} blend {:0.3} s2 {:0.3}",
        //     s1.duration,
        //     blend_duration,
        //     s2.duration
        // );

        // if blend_duration > s1.duration || blend_duration > s2.duration {
        //     let reduction_factor = (s1.duration.min(s2.duration) / blend_duration).sqrt();

        //     log::debug!("Clamp {}", reduction_factor);

        //     // blend_duration *= reduction_factor;

        //     // s1.velocity *= reduction_factor;
        //     // s2.velocity *= reduction_factor;
        // }

        let offset = blend_duration / 2.0;

        let s1_duration = s1.duration - offset;
        let s1_end_time = s1.start_time + s1_duration;

        let blend = Blend {
            start: s1.start + (s1.velocity * s1_duration),
            start_time: s1_end_time,
            duration: blend_duration,
            acceleration: blend_accel,
            start_velocity: s1.velocity,
        };

        // Modify end of first segment
        let s1 = LinearSegment {
            duration: s1_duration,
            ..s1
        };

        // Modify start of next segment
        let s2 = LinearSegment {
            duration: s2.duration - offset,
            start_time: s2.start_time + offset,
            start: {
                // Position at end of blend
                blend.start + 0.5 * (s2.velocity + blend.start_velocity) * blend.duration
            },
            ..s2
        };

        (s1, blend, s2)
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

                    // log::debug!(
                    //     "delta {:?}, duration {:?}, start_time {:?}",
                    //     x_delta,
                    //     duration,
                    //     start_time,
                    // );

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

        let mut it = segments.into_iter();

        let mut s1: Option<LinearSegment> = None;

        let mut new_segments = Vec::new();

        new_segments.push(Move::Blend(Blend {
            start: 0.0,
            duration: 0.0,
            start_time: 0.0,
            acceleration: 0.0,
            start_velocity: 0.0,
        }));

        while let Some(mut s2) = it.next() {
            // If there's a previous segment
            if let Some(s1) = s1.as_mut() {
                let (mut new_s1, mut blend, mut new_s2) = self.compute_blend(*s1, s2);

                // // Clamp velocities
                // if new_s1.duration < 0.0 || new_s2.duration < 0.0 {
                //     // let duration_reduction = (s1.duration - (blend_duration / 2.0))
                //     //     .min(new_s2.duration - (blend_duration / 2.0));

                //     log::debug!("delta {} {}", new_s1.duration, new_s2.duration);

                //     let reduce = new_s1.duration;

                //     log::debug!(
                //         "new_s2 {} -> {}",
                //         new_s2.velocity,
                //         new_s2.velocity - reduce.abs() * new_s2.velocity.signum()
                //     );

                //     new_s2.velocity -= reduce.abs() * new_s2.velocity.signum();

                //     let (a, b, c) = self.compute_blend(new_s1, new_s2);

                //     new_s1 = a;
                //     blend = b;
                //     new_s2 = c;
                // }

                // Push first segment
                new_segments.push(Move::Linear(new_s1));

                // Push blend
                new_segments.push(Move::Blend(blend));

                // Set up first blend for next iteration
                *s1 = new_s2;
            }
            // Otherwise push a new one and continue
            else {
                // new_segments.push(s2);
                s1 = Some(s2);
            }
        }

        // new_segments.push(Move::Blend(Blend {
        //     start: 0.0,
        //     duration: 0.0,
        //     start_time: new_segments.last().unwrap().end_time(),
        //     acceleration: 0.0,
        //     start_velocity: 0.0,
        // }));

        self.queue = new_segments;

        self.clamp_blends();
    }

    fn clamp_blends(&mut self) {
        for i in 0..(self.queue.len().saturating_sub(2)) {
            // let a = ;
            // let b = self.queue.get_mut(i + 1);
            // let c = self.queue.get_mut(i + 2);

            match self.queue.get_mut(i..=(i + 2)) {
                // Two accel/decel phases with a cruise in between: a trapezoidal profile.
                Some([Move::Blend(b1), Move::Linear(s1), Move::Blend(b2)]) => {
                    // log::debug!(
                    //     "blend idx {}, s1 {:0.3} blend {:0.3} s2 {:0.3}",
                    //     i + 1,
                    //     b1.duration,
                    //     s1.duration,
                    //     b2.duration,
                    // );

                    // Don't have enough time to reach full velocity and do the blend
                    if s1.duration < 0.0 {
                        // log::debug!("i {} overload", i + 1);

                        let reduce = s1.duration.abs();

                        b1.duration -= reduce;

                        let b1_end = b1.start
                            + ((b1.start_velocity * b1.duration)
                                + (0.5 * b1.acceleration * b1.duration.powi(2)));

                        let new_velocity = b1.start_velocity + b1.duration * b1.acceleration;

                        s1.velocity = new_velocity;
                        s1.duration = 0.0;
                        s1.start_time = b1.start_time + b1.duration;
                        s1.start = b1_end;
                        b2.start = b1_end;
                        b2.start_velocity = new_velocity;
                        b2.start_time = s1.start_time;
                        b2.duration -= reduce;
                    }
                }
                _ => {
                    // Unknown combo, skip
                }
            }
        }
    }

    /// Position, velocity and acceleration for a given time
    pub fn position(&self, time: f32) -> Option<(f32, f32, f32)> {
        self.queue
            .iter()
            .find(|segment| segment.end_time() > time)
            .map(|segment| {
                let delta_t = time - segment.start_time();

                match segment {
                    Move::Linear(segment) => {
                        let pos = segment.start + (segment.velocity * delta_t);

                        let acc = 0.0;

                        let vel = segment.velocity;

                        (pos, vel, acc)
                    }
                    Move::Blend(blend) => {
                        let pos = blend.start
                            + blend.start_velocity * delta_t
                            + 0.5 * blend.acceleration * delta_t.powi(2);

                        let vel = blend.start_velocity + (blend.acceleration * delta_t);

                        let acc = blend.acceleration;

                        (pos, vel, acc)
                    }
                }
            })
    }

    pub fn duration(&self) -> f32 {
        self.queue
            .last()
            .map(|segment| segment.end_time())
            .unwrap_or(0.0)
    }

    pub fn set_acceleration_limit(&mut self, acceleration: f32) {
        self.limits.acceleration = acceleration;

        self.recompute();
    }

    pub fn set_velocity_limit(&mut self, velocity: f32) {
        self.limits.velocity = velocity;

        self.recompute();
    }
}

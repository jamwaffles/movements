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
    start: f32,
    duration: f32,
    start_time: f32,
    acceleration: f32,
    start_velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Linear(LinearSegment),
    Blend(Blend),
}

impl Move {
    fn start_time(&self) -> f32 {
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
    start: f32,
    end: f32,
    velocity: f32,
    duration: f32,
    start_time: f32,
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    limits: Limits,
    queue: Vec<Move>,
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

        // new_segments.push(Move::Blend(Blend {
        //     start: 0.0,
        //     duration: 0.0,
        //     start_time: 0.0,
        //     acceleration: 0.0,
        //     start_velocity: 0.0,
        // }));

        while let Some(mut s2) = it.next() {
            // If there's a previous segment
            if let Some(s1) = s1.as_mut() {
                let blend_accel = (s2.velocity - s1.velocity).signum() * self.limits.acceleration;

                let mut blend_duration =
                    (s2.velocity - s1.velocity).abs() / self.limits.acceleration;

                log::debug!(
                    "s1 {:0.3} blend {:0.3} s2 {:0.3}",
                    s1.duration,
                    blend_duration,
                    s2.duration
                );

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
                *s1 = LinearSegment {
                    duration: s1_duration,
                    ..*s1
                };

                // Modify start of next segment
                s2 = LinearSegment {
                    duration: s2.duration - offset,
                    start_time: s2.start_time + offset,
                    start: {
                        // Position at end of blend
                        blend.start + 0.5 * (s2.velocity + blend.start_velocity) * blend.duration
                    },
                    ..s2
                };

                // Push first segment
                new_segments.push(Move::Linear(*s1));

                // Push blend
                new_segments.push(Move::Blend(blend));

                // Set up first blend for next iteration
                *s1 = s2;
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

        // self.clamp_blends();
    }

    // fn clamp_blends(&mut self) {
    //     for i in 0..(self.queue.len().saturating_sub(2)) {
    //         // let a = ;
    //         // let b = self.queue.get_mut(i + 1);
    //         // let c = self.queue.get_mut(i + 2);

    //         match self.queue.get_mut(i..=(i + 2)) {
    //             // A blend between two linear segments. We might need to clamp them.
    //             Some([Move::Linear(s1), Move::Blend(blend), Move::Linear(s2)]) => {
    //                 log::debug!(
    //                     "blend idx {}, s1 {:0.3} blend {:0.3} s2 {:0.3}",
    //                     i + 1,
    //                     s1.duration,
    //                     blend.duration,
    //                     s2.duration
    //                 );

    //                 if blend.duration > s1.duration / 2.0 || blend.duration > s2.duration / 2.0 {
    //                     log::debug!("i {} overload", i);
    //                 }
    //             }
    //             _ => {
    //                 // Unknown combo, skip
    //             }
    //         }
    //     }
    // }

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

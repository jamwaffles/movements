use nalgebra::Vector3;

/// Second order polynomial, x(t) paper equation `(1)`.
pub fn second_order(
    t: Vector3<f32>,
    initial_pos: Vector3<f32>,
    initial_vel: Vector3<f32>,
    accel: Vector3<f32>,
) -> Vector3<f32> {
    initial_pos + initial_vel.component_mul(&t) + 0.5 * accel.component_mul(&t.component_mul(&t))
}

/// Second order polynomial, single axis version, x(t) paper equation `(1)`.
fn second_order_single_axis(t: f32, initial_pos: f32, initial_vel: f32, accel: f32) -> f32 {
    initial_pos + initial_vel * t + 0.5 * accel * t.powi(2)
}

#[derive(Debug, Clone, Copy)]
pub struct Phase {
    // Duration of each axis in this phase.
    pub duration: Vector3<f32>,
    pub distance: Vector3<f32>,
    start_velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
}

impl Phase {
    pub fn new(
        start_velocity: Vector3<f32>,
        end_velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
    ) -> Self {
        let duration = (end_velocity - start_velocity).component_div(&acceleration);

        let distance = second_order(duration, Vector3::zeros(), start_velocity, acceleration);

        Self {
            distance,
            duration,
            start_velocity,
            acceleration,
        }
    }

    fn change_duration(&mut self, duration: Vector3<f32>) {
        let distance = second_order(
            duration,
            Vector3::zeros(),
            self.start_velocity,
            self.acceleration,
        );

        *self = Self {
            duration,
            distance,
            ..*self
        };
    }

    fn cruise(duration: Vector3<f32>, start_velocity: Vector3<f32>) -> Self {
        let acceleration = Vector3::zeros();
        let distance = second_order(duration, Vector3::zeros(), start_velocity, acceleration);

        Self {
            duration,
            start_velocity,
            distance,
            acceleration,
        }
    }

    // fn zero() -> Self {
    //     Self {
    //         duration: Vector3::zeros(),
    //         distance: Vector3::zeros(),
    //     }
    // }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

impl Point {
    fn zero() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub acceleration: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct TrapezoidalLineSegment {
    limits: Limits,
    start: Point,
    end: Point,

    start_accel: Vector3<f32>,
    end_accel: Vector3<f32>,
    max_velocity: Vector3<f32>,

    /// Time at end of accel phase.
    t1: Vector3<f32>,

    /// Time at end of cruise phase
    t2: Vector3<f32>,

    /// Aka total duration, or time at end of decel phase.
    t3: Vector3<f32>,

    /// Accel phase duration
    delta_t1: Vector3<f32>,

    /// Cruise phase duration
    delta_t2: Vector3<f32>,

    /// Decel phase duration
    delta_t3: Vector3<f32>,

    /// Distance change from accel phase
    delta_x1: Vector3<f32>,

    /// Distance change from accel phase
    delta_x2: Vector3<f32>,
}

impl TrapezoidalLineSegment {
    pub fn new(limits: Limits, start: Point, end: Point) -> Self {
        let Limits {
            acceleration: accel_limit,
            velocity: vel_limit,
        } = limits;

        let Point {
            position: start_pos,
            velocity: start_vel,
        } = start;

        let Point {
            position: end_pos,
            velocity: end_vel,
        } = end;

        let start_accel = accel_limit;
        let end_accel = -accel_limit;

        let (start_accel, end_accel) = {
            let mut start_accel_direction = (end_pos - start_pos).map(f32::signum);
            let end_accel_direction = Vector3::repeat(-1.0);

            // Swap acceleration direction if start velocity is higher than velocity limit.
            start_accel_direction
                .iter_mut()
                .enumerate()
                .for_each(|(idx, axis)| {
                    if start_vel[idx] > vel_limit[idx] {
                        *axis *= -1.0
                    }
                });

            let start_accel = accel_limit.component_mul(&start_accel_direction);
            let end_accel = accel_limit.component_mul(&end_accel_direction);

            (start_accel, end_accel)
        };

        // (3) Calculate phase durations
        let delta_t1 = (vel_limit - start_vel).component_div(&start_accel);

        // (4)
        let delta_t3 = (end_vel - vel_limit).component_div(&end_accel);

        // (3)
        let delta_x1 = second_order(delta_t1, Vector3::zeros(), start_vel, start_accel);
        // (4)
        let delta_x3 = second_order(delta_t3, Vector3::zeros(), vel_limit, end_accel);

        // (5)
        let delta_t2 = (end_pos - (start_pos + delta_x1 + delta_x3)).component_div(&vel_limit);

        let delta_x2 = second_order(delta_t2, Vector3::zeros(), vel_limit, Vector3::zeros());

        let t1 = delta_t1;
        let t2 = delta_t1 + delta_t2;
        let t3 = delta_t1 + delta_t2 + delta_t3;

        let mut self_ = Self {
            start,
            end,
            limits,
            delta_t1,
            delta_t2,
            delta_t3,
            t1,
            t2,
            t3,

            delta_x1,
            delta_x2,

            start_accel,
            end_accel,
            max_velocity: vel_limit,
        };

        self_.clamp();

        self_.sync_multi();

        self_
    }

    /// Reduce profiles of axes that are too short to contain a cruise phase.
    ///
    /// In this case, a wedge shaped profile must be created instead, with a lower maximum velocity.
    fn clamp(&mut self) {
        let Self {
            limits:
                Limits {
                    acceleration: accel_limit,
                    ..
                },
            start:
                Point {
                    position: start_pos,
                    velocity: start_vel,
                },
            end:
                Point {
                    position: end_pos,
                    velocity: end_vel,
                },
            max_velocity,
            start_accel,
            end_accel,
            delta_t2,
            ..
        } = self;

        // FIXME: Cheeki breeki clone
        for (idx, axis_cruise_duration) in delta_t2.clone().iter().enumerate() {
            // A negative cruise duration signifies that there wasn't enough time to reach a
            // cruising velocity (trapezoidal profile), so we need to turn this axis' profile into a
            // wedge instead.
            if *axis_cruise_duration < 0.0 {
                // Convert everything into the current single axis
                let accel_limit = accel_limit[idx];
                let end_pos = end_pos[idx];
                let end_vel = end_vel[idx];
                let end_accel = end_accel[idx];
                let start_pos = start_pos[idx];
                let start_vel = start_vel[idx];
                let start_accel = start_accel[idx];

                // New peak velocity |v|
                let clamped_max_vel =
                    (accel_limit * (end_pos - start_pos) + 0.5 * start_vel.powi(2)).sqrt();

                // Modify max velocity allowed for this axis as peak of wedge velocity profile
                max_velocity[idx] = clamped_max_vel;

                // Set parameters for all variables, but this axis only (specified by `idx`)
                {
                    // (3) Calculate phase durations
                    let delta_t1 = (clamped_max_vel - start_vel) / start_accel;

                    // Cruise phase now has no duration
                    let delta_t2 = 0.0;

                    // (4)
                    let delta_t3 = (end_vel - clamped_max_vel) / end_accel;

                    let delta_x1 = second_order_single_axis(delta_t1, 0.0, start_vel, start_accel);

                    // Cruise phase has no duration, therefore no displacement
                    let delta_x2 = 0.0;

                    // Largely redundant, but maintains symmetry with new()
                    let t1 = delta_t1;
                    let t2 = delta_t1 + delta_t2;
                    let t3 = delta_t1 + delta_t2 + delta_t3;

                    // Modify any required values
                    self.delta_t1[idx] = delta_t1;
                    self.delta_t2[idx] = delta_t2;
                    self.delta_t3[idx] = delta_t3;
                    self.delta_x1[idx] = delta_x1;
                    self.delta_x2[idx] = delta_x2;
                    self.t1[idx] = t1;
                    self.t2[idx] = t2;
                    self.t3[idx] = t3;
                }
            }
        }
    }

    /// Synchronise multiple axes to end at the same time.
    fn sync_multi(&mut self) {
        let Self {
            t1,
            t2,
            t3,
            delta_t1,
            delta_t2,
            delta_t3,
            limits:
                Limits {
                    acceleration: accel_limit,
                    velocity: vel_limit,
                },
            start:
                Point {
                    velocity: start_vel,
                    position: start_pos,
                },
            end: Point {
                position: end_pos, ..
            },
            ..
        } = *self;

        // log::debug!("Max {:?}", t3);

        let max_duration = Vector3::repeat(t3.max());

        let a = max_duration - (t3 - delta_t2);

        let a_sq = a.component_mul(&a);

        let delta = -(a / 2.0)
            + ((a_sq / 4.0)
                + (max_duration - t3)
                    .component_mul(&self.max_velocity.component_div(&accel_limit)))
            .map(|axis| axis.sqrt());

        // log::debug!("Delta {:?}", delta);

        let mut new_t1 = t1;
        let mut new_t2 = t2;
        let mut new_t3 = t3;

        // log::debug!("Old start_accel {:?}", self.start_accel);

        delta_t1.clone().iter().enumerate().for_each(|(idx, axis)| {
            // log::debug!("BUMS {:?} : {:?} < {:?}", idx, axis, delta[idx]);
            // Decelerate to lower cruise phase to extend this axis' duration to fit t3
            if *axis * self.start_accel[idx].signum() < delta[idx] {
                // TODO: Deal with non-zero final velocities

                let t_stop = start_vel[idx].abs() / accel_limit[idx];

                // log::debug!("Decel");

                // log::debug!(
                //     "Decel {:?} {:?} -> {:?}, tstop {:?}",
                //     idx,
                //     axis,
                //     delta[idx],
                //     t_stop
                // );

                let x_stop = start_pos[idx] + 0.5 * start_vel[idx] * t_stop;

                let v = (end_pos[idx] - x_stop) / (max_duration[idx] - t_stop);

                // log::debug!("{} t_stop {:?}, x_stop {:?}", idx, t_stop, x_stop);

                new_t1[idx] = (v - start_vel[idx]).abs() / accel_limit[idx];
                new_t2[idx] = max_duration[idx] - (t_stop - new_t1[idx]);

                // This is now a deceleration phase
                self.start_accel[idx] = -accel_limit[idx];
            }
            // Compute new acceleration times to extend cruise phase
            else {
                // log::debug!("Accel {}", idx);

                new_t1[idx] = t1[idx] - delta[idx];
                new_t2[idx] = max_duration[idx] - (delta_t3[idx] - delta[idx]);
                // log::debug!("Accel {:?} {:?} -> {:?}", idx, axis, new_t1[idx]);
            }

            new_t3[idx] = max_duration[idx];
        });

        // log::debug!("New start_accel {:?}", self.start_accel);

        let old = self.max_velocity;

        // Compute new velocity limit based on new acceleration phase
        self.max_velocity = {
            // v = u + at
            start_vel + self.start_accel.component_mul(&new_t1)
        };

        // log::debug!("Max velocity {:?} -> {:?}", old, self.max_velocity);

        let new_delta_t2 = new_t2 - new_t1;
        let new_delta_t3 = new_t3 - new_t2;

        // log::debug!("t1 {:?} -> {:?}", self.t1, new_t1);
        // log::debug!("t2 {:?} -> {:?}", self.t2, new_t2);
        // log::debug!("delta_t2 {:?} -> {:?}", self.delta_t2, new_delta_t2);
        // log::debug!("delta_t3 {:?} -> {:?}", self.delta_t3, new_delta_t3);

        let new_delta_x1 = second_order(new_t1, Vector3::zeros(), start_vel, self.start_accel);
        let new_delta_x2 = second_order(
            new_delta_t2,
            Vector3::zeros(),
            self.max_velocity,
            Vector3::zeros(),
        );

        // log::debug!("delta_x1 {:?} -> {:?}", self.delta_x1, new_delta_x1);
        // log::debug!("delta_x2 {:?} -> {:?}", self.delta_x2, new_delta_x2);

        self.t1 = new_t1;
        self.t2 = new_t2;
        self.t3 = new_t3;
        self.delta_t1 = new_t1;
        self.delta_t2 = new_delta_t2;
        self.delta_t3 = new_delta_t3;
        self.delta_x1 = new_delta_x1;
        self.delta_x2 = new_delta_x2;

        // log::debug!("t1 {:?} -> new t1 {:?}", t1, new_t1);
        // log::debug!("t2 {:?} -> new t2 {:?}", t2, new_t2);

        // let mut t1 = t1 - delta;
        // let mut t2 = max_duration - (delta_t3 - delta);
        // let t3 = max_duration;

        // log::debug!("{:?} {:?}", t1, delta);

        // // Turn accel into decel if necessary
        // t1.iter_mut().enumerate().for_each(|(idx, axis)| {
        //     if *axis < delta[idx] {
        //         let t_stop = max_velocity[idx].abs() / accel_limit[idx];

        //         // TODO: Handle non-zero final velocity

        //         // Position at full stop
        //         let x_stop = second_order_single_axis(
        //             t_stop,
        //             start_pos[idx],
        //             start_vel[idx],
        //             -accel_limit[idx],
        //         );

        //         let x_goal = end_pos[idx];

        //         let reduced_velocity = (x_goal - x_stop) / (max_duration[idx] - t_stop);

        //         // We're decelerating now, so make start accel negative
        //         self.start_accel[idx] = -accel_limit[idx];

        //         *axis = (reduced_velocity - start_vel[idx]).abs() / accel_limit[idx];

        //         t2[idx] = max_duration[idx] - t_stop - *axis;
        //     } else {
        //         *axis -= delta[idx]
        //     }
        // });

        // let delta_t1 = t1;
        // let delta_t2 = t2 - t1;
        // let delta_t3 = t3 - t2 - t1;

        // let delta_x1 = second_order(delta_t1, Vector3::zeros(), start_vel, start_accel);
        // let delta_x2 = {
        //     // Velocity at end of accel phase
        //     let t1_vel = start_vel + self.start_accel.component_mul(&delta_t1);

        //     second_order(delta_t2, Vector3::zeros(), t1_vel, Vector3::zeros())
        // };

        // self.t1 = t1;
        // self.t2 = t2;
        // self.t3 = t3;
        // self.delta_t1 = delta_t1;
        // self.delta_t2 = delta_t2;
        // self.delta_t3 = delta_t3;
        // self.delta_x1 = delta_x1;
        // self.delta_x2 = delta_x2;
    }

    /// Get position, velocity and acceleration of a single axis at `time`.
    fn pos(&self, time: f32, idx: usize) -> Option<(f32, f32, f32)> {
        // Accel
        if 0.0 <= time && time < self.t1[idx] {
            let pos = second_order_single_axis(
                time,
                self.start.position[idx],
                self.start.velocity[idx],
                self.start_accel[idx],
            );

            let vel = self.start.velocity[idx] + self.start_accel[idx] * time;

            let acc = self.start_accel[idx];

            return Some((pos, vel, acc));
        }

        // Cruise
        if time < self.t2[idx] {
            let time = time - self.t1[idx];

            // Position at end of acceleration phase
            let initial_pos = self.start.position[idx] + self.delta_x1[idx];

            // Velocity at end of accel phase
            // FIXME: Use self.max_velocity when that's recalculated properly
            // let cruise_vel = self.start.velocity[idx] + self.start_accel[idx] * self.delta_t1[idx];
            let cruise_vel = self.max_velocity[idx];

            let pos = second_order_single_axis(time, initial_pos, cruise_vel, 0.0);

            let vel = cruise_vel;

            let acc = 0.0;

            return Some((pos, vel, acc));
        }

        // Decel
        if time <= self.t3[idx] {
            let time = time - self.t2[idx];

            // Velocity at end of accel phase. Cruise phase velocity remains at this value so we can
            // use it in the calculations.
            // FIXME: Use self.max_velocity when that's recalculated properly
            // let cruise_vel = self.start.velocity[idx] + self.start_accel[idx] * self.delta_t1[idx];
            let cruise_vel = self.max_velocity[idx];

            // End of cruise phase
            let initial_pos = self.start.position[idx] + self.delta_x1[idx] + self.delta_x2[idx];

            let pos = second_order_single_axis(time, initial_pos, cruise_vel, self.end_accel[idx]);

            let vel = cruise_vel + self.end_accel[idx] * time;

            let acc = self.end_accel[idx];

            return Some((pos, vel, acc));
        }

        None
    }

    pub fn position(&self, time: f32) -> Option<(Point, Vector3<f32>)> {
        let mut point = Point::zero();
        let mut accel = Vector3::zeros();

        for i in 0..self.max_velocity.len() {
            let (pos, vel, acc) = self.pos(time, i).unwrap_or((0.0, 0.0, 0.0));

            point.position[i] = pos;
            point.velocity[i] = vel;
            accel[i] = acc;
        }

        Some((point, accel))
    }

    /// Get durations for all DOF
    pub fn duration(&self) -> Vector3<f32> {
        // self.delta_t1 + self.delta_t2 + self.delta_t3
        self.t3
    }

    /// The time taken for the slowest DOF to complete its move.
    pub fn max_duration(&self) -> f32 {
        self.duration().max()
    }

    pub fn set_velocity_limit(&mut self, velocity: Vector3<f32>) {
        *self = Self::new(
            Limits {
                velocity,
                ..self.limits
            },
            self.start,
            self.end,
        );
    }

    pub fn set_acceleration_limit(&mut self, acceleration: Vector3<f32>) {
        *self = Self::new(
            Limits {
                acceleration,
                ..self.limits
            },
            self.start,
            self.end,
        );
    }

    pub fn set_start_velocity(&mut self, velocity: Vector3<f32>) {
        *self = Self::new(
            self.limits,
            Point {
                velocity,
                ..self.start
            },
            self.end,
        );
    }

    pub fn set_end_velocity(&mut self, velocity: Vector3<f32>) {
        *self = Self::new(
            self.limits,
            self.start,
            Point {
                velocity,
                ..self.end
            },
        );
    }
}

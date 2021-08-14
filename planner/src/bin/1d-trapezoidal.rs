// Favourite calculator: <https://www.calculatorsoup.com/calculators/physics/>

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: f32,
    pub velocity: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Limits {
    acceleration: f32,
    velocity: f32,
}

/// Second order polynomial.
///
/// - t:
fn p_2(t: f32, initial_position: f32, initial_velocity: f32, acceleration: f32) -> f32 {
    initial_position + (initial_velocity * t) + (0.5 * acceleration * t.powi(2))
}

#[derive(Debug, Copy, Clone)]
pub struct Segment {
    delta_t1: f32,
    delta_t2: f32,
    delta_t3: f32,

    time: f32,

    delta_x1: f32,
    delta_x2: f32,
    delta_x3: f32,

    start: Vertex,
    end: Vertex,

    acceleration: f32,
    deceleration: f32,
    cruise_velocity: f32,
}

impl Segment {
    pub fn new(start: Vertex, end: Vertex, limits: &Limits) -> Self {
        // Position reached when decelerating from start velocity to full stop
        let x_stop = {
            let final_velocity = 0.0f32;

            (final_velocity.powi(2) - start.velocity.powi(2)) / 2.0 * limits.acceleration
        };

        // Sign of cruising (general direction)
        let sign = (end.position - x_stop).signum();

        // Acceleration sign
        let accel_t1 = limits.acceleration
            * if start.velocity <= limits.velocity {
                // Below velocity limit - accelerate
                sign
            } else {
                // Too fast for current limit - decelerate. This will result in a double-deceleration
                // profile.
                -sign
            };

        // Deceleration
        let accel_t3 = -limits.acceleration;

        // Maximum cruise velocity
        let mut cruise_velocity = limits.velocity;

        // First phase accel/decel time
        let mut delta_t1 = (cruise_velocity - start.velocity) / accel_t1;

        // First phase displacement
        let mut delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

        // Third phase decel time
        let mut delta_t3 = cruise_velocity / accel_t3.abs();

        // Third phase displacement
        let mut delta_x3 = p_2(delta_t3, 0.0, cruise_velocity, accel_t3);

        let mut delta_t2 =
            (end.position - (start.position + delta_x1 + delta_x3)) / cruise_velocity;

        // Not enough space/time to create a trapezoidal profile. We'll reduce the maximum velocity and
        // recalculate everything to form a "wedge" shaped profile.
        if delta_t2 < 0.0 {
            // New limit for cruise velocity
            cruise_velocity = f32::sqrt(
                accel_t1 * (end.position - start.position) + (0.5 * start.velocity.powi(2)),
            );

            delta_t2 = 0.0;

            // First phase accel/decel time
            delta_t1 = (cruise_velocity - start.velocity) / accel_t1;

            // First phase displacement
            delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

            // Third phase decel time
            delta_t3 = cruise_velocity / accel_t3.abs();

            // Third phase displacement
            delta_x3 = p_2(delta_t3, 0.0, cruise_velocity, accel_t3);
        }

        // Cruise displacement (will be 0 if a wedge shaped profile is formed)
        let delta_x2 = cruise_velocity * delta_t2;

        // Total segment time
        let t3 = delta_t1 + delta_t2 + delta_t3;

        Self {
            delta_x1,
            delta_x2,
            delta_x3,

            delta_t1,
            delta_t2,
            delta_t3,

            time: t3,

            start,
            end,

            acceleration: accel_t1,
            deceleration: accel_t3,
            cruise_velocity,
        }
    }

    pub fn position(&self, t: f32) -> f32 {
        let t1 = 0.0..self.delta_t1;
        let t2 = self.delta_t1..self.delta_t1 + self.delta_t2;
        let t3 = self.delta_t1 + self.delta_t2..self.delta_t1 + self.delta_t2 + self.delta_t2;

        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if t1.contains(&t) {
            p_2(
                t,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            )
        }
        // Cruise phase t2
        else if t2.contains(&t) {
            // Position at end of t1
            let x1 = p_2(
                self.delta_t1,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            );

            p_2(t - self.delta_t1, x1, self.cruise_velocity, 0.0)
        }
        // Deceleration t3
        else if t3.contains(&t) {
            // Position at end of t1
            let x1 = p_2(
                self.delta_t1,
                self.start.position,
                self.start.velocity,
                self.acceleration,
            );

            // Position at end of cruise phase
            let x2 = x1 + (self.cruise_velocity * self.delta_t2);

            let t2 = self.delta_t1 + self.delta_t2;

            p_2(t - t2, x2, self.cruise_velocity, self.deceleration)
        }
        // The absolute last position of this segment
        else {
            self.end.position
        }
    }

    pub fn velocity(&self, t: f32) -> f32 {
        let t1 = 0.0..self.delta_t1;
        let t2 = self.delta_t1..self.delta_t1 + self.delta_t2;
        let t3 = self.delta_t1 + self.delta_t2..self.delta_t1 + self.delta_t2 + self.delta_t2;

        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if t1.contains(&t) {
            self.start.velocity + self.acceleration * t
        }
        // Cruise phase t2
        else if t2.contains(&t) {
            self.cruise_velocity
        }
        // Deceleration t3
        else if t3.contains(&t) {
            self.cruise_velocity + self.deceleration * (t - (self.delta_t1 + self.delta_t2))
        }
        // The absolute last position of this segment
        else {
            0.0
        }
    }

    pub fn acceleration(&self, t: f32) -> f32 {
        let t1 = 0.0..self.delta_t1;
        let t2 = self.delta_t1..self.delta_t1 + self.delta_t2;
        let t3 = self.delta_t1 + self.delta_t2..self.delta_t1 + self.delta_t2 + self.delta_t2;

        // Acceleration phase t1 (or deceleration if we were originally moving too fast)
        if t1.contains(&t) {
            self.acceleration
        }
        // Cruise phase t2
        else if t2.contains(&t) {
            0.0
        }
        // Deceleration t3
        else if t3.contains(&t) {
            self.deceleration
        }
        // The absolute last position of this segment
        else {
            self.deceleration
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let start = Vertex {
        position: 0.0,
        velocity: 0.0,
    };
    let end = Vertex {
        position: 3.0,
        velocity: 0.0,
    };

    let limits = Limits {
        acceleration: 5.0,
        velocity: 1.0,
    };

    let segment = Segment::new(start, end, &limits);

    dbg!(segment);

    let mut x = 0.0;

    while x <= segment.time {
        let pos = segment.position(x);
        let vel = segment.velocity(x);
        let acc = segment.acceleration(x);

        println!("{:+04.2} -> {:+04.2} {:+04.2} {:+04.2}", x, pos, vel, acc);

        x += 0.1;
    }
}

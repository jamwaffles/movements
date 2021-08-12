// Favourite calculator: <https://www.calculatorsoup.com/calculators/physics/>

pub struct Vertex {
    pub position: f32,
    pub velocity: f32,
}

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
    let accel_t3 = limits.acceleration - sign;

    // Maximum cruise velocity
    let mut cruise_velocity = limits.velocity * sign;

    // First phase accel/decel time
    let mut delta_t1 = (cruise_velocity - start.velocity) / accel_t1;

    // First phase displacement
    let mut delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

    // Third phase decel time
    let mut delta_t3 = cruise_velocity / accel_t3;

    // Third phase displacement
    let mut delta_x3 = p_2(delta_t3, 0.0, cruise_velocity, accel_t3);

    let mut delta_t2 = (end.position - (start.position + delta_x1 + delta_x3)) / cruise_velocity;

    // Not enough space/time to create a trapezoidal profile. We'll reduce the maximum velocity and
    // recalculate everything to form a "wedge" shaped profile.
    if delta_t2 < 0.0 {
        // New limit for cruise velocity
        cruise_velocity = f32::sqrt(
            sign * limits.acceleration * (end.position - start.position)
                + (0.5 * start.velocity.powi(2)),
        );

        delta_t2 = 0.0;

        // First phase accel/decel time
        delta_t1 = (cruise_velocity - start.velocity) / accel_t1;

        // First phase displacement
        delta_x1 = p_2(delta_t1, 0.0, start.velocity, accel_t1);

        // Third phase decel time
        delta_t3 = cruise_velocity / accel_t3;

        // Third phase displacement
        delta_x3 = p_2(delta_t3, 0.0, cruise_velocity, accel_t3);
    }

    // Cruise displacement (will be 0 if a wedge shaped profile is formed)
    let delta_x2 = cruise_velocity * delta_t2;

    // Total segment time
    let t3 = delta_t1 + delta_t2 + delta_t3;
}

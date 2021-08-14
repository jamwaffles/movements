// Favourite calculator: <https://www.calculatorsoup.com/calculators/physics/>

use planner::one_d::*;

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

    dbg!(core::mem::size_of::<Segment>());

    dbg!(&segment);

    let mut x = 0.0;

    while x <= segment.duration() {
        let pos = segment.position(x);
        let vel = segment.velocity(x);
        let acc = segment.acceleration(x);

        println!("{:+04.2} -> {:+04.2} {:+04.2} {:+04.2}", x, pos, vel, acc);

        x += 0.1;
    }
}

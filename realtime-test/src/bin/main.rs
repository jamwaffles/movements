use std::{
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use chrono::SecondsFormat;
use realtime_test::{spawn_unchecked, SchedPolicy};

fn main() {
    let run_name = hostname::get().unwrap().into_string().unwrap();

    let policy = std::env::args()
        .nth(1)
        .expect("Need policy")
        .parse::<SchedPolicy>()
        .expect("Invalid policy");
    let prio = std::env::args()
        .nth(2)
        .expect("Need prio")
        .parse::<i32>()
        .expect("Prio must be a number");

    let run_name = format!(
        "{}-{}-{}-p{}",
        run_name,
        chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        policy,
        prio
    );

    // Servo period from LCNC
    let period = Duration::from_micros(1000);
    // Base thread period from LCNC
    // let period = Duration::from_micros(25);

    let thread = spawn_unchecked(policy, prio, move || {
        let mut start = Instant::now();

        loop {
            thread::sleep(period);

            let time = start.elapsed();
            start = start + start.elapsed();

            println!("{}", time.as_nanos() as i64 - period.as_nanos() as i64);
        }
    })
    .expect("Failed to spawn thread");

    thread.join().unwrap();
}

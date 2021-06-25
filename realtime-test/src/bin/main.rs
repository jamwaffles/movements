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

    let period = Duration::from_millis(1);
    // let (tx, rx) = channel();

    println!("Running {}", run_name);

    let thread = spawn_unchecked(policy, prio, move || {
        let mut start = Instant::now();

        for _i in 0..5000 {
            thread::sleep(period);

            let time = start.elapsed();
            start = start + start.elapsed();

            println!("{}", time.as_nanos() as i64 - period.as_nanos() as i64);
        }
    })
    .expect("Failed to spawn thread");

    thread.join().unwrap();
}

use chrono::SecondsFormat;
use crossbeam::crossbeam_channel::tick;
use histogram::*;
use realtime_test::{spawn_unchecked, SchedPolicy};
use std::{
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

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
    // let period = Duration::from_micros(1000);
    // Base thread period from LCNC
    let period = Duration::from_micros(25);

    let (tx, rx) = channel();

    let mut histogram = Histogram::new();

    let thread = spawn_unchecked(policy, prio, move || {
        let mut start = Instant::now();

        let ticker = tick(period);

        for _ in 0..10000 {
            ticker.recv().unwrap();
            let time = start.elapsed();
            start = start + start.elapsed();

            tx.send(time.as_nanos() as u64).unwrap();
            // println!("elapsed: {:?}", time);
            // println!("{}", time.as_nanos() as i64 - period.as_nanos() as i64);
        }

        // for _ in 0..10000 {
        //     thread::sleep(period);

        //     let time = start.elapsed();
        //     start = start + start.elapsed();

        //     // let value = time.as_nanos() as i64 - period.as_nanos() as i64;

        //     // println!("{}", value);
        //     tx.send(time.as_nanos() as u64).unwrap();
        // }
    })
    .expect("Failed to spawn thread");

    while let Ok(value) = rx.recv() {
        histogram.increment(value).unwrap();
    }

    let stats = format!(
        "Scheduling policy {:?}\nLatency (ns): Min: {:?} Avg: {:?} Max: {:?} StdDev: {:?}",
        policy,
        Duration::from_nanos(histogram.minimum().unwrap()).as_micros(),
        Duration::from_nanos(histogram.mean().unwrap()).as_micros(),
        Duration::from_nanos(histogram.maximum().unwrap()).as_micros(),
        Duration::from_nanos(histogram.stddev().unwrap()).as_micros(),
    );

    println!("{}", stats);

    thread.join().unwrap();
}

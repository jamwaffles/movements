use chrono::SecondsFormat;
use crossbeam::crossbeam_channel::tick;
use histogram::*;
use realtime_test::{spawn_unchecked, SchedPolicy};
use std::fmt;
use std::{
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
enum Stats {
    Base,
    Servo,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base => f.write_str("base "),
            Self::Servo => f.write_str("servo"),
        }
    }
}

struct Payload {
    stats: Stats,
    min_nanos: u64,
    max_nanos: u64,
    mean_nanos: u64,
    stddev_nanos: u64,
}

fn gen_stats(thread: Stats, histogram: &Histogram) -> Payload {
    Payload {
        stats: thread,
        min_nanos: histogram.minimum().unwrap(),
        max_nanos: histogram.maximum().unwrap(),
        mean_nanos: histogram.mean().unwrap(),
        stddev_nanos: histogram.stddev().unwrap(),
    }
}

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
    let servo_period = Duration::from_micros(1000);
    // Base thread period from LCNC
    let base_period = Duration::from_micros(25);

    let log_period = Duration::from_millis(1000);

    let (stats_tx, stats_rx) = channel();

    let stats_tx2 = stats_tx.clone();

    let servo_thread = spawn_unchecked(policy, prio, move || {
        let mut start = Instant::now();

        let ticker = tick(servo_period);

        let mut histogram = Histogram::new();

        let mut interval = Instant::now();

        // for _ in 0..100000 {
        loop {
            ticker.recv().unwrap();
            let time = start.elapsed();
            start = start + start.elapsed();

            histogram.increment(time.as_nanos() as u64).unwrap();

            if interval.elapsed() > log_period {
                interval = Instant::now();

                stats_tx2.send(gen_stats(Stats::Servo, &histogram));
            }
        }
    })
    .expect("Failed to spawn servo thread");

    let base_thread = spawn_unchecked(policy, prio, move || {
        let mut start = Instant::now();

        let mut histogram = Histogram::new();

        let ticker = tick(base_period);

        let mut interval = Instant::now();

        // for _ in 0..100000 {
        loop {
            ticker.recv().unwrap();
            let time = start.elapsed();
            start = start + start.elapsed();

            histogram.increment(time.as_nanos() as u64).unwrap();

            if interval.elapsed() > log_period {
                interval = Instant::now();

                stats_tx.send(gen_stats(Stats::Base, &histogram));
            }
        }
    })
    .expect("Failed to spawn base thread");

    let mut interval = 0;

    while let Ok(stats) = stats_rx.recv() {
        println!(
            "{} latency: min: {:<5} us, avg: {:<5} us, max: {:<5} us stddev: {:<5} us",
            stats.stats,
            stats.min_nanos / 1000,
            stats.mean_nanos / 1000,
            stats.max_nanos / 1000,
            stats.stddev_nanos / 1000
        )
    }

    servo_thread.join().unwrap();
    base_thread.join().unwrap();
}

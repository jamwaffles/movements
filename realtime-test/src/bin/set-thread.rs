use chrono::SecondsFormat;
use crossbeam::tick;
use gpio::GpioOut;
use histogram::Histogram;
use realtime_test::{spawn_unchecked, SchedPolicy};
use std::{
    mem,
    sync::mpsc::channel,
    thread,
    time::{self, Duration, Instant},
};

fn set_thread_prio(prio: i32, policy: SchedPolicy) {
    let mut param: libc::sched_param = unsafe { mem::zeroed() };
    param.sched_priority = prio;

    assert_eq!(
        unsafe { libc::sched_setscheduler(0, policy as i32, &param) },
        0,
        "failed to set prio"
    );
}

fn main() {
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

    // Histogram
    {
        let period = Duration::from_micros(1000);
        let count = 5000;

        eprintln!("Running for {} ms", period.as_millis() * count);

        let mut histogram = Histogram::new();

        let (tx, rx) = channel();

        let handle = thread::spawn(move || {
            set_thread_prio(prio, policy);

            let mut start = Instant::now();
            let ticker = tick(period);

            for _ in 0..count {
                ticker.recv().unwrap();
                let time = start.elapsed();
                start = start + start.elapsed();

                // histogram.increment(time.as_nanos() as u64).unwrap();
                tx.send(time);
                // println!("elapsed: {:?}", time);
                // println!("{}", time.as_nanos() as i64 - period.as_nanos() as i64);
            }
        });

        handle.join().unwrap();

        while let Ok(time) = rx.recv() {
            histogram.increment(time.as_nanos() as u64).unwrap();
        }

        let stats = format!(
        "Period {:?} us\nRan for {} ms\nScheduling policy {:?}\nLatency (ns): Min: {:?} Avg: {:?} Max: {:?} StdDev: {:?}",
            period,
            period.as_millis() * count,
            policy,
            Duration::from_nanos(histogram.minimum().unwrap()),
            Duration::from_nanos(histogram.mean().unwrap()),
            Duration::from_nanos(histogram.maximum().unwrap()),
            Duration::from_nanos(histogram.stddev().unwrap()),
        );

        eprintln!("{}", stats);
    }
}

use crossbeam::crossbeam_channel::tick;
use histogram::*;
use std::thread;
use std::time::{Duration, Instant};
use thread_priority::*;

fn main() {
    let thread_id = thread_native_id();
    let default_prio = thread_priority().unwrap();
    let default_policy = thread_schedule_policy().unwrap();

    // All new threads spawned by main() will have this priority.
    set_thread_priority_and_policy(
        thread_id,
        ThreadPriority::Min,
        // ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo),
        ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Normal),
    )
    .expect("Failed to set priority");

    // // Test setting priority
    // thread::spawn(|| {
    //     println!(
    //         "Priority {:?}, scheduling {:?}",
    //         thread_priority().unwrap(),
    //         thread_schedule_policy().unwrap()
    //     );
    // })
    // .join()
    // .unwrap();

    let period = 1;
    let count = 10000;

    let mut histogram = Histogram::new();
    let mut start = Instant::now();
    let ticker = tick(Duration::from_millis(period));

    for _ in 0..count {
        ticker.recv().unwrap();
        let time = start.elapsed();
        start = start + start.elapsed();

        histogram.increment(time.as_nanos() as u64).unwrap();
        // println!("elapsed: {:?}", time);
        println!("{}", time.as_nanos());
    }

    // let min = histogram.minimum().unwrap();
    // let max = histogram.maximum().unwrap();

    // for bucket in histogram
    //     .into_iter()
    //     .skip_while(|bucket| bucket.value() < min)
    //     .take_while(|bucket| bucket.value() <= max)
    // {
    //     println!("{}, {}", bucket.value(), bucket.count());
    // }

    // println!(
    //     "Latency (ns): Min: {:?} Avg: {:?} Max: {:?} StdDev: {:?}",
    //     Duration::from_nanos(histogram.minimum().unwrap()),
    //     Duration::from_nanos(histogram.mean().unwrap()),
    //     Duration::from_nanos(histogram.maximum().unwrap()),
    //     Duration::from_nanos(histogram.stddev().unwrap()),
    // );

    // thread::spawn(move || {
    //     for _ in 0..count {
    //         thread::sleep(Duration::from_millis(period));
    //         println!("A elapsed: {:?}", start.elapsed());
    //     }
    // })
    // .join()
    // .unwrap();

    // Reset priority to normal
    set_thread_priority_and_policy(thread_id, default_prio, default_policy)
        .expect("Failed to reset priority");

    // Test resetting priority
    thread::spawn(|| {
        println!(
            "Reset priority {:?}, scheduling {:?}",
            thread_priority().unwrap(),
            thread_schedule_policy().unwrap()
        );
    })
    .join()
    .unwrap();
}

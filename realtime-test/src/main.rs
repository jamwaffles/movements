use histogram::*;
use std::thread;
use thread_priority::*;

fn main() {
    println!("Hello, world!");

    let thread_id = thread_native_id();

    set_thread_priority_and_policy(
        thread_id,
        ThreadPriority::Min,
        ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo),
        // ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Normal),
    )
    .unwrap();

    thread::spawn(|| {
        println!(
            "Priority {:?}, scheduling {:?}",
            thread_priority().unwrap(),
            thread_schedule_policy().unwrap()
        );

        println!("I'm in a thread");

        // let thread_id = thread_native_id();

        // set_thread_priority_and_policy(
        //     thread_id,
        //     ThreadPriority::Specific(0),
        //     ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo),
        //     // ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Normal),
        // )
        // .unwrap();

        // println!(
        //     "Priority {:?}, scheduling {:?}",
        //     thread_priority().unwrap(),
        //     thread_schedule_policy().unwrap()
        // );
    })
    .join()
    .unwrap();
}

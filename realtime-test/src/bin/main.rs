use realtime_test::{spawn_with_policy, SchedPolicy};

fn main() {
    let thread = spawn_with_policy(move || println!("Thread!"), SchedPolicy::Fifo);

    thread.join().unwrap();
}

use realtime_test::{spawn_unchecked, SchedPolicy};

// use realtime_test::{spawn_with_policy, SchedPolicy};

// fn main() {
//     let thread = spawn_with_policy(
//         move || {
//             println!("Thread!");
//             // 123456i32
//         },
//         // SchedPolicy::Fifo,
//         SchedPolicy::Other,
//     );

//     let out: () = thread.join().unwrap();
// }

fn main() {
    let thread = unsafe {
        spawn_unchecked(
            SchedPolicy::Fifo,
            // SchedPolicy::Other,
            move || {
                println!("Thread!");
                123456i32
            },
        )
        .expect("Failed to spawn thread")
    };

    let out: i32 = thread.join().unwrap();

    dbg!(out);
}

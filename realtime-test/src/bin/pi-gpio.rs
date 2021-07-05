use chrono::SecondsFormat;
use gpio::GpioOut;
use realtime_test::{spawn_unchecked, SchedPolicy};
use std::{thread, time};

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

    // let mut value = false;
    // let mut gpio = gpio::sysfs::SysFsGpioOutput::open(14).unwrap();

    // let thread = spawn_unchecked(policy, prio, move || loop {
    //     gpio.set_value(value).unwrap();

    //     value = !value;
    // })
    // .expect("Failed to spawn thread");

    // thread.join().unwrap();

    // Let's open GPIO23 and -24, e.g. on a Raspberry Pi 2.
    let mut gpio23 = gpio::sysfs::SysFsGpioInput::open(23).unwrap();
    let mut gpio24 = gpio::sysfs::SysFsGpioOutput::open(24).unwrap();

    // GPIO24 will be toggled every second in the background by a different thread
    let mut value = false;

    let thread = spawn_unchecked(policy, prio, move || loop {
        gpio24.set_value(value).unwrap();

        thread::sleep(time::Duration::from_millis(5));

        value = !value;
    })
    .expect("Failed to spawn thread")
    .join()
    .unwrap();

    // thread::spawn(move || loop {
    //     gpio24.set_value(value).expect("could not set gpio24");
    // thread::sleep(time::Duration::from_nanos(1000));
    //     value = !value;
    // }).join() .unwrap();
}

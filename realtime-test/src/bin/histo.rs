use crossbeam::tick;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use histogram::Histogram;
use realtime_test::{set_thread_prio, SchedPolicy};
use std::{
    process,
    sync::{Arc, Mutex},
    thread::{self},
    time::{Duration, Instant},
};

fn draw_graph<D>(target: &mut D, histo_limits: Size, sums: &[u64]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let bb = target.bounding_box().size;
    let margin = Size::new_equal((bb.width - histo_limits.width) / 2);
    let inner = bb - margin * 2;

    for (x, value) in sums.iter().enumerate() {
        let value = *value as u32;

        let x = margin.width + x as u32;
        let x = x as i32;

        let scaled = (value as f32 / histo_limits.height as f32) * inner.height as f32;
        let scaled = scaled as i32;

        let bottom = Point::new(x, margin.height as i32 + inner.height as i32);
        let top = Point::new(x, margin.height as i32 + inner.height as i32 - scaled);

        Line::new(bottom, top)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(target)?;
    }

    // // Scale size to fit in height
    // size.height = bb.height - margin.height * 2;

    // for x in margin.width..(margin.width + size.width) {
    //     let x = x as i32;

    //     let bottom = Point::new(x, bb.height as i32) - margin.y_axis();
    //     let top = Point::new(x, margin.height as i32);

    //     Line::new(bottom, top)
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
    //         .draw(target)?;
    // }

    Ok(())
}

fn main() -> Result<(), core::convert::Infallible> {
    let display_width = 800;
    let width = display_width - 40;

    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(display_width, 500));

    let output_settings = OutputSettingsBuilder::new().build();
    let mut window = Window::new("Latency test", &output_settings);

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

    let period = Duration::from_micros(1000);
    let count = 1000;

    eprintln!("Running for {} ms", period.as_millis() * count);

    let mut histogram = Arc::new(Mutex::new(
        Histogram::configure().precision(2).build().unwrap(),
    ));

    // let (tx, rx) = channel();

    let histo = histogram.clone();

    let handle = thread::spawn(move || {
        set_thread_prio(prio, policy);

        let mut start = Instant::now();
        let ticker = tick(period);

        println!("Spawn");

        // for _ in 0..count {
        while let Ok(tick) = ticker.recv() {
            let time = start.elapsed();
            start = start + start.elapsed();

            // histogram.increment(time.as_nanos() as u64).unwrap();
            // tx.send(time).unwrap();
            histo
                .lock()
                .unwrap()
                .increment(time.as_nanos() as u64)
                .unwrap();

            // println!("elapsed: {:?}", time);
            // println!("{}", time.as_nanos() as i64 - period.as_nanos() as i64);
        }
    });

    // // Collect histogram results
    // thread::spawn(move || {
    //     while let Ok(time) = rx.recv() {
    //         histogram.increment(time.as_nanos() as u64).unwrap();
    //     }

    // });

    loop {
        let sums = if let Ok(histogram) = histogram.try_lock() {
            println!(
                "Doing stuff, {:?} total buckets {}",
                histogram.into_iter().next(),
                histogram.buckets_total()
            );
            let range = histogram.minimum().unwrap_or(0)..histogram.maximum().unwrap_or(0);

            let value_span = range.end - range.start;

            let span_per_pixel = value_span / width as u64;

            let mut buckets = histogram.into_iter();

            dbg!(span_per_pixel);

            let mut sums = Vec::new();

            let mut bucket_sum = 0;
            let mut bucket_start = range.start;

            while let Some(bucket) = buckets.next() {
                let value = bucket.value();
                let count = bucket.count();

                bucket_sum += count;

                if value > bucket_start + span_per_pixel {
                    sums.push(bucket_sum);
                    bucket_sum = 0;
                    bucket_start += span_per_pixel;
                }
            }

            // for x in 0..display_width {
            //     let x = x as u64;

            //     let start = range.start + x * span_per_pixel;

            //     let span = start..(start + span_per_pixel);

            //     // dbg!(span);

            //     // let sum: u64 = buckets
            //     //     .take(span_per_pixel as usize)
            //     //     .map(Bucket::value)
            //     //     .sum();

            //     sums.push(sum)
            // }

            sums
        } else {
            panic!("Oops");
        };

        let height = sums.iter().max().cloned().unwrap_or(0);

        let size = Size::new(width, height as u32);

        display.clear(Rgb888::BLACK)?;

        draw_graph(&mut display, size, &sums)?;

        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            process::exit(0);
        }

        thread::sleep(Duration::from_millis(50));
    }

    handle.join().unwrap();

    // let histogram = histogram.read().unwrap();

    // eprintln!(
    //     "Period {:?}\nRan for {} ms\nScheduling policy {:?}\nLatency (ns): Min: {:?} Avg: {:?} Max: {:?} StdDev: {:?}",
    //         period,
    //         period.as_millis() * count,
    //         policy,
    //         Duration::from_nanos(histogram.minimum().unwrap_or(0)),
    //         Duration::from_nanos(histogram.mean().unwrap_or(0)),
    //         Duration::from_nanos(histogram.maximum().unwrap_or(0)),
    //         Duration::from_nanos(histogram.stddev().unwrap_or(0)),
    //     );

    Ok(())
}

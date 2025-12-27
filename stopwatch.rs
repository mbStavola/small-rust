#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
humantime = "2.1.0"
---

use std::thread;
use std::time::{Duration, Instant};

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "Keep track of how long a task runs or set a timer for some period of time.")]
struct App {
    #[argh(
        option,
        description = "how long to wait before starting the stopwatch."
    )]
    delay: Option<humantime::Duration>,

    #[argh(
        option,
        description = "how long to wait between timer updates.",
        default = "\"1ms\".parse::<humantime::Duration>().unwrap()"
    )]
    pause: humantime::Duration,

    #[argh(option, description = "how long to run the stopwatch.")]
    duration: Option<humantime::Duration>,

    #[argh(switch, description = "disable terminal bell when stopped.")]
    no_ring: bool,
}

fn main() {
    let app: App = argh::from_env();

    let delay: Option<Duration> = app.delay.map(|d| d.into());
    let pause = app.pause.into();
    let duration = app
        .duration
        .map(|d| d.into())
        .unwrap_or_else(|| Duration::new(u64::MAX, 0));
    let no_ring = app.no_ring;

    if let Some(delay) = delay {
        println!("Waiting {:?}...", delay);
        thread::sleep(delay);
    }

    let bell = if no_ring { "" } else { "\x07" };

    let now = Instant::now();

    print!("\x1B[?25l");
    let mut elapsed = now.elapsed();
    while elapsed < duration {
        print!("\rElapsed: {}\x1B[K", humantime::format_duration(elapsed));
        thread::sleep(pause);
        elapsed = now.elapsed();
    }
    print!("\x1B[?25h");

    print!(
        "\x1B[2K\rElapsed: {}\nStopped{}",
        humantime::format_duration(elapsed),
        bell
    );
}

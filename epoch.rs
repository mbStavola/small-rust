#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
chrono = "0.4"
---

use argh::FromArgs;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

#[derive(FromArgs)]
#[argh(description = "Convert between Unix timestamps and human-readable dates.")]
struct App {
    #[argh(switch, description = "use UTC instead of local time.")]
    utc: bool,

    #[argh(switch, description = "treat timestamp as milliseconds.")]
    millis: bool,

    #[argh(positional, description = "timestamp, date (YYYY-MM-DD HH:MM:SS), or 'now'.")]
    input: Option<String>,
}

fn main() {
    let app: App = argh::from_env();
    let input = app.input.as_deref().unwrap_or("now");

    match input {
        "now" => {
            let now = Utc::now();
            if app.millis {
                println!("{}", now.timestamp_millis());
            } else {
                println!("{}", now.timestamp());
            }
        }
        s if s.chars().all(|c| c.is_ascii_digit()) => {
            let ts: i64 = s.parse().expect("invalid timestamp");
            let dt = if app.millis {
                Utc.timestamp_millis_opt(ts).single()
            } else {
                Utc.timestamp_opt(ts, 0).single()
            }
            .expect("invalid timestamp");

            if app.utc {
                println!("{}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            } else {
                let local: DateTime<Local> = dt.into();
                println!("{}", local.format("%Y-%m-%d %H:%M:%S"));
            }
        }
        s => {
            let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .expect("expected format: YYYY-MM-DD HH:MM:SS");

            let dt: DateTime<Utc> = if app.utc {
                Utc.from_utc_datetime(&naive)
            } else {
                Local
                    .from_local_datetime(&naive)
                    .single()
                    .expect("invalid local time")
                    .into()
            };

            if app.millis {
                println!("{}", dt.timestamp_millis());
            } else {
                println!("{}", dt.timestamp());
            }
        }
    }
}

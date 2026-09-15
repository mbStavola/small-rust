#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
humantime = "2.1.0"
---

use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::{Duration, SystemTime};

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(
    description = "Measure battery charge rate, adapter power, and pack health.\n\nPower is signed: positive means the pack is charging, negative means it is \
                   draining. A laptop can report \"charging\" while still running at a \
                   deficit if the adapter cannot cover the system load."
)]
struct App {
    #[argh(
        option,
        description = "how many samples to take. defaults to 1.",
        default = "1"
    )]
    count: usize,

    #[argh(
        option,
        description = "how long to wait between samples.",
        default = "\"30s\".parse::<humantime::Duration>().unwrap()"
    )]
    interval: humantime::Duration,

    #[argh(
        option,
        description = "append each sample to a tsv file, building history across runs."
    )]
    log: Option<String>,

    #[argh(
        switch,
        description = "emit one json object per sample instead of a table."
    )]
    json: bool,
}

struct Reading {
    percent: f64,
    power_w: f64,
    voltage_v: f64,
    now_mah: f64,
    full_mah: f64,
    design_mah: Option<f64>,
    cycles: Option<u64>,
    status: String,
    adapter: Option<Adapter>,
    to_full_min: Option<u64>,
    to_empty_min: Option<u64>,
}

struct Adapter {
    watts: f64,
    volts: Option<f64>,
    amps: Option<f64>,
}

const RULE: &str =
    "──────────  ─────────  ────────  ───────────────────  ────────────  ──────────────────";

fn main() {
    let app: App = argh::from_env();

    let interval: Duration = app.interval.into();
    let mut samples = Vec::with_capacity(app.count);

    for i in 0..app.count {
        let reading = read_battery();

        if app.json {
            println!("{}", reading.to_json());
        } else {
            if i == 0 {
                reading.print_summary();
                print_table_header();
            }
            println!("{}", reading.row(interval * i as u32));
        }

        if let Some(path) = &app.log {
            reading.append_to(path);
        }

        samples.push(reading.power_w);

        if i + 1 < app.count {
            thread::sleep(interval);
        }
    }

    if !app.json && samples.len() > 1 {
        let avg = samples.iter().sum::<f64>() / samples.len() as f64;
        let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        println!("{}", RULE);
        println!(
            "{:>10}  {:>9}  {} samples, min {:+.1} W, max {:+.1} W",
            "avg",
            format!("{:+.1} W", avg),
            samples.len(),
            min,
            max
        );
    }
}

fn print_table_header() {
    println!(
        "{:>10}  {:>9}  {:>8}  {:>19}  {:<12}  {}",
        "elapsed", "power", "charge", "capacity", "status", "estimate"
    );
    println!("{}", RULE);
}

impl Reading {
    fn print_summary(&self) {
        let health = self
            .design_mah
            .map(|d| {
                format!(
                    "   {:.1}% of {:.0} mAh design",
                    100.0 * self.full_mah / d,
                    d
                )
            })
            .unwrap_or_default();
        let cycles = self
            .cycles
            .map(|c| format!("   {} cycles", c))
            .unwrap_or_default();

        println!(
            "battery   {:.0} mAh full{}{}",
            self.full_mah, health, cycles
        );

        match &self.adapter {
            Some(a) => {
                let contract = match (a.volts, a.amps) {
                    (Some(v), Some(c)) => format!("   {:.1} V x {:.2} A", v, c),
                    _ => String::new(),
                };
                println!("adapter   {:.0} W{}", a.watts, contract);
            }
            None => println!("adapter   none detected"),
        }
        println!();
    }

    fn row(&self, elapsed: Duration) -> String {
        format!(
            "{:>10}  {:>9}  {:>8}  {:>19}  {:<12}  {}",
            humantime::format_duration(elapsed).to_string(),
            format!("{:+.1} W", self.power_w),
            format!("{:.1}%", self.percent),
            format!("{:.0} / {:.0} mAh", self.now_mah, self.full_mah),
            self.status,
            self.estimate(),
        )
    }

    fn estimate(&self) -> String {
        let pretty = |m: u64| humantime::format_duration(Duration::from_secs(m * 60)).to_string();

        match (self.power_w > 0.0, self.to_full_min, self.to_empty_min) {
            (true, Some(m), _) => format!("to full {}", pretty(m)),
            (false, _, Some(m)) => format!("to empty {}", pretty(m)),
            _ => String::new(),
        }
    }

    fn append_to(&self, path: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("should be able to open the log file");

        if file
            .metadata()
            .map(|m| m.len() == 0)
            .expect("should be able to stat the log file")
        {
            writeln!(
                file,
                "timestamp\tpower_w\tpercent\tnow_mah\tfull_mah\tstatus\tadapter_w"
            )
            .expect("should be able to write the log header");
        }

        writeln!(
            file,
            "{}\t{:+.2}\t{:.1}\t{:.0}\t{:.0}\t{}\t{}",
            humantime::format_rfc3339_seconds(SystemTime::now()),
            self.power_w,
            self.percent,
            self.now_mah,
            self.full_mah,
            self.status,
            self.adapter
                .as_ref()
                .map(|a| format!("{:.0}", a.watts))
                .unwrap_or_else(|| "-".into()),
        )
        .expect("should be able to write the log row");
    }

    fn to_json(&self) -> String {
        let opt = |v: Option<f64>| v.map(|v| format!("{:.2}", v)).unwrap_or("null".into());
        let opt_u = |v: Option<u64>| v.map(|v| v.to_string()).unwrap_or("null".into());

        format!(
            r#"{{"timestamp":"{}","percent":{:.2},"power_w":{:.2},"voltage_v":{:.3},"now_mah":{:.0},"full_mah":{:.0},"design_mah":{},"cycles":{},"status":"{}","adapter_w":{},"to_full_min":{},"to_empty_min":{}}}"#,
            humantime::format_rfc3339_seconds(SystemTime::now()),
            self.percent,
            self.power_w,
            self.voltage_v,
            self.now_mah,
            self.full_mah,
            opt(self.design_mah),
            opt_u(self.cycles),
            self.status,
            opt(self.adapter.as_ref().map(|a| a.watts)),
            opt_u(self.to_full_min),
            opt_u(self.to_empty_min),
        )
    }
}

#[cfg(target_os = "macos")]
fn read_battery() -> Reading {
    use std::process::Command;

    let output = Command::new("ioreg")
        .args(["-r", "-n", "AppleSmartBattery"])
        .output()
        .expect("failed to execute ioreg");

    let raw = String::from_utf8(output.stdout).expect("must be valid utf8");

    let now_mah = top_num(&raw, "AppleRawCurrentCapacity").expect("should report current capacity");
    let full_mah = top_num(&raw, "AppleRawMaxCapacity").expect("should report max capacity");
    let voltage_v = top_num(&raw, "Voltage").expect("should report pack voltage") / 1000.0;

    let amps = top_field(&raw, "Amperage")
        .and_then(|v| v.parse::<u64>().ok())
        .map(as_signed)
        .expect("should report amperage") as f64
        / 1000.0;

    let charging = raw.contains("\"IsCharging\" = Yes");
    let external = raw.contains("\"ExternalConnected\" = Yes");
    let full = raw.contains("\"FullyCharged\" = Yes");

    let status = match (full, charging, external) {
        (true, _, _) => "full",
        (_, true, _) => "charging",
        (_, _, true) => "plugged in",
        _ => "on battery",
    }
    .to_string();

    Reading {
        percent: 100.0 * now_mah / full_mah,
        power_w: amps * voltage_v,
        voltage_v,
        now_mah,
        full_mah,
        design_mah: top_num(&raw, "DesignCapacity"),
        cycles: top_field(&raw, "CycleCount").and_then(|v| v.parse().ok()),
        status,
        adapter: macos_adapter(&raw),
        to_full_min: top_field(&raw, "AvgTimeToFull").and_then(parse_minutes),
        to_empty_min: top_field(&raw, "AvgTimeToEmpty").and_then(parse_minutes),
    }
}

#[cfg(target_os = "macos")]
fn macos_adapter(raw: &str) -> Option<Adapter> {
    let start = raw.find("\"AdapterDetails\" = {")?;
    let rest = &raw[start..];

    let block = &rest[..rest.find('}').unwrap_or(rest.len())];

    let watts = nested_num(block, "Watts")?;
    if watts == 0.0 {
        return None;
    }

    Some(Adapter {
        watts,
        volts: nested_num(block, "AdapterVoltage").map(|v| v / 1000.0),
        amps: nested_num(block, "Current").map(|c| c / 1000.0),
    })
}

#[cfg(target_os = "macos")]
fn parse_minutes(value: &str) -> Option<u64> {
    match value.parse::<u64>().ok()? {
        65535 => None,
        minutes => Some(minutes),
    }
}

#[cfg(target_os = "macos")]
fn as_signed(value: u64) -> i64 {
    if value >= 1 << 63 {
        (value as i128 - (1i128 << 64)) as i64
    } else {
        value as i64
    }
}

#[cfg(target_os = "macos")]
fn field<'a>(haystack: &'a str, key: &str, spaced: bool) -> Option<&'a str> {
    let needle = if spaced {
        format!("\"{key}\" = ")
    } else {
        format!("\"{key}\"=")
    };

    let start = haystack.find(&needle)? + needle.len();
    let rest = &haystack[start..];
    let end = rest.find([',', '}', ')', '\n']).unwrap_or(rest.len());

    Some(rest[..end].trim())
}

#[cfg(target_os = "macos")]
fn top_field<'a>(haystack: &'a str, key: &str) -> Option<&'a str> {
    field(haystack, key, true)
}

#[cfg(target_os = "macos")]
fn top_num(haystack: &str, key: &str) -> Option<f64> {
    top_field(haystack, key)?.parse().ok()
}

#[cfg(target_os = "macos")]
fn nested_num(haystack: &str, key: &str) -> Option<f64> {
    field(haystack, key, false)?.parse().ok()
}

#[cfg(target_os = "linux")]
fn read_battery() -> Reading {
    use std::fs;

    let supplies = fs::read_dir("/sys/class/power_supply")
        .expect("should be able to read /sys/class/power_supply");

    let mut battery = None;
    let mut mains = None;

    for entry in supplies.flatten() {
        let path = entry.path();
        match sysfs_str(&path, "type").as_deref() {
            Some("Battery") if battery.is_none() => battery = Some(path),
            Some("Mains") | Some("USB") if mains.is_none() => mains = Some(path),
            _ => {}
        }
    }

    let battery = battery.expect("should find a battery under /sys/class/power_supply");

    let voltage_v = sysfs_num(&battery, "voltage_now")
        .map(|v| v / 1_000_000.0)
        .expect("should report pack voltage");

    let (now_mah, full_mah, design_mah) = match sysfs_num(&battery, "charge_now") {
        Some(now) => (
            now / 1000.0,
            sysfs_num(&battery, "charge_full").expect("should report full charge") / 1000.0,
            sysfs_num(&battery, "charge_full_design").map(|d| d / 1000.0),
        ),
        None => {
            let to_mah = |uwh: f64| uwh / voltage_v / 1000.0;
            (
                to_mah(sysfs_num(&battery, "energy_now").expect("should report charge or energy")),
                to_mah(sysfs_num(&battery, "energy_full").expect("should report full energy")),
                sysfs_num(&battery, "energy_full_design").map(to_mah),
            )
        }
    };

    let status = sysfs_str(&battery, "status").unwrap_or_else(|| "unknown".into());

    let magnitude_w = match sysfs_num(&battery, "power_now") {
        Some(uw) => uw / 1_000_000.0,
        None => sysfs_num(&battery, "current_now")
            .map(|ua| (ua / 1_000_000.0) * voltage_v)
            .unwrap_or(0.0),
    }
    .abs();

    let power_w = if status.eq_ignore_ascii_case("discharging") {
        -magnitude_w
    } else if status.eq_ignore_ascii_case("charging") {
        magnitude_w
    } else {
        0.0
    };

    let remaining_min = |mah: f64| {
        if magnitude_w <= 0.0 {
            None
        } else {
            Some(((mah / 1000.0 * voltage_v) / magnitude_w * 60.0) as u64)
        }
    };

    Reading {
        percent: 100.0 * now_mah / full_mah,
        power_w,
        voltage_v,
        now_mah,
        full_mah,
        design_mah,
        cycles: sysfs_num(&battery, "cycle_count").map(|c| c as u64),
        adapter: mains.and_then(|p| linux_adapter(&p)),
        to_full_min: (power_w > 0.0)
            .then(|| remaining_min(full_mah - now_mah))
            .flatten(),
        to_empty_min: (power_w < 0.0).then(|| remaining_min(now_mah)).flatten(),
        status: status.to_lowercase(),
    }
}

#[cfg(target_os = "linux")]
fn linux_adapter(path: &std::path::Path) -> Option<Adapter> {
    if sysfs_num(path, "online")? == 0.0 {
        return None;
    }

    let volts = sysfs_num(path, "voltage_now").map(|v| v / 1_000_000.0);
    let amps = sysfs_num(path, "current_max")
        .or_else(|| sysfs_num(path, "current_now"))
        .map(|c| c / 1_000_000.0);

    let watts = match (volts, amps) {
        (Some(v), Some(c)) => v * c,
        _ => return None,
    };

    Some(Adapter { watts, volts, amps })
}

#[cfg(target_os = "linux")]
fn sysfs_str(path: &std::path::Path, name: &str) -> Option<String> {
    std::fs::read_to_string(path.join(name))
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(target_os = "linux")]
fn sysfs_num(path: &std::path::Path, name: &str) -> Option<f64> {
    sysfs_str(path, name)?.parse().ok()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("batt_stats supports macos and linux only");

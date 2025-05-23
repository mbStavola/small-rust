#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
crono = "0.4"
---

use argh::FromArgs;
use chrono::prelude::*;

#[derive(FromArgs)]
#[argh(description = "Utility to help tag a git release.")]
struct App {
    #[argh(
        option,
        description = "how to format the date tag.",
        default = "\"v%Y%m%d-%H%M\".to_string()"
    )]
    format: String,
}

fn main() {
    let app: App = argh::from_env();

    let tag = Utc::now().format(&app.format).to_string();

    println!("# Run the following to tag and release this version");
    println!();
    println!("git tag -a {} -m \"\"", tag);
    println!("git push origin --tags");
}

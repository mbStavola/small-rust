#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
qrcode = "0.14.1"
---

use std::fs;
use std::path::PathBuf;

use argh::FromArgs;

use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;

#[derive(FromArgs)]
#[argh(description = "Create a QR code from anything.")]
struct App {
    #[argh(option, description = "module width in characters.", default = "1")]
    scale: u32,

    #[argh(switch, description = "disable quiet zone border.")]
    no_quiet_zone: bool,

    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    File(FileCmd),
    Raw(RawCmd),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "file")]
#[argh(description = "Read from a file")]
struct FileCmd {
    #[argh(positional)]
    path: PathBuf,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "raw")]
#[argh(description = "Use raw input")]
struct RawCmd {
    #[argh(positional)]
    input: String,
}

fn main() {
    let app: App = argh::from_env();

    let data = match app.command {
        Command::File(FileCmd { path }) => fs::read_to_string(path).expect("could not read file"),
        Command::Raw(RawCmd { input }) => input,
    };

    let code = QrCode::new(data.into_bytes()).expect("could not create qr code");
    let rendered = code
        .render::<Dense1x2>()
        .module_dimensions(app.scale, app.scale)
        .quiet_zone(!app.no_quiet_zone)
        .build();
    println!("{}", rendered);
}

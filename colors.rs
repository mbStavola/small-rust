#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
owo-colors = "4"
---

use argh::FromArgs;
use owo_colors::{AnsiColors, OwoColorize, Rgb, XtermColors};

#[derive(FromArgs)]
#[argh(description = "Display terminal color capabilities")]
struct App {
    #[argh(switch, description = "show only basic 16 ANSI colors.")]
    basic: bool,

    #[argh(switch, description = "show 256 color palette.")]
    palette: bool,

    #[argh(switch, description = "show true color (24-bit) gradient.")]
    truecolor: bool,
}

const BLOCK: &str = "  ";

fn main() {
    let app: App = argh::from_env();
    let show_all = !app.basic && !app.palette && !app.truecolor;

    if show_all || app.basic {
        print_basic_colors();
    }
    if show_all || app.palette {
        print_256_palette();
    }
    if show_all || app.truecolor {
        print_truecolor();
    }
}

fn print_basic_colors() {
    use AnsiColors::*;
    const COLORS: [AnsiColors; 8] = [Black, Red, Green, Yellow, Blue, Magenta, Cyan, White];

    println!("ANSI 16 Colors:\n");

    print!("  Normal:  ");
    for color in COLORS {
        print!("{}", BLOCK.on_color(color));
    }
    println!();

    print!("  Bright:  ");
    for color in COLORS {
        print!("{}", BLOCK.on_color(color).bold());
    }
    println!("\n");
}

fn print_256_palette() {
    println!("256 Color Palette:\n");

    // Standard 0-15
    print!("  ");
    for i in 0..16u8 {
        print!("{}", BLOCK.on_color(XtermColors::from(i)));
    }
    println!("\n");

    // Color cube 16-231
    for row in 0..6u8 {
        print!("  ");
        for green in 0..6u8 {
            for red in 0..6u8 {
                let color = 16 + row * 36 + green * 6 + red;
                print!("{}", BLOCK.on_color(XtermColors::from(color)));
            }
            print!(" ");
        }
        println!();
    }
    println!();

    // Grayscale 232-255
    print!("  ");
    for i in 232..=255u8 {
        print!("{}", BLOCK.on_color(XtermColors::from(i)));
    }
    println!("\n");
}

fn print_truecolor() {
    println!("True Color (24-bit) Test:\n");

    // Red gradient
    print!("  ");
    for i in 0..64u8 {
        print!("{}", " ".on_color(Rgb(i * 4, 0, 0)));
    }
    println!();

    // Green gradient
    print!("  ");
    for i in 0..64u8 {
        print!("{}", " ".on_color(Rgb(0, i * 4, 0)));
    }
    println!();

    // Blue gradient
    print!("  ");
    for i in 0..64u8 {
        print!("{}", " ".on_color(Rgb(0, 0, i * 4)));
    }
    println!();

    // Rainbow
    print!("  ");
    for i in 0..64 {
        let (r, g, b) = hsv_to_rgb(i as f32 / 64.0 * 360.0);
        print!("{}", " ".on_color(Rgb(r, g, b)));
    }
    println!("\n");

    println!("  If gradients appear smooth, your terminal supports true color.\n");
}

fn hsv_to_rgb(h: f32) -> (u8, u8, u8) {
    let x = 1.0 - ((h / 60.0) % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0..=59 => (1.0, x, 0.0),
        60..=119 => (x, 1.0, 0.0),
        120..=179 => (0.0, 1.0, x),
        180..=239 => (0.0, x, 1.0),
        240..=299 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

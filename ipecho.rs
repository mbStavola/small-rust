#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
reqwest = { version = "0.11.3", features = ["blocking"] }
---

use reqwest::blocking::Client;

fn main() {
    let client = Client::new();
    let res = client
        .get("https://ipecho.net/plain")
        .send()
        .expect("unexpected error while trying to reach ipecho.net");

    if res.status() != 200 {
        panic!("Unexpected status code {}", res.status());
    }

    let ip = res.text().expect("should be able to deserialize body");
    println!("{}", ip);
}

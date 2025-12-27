#!/usr/bin/env -S cargo +nightly -Zscript

---
[package]
edition = "2024"
[dependencies]
argh = "0.1"
---

use std::net::{TcpStream, ToSocketAddrs};
use std::process;
use std::time::Duration;

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "Check if a TCP port is reachable.")]
struct App {
    #[argh(option, description = "timeout in seconds.", default = "5")]
    timeout: u64,

    #[argh(positional, description = "host to connect to.")]
    host: String,

    #[argh(positional, description = "port to connect to.")]
    port: u16,
}

fn main() {
    let app: App = argh::from_env();
    let address = format!("{}:{}", app.host, app.port);
    let timeout = Duration::from_secs(app.timeout);

    let addrs: Vec<_> = address
        .to_socket_addrs()
        .expect("failed to resolve address")
        .collect();

    for addr in &addrs {
        if TcpStream::connect_timeout(addr, timeout).is_ok() {
            println!("reachable");
            process::exit(0);
        }
    }

    println!("unreachable");
    process::exit(1);
}

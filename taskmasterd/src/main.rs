mod daemon;
mod job;

use anyhow::{Context, Result};
use clap::*;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const FILES_DIR: &str = "/tmp/taskmasterd";
const SOCKET_PATH: &str = "/tmp/taskmasterd/taskmasterd.sock";

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    nodaemon: bool,
}

pub fn main_loop() -> Result<()> {
    if std::fs::metadata(SOCKET_PATH).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(SOCKET_PATH)
            .with_context(|| format!("could not delete previous socket at {:?}", SOCKET_PATH))?;
    }
    let unix_listener = UnixListener::bind(SOCKET_PATH)?;
    unix_listener
        .set_nonblocking(true)
        .context("could not set non-blocking mode on socket")?;
    let term = Arc::new(AtomicBool::new(false));
    let mut response = String::new();
    signal_hook::flag::register(signal_hook::consts::SIGHUP, Arc::clone(&term))?;
    while !term.load(Ordering::Relaxed) {
        if let Ok((mut stream, _)) = unix_listener.accept() {
            stream
                .read_to_string(&mut response)
                .context("Failed at reading the request")?;
            eprintln!("message: {}", response);
            stream
                .write_all(format!("ack {}", response).as_bytes())
                .context("Failed at writing the response")?;
            if response == "quit" {
                break;
            }
            response.clear();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    if opts.nodaemon {
        main_loop()?;
    } else {
        daemon::init()?;
    }
    Ok(())
}

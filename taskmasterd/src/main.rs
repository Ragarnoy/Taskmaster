mod daemon;
mod job;

use anyhow::{Context, Result};
use clap::*;
use job::Jobs;
use signal_hook::consts::signal::SIGHUP;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

pub const FILES_DIR: &str = "/tmp/taskmasterd";
const SOCKET_PATH: &str = "/tmp/taskmasterd/taskmasterd.sock";

#[derive(Parser)]
#[command(version, author, about)]
struct Opts {
    #[clap(short, long)]
    nodaemon: bool,
}

fn create_socket() -> Result<UnixListener> {
    if fs::metadata(SOCKET_PATH).is_ok() {
        eprintln!("A socket is already present. Deleting...");
        fs::remove_file(SOCKET_PATH)
            .with_context(|| format!("could not delete previous socket at {:?}", SOCKET_PATH))?;
    }
    let unix_listener = UnixListener::bind(SOCKET_PATH)
        .with_context(|| format!("could not bind to socket at {:?}", SOCKET_PATH))?;
    unix_listener
        .set_nonblocking(true)
        .context("could not set non-blocking mode on socket")?;
    Ok(unix_listener)
}

fn create_signal_handler() -> Result<Arc<AtomicBool>> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGHUP, Arc::clone(&term))
        .context("Failed to register SIGHUP handler")?;
    Ok(term)
}

fn get_jobs() -> Result<Jobs> {
    let jobs = match job::find_config() {
        Some(path) => job::load_config_file(path).context("Failed to load config file")?,
        None => Default::default(),
    };
    Ok(jobs)
}

fn check_socket_message(stream: &mut UnixStream, response: &mut String) -> Result<()> {
    stream
        .read_to_string(response)
        .context("Failed at reading the request")?;
    eprintln!("message: {}", response);
    stream
        .write_all(format!("ack {}", response).as_bytes())
        .context("Failed at writing the response")?;
    Ok(())
}

pub fn main_loop() -> Result<()> {
    let unix_listener = create_socket()?;
    let term = create_signal_handler()?;
    let mut jobs = get_jobs()?;
    let mut response = String::new();
    jobs.auto_start().context("Jobs auto-start failed")?;
    while !term.load(Ordering::Relaxed) {
        if let Ok((mut stream, _)) = unix_listener.accept() {
            check_socket_message(&mut stream, &mut response)?;
            if response == "quit" {
                break;
            }
            response.clear();
        }
        sleep(Duration::from_millis(100));
    }
    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    // create a directory for the tmp files if it doesn't exist
    fs::create_dir_all(FILES_DIR).context("could not create files directory")?;
    if opts.nodaemon {
        main_loop()?;
    } else {
        daemon::init()?;
    }
    Ok(())
}

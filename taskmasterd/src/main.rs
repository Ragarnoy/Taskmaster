mod daemon;
mod job;
mod sleeper;
mod socket;

use crate::socket::Socket;
use crate::sleeper::Sleeper;
use anyhow::{Context, Result};
use clap::*;
use job::Jobs;
use signal_hook::consts::signal::SIGHUP;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const FILES_DIR: &str = "/tmp/taskmasterd";
const SOCKET_PATH: &str = "/tmp/taskmasterd/taskmasterd.sock";

#[derive(Parser)]
#[command(version, author, about)]
struct Opts {
    #[clap(short, long)]
    nodaemon: bool,
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

pub fn main_loop() -> Result<()> {
    let socket = Socket::new(SOCKET_PATH)?;
    let term = create_signal_handler()?;
    let mut jobs = get_jobs()?;
    let mut response = String::new();
    let mut sleeper = Sleeper::new(100)?;
    jobs.auto_start().context("Jobs auto-start failed")?;
    while !term.load(Ordering::Relaxed) {
        if socket.read(&mut response)? {
            if response == "quit" {
                break;
            }
            response.clear();
        }
        jobs.check_status().context("Jobs status check failed")?;
        sleeper.sleep()?;
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

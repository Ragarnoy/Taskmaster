mod daemon;
mod job;
mod jobs;
mod listener;
mod sleeper;
mod socket;

use crate::jobs::load_config_file;
use crate::sleeper::Sleeper;
use crate::socket::Socket;
use anyhow::{Context, Result};
use clap::Parser;
use dirs::home_dir;
use jobs::Jobs;
use listener::Action;
use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const FILES_DIR: &str = ".taskmasterd";
const SOCKET_PATH: &str = ".taskmasterd/taskmasterd.sock";

#[derive(Parser)]
#[command(version, author, about)]
struct Opts {
    #[clap(short, long)]
    nodaemon: bool,
}

fn create_signal_handler() -> Result<(Arc<AtomicBool>, Arc<AtomicBool>)> {
    let hup = Arc::new(AtomicBool::new(false));
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGHUP, Arc::clone(&hup))
        .context("Failed to register SIGHUP handler")?;
    signal_hook::flag::register(SIGTERM, Arc::clone(&term))
        .context("Failed to register SIGTERM handler")?;
    signal_hook::flag::register(SIGINT, Arc::clone(&term))
        .context("Failed to register SIGTERM handler")?;
    signal_hook::flag::register(SIGQUIT, Arc::clone(&term))
        .context("Failed to register SIGTERM handler")?;
    Ok((hup, term))
}

pub fn main_loop() -> Result<()> {
    let socket_path = home_dir()
        .context("could not find home directory")?
        .join(SOCKET_PATH);
    let socket = Socket::new(&socket_path)?;
    let (hup, term) = create_signal_handler()?;
    let mut jobs = Jobs::new().context("Jobs creation failed")?;
    let mut response = String::new();
    let mut sleeper = Sleeper::new(100)?;
    jobs.auto_start();
    while !term.load(Ordering::Relaxed) {
        if hup.load(Ordering::Relaxed) {
            hup.store(false, Ordering::Relaxed);
            jobs.reread().context("Jobs reload failed")?;
        }
        if let Some(stream) = socket.read(&mut response)? {
            let action = Action::from_str(&response)?;
            match action {
                Action::Start(name) => jobs.start(&name),
                Action::Stop(name) => jobs.stop(&name).context("Job stop failed")?,
                Action::Restart(name) => jobs.restart(&name).context("Job restart failed")?,
                Action::Status(name) => {
                    let status = jobs.status(&name).context("Job status failed")?;
                    socket.write(&status, stream)?;
                }
                Action::Reload => {
                    jobs.reload().context("Jobs reload failed")?;
                }
                Action::Shutdown => {
                    break;
                }
                Action::Load(path) => {
                    if let Ok(new_jobs) = load_config_file(PathBuf::from(path.clone())) {
                        jobs.load_new_jobs(new_jobs).context("Jobs load failed")?;
                    } else {
                        // if the config file is invalid, we keep the old one
                        socket.write("Invalid config file", stream)?;
                        eprintln!("Received invalid config file: {}", path);
                    }
                }
            }
            response.clear();
        }
        jobs.check_status().context("Jobs status check failed")?;
        sleeper.sleep()?;
    }
    println!("Shutting down");
    jobs.stop_all().context("Jobs stop failed")?;
    try_wait_processes_end(&mut jobs, &mut sleeper)?;
    println!("All jobs stopped");
    Ok(())
}

fn try_wait_processes_end(jobs: &mut Jobs, sleeper: &mut Sleeper) -> Result<()> {
    while jobs.programs.iter().any(|p| p.1.is_running()) {
        jobs.check_status().context("Jobs status check failed")?;
        sleeper.sleep()?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    // create a directory for the tmp files if it doesn't exist
    let path = home_dir()
        .context("could not find home directory")?
        .join(FILES_DIR);
    fs::create_dir_all(path).context("could not create files directory")?;
    if opts.nodaemon {
        main_loop()?;
    } else {
        daemon::init()?;
    }
    Ok(())
}

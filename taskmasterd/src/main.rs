mod daemon;
mod job;
mod jobs;
mod listener;
mod sleeper;
mod socket;

use crate::sleeper::Sleeper;
use crate::socket::Socket;
use anyhow::{Context, Result};
use clap::Parser;
use dirs::home_dir;
use jobs::Jobs;
use listener::Action;
use signal_hook::consts::signal::{SIGHUP, SIGTERM};
use std::fs;
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
    Ok((hup, term))
}

fn get_jobs() -> Result<Jobs> {
    let jobs = match job::find_config() {
        Some(path) => job::load_config_file(path).context("Failed to load config file")?,
        None => Jobs::default(),
    };
    Ok(jobs)
}

pub fn main_loop() -> Result<()> {
    let socket_path = home_dir()
        .context("could not find home directory")?
        .join(SOCKET_PATH);
    let socket = Socket::new(&socket_path)?;
    let (hup, term) = create_signal_handler()?;
    let mut jobs = get_jobs()?;
    let mut response = String::new();
    let mut sleeper = Sleeper::new(100)?;
    jobs.auto_start().context("Jobs auto-start failed")?;
    while !term.load(Ordering::Relaxed) {
        if hup.load(Ordering::Relaxed) {
            jobs = get_jobs()?;
            jobs.auto_start().context("Jobs auto-start failed")?;
            hup.store(false, Ordering::Relaxed);
        }
        if let Some(stream) = socket.read(&mut response)? {
            let action = Action::from_str(&response)?;
            match action {
                Action::Start(name) => jobs.start(&name).context("Job start failed")?,
                Action::Stop(name) => jobs.stop(&name).context("Job stop failed")?,
                Action::Restart(name) => jobs.restart(&name).context("Job restart failed")?,
                Action::Status(name) => {
                    let status = jobs.status(&name).context("Job status failed")?;
                    socket.write(&status, stream)?;
                }
                Action::Reload => {
                    jobs.reload().context("Jobs reload failed")?;
                    jobs = get_jobs()?;
                    jobs.auto_start().context("Jobs auto-start failed")?;
                }
                Action::Shutdown => {
                    break;
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

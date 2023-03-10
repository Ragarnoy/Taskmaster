use crate::job::jobconfig::stopsignal::StopSignal;
use crate::job::jobconfig::JobConfig;
use anyhow::{Context, Result};
use nix::sys::signal::Signal;
use nix::sys::stat::{umask, Mode};
use nix::unistd::Pid;
use std::fmt::{Debug, Display, Formatter};
use std::process::{Child, Command};
use std::time::Instant;

// TODO Restrain PID to Running states
#[derive(Debug, Clone, Copy)]
pub enum RunningStatus {
    Running,
    StartRequested { start: Instant, tries: u32 },
    StopRequested { since: Instant, restart: bool },
}

/// Status of a stopped process
#[derive(Debug, Default)]
pub enum StoppedStatus {
    /// Process exited before being fully started
    Backoff {
        /// Number of times the process has been restarted
        tries: u32,
        /// Time at which the process was put in backoff
        started_at: Instant,
    },
    /// Process could not be started
    Fatal,
    /// Process exited unexpectedly
    Unexpected,
    /// Process exited safely
    Exited,
    /// Process was stopped or never started
    #[default]
    Stopped,
}

#[derive(Debug)]
pub enum State {
    Stopped(StoppedStatus),
    Running {
        pid: Pid,
        child: Child,
        status: RunningStatus,
    },
}

impl Display for RunningStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunningStatus::Running => write!(f, "RUNNING"),
            RunningStatus::StartRequested { start, tries } => write!(
                f,
                "START_REQUESTED (tries: {}, since: {})",
                tries,
                start.elapsed().as_secs()
            ),
            RunningStatus::StopRequested { since, restart } => {
                write!(
                    f,
                    "STOP_REQUESTED (since: {}, restarting: {})",
                    since.elapsed().as_secs(),
                    restart
                )
            }
        }
    }
}

impl Display for StoppedStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StoppedStatus::Backoff {
                tries: restarts,
                started_at: start,
            } => write!(
                f,
                "BACKOFF (restarts: {}, since: {})",
                restarts,
                start.elapsed().as_secs()
            ),
            StoppedStatus::Fatal => write!(f, "FATAL"),
            StoppedStatus::Unexpected => write!(f, "UNEXPECTED"),
            StoppedStatus::Exited => write!(f, "EXITED"),
            StoppedStatus::Stopped => write!(f, "STOPPED"),
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = 10;
        match self {
            State::Stopped(fatal) => write!(f, "pid: {:>width$} - {}", "N/A", fatal),
            State::Running { pid, status, .. } => {
                write!(f, "pid: {:>width$} - {}", pid, status)
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Stopped(StoppedStatus::default())
    }
}

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub state: State,
    config: JobConfig,
}

impl Process {
    pub fn new(name: String, config: &JobConfig) -> Self {
        Self {
            name,
            state: State::default(),
            config: config.clone(),
        }
    }

    fn get_tries(&self) -> u32 {
        match &self.state {
            State::Stopped(StoppedStatus::Backoff { tries: t, .. }) => *t,
            State::Running {
                status: RunningStatus::StartRequested { tries: t, .. },
                ..
            } => *t,
            _ => 0,
        }
    }

    fn try_start(&mut self) -> Result<()> {
        let mut command = Command::new(
            std::fs::canonicalize(&self.config.cmd).context("Failed to find command")?,
        );
        command.current_dir(self.config.workingdir.0.clone());
        if let Some(env) = &self.config.env {
            command.envs(env.0.iter());
        }
        if let Some(stdout) = &self.config.stdout {
            command.stdout(std::fs::File::create(stdout).context("Failed to open stdout file")?);
        }
        if let Some(stderr) = &self.config.stderr {
            command.stderr(std::fs::File::create(stderr).context("Failed to open stderr file")?);
        }

        if let Some(umask_value) = &self.config.umask {
            umask(Mode::from_bits_truncate(umask_value.0));
        }

        let child = command.spawn()?;
        // FIXME Needs to be configurable
        umask(Mode::from_bits_truncate(0o022));

        let tries = self.get_tries();
        self.state = State::Running {
            pid: Pid::from_raw(child.id() as i32),
            child,
            status: RunningStatus::StartRequested {
                start: Instant::now(),
                tries,
            },
        };
        Ok(())
    }

    /// Returns the adequate `StoppedStatus` for the current state
    fn get_stopped_status(&self) -> StoppedStatus {
        let tries = self.get_tries();
        if tries < self.config.startretries {
            println!("{}: backing off", self.name);
            StoppedStatus::Backoff {
                tries: tries + 1,
                started_at: Instant::now(),
            }
        } else {
            println!("{}: giving up", self.name);
            StoppedStatus::Fatal
        }
    }

    pub fn start(&mut self) {
        if let State::Stopped(_) = self.state {
            if let Err(e) = self.try_start() {
                eprintln!("{}: failed to start: {}", self.name, e);
                self.state = State::Stopped(self.get_stopped_status());
            };
        } else {
            eprintln!("{}: already running", self.name);
        }
    }

    pub fn stop(&mut self, stop_signal: StopSignal, restart: bool) -> Result<()> {
        if let State::Running { pid, status, .. } = &mut self.state {
            nix::sys::signal::kill(*pid, Signal::from(stop_signal))?;
            *status = RunningStatus::StopRequested {
                since: Instant::now(),
                restart,
            };
        } else if let State::Stopped(StoppedStatus::Backoff { .. }) = &self.state {
            self.state = State::Stopped(StoppedStatus::Stopped);
        }
        Ok(())
    }

    /// Kill the process
    /// This is a shortcut for `stop(StopSignal::Kill)`
    pub fn kill(&mut self) -> Result<()> {
        self.stop(StopSignal::Kill, false)?;
        self.state = State::Stopped(StoppedStatus::Stopped);
        // not sure if that's the right way to do it
        Ok(())
    }

    pub fn restart(&mut self, config: &JobConfig) -> Result<()> {
        match &self.state {
            State::Running { .. } => self.stop(config.stopsignal, true)?,
            State::Stopped(status) => {
                if let StoppedStatus::Backoff { .. } = status {
                    self.state = State::Stopped(StoppedStatus::Stopped);
                }
                self.start();
            }
        }
        Ok(())
    }

    /// Update the process state
    pub fn update_status(&mut self, config: &JobConfig) -> Result<()> {
        if let State::Running { child, status, .. } = &mut self.state {
            if let Some(exit_status) = child.try_wait()? {
                let expected = if let Some(exit_code) = exit_status.code() {
                    config.exitcodes.is_valid(exit_code)
                } else {
                    true // <== process received a signal, so it exiting is expected
                };
                self.state = State::Stopped(match status {
                    RunningStatus::StartRequested { tries, .. } => {
                        println!(
                            "{}: exited before being fully started ({} tries)",
                            self.name, tries
                        );
                        self.get_stopped_status()
                    }
                    RunningStatus::StopRequested { restart, .. } => {
                        if *restart {
                            self.state = State::Stopped(StoppedStatus::Stopped);
                            println!("{}: exited and will be restarted", self.name);
                            self.start();
                            return Ok(()); // sale
                        } else {
                            println!("{}: has been stopped", self.name);
                            StoppedStatus::Stopped
                        }
                    }
                    RunningStatus::Running => {
                        if expected {
                            println!("{}: exited", self.name);
                            StoppedStatus::Exited
                        } else {
                            StoppedStatus::Unexpected
                        }
                    }
                });
            }
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        match &self.state {
            State::Stopped(_) => false,
            State::Running { .. } => true,
        }
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.name, self.state)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if let State::Running { .. } = &self.state {
            panic!("Process {} was not stopped", self.name);
        }
    }
}

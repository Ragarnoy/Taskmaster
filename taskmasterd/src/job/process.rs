use crate::job::jobconfig::stopsignal::StopSignal;
use crate::job::jobconfig::umask::Umask;
use crate::job::jobconfig::JobConfig;
use anyhow::{anyhow, Result};
use nix::sys::signal::Signal;
use nix::sys::stat::{umask, Mode};
use nix::unistd::Pid;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::process::{Child, Command};
use std::time::Instant;

// TODO Restrain PID to Running states
#[derive(Debug, Clone, Copy)]
pub enum RunningStatus {
    Running,
    StartRequested(Instant),
    StopRequested(Instant),
}

/// Status of a stopped process
#[derive(Debug, Default)]
pub enum StoppedStatus {
    /// Process exited before being fully started
    Backoff,
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
enum State {
    Stopped(StoppedStatus),
    Running {
        pid: Pid,
        child: Child,
        status: RunningStatus,
    },
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Stopped(fatal) => match fatal {
                StoppedStatus::Backoff => write!(f, "BACKOFF"),
                StoppedStatus::Fatal => write!(f, "FATAL"),
                StoppedStatus::Unexpected => write!(f, "UNEXPECTED"),
                StoppedStatus::Exited => write!(f, "EXITED"),
                StoppedStatus::Stopped => write!(f, "STOPPED"),
            },
            State::Running { pid, status, .. } => {
                write!(f, "RUNNING (pid: {}, status: {:?})", pid, status)
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
    pub command: Command,
    pub umask: Option<Umask>,
    state: State,
}

impl Process {
    pub fn new(name: String, config: &JobConfig) -> Result<Self> {
        let mut command = Command::new(std::fs::canonicalize(&config.cmd)?);
        command.current_dir(config.workingdir.0.clone());
        if let Some(env) = &config.env {
            command.envs(env.0.iter());
        }
        if let Some(stdout) = &config.stdout {
            command.stdout(std::fs::File::create(stdout)?);
        }
        if let Some(stderr) = &config.stderr {
            command.stderr(std::fs::File::create(stderr)?);
        }

        Ok(Self {
            name,
            command,
            umask: config.umask,
            state: State::default(),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        if let Some(umask_value) = self.umask {
            umask(Mode::from_bits_truncate(umask_value.0));
        }

        let child = self.command.spawn()?;
        // FIXME Needs to be configurable
        umask(Mode::from_bits_truncate(0o022));

        self.state = State::Running {
            pid: Pid::from_raw(child.id() as i32),
            child,
            status: RunningStatus::StartRequested(Instant::now()),
        };
        Ok(())
    }

    pub fn stop(&mut self, stop_signal: StopSignal) -> Result<()> {
        if let State::Running { pid, status, .. } = &mut self.state {
            nix::sys::signal::kill(*pid, Signal::from(stop_signal))?;
            *status = RunningStatus::StopRequested(Instant::now());
        } else {
            return Err(anyhow!(StopError::NotRunning));
        }
        Ok(())
    }

    pub fn restart(&mut self, config: &JobConfig) -> Result<()> {
        self.stop(config.stopsignal)?;
        self.start()?;
        Ok(())
    }

    /// Check the status of the process
    /// Returns None if the process is still running
    /// Returns Some(true) if the process exited with an expected exit code
    /// Returns Some(false) if the process exited with an unexpected exit code
    pub fn check_status(&mut self, config: &JobConfig) -> Result<Option<bool>> {
        match &mut self.state {
            State::Stopped(_) => Err(anyhow!(CheckStatusError::NoChildProcess)),
            State::Running { child, status, .. } => match child.try_wait()? {
                Some(exit_status) => {
                    let expected = if let Some(exit_code) = exit_status.code() {
                        config.exitcodes.is_valid(exit_code)
                    } else {
                        true // <== process received a signal, so it's expected
                    };
                    self.state = State::Stopped(match status {
                        RunningStatus::StartRequested(_) => {
                            StoppedStatus::Backoff // TODO implement max retries
                        }
                        RunningStatus::StopRequested(_) => StoppedStatus::Stopped,
                        RunningStatus::Running => {
                            if expected {
                                StoppedStatus::Exited
                            } else {
                                StoppedStatus::Unexpected
                            }
                        }
                    });
                    Ok(Some(expected))
                }
                None => {
                    match status {
                        RunningStatus::StartRequested(start_time) => {
                            if start_time.elapsed().as_secs() >= config.starttimeout.0 {
                                *status = RunningStatus::Running;
                            }
                        }
                        RunningStatus::StopRequested(stop_time) => {
                            if stop_time.elapsed().as_secs() >= config.stoptimeout.0 {
                                self.stop(StopSignal::Kill)?;
                            }
                        }
                        RunningStatus::Running => {}
                    }
                    Ok(None)
                }
            },
        }
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

#[derive(Debug)]
pub enum CheckStatusError {
    NoChildProcess,
    TryWaitError,
}

impl Display for CheckStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatusError::NoChildProcess => {
                write!(f, "No child process")
            }
            CheckStatusError::TryWaitError => {
                write!(f, "Try wait error")
            }
        }
    }
}

#[derive(Debug)]
pub enum StopError {
    NotRunning,
}

impl Display for StopError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StopError::NotRunning => {
                write!(f, "Process is not running")
            }
        }
    }
}

impl Error for StopError {}

impl Error for CheckStatusError {}

impl Drop for Process {
    fn drop(&mut self) {
        if let State::Running { .. } = &self.state {
            panic!("Process {} was not stopped", self.name);
        }
    }
}

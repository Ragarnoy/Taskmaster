use crate::job::jobconfig::stopsignal::StopSignal;
use crate::job::jobconfig::umask::Umask;
use crate::job::jobconfig::JobConfig;
use anyhow::{anyhow, Result};
use nix::sys::signal::Signal;
use nix::sys::stat::{umask, Mode};
use nix::unistd::Pid;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::process::{Child, Command, ExitStatus};

// TODO Restrain PID to Running states
#[derive(Debug, Clone, Copy)]
pub enum RunningStatus {
    Running,
    StartRequested,
    StopRequested,
}

type Fatal = bool;

#[derive(Debug)]
enum State {
    Stopped(Fatal),
    Running {
        pid: Pid,
        child: Child,
        status: RunningStatus,
    },
}

impl Default for State {
    fn default() -> Self {
        Self::Stopped(false)
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
        let mut command = Command::new(&config.cmd);
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
            status: RunningStatus::StartRequested,
        };
        Ok(())
    }

    pub fn stop(&mut self, stop_signal: StopSignal) -> Result<()> {
        if let State::Running { pid, status, .. } = &mut self.state {
            nix::sys::signal::kill(*pid, Signal::from(stop_signal))?;
            *status = RunningStatus::StopRequested;
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

    pub fn check_status(&mut self) -> Result<Option<ExitStatus>> {
        match &mut self.state {
            State::Stopped(_) => Err(anyhow!(CheckStatusError::NoChildProcess)),
            State::Running { child, status, .. } => match child.try_wait()? {
                Some(exit_status) => {
                    self.state = State::Stopped(false);
                    Ok(Some(exit_status))
                }
                None => {
                    if let RunningStatus::StartRequested = status {
                        *status = RunningStatus::Running;
                    }
                    Ok(None)
                }
            },
        }
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

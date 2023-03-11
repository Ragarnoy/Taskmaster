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
    StartRequested { start: Instant, tries: u32 },
    StopRequested(Instant),
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

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Stopped(fatal) => match fatal {
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
    pub state: State,
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

        let tries = if let State::Stopped(StoppedStatus::Backoff { tries: t, .. }) = &self.state {
            *t
        } else {
            0
        };
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

    pub fn stop(&mut self, stop_signal: StopSignal) -> Result<()> {
        if let State::Running { pid, status, .. } = &mut self.state {
            nix::sys::signal::kill(*pid, Signal::from(stop_signal))?;
            *status = RunningStatus::StopRequested(Instant::now());
        } else {
            return Err(anyhow!(StopError::NotRunning));
        }
        Ok(())
    }

    /// Kill the process
    /// This is a shortcut for `stop(StopSignal::Kill)`
    pub fn kill(&mut self) -> Result<()> {
        self.stop(StopSignal::Kill)?;
        self.state = State::Stopped(StoppedStatus::Stopped);
        // not sure if that's the right way to do it
        Ok(())
    }

    pub fn restart(&mut self, config: &JobConfig) -> Result<()> {
        self.stop(config.stopsignal)?;
        self.start()?;
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
                        print!(
                            "{}: exited before being fully started ({} tries)",
                            self.name, tries
                        );
                        if *tries < config.startretries {
                            println!(", backing off");
                            StoppedStatus::Backoff {
                                tries: *tries + 1,
                                started_at: Instant::now(),
                            }
                        } else {
                            println!(", giving up");
                            StoppedStatus::Fatal
                        }
                    }
                    RunningStatus::StopRequested(_) => StoppedStatus::Stopped,
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

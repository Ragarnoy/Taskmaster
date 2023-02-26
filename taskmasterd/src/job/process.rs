use anyhow::Result;
use std::process::{Child, Command, ExitStatus};

#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Stopped,
    Running,
    Fatal,
    Restarting,
    Starting,
}

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub command: Command,
    pub child: Option<Child>,
    pub pid: Option<u32>,
    pub status: Status,
}

impl Process {
    pub fn new(name: String, command: &str) -> Result<Self> {
        let command = Command::new(command);
        Ok(Self {
            name,
            command,
            child: None,
            pid: None,
            status: Status::default(),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let child = self.command.spawn()?;
        self.pid = Some(child.id());
        self.child = Some(child);
        self.status = Status::Starting;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            child.kill()?;
        }
        self.check_status()?;
        Ok(())
    }

    pub fn restart(&mut self) -> Result<()> {
        self.status = Status::Restarting;
        self.stop()?;
        self.start()?;
        self.status = Status::Starting;
        Ok(())
    }

    pub fn check_status(&mut self) -> Result<Option<ExitStatus>, CheckStatusError> {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait()? {
                Some(status) => {
                    self.status = Status::Stopped;
                    self.pid = None;
                    self.child = None;
                    Ok(Some(status))
                }
                None => {
                    self.status = Status::Running;
                    Ok(None)
                }
            }
        } else {
            Err(CheckStatusError::NoChildProcess)
        }
    }
}

pub enum CheckStatusError {
    NoChildProcess,
    TryWaitError,
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            child.kill().unwrap();
        }
    }
}

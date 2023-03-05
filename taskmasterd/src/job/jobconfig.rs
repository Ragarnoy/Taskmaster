use autorestart::AutoRestart;
use env::Env;
use exitcodes::ExitCodes;
use numprocs::NumProcs;
use serde::Deserialize;
use starttimeout::StartTimeout;
use std::path::PathBuf;
use stopsignal::StopSignal;
use stoptimeout::StopTimeout;
use umask::Umask;
use workingdir::WorkingDir;

pub mod autorestart;
pub mod env;
pub mod exitcodes;
pub mod numprocs;
pub mod starttimeout;
pub mod stopsignal;
pub mod stoptimeout;
pub mod umask;
pub mod workingdir;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct JobConfig {
    pub cmd: String,
    #[serde(default)]
    pub numprocs: NumProcs,
    pub umask: Option<Umask>,
    #[serde(default)]
    pub workingdir: WorkingDir,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub autorestart: AutoRestart,
    #[serde(default)]
    pub exitcodes: ExitCodes,
    #[serde(default)]
    pub startretries: u32,
    #[serde(default)]
    pub stoptimeout: StopTimeout,
    #[serde(default)]
    pub starttimeout: StartTimeout,
    #[serde(default)]
    pub stopsignal: StopSignal,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub env: Option<Env>,
}

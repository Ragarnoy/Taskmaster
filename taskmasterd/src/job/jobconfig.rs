use std::collections::HashMap;
use crate::job::jobconfig::autorestart::AutoRestart;
use crate::job::jobconfig::exitcodes::ExitCodes;
use crate::job::jobconfig::numprocs::NumProcs;
use crate::job::jobconfig::stopsignal::StopSignal;
use crate::job::jobconfig::workingdir::WorkingDir;
use serde::Deserialize;
use std::path::PathBuf;
use serde_yaml::Value;
use crate::job::jobconfig::umask::Umask;

pub mod autorestart;
pub mod exitcodes;
pub mod numprocs;
pub mod stopsignal;
pub mod workingdir;
mod umask;

#[derive(Debug, Clone, Deserialize)]
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
    pub stoptimeout: u32,
    #[serde(default)]
    pub starttimeout: u32,
    #[serde(default)]
    pub stopsignal: StopSignal,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub env: Option<HashMap<String, Value>>,
}

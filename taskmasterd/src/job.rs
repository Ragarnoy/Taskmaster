use crate::job::jobconfig::JobConfig;
use crate::job::process::Process;
use serde::Deserialize;

pub mod jobconfig;
pub mod process;

pub type Jobs = Vec<Job>;

#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub name: String,
    pub config: JobConfig,
    #[serde(skip)]
    pub processes: Vec<Process>,
}

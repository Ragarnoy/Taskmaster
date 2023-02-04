use crate::job::jobconfig::JobConfig;
use crate::job::process::Process;
use serde::Deserialize;
use anyhow::Result;

pub mod jobconfig;
pub mod process;

const DEFAULT_CONFIG_PATHS: [&str; 3] = ["config.yml", "../config.yml", "/etc/taskmasterd/config.yml"];

pub type Jobs = Vec<Job>;

#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub name: String,
    pub config: JobConfig,
    #[serde(skip)]
    pub processes: Vec<Process>,
}

pub fn find_config() -> Option<String> {
    for path in DEFAULT_CONFIG_PATHS.iter() {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

pub fn load_config(path: &str) -> Result<Jobs> {
    let file = std::fs::File::open(path)?;
    let jobs: Jobs = serde_yaml::from_reader(file)?;
    Ok(jobs)
}
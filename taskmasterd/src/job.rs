use crate::job::jobconfig::JobConfig;
use crate::job::process::Process;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod jobconfig;
pub mod process;

const DEFAULT_CONFIG_PATHS: [&str; 3] =
    ["config.yml", "../config.yml", "/etc/taskmasterd/config.yml"];

#[derive(Debug, Deserialize, Default)]
pub struct Jobs {
    pub programs: HashMap<String, Job>,
}

impl Jobs {
    pub fn auto_start(&mut self) -> Result<()> {
        for (name, job) in self.programs.iter_mut() {
            if job.config.autostart {
                job.start(name.clone())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Job {
    #[serde(flatten)]
    pub config: JobConfig,
    #[serde(skip)]
    pub processes: Vec<Process>,
}

impl Job {
    pub fn start(&mut self, mut name: String) -> Result<()> {
        for i in 0..self.config.numprocs.0.into() {
            name = format!("{}-{}", name, i);
            let mut process = Process::new(name.clone(), &self.config.cmd)?;
            process.start()?;
            self.processes.push(process);
        }
        Ok(())
    }
}

pub fn find_config() -> Option<PathBuf> {
    for path in DEFAULT_CONFIG_PATHS.iter() {
        if std::path::Path::new(path).exists() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

pub fn load_config_file(path: PathBuf) -> Result<Jobs> {
    let file = std::fs::File::open(path)?;
    let jobs: Jobs = serde_yaml::from_reader(file)?;
    Ok(jobs)
}

pub fn load_config(str: &str) -> Result<Jobs> {
    let jobs: Jobs = serde_yaml::from_str(str)?;
    Ok(jobs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_EXAMPLE: &str = r#"
    programs:
        nginx:
            cmd: "/usr/local/bin/nginx -c /etc/nginx/test.conf"
            numprocs: 1
            umask: 022
            workingdir: /tmp
            autostart: true
            autorestart: unexpected
            exitcodes:
                - 0
                - 2
            startretries: 3
            starttime: 5
            stopsignal: TERM
            stoptime: 10
            stdout: /tmp/nginx.stdout
            stderr: /tmp/nginx.stderr
            env:
                STARTED_BY: taskmaster
                ANSWER: 42
"#;

    #[test]
    fn test_find_config() {
        assert!(find_config().is_some());
    }

    #[test]
    fn test_load_config_file() {
        let path = find_config().unwrap();
        let jobs = load_config_file(path).unwrap();
        assert_eq!(jobs.programs.len(), 2);
    }

    #[test]
    fn test_load_config() {
        let jobs = load_config(CONFIG_EXAMPLE).unwrap();
        assert_eq!(jobs.programs.len(), 1);
    }
}

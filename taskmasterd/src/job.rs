use anyhow::{Ok, Result};
use jobconfig::JobConfig;
use process::{Process, State};
use serde::Deserialize;

use std::path::PathBuf;

pub mod jobconfig;
pub mod process;

const DEFAULT_CONFIG_PATHS: [&str; 3] =
    ["config.yml", "../config.yml", "/etc/taskmasterd/config.yml"];

#[derive(Debug, Deserialize)]
pub struct Job {
    #[serde(flatten)]
    pub config: JobConfig,
    #[serde(skip)]
    pub processes: Vec<Process>,
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl Eq for Job {}

impl Job {
    pub fn init(&mut self, name: &str) -> Result<()> {
        for i in 0..self.config.numprocs.0.into() {
            let process = Process::new(format!("{}-{}", name, i), &self.config);
            self.processes.push(process);
        }
        Ok(())
    }

    pub fn start(&mut self) {
        for process in self.processes.iter_mut() {
            process.start();
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        for process in self.processes.iter_mut() {
            process.stop(self.config.stopsignal)?;
        }
        Ok(())
    }

    pub fn restart(&mut self) -> Result<()> {
        for process in self.processes.iter_mut() {
            process.restart(&self.config)?;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.processes.iter().any(Process::is_running)
    }

    pub fn check_status(&mut self) -> Result<()> {
        use crate::job::jobconfig::autorestart::AutoRestart;
        for process in self.processes.iter_mut() {
            process.update_status(&self.config)?;
            match &mut process.state {
                State::Stopped(status) => match status {
                    process::StoppedStatus::Backoff { tries, started_at } => {
                        if started_at.elapsed().as_secs() >= (*tries).into() {
                            println!("{}: backoff expired, restart", process.name);
                            process.start();
                        }
                    }
                    process::StoppedStatus::Unexpected => {
                        // if autorestart is at true or unexpected, restart
                        if self.config.autorestart == AutoRestart::Always
                            || self.config.autorestart == AutoRestart::Unexpected
                        {
                            println!("{}: unexpected exit, restart", process.name);
                            process.start();
                        } else {
                            println!("{}: unexpected exit", process.name);
                        }
                    }
                    process::StoppedStatus::Exited => {
                        if self.config.autorestart == AutoRestart::Always {
                            println!("{}: exited, restart", process.name);
                            process.start();
                        }
                    }
                    process::StoppedStatus::Fatal => {}
                    process::StoppedStatus::Stopped => {}
                },
                State::Running { status, .. } => match status {
                    process::RunningStatus::StopRequested(since) => {
                        if since.elapsed().as_secs() >= self.config.stoptimeout.0 {
                            println!("{}: stop timeout expired, kill", process.name);
                            process.kill()?;
                        }
                    }
                    process::RunningStatus::StartRequested { start, .. } => {
                        if start.elapsed().as_secs() >= self.config.starttime.0 {
                            println!(
                                "{}: start period ended ({}s)",
                                process.name, self.config.starttime.0
                            );
                            *status = process::RunningStatus::Running;
                        }
                    }
                    process::RunningStatus::Running => {}
                },
            }
        }
        Ok(())
    }

    pub fn print_status(&self) -> String {
        let mut status = String::new();
        if self.processes.is_empty() {
            status.push_str("No process running\n");
        } else {
            self.processes.iter().for_each(|p| {
                status.push_str(&p.to_string());
            });
        }
        status
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        assert!(
            !self.is_running(),
            "Job processes should be stopped before being dropped"
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::{load_config_file, Jobs};

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
    fn test_config_diff() {
        let jobs: Jobs = serde_yaml::from_str(CONFIG_EXAMPLE).unwrap();
        let new_jobs: Jobs = serde_yaml::from_str(CONFIG_EXAMPLE).unwrap();
        assert_eq!(jobs.programs, new_jobs.programs);
    }

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
        let jobs: Jobs = serde_yaml::from_str(CONFIG_EXAMPLE).unwrap();
        assert_eq!(jobs.programs.len(), 1);
    }
}

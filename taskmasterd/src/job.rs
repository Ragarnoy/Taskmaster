use crate::job::jobconfig::JobConfig;
use crate::job::process::Process;
use anyhow::{anyhow, Ok, Result};
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
    pub fn load_new_config(&mut self, new_conf: &str) -> Result<()> {
        let new_jobs: Jobs = load_config(new_conf)?;
        // if job is in new config, and is not equal to old config, update it and restart it
        // if job is in new config but not in old config, insert it
        // if job is in old config but not in new config, remove it
        self.programs
            .retain(|name, _| new_jobs.programs.contains_key(name));

        for (name, job) in new_jobs.programs {
            if let Some(old_job) = self.programs.get_mut(&name) {
                if old_job.config != job.config {
                    old_job.stop()?;
                    old_job.config = job.config.clone();
                }
            } else {
                self.programs.insert(name, job);
            }
        }
        self.auto_start()?;
        Ok(())
    }

    pub fn auto_start(&mut self) -> Result<()> {
        for (name, job) in self.programs.iter_mut() {
            if job.config.autostart && !job.is_running() {
                job.start(name)?;
            }
        }
        Ok(())
    }

    pub fn status(&self, name: &str) -> Result<String> {
        if name.is_empty() {
            return self.status_all();
        } else if let Some(job) = self.programs.get(name) {
            return Ok(job.print_status());
        }
        Err(anyhow!("Job {} not found", name))
    }

    pub fn status_all(&self) -> Result<String> {
        let mut status = String::new();
        for (name, job) in self.programs.iter() {
            status.push_str(format!("Job status {}:\n", name).as_str());
            status.push_str(&job.print_status());
            status.push('\n');
        }
        Ok(status)
    }

    pub fn start(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            return self.start_all();
        }
        if let Some(job) = self.programs.get_mut(name) {
            job.start(name)?;
        }
        Ok(())
    }

    pub fn start_all(&mut self) -> Result<()> {
        for (name, job) in self.programs.iter_mut() {
            job.start(name)?;
        }
        Ok(())
    }

    pub fn stop(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            return self.stop_all();
        }
        if let Some(job) = self.programs.get_mut(name) {
            job.stop()?;
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<()> {
        for job in self.programs.values_mut() {
            job.stop()?;
        }
        Ok(())
    }

    pub fn restart(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            return self.restart_all();
        }
        if let Some(job) = self.programs.get_mut(name) {
            job.restart()?;
        }
        Ok(())
    }

    pub fn restart_all(&mut self) -> Result<()> {
        for job in self.programs.values_mut() {
            job.restart()?;
        }
        Ok(())
    }

    pub fn check_status(&mut self) -> Result<()> {
        for job in self.programs.values_mut() {
            job.check_status()?;
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

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl Eq for Job {}

impl Job {
    pub fn start(&mut self, name: &str) -> Result<()> {
        for i in 0..self.config.numprocs.0.into() {
            let name = format!("{}-{}", name, i);
            let mut process = Process::new(name.clone(), &self.config)?;
            process.start()?;
            self.processes.push(process);
        }
        Ok(())
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
        self.processes.iter().any(|p| p.is_running())
    }

    pub fn check_status(&mut self) -> Result<()> {
        use crate::job::process::CheckStatusError;

        for process in self.processes.iter_mut() {
            let status = process.check_status(&self.config);
            match status {
                Err(e) => {
                    if let CheckStatusError::NoChildProcess =
                        e.downcast_ref::<CheckStatusError>().unwrap()
                    {
                        continue;
                    } else {
                        return Err(e);
                    }
                }
                Result::Ok(status) => {
                    if let Some(fatal) = status {
                        use crate::job::jobconfig::autorestart::*;
                        match self.config.autorestart {
                            AutoRestart::True => {
                                process.start()?;
                            }
                            AutoRestart::Unexpected => {
                                if fatal {
                                    process.start()?;
                                }
                            }
                            AutoRestart::False => (),
                        }
                    }
                }
            }
        }
        self.processes.retain(|p| p.is_running());
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
        self.stop().unwrap();
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
    fn test_config_diff() {
        let jobs = load_config(CONFIG_EXAMPLE).unwrap();
        let new_jobs = load_config(CONFIG_EXAMPLE).unwrap();
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
        let jobs = load_config(CONFIG_EXAMPLE).unwrap();
        assert_eq!(jobs.programs.len(), 1);
    }
}

use crate::job::{find_config, Job};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Jobs {
    pub programs: HashMap<String, Job>,
}

impl Jobs {
    pub fn new() -> Result<Jobs> {
        let jobs = match find_config() {
            Some(path) => load_config_file(path).context("Failed to load config file")?,
            None => Jobs::default(),
        };
        Ok(jobs)
    }

    pub fn load_new_jobs(&mut self, new_jobs: Jobs) -> Result<()> {
        // if job is in new config, and is not equal to old config, update it and restart it
        // if job is in new config but not in old config, insert it
        // if job is in old config but not in new config, remove it
        let to_remove = self
            .programs
            .iter()
            .filter(|(name, jobs)| {
                !new_jobs.programs.contains_key(*name) || *jobs != &new_jobs.programs[*name]
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<String>>();

        let to_add = new_jobs
            .programs
            .into_iter()
            .filter(|(name, jobs)| {
                !self.programs.contains_key(name) || self.programs[name] != *jobs
            })
            .collect::<Vec<(String, Job)>>();

        // FIXME This is slow
        for name in to_remove {
            self.remove_job(&name)?;
        }
        for (name, job) in to_add {
            self.programs.insert(name, job);
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
            Ok(self.status_all())
        } else if let Some(job) = self.programs.get(name) {
            Ok(job.print_status())
        } else {
            Err(anyhow!("Job {} not found", name))
        }
    }

    pub fn status_all(&self) -> String {
        let mut status = String::new();
        for (name, job) in self.programs.iter() {
            status.push_str(format!("Job status {}:\n", name).as_str());
            status.push_str(&job.print_status());
            status.push('\n');
        }
        status
    }

    pub fn init(&mut self) -> Result<()> {
        self.programs
            .iter_mut()
            .try_for_each(|(name, job)| job.init(name))?;
        Ok(())
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

    pub fn reload(&mut self) -> Result<()> {
        println!("Reloading config");
        self.stop_all()?;
        self.try_wait_job_stop()?;
        self.clear_jobs();
        *self = Jobs::new().context("Failed to load new config")?;
        self.auto_start().context("Jobs auto-start failed")?;
        Ok(())
    }

    pub fn reread(&mut self) -> Result<()> {
        println!("Rereading config");
        self.stop_all()?;
        let path = find_config().context("Failed to find config")?;
        let new_jobs = load_config_file(path).context("Failed to load config")?;
        self.try_wait_job_stop()?;
        *self = new_jobs;
        self.auto_start()?;
        Ok(())
    }

    fn try_wait_job_stop(&mut self) -> Result<()> {
        while self.programs.values().any(Job::is_running) {
            self.check_status()?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
    }

    pub fn remove_job(&mut self, name: &str) -> Result<()> {
        if let Some(job) = self.programs.get_mut(name) {
            job.stop()?;
        }
        self.try_wait_job_stop()?;
        self.programs.remove(name);
        Ok(())
    }

    pub fn clear_jobs(&mut self) {
        self.programs.clear();
    }
}

pub fn load_config_file(path: PathBuf) -> Result<Jobs> {
    let file = std::fs::File::open(path)?;
    let mut jobs: Jobs = serde_yaml::from_reader(file)?;
    jobs.init()?;
    Ok(jobs)
}

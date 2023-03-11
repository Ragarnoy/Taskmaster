use crate::job;
use crate::job::Job;
use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct Jobs {
    pub programs: HashMap<String, Job>,
}

impl Jobs {
    pub fn load_new_config(&mut self, new_conf: &str) -> anyhow::Result<()> {
        let new_jobs: Jobs = job::load_config(new_conf)?;
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
        anyhow::Ok(())
    }

    pub fn auto_start(&mut self) -> anyhow::Result<()> {
        for (name, job) in self.programs.iter_mut() {
            if job.config.autostart && !job.is_running() {
                job.start(name)?;
            }
        }
        anyhow::Ok(())
    }

    pub fn status(&self, name: &str) -> anyhow::Result<String> {
        if name.is_empty() {
            return self.status_all();
        } else if let Some(job) = self.programs.get(name) {
            return anyhow::Ok(job.print_status());
        }
        Err(anyhow!("Job {} not found", name))
    }

    pub fn status_all(&self) -> anyhow::Result<String> {
        let mut status = String::new();
        for (name, job) in self.programs.iter() {
            status.push_str(format!("Job status {}:\n", name).as_str());
            status.push_str(&job.print_status());
            status.push('\n');
        }
        anyhow::Ok(status)
    }

    pub fn start(&mut self, name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return self.start_all();
        }
        if let Some(job) = self.programs.get_mut(name) {
            job.start(name)?;
        }
        anyhow::Ok(())
    }

    pub fn start_all(&mut self) -> anyhow::Result<()> {
        for (name, job) in self.programs.iter_mut() {
            job.start(name)?;
        }
        anyhow::Ok(())
    }

    pub fn stop(&mut self, name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return self.stop_all();
        }
        if let Some(job) = self.programs.get_mut(name) {
            job.stop()?;
        }
        anyhow::Ok(())
    }

    pub fn stop_all(&mut self) -> anyhow::Result<()> {
        for job in self.programs.values_mut() {
            job.stop()?;
        }
        anyhow::Ok(())
    }

    pub fn restart(&mut self, name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return self.restart_all();
        }
        if let Some(job) = self.programs.get_mut(name) {
            job.restart()?;
        }
        anyhow::Ok(())
    }

    pub fn restart_all(&mut self) -> anyhow::Result<()> {
        for job in self.programs.values_mut() {
            job.restart()?;
        }
        anyhow::Ok(())
    }

    pub fn check_status(&mut self) -> anyhow::Result<()> {
        for job in self.programs.values_mut() {
            job.check_status()?;
        }
        anyhow::Ok(())
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        println!("Reloading config");
        self.stop_all()?;
        while self.programs.values().any(|j| j.is_running()) {
            self.check_status()?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        self.clear_jobs();
        anyhow::Ok(())
    }

    pub fn clear_jobs(&mut self) {
        self.programs.clear();
    }
}

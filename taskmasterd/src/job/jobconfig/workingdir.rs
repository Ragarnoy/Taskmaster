use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkingDir(pub PathBuf);

impl Default for WorkingDir {
    fn default() -> Self {
        unimplemented!("Needs taskmasterd config path")
    }
}

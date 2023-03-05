use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct WorkingDir(pub PathBuf);

impl Default for WorkingDir {
    fn default() -> Self {
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        Self(cwd)
    }
}

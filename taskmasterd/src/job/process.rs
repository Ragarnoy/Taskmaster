use std::process::Child;

#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Stopped,
    Running,
    Fatal,
    Restarting,
    Starting,
}

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub pid: i32,
    pub child: Child,
    pub status: Status,
}

impl Process {
    pub fn new(name: String, command: &str) -> Self {
        let child = std::process::Command::new(command)
            .spawn()
            .expect("Failed to spawn process");
        let pid = child.id() as i32;
        Self {
            name,
            pid,
            child,
            status: Status::Starting,
        }
    }
}

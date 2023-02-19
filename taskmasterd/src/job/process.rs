#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Stopped,
    Running,
    Fatal,
    Restarting,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub name: String,
    pub pid: i32,
    pub status: Status,
}

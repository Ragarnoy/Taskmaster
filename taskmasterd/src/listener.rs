use std::str::FromStr;

pub enum Action {
    Start(String),
    Stop(String),
    Restart(String),
    Status(String),
    Reload,
    Shutdown,
}

impl FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        let parts = s.split_once(' ').unwrap_or((s, ""));
        let action = parts.0;
        let name = parts.1;
        match action {
            "start" => Ok(Action::Start(name.to_string())),
            "stop" => Ok(Action::Stop(name.to_string())),
            "restart" => Ok(Action::Restart(name.to_string())),
            "status" => Ok(Action::Status(name.to_string())),
            "reload" => Ok(Action::Reload),
            "shutdown" => Ok(Action::Shutdown),
            _ => Err(anyhow::anyhow!("Unknown action")),
        }
    }
}

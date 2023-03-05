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
        let mut parts = s.split_whitespace();
        let action = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid action"))?;
        let name = parts.next().unwrap_or("");
        match action {
            "start" => Ok(Action::Start(name.to_string())),
            "stop" => Ok(Action::Stop(name.to_string())),
            "restart" => Ok(Action::Restart(name.to_string())),
            "status" => Ok(Action::Status(name.to_string())),
            "reload" => Ok(Action::Reload),
            "shutdown" => Ok(Action::Shutdown),
            _ => Err(anyhow::anyhow!("Invalid action")),
        }
    }
}

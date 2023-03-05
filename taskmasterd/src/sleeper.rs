use std::thread::sleep;
use std::time::Duration;
use anyhow::Result;

pub struct Sleeper {
    duration: Duration,
    timestamp: Duration,
}

impl Sleeper {
    pub fn new(duration_ms: u64) -> Result<Self> {
        use std::time::{SystemTime, UNIX_EPOCH};
        Ok(Self {
            duration: Duration::from_millis(duration_ms),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?,
        })
    }

    pub fn sleep(&mut self) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now();
        let elapsed = now.duration_since(UNIX_EPOCH)?;
        let diff = elapsed - self.timestamp;
        if diff < self.duration {
            sleep(self.duration - diff);
        }
        self.timestamp = now.duration_since(UNIX_EPOCH)?;
        Ok(())
    }
}

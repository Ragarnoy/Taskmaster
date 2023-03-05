use serde::Deserialize;

/// Stop timeout in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct StopTimeout(pub u64);

/// Default stop timeout is 10 seconds
impl Default for StopTimeout {
    fn default() -> Self {
        StopTimeout(10)
    }
}

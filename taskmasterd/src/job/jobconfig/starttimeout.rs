use serde::Deserialize;

/// Start timeout in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct StartTimeout(pub u64);

/// Default start timeout is 1 second
impl Default for StartTimeout {
    fn default() -> Self {
        StartTimeout(1)
    }
}

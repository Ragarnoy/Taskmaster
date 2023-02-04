use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ExitCodes {
    pub codes: Vec<i32>,
}

impl Default for ExitCodes {
    fn default() -> Self {
        Self { codes: vec![0] }
    }
}

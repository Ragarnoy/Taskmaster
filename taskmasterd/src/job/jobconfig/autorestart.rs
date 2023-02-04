use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum AutoRestart {
    True,
    #[default]
    Unexpected,
    False,
}

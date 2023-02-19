use serde::Deserialize;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NumProcs(NonZeroU32);

impl Default for NumProcs {
    fn default() -> Self {
        Self(NonZeroU32::new(1).unwrap())
    }
}

use serde::Deserialize;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NumProcs {
    pub numprocs: NonZeroU32,
}

impl Default for NumProcs {
    fn default() -> Self {
        NumProcs {
            numprocs: NonZeroU32::new(1).unwrap(),
        }
    }
}

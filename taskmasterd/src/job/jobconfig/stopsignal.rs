use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum StopSignal {
    #[default]
    SigTerm,
    SigKill,
    SigInt,
    SigQuit,
    SigHup,
    SigUsr1,
    SigUsr2,
}

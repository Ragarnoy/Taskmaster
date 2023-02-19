use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StopSignal {
    #[default]
    Term,
    Kill,
    Int,
    Quit,
    Hup,
    Usr1,
    Usr2,
}

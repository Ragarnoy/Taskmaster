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

impl From<StopSignal> for nix::sys::signal::Signal {
    fn from(signal: StopSignal) -> Self {
        match signal {
            StopSignal::Term => nix::sys::signal::Signal::SIGTERM,
            StopSignal::Kill => nix::sys::signal::Signal::SIGKILL,
            StopSignal::Int => nix::sys::signal::Signal::SIGINT,
            StopSignal::Quit => nix::sys::signal::Signal::SIGQUIT,
            StopSignal::Hup => nix::sys::signal::Signal::SIGHUP,
            StopSignal::Usr1 => nix::sys::signal::Signal::SIGUSR1,
            StopSignal::Usr2 => nix::sys::signal::Signal::SIGUSR2,
        }
    }
}

use anyhow::{Context, Result};
use clap::*;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/taskmasterd/taskmasterd.sock";

#[derive(Parser)]
#[command(author, name = "taskmasterctl", about)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Parser)]
enum Command {
    /// Start processes
    Start {
        /// The name of the processes to start, or all if not specified
        #[clap(name = "name")]
        processes: Vec<String>,
    },
    /// Stop processes
    Stop {
        /// The name of the processes to stop, or all if not specified
        #[clap(name = "name")]
        processes: Vec<String>,
    },
    /// Restart processes
    Restart {
        /// The name of the processes to restart, or all if not specified
        #[clap(name = "name")]
        processes: Vec<String>,
    },
    /// Get the status of processes
    Status {
        /// The name of the processes to get the status of, or all if not specified
        name: Vec<String>,
    },
    /// Reload the configuration
    Reload,
    /// Shutdown the daemon
    Shutdown,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let message = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if !message.is_empty() {
        let mut unix_stream =
            UnixStream::connect(SOCKET_PATH).context("Could not create stream")?;
        write_request_and_shutdown(&mut unix_stream, message)?;
        read_from_stream(&mut unix_stream)?;
    }
    Ok(())
}

fn write_request_and_shutdown(unix_stream: &mut UnixStream, message: String) -> Result<()> {
    unix_stream
        .write_all(message.as_bytes())
        .context("Failed at writing onto the unix stream")?;
    unix_stream
        .shutdown(std::net::Shutdown::Write)
        .context("Could not shutdown writing on the stream")?;
    Ok(())
}

fn read_from_stream(unix_stream: &mut UnixStream) -> Result<()> {
    let mut response = String::new();
    unix_stream
        .read_to_string(&mut response)
        .context("Failed at reading from the unix stream")?;
    println!("response: {}", response);
    Ok(())
}

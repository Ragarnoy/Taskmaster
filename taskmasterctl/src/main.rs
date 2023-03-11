use anyhow::{Context, Result};
use clap::*;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use dirs::home_dir;

const SOCKET_PATH: &str = ".taskmasterd/taskmasterd.sock";

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
    let message = match args.command {
        Some(Command::Start { processes }) => {
            if processes.is_empty() {
                "start".to_string()
            } else {
                format!("start {}", processes.join(" "))
            }
        }
        Some(Command::Stop { processes }) => {
            if processes.is_empty() {
                "stop".to_string()
            } else {
                format!("stop {}", processes.join(" "))
            }
        }
        Some(Command::Restart { processes }) => {
            if processes.is_empty() {
                "restart".to_string()
            } else {
                format!("restart {}", processes.join(" "))
            }
        }
        Some(Command::Status { name }) => {
            if name.is_empty() {
                "status".to_string()
            } else {
                format!("status {}", name.join(" "))
            }
        }
        Some(Command::Reload) => "reload".to_string(),
        Some(Command::Shutdown) => "shutdown".to_string(),
        None => "".to_string(),
    };
    if !message.is_empty() {
        let socket_path = home_dir().context("Could not get home directory")?.join(SOCKET_PATH);
        let mut unix_stream =
            UnixStream::connect(socket_path).context("Could not create stream")?;
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
    println!("{}", response);
    Ok(())
}

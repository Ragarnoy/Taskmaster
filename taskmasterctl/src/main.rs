use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/taskmaster.sock";

fn main() -> Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH).context("Could not create stream")?;
    stream
        .write_all(b"hello")
        .context("Failed at writing onto the unix stream")?;
    let mut response = String::new();
    // stream.read_to_string(&mut response)?;
    println!("{response}");
    Ok(())
}

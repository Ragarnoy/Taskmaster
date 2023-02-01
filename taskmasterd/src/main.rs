use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const SOCKET_PATH: &str = "/tmp/taskmaster.sock";

fn main() -> Result<()> {
    if std::fs::metadata(SOCKET_PATH).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(SOCKET_PATH)
            .with_context(|| format!("could not delete previous socket at {:?}", SOCKET_PATH))?;
    }
    let unix_listener = UnixListener::bind(SOCKET_PATH)?;
    let (mut stream, _) = unix_listener
        .accept()
        .context("Failed at accepting a connection on the unix listener")?;
    let term = Arc::new(AtomicBool::new(false));
    let mut response = String::new();
    signal_hook::flag::register(signal_hook::consts::SIGHUP, Arc::clone(&term))?;
    println!("hi");
    // while !term.load(Ordering::Relaxed) {
    // std::thread::sleep(std::time::Duration::from_millis(100));
    stream.read_to_string(&mut response)?;
    eprintln!("response: {}", response);
    // if response.contains("hello") {
    stream.write_all(b"world")?;
    // break;
    // }
    // }
    println!("\nbye");
    Ok(())
}

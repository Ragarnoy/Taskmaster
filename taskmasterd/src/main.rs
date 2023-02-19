mod job;

use anyhow::{Context, Result};
use daemonize_me::Daemon;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const FILES_DIR: &str = "/tmp/taskmasterd";
const SOCKET_PATH: &str = "/tmp/taskmasterd/taskmasterd.sock";
// const PID_FILE: &str = "/tmp/taskmasterd/taskmasterd.pid";
const STDOUT_FILE: &str = "/tmp/taskmasterd/taskmasterd.stdout";
const STDERR_FILE: &str = "/tmp/taskmasterd/taskmasterd.stderr";

fn daemon_loop() -> Result<()> {
    if std::fs::metadata(SOCKET_PATH).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(SOCKET_PATH)
            .with_context(|| format!("could not delete previous socket at {:?}", SOCKET_PATH))?;
    }
    let unix_listener = UnixListener::bind(SOCKET_PATH)?;
    unix_listener
        .set_nonblocking(true)
        .context("could not set non-blocking mode on socket")?;
    let term = Arc::new(AtomicBool::new(false));
    let mut response = String::new();
    signal_hook::flag::register(signal_hook::consts::SIGHUP, Arc::clone(&term))?;
    while !term.load(Ordering::Relaxed) {
        if let Ok((mut stream, _)) = unix_listener.accept() {
            stream
                .read_to_string(&mut response)
                .context("Failed at reading the request")?;
            eprintln!("message: {}", response);
            stream
                .write_all(format!("ack {}", response).as_bytes())
                .context("Failed at writing the response")?;
            if response == "quit" {
                break;
            }
            response.clear();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

fn daemon_hook(_ppid: i32, _cpid: i32) {
    if let Err(e) = daemon_loop() {
        eprintln!("{}", e);
    }
}

fn main() -> Result<()> {
    // create a directory for the tmp files if it doesn't exist
    std::fs::create_dir_all(FILES_DIR).context("could not create files directory")?;
    let stdout = std::fs::File::create(STDOUT_FILE).context("could not create stdout file")?;
    let stderr = std::fs::File::create(STDERR_FILE).context("could not create stderr file")?;
    Daemon::new()
        .umask(0o777)
        //.pid_file(PID_FILE, None)
        .stdout(stdout)
        .stderr(stderr)
        .work_dir("/")
        .name(std::ffi::OsStr::new("taskmasterd"))
        .setup_post_fork_child_hook(daemon_hook)
        .start()
        .context("Failed at daemonizing")?;
    Ok(())
}

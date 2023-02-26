use crate::main_loop;
use crate::FILES_DIR;
use anyhow::{Context, Result};
use daemonize_me::Daemon;

// const PID_FILE: &str = "/tmp/taskmasterd/taskmasterd.pid";
const STDOUT_FILE: &str = "/tmp/taskmasterd/taskmasterd.stdout";
const STDERR_FILE: &str = "/tmp/taskmasterd/taskmasterd.stderr";

fn hook(_ppid: i32, _cpid: i32) {
    if let Err(e) = main_loop() {
        eprintln!("{}", e);
    }
}

pub fn init() -> Result<()> {
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
        .setup_post_fork_child_hook(hook)
        .start()
        .context("Failed at daemonizing")?;
    Ok(())
}

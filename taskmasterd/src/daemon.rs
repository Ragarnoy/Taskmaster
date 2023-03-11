use crate::main_loop;
use anyhow::{Context, Result};
use daemonize_me::Daemon;
use dirs::home_dir;

const STDOUT_FILE: &str = ".taskmasterd/taskmasterd.stdout";
const STDERR_FILE: &str = ".taskmasterd/taskmasterd.stderr";

fn hook(_ppid: i32, _cpid: i32) {
    if let Err(e) = main_loop() {
        eprintln!("{}", e);
    }
}

pub fn init() -> Result<()> {
    let stdout_path = home_dir()?.join(STDOUT_FILE);
    let stderr_path = home_dir()?.join(STDERR_FILE);
    let stdout = std::fs::File::create(stdout_path).context("could not create stdout file")?;
    let stderr = std::fs::File::create(stderr_path).context("could not create stderr file")?;
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

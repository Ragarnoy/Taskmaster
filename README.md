# Taskmaster

[![Rust](https://github.com/Ragarnoy/Taskmaster/actions/workflows/rust.yml/badge.svg)](https://github.com/Ragarnoy/Taskmaster/actions/workflows/rust.yml)

Taskmaster is a lightweight, Rust-based process manager similar to Supervisor. It is designed to simplify the process of managing processes and ensure they run reliably.

This is a School 42 project.

## Prerequisites

Before you can use Taskmaster, you will need to install Rust's toolchain on your machine.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Running Taskmaster

Once you have installed Rust, you can run the Taskmaster dæmon by running the following command:

```
cargo run
```

and the client by running the following command:

```
cargo run --package taskmasterctl
```

## Usage
```
Usage: taskmasterd [OPTIONS]

Options:
  -n, --nodaemon
  -h, --help      Print help
  -V, --version   Print version
```

```
Usage: taskmasterctl [COMMAND]

Commands:
  start     Start processes
  stop      Stop processes
  restart   Restart processes
  status    Get the status of processes
  load      Load a configuration file
  reload    Reload the configuration
  shutdown  Shutdown the daemon
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Configuration

```yaml
programs:
  job_name:
    cmd: "command"
    numprocs: number of processes to start in parallel
    umask: umask to apply to the process (octal value like 077)
    workingdir: working directory for the processes
    autostart: true/false whether to start the program on startup
    autorestart: unexpected/always/never whether to restart the program on exit
    exitcodes: list of exit codes that should be considered as normal
    startretries: number of times to retry starting the program before giving up if it exits before being fully started
    starttime: time to wait before considering the program as fully started
    stopsignal: signal to send to the program to stop it (TERM, INT, KILL, etc.)
    stoptime: time to wait before sending a KILL signal to the program after sending the stop signal
    stdout: path to the file to redirect stdout to
    stderr: path to the file to redirect stderr to
    env: environment variables to set for the program
```

See the [example configuration file](./config.yml) for a more detailed example.

## Authors

- [Tiago Lernould](https://github.com/Ragarnoy)
- [Youva Gaudé](https://github.com/Eviber)

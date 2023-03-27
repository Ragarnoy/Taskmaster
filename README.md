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

```bash
cargo run
```

and the client by running the following command:

```bash
cargo run --package taskmasterctl
```

## Usage
```bash
Usage: taskmasterd [OPTIONS]

Options:
  -n, --nodaemon
  -h, --help      Print help
  -V, --version   Print version
```

```bash
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

## Authors

- Tiago Lernould (@Ragarnoy)
- Youva Gaudé (@Eviber)

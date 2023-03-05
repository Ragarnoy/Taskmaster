use crate::SOCKET_PATH;
use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

pub struct Socket {
    listener: UnixListener,
}

impl Socket {
    pub fn new(path: &str) -> Result<Self> {
        if fs::metadata(path).is_ok() {
            eprintln!("A socket is already present. Deleting...");
            fs::remove_file(path)
                .with_context(|| format!("could not delete previous socket at {:?}", path))?;
        }
        let listener = UnixListener::bind(path)
            .with_context(|| format!("could not bind to socket at {:?}", path))?;
        listener
            .set_nonblocking(true)
            .context("could not set non-blocking mode on socket")?;
        Ok(Self { listener })
    }

    pub fn read(&self, response: &mut String) -> Result<Option<UnixStream>> {
        if let Ok((mut stream, _)) = self.listener.accept() {
            stream
                .read_to_string(response)
                .context("Failed at reading the request")?;
            Ok(Some(stream))
        } else {
            Ok(None)
        }
    }

    pub fn write(&self, response: &str, mut stream: UnixStream) -> Result<()> {
        stream
            .write_all(response.as_bytes())
            .context("Failed at writing the response")?;
        Ok(())
    }
}

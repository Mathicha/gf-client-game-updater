use std::{
    io::Error,
    pin::Pin,
    task::{Context, Poll},
};

use sha1::{Digest, Sha1};
use tokio::{
    fs::File,
    io::{self, AsyncWrite, BufReader},
};

/// async sha1 calc
pub async fn calc_sha(file: File) -> Result<String, Box<dyn std::error::Error>> {
    let mut read = BufReader::new(file);
    let mut write = Writer::new();
    io::copy(&mut read, &mut write).await?;

    Ok(write.finalize())
}

pub struct Writer {
    sha1: Sha1,
}

impl Default for Writer {
    fn default() -> Self {
        Self { sha1: Sha1::new() }
    }
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finalize(self) -> String {
        let hash = self.sha1.finalize();
        format!("{:x}", hash)
    }
}

impl AsyncWrite for Writer {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        let s = self.get_mut();
        s.sha1.update(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

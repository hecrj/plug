use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io;
use tokio::net;

use std::fmt;
use std::marker::PhantomData;

/// A type-safe TCP socket connection.
///
///
/// [`write`](Self::write) accepts input of type `I`.
/// [`read`](Self::read) will produce an output of type `O`.
pub struct Connection<I, O> {
    stream: net::TcpStream,
    buffer: Vec<u8>,
    _types: PhantomData<(I, O)>,
}

impl<I, O> Connection<I, O> {
    pub(crate) fn new(stream: net::TcpStream, buffer: Vec<u8>) -> Self {
        Self {
            stream,
            buffer,
            _types: PhantomData,
        }
    }

    /// Creates a [`Connection`] that will take over the given [`net::TcpStream`].
    ///
    /// Unless you know what you are doing, use a [`Plug`](crate::Plug) instead!
    pub fn seize(stream: net::TcpStream) -> Self {
        Self::new(stream, Vec::new())
    }

    /// Writes the given input to the [`Connection`].
    pub async fn write(&mut self, input: I) -> io::Result<()>
    where
        I: Serialize,
    {
        write_json(&mut self.stream, input).await
    }

    /// Writes raw bytes to the [`Connection`].
    pub async fn write_bytes(&mut self, input: &[u8]) -> io::Result<()>
    where
        I: Serialize,
    {
        write(&mut self.stream, input).await
    }

    /// Reads some output from the [`Connection`].
    pub async fn read(&mut self) -> io::Result<O>
    where
        O: DeserializeOwned,
    {
        read_json(&mut self.stream, &mut self.buffer).await
    }

    /// Reads raw bytes from the [`Connection`].
    pub async fn read_bytes(&mut self) -> io::Result<&[u8]> {
        let n = read(&mut self.stream, &mut self.buffer).await?;

        Ok(&self.buffer[..n])
    }

    /// Copies all the output of the [`Connection`] to the provided one, effectively
    /// creating a proxy.
    pub async fn copy<T>(&mut self, to: &mut Connection<T, O>) -> io::Result<u64> {
        io::copy(&mut self.stream, &mut to.stream).await
    }

    /// Connects this [`Connection`] with another, copying in both directions.
    pub async fn connect<T>(&mut self, with: &mut Connection<T, O>) -> io::Result<(u64, u64)> {
        io::copy_bidirectional(&mut self.stream, &mut with.stream).await
    }
}

impl<I, O> fmt::Debug for Connection<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection")
            .field("stream", &self.stream)
            .field("buffer", &format!("{} bytes", self.buffer.len()))
            .finish()
    }
}

async fn read(stream: &mut net::TcpStream, buffer: &mut Vec<u8>) -> io::Result<usize> {
    use tokio::io::AsyncReadExt;

    let message_size = stream.read_u64().await? as usize;

    if buffer.len() < message_size {
        buffer.resize(message_size, 0);
    }

    stream.read_exact(&mut buffer[..message_size]).await
}

async fn write(stream: &mut net::TcpStream, bytes: &[u8]) -> io::Result<()> {
    use tokio::io::AsyncWriteExt;

    stream.write_u64(bytes.len() as u64).await?;
    stream.write_all(bytes).await?;
    stream.flush().await?;

    Ok(())
}

pub async fn read_json<T: DeserializeOwned>(
    stream: &mut net::TcpStream,
    buffer: &mut Vec<u8>,
) -> io::Result<T> {
    let message_size = read(stream, buffer).await?;
    let data = serde_json::from_reader(&buffer[..message_size])?;

    Ok(data)
}

pub async fn write_json<T: Serialize>(stream: &mut net::TcpStream, data: T) -> io::Result<()> {
    let bytes = serde_json::to_vec(&data)?;

    write(stream, &bytes).await
}

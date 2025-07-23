use crate::connection::{self, Connection};

use tokio::io;
use tokio::net;

use std::marker::PhantomData;

/// A type-safe connection endpoint with a name.
#[derive(Debug, Clone, Copy)]
pub struct Plug<I, O> {
    pub(crate) name: &'static str,
    _types: PhantomData<(I, O)>,
}

impl<I, O> Plug<I, O> {
    /// Creates a new [`Plug`] with the given name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _types: PhantomData,
        }
    }

    /// Connects to the [`Plug`] in the given server address, producing a type-safe [`Connection`].
    pub async fn connect(self, address: impl net::ToSocketAddrs) -> io::Result<Connection<I, O>> {
        let stream = net::TcpStream::connect(address).await?;

        self.seize(stream).await
    }

    /// Seizes an existing TCP connection and redirects it through the [`Plug`].
    pub async fn seize(self, mut stream: net::TcpStream) -> io::Result<Connection<I, O>> {
        connection::write_json(&mut stream, self.name).await?;

        Ok(Connection::seize(stream))
    }
}

use crate::connection;
use crate::{Connection, Plug};

use tokio::io;
use tokio::net;

use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;

/// A set of plugs that can be used to accept incoming connections.
///
/// This serves as a "plug router" in your server implementation.
#[derive(Debug)]
pub struct Strip<E> {
    plugs: HashMap<&'static str, Handler<E>>,
}

impl<E> Strip<E> {
    /// Creates a new empty [`Strip`].
    pub fn new() -> Self {
        Self {
            plugs: HashMap::new(),
        }
    }

    /// Adds a new [`Plug`] to the [`Strip`] with the given [`Connection`] handler.
    pub fn plug<I, O, F>(
        mut self,
        plug: Plug<I, O>,
        handler: impl Fn(Connection<O, I>) -> F + Send + Sync + 'static,
    ) -> Self
    where
        F: Future<Output = Result<(), E>> + Send + 'static,
    {
        let _ = self.plugs.insert(
            plug.name,
            Handler(Box::new(move |client, buffer| {
                Box::pin(handler(Connection::new(client, buffer)))
            })),
        );

        self
    }

    /// Serves a new connection by attaching it to the [`Strip`].
    ///
    /// It will route the connection to the proper [`Plug`] and [`Connection`] handler
    /// automatically.
    pub async fn attach(&self, mut client: net::TcpStream) -> Result<(), E>
    where
        E: From<io::Error>,
    {
        let mut buffer = Vec::new();

        let route: String = connection::read_json(&mut client, &mut buffer).await?;

        if let Some(handler) = self.plugs.get(route.as_str()) {
            (handler.0)(client, buffer).await?;
        }

        Ok(())
    }
}

impl<E> Default for Strip<E> {
    fn default() -> Self {
        Self::new()
    }
}

struct Handler<E>(Box<dyn Fn(net::TcpStream, Vec<u8>) -> BoxFuture<Result<(), E>> + Send + Sync>);

impl<E> fmt::Debug for Handler<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handler").finish()
    }
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

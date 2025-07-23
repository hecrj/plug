//! A library for type-safe interprocess communication.
//!
//! Define the input and output of a wired connection:
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct Credentials {
//!     username: String,
//!     password: String,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! pub enum Authentication {
//!     Success { token: String },
//!     Error { description: String },
//! }
//! ```
//!
//! Create a [`Plug`] for this specific action:
//!
//! ```
//! # use serde::{Deserialize, Serialize};
//! #
//! # #[derive(Serialize, Deserialize)]
//! # pub struct Credentials {
//! #     username: String,
//! #     password: String,
//! # }
//! #
//! # #[derive(Serialize, Deserialize)]
//! # pub enum Authentication {
//! #     Success { token: String },
//! #     Error { description: String },
//! # }
//! use plug::Plug;
//!
//! pub const LOG_IN: Plug<Credentials, Authentication> = Plug::new("log_in");
//! ```
//!
//! Connect the [`Plug`] to the server leveraging type-safety:
//!
//! ```
//! # use serde::{Deserialize, Serialize};
//! #
//! # #[derive(Serialize, Deserialize)]
//! # pub struct Credentials {
//! #     username: String,
//! #     password: String,
//! # }
//! #
//! # #[derive(Serialize, Deserialize)]
//! # pub enum Authentication {
//! #     Success { token: String },
//! #     Error { description: String },
//! # }
//! # use plug::Plug;
//! #
//! # pub const LOG_IN: Plug<Credentials, Authentication> = Plug::new("log_in");
//! #
//! use std::io;
//!
//! async fn log_in(credentials: Credentials) -> io::Result<Authentication> {
//!     let mut connection = LOG_IN.connect("127.0.0.1:1234").await?;
//!
//!     connection.write(credentials).await?;
//!     connection.read().await
//! }
//! ```
//!
//! Implement the server by gathering all the [`Plug`] definitions in a [`Strip`]:
//!
//! ```
//! # use serde::{Deserialize, Serialize};
//! #
//! # #[derive(Serialize, Deserialize)]
//! # pub struct Credentials {
//! #     username: String,
//! #     password: String,
//! # }
//! #
//! # #[derive(Serialize, Deserialize)]
//! # pub enum Authentication {
//! #     Success { token: String },
//! #     Error { description: String },
//! # }
//! # use plug::Plug;
//! #
//! # pub const LOG_IN: Plug<Credentials, Authentication> = Plug::new("log_in");
//! #
//! use plug::{Connection, Strip};
//!
//! use std::io;
//! use tokio::net;
//! use tokio::task;
//!
//! async fn run_server() -> io::Result<()> {
//!     let strip = Strip::new().plug(LOG_IN, log_in);
//!     let server = net::TcpListener::bind("127.0.0.1:1234").await?;
//!
//!     loop {
//!         let Ok((client, _address)) = server.accept().await else {
//!             continue;
//!         };
//!
//!         let _ = strip.attach(client).await;
//!     }
//!
//!     Ok(())
//! }
//!
//! async fn log_in(mut connection: Connection<Authentication, Credentials>) -> io::Result<()> {
//!     let credentials = connection.read().await?;
//!
//!     let result = if credentials.username == "admin" && credentials.password == "1234" {
//!         Authentication::Success { token: "verysecure".to_owned() }
//!     } else {
//!         Authentication::Error { description: "Invalid credentials!".to_owned() }
//!     };
//!
//!     connection.write(result).await
//! }
//! ```
mod connection;
mod plug;
mod strip;

pub use connection::Connection;
pub use plug::Plug;
pub use strip::Strip;

pub use std::convert::Infallible as Never;

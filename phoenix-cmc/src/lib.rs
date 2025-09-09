#![deny(rust_2018_idioms, nonstandard_style, unused_must_use, clippy::all)]
#![warn(future_incompatible, missing_docs, clippy::pedantic, clippy::cargo)]

pub mod constants;
pub mod discussion;
pub mod name;
pub mod output;
pub mod sendlist;
pub mod server;
pub mod session;
pub mod telnet;
pub mod text;
pub mod timestamp;
pub mod types;
pub mod user;

pub use server::Server;
pub use types::*;

/// Phoenix server version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

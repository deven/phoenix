pub mod constants;
pub mod discussion;
pub mod event;
pub mod name;
pub mod output;
pub mod sendlist;
pub mod server;
pub mod session;
pub mod telnet;
pub mod timestamp;
pub mod types;
pub mod user;

pub use server::PhoenixServer;
pub use types::*;

/// Phoenix server version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

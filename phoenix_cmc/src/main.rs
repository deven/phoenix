// -*- Rust -*-
//
// $Id$
//
// Main program.
//
// Copyright 2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

use std::error::Error;
use std::fmt;
use std::io;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[derive(Debug, StructOpt)]
struct Opts {
    /// Running from cron to restart server
    #[structopt(long)]
    cron: bool,

    /// Enable debug mode
    #[structopt(long)]
    _debug: bool,

    /// Use IPv6 instead of IPv4
    #[structopt(long)]
    _ipv6: bool,

    /// Set listening port number
    #[structopt(long, default_value = "9999")]
    port: u16,
}

#[derive(Debug)]
pub enum AppError {
    SocketReadError { addr: SocketAddr, source: io::Error },
    SocketWriteError { addr: SocketAddr, source: io::Error },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SocketReadError { addr, source } => write!(
                f,
                "failed to read from socket ({:?}); err = {:?}",
                addr, source
            ),
            Self::SocketWriteError { addr, source } => write!(
                f,
                "failed to write to socket ({:?}); err = {:?}",
                addr, source
            ),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SocketReadError { addr: _, source } => Some(source),
            Self::SocketWriteError { addr: _, source } => Some(source),
        }
    }
}

/// Shared state between async tasks.
struct SharedState {}

impl SharedState {
    /// Create a new, empty, instance of `SharedState`.
    fn new() -> Self {
        SharedState {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("phoenix_cmc=info".parse()?))
        .with_span_events(FmtSpan::FULL)
        .init();

    let state = Arc::new(Mutex::new(SharedState::new()));
    let opts = Opts::from_args();
    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), opts.port);

    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            if opts.cron && e.kind() == ErrorKind::AddrInUse {
                return Ok(());
            } else {
                error!("Error binding to TCP port {}: {:?}", opts.port, e);
                return Err(Box::new(e) as Box<dyn Error>);
            }
        }
    };

    info!(
        "Phoenix CMC running, accepting connections on port {}.",
        opts.port
    );

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("Accepted TCP connection from {:?}", addr);

                let state = Arc::clone(&state);

                tokio::spawn(async move {
                    if let Err(e) = process(socket, addr, state).await {
                        warn!("Error processing TCP connection from {:?}: {:?}", addr, e);
                    }
                });
            }
            Err(e) => warn!("Error accepting TCP connection: {:?}", e),
        }
    }
}

/// Process an individual TCP connection.
async fn process(
    mut socket: TcpStream,
    addr: SocketAddr,
    _state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        let n = match socket.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => return Ok(()),
            Ok(n) => n,
            Err(e) => {
                return Err(Box::new(AppError::SocketReadError {
                    addr: addr,
                    source: e,
                }));
            }
        };

        // Write the data back
        if let Err(e) = socket.write_all(&buf[0..n]).await {
            return Err(Box::new(AppError::SocketWriteError {
                addr: addr,
                source: e,
            }));
        }
    }
}

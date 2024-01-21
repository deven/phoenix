// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: server module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::Client;
use crate::Options;
use async_backtrace::framed;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[derive(Debug)]
pub struct Server {
    pub cron: bool,
    pub port: u16,
}

impl Server {
    pub fn new(opts: Options) -> Self {
        Self {
            cron: opts.cron,
            port: opts.port,
        }
    }

    #[framed]
    pub async fn run(&self) -> Result<(), ServerError> {
        let port = self.port;
        let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

        let listener = match TcpListener::bind(socket).await {
            Ok(listener) => listener,
            Err(err) => {
                if self.cron && err.kind() == ErrorKind::AddrInUse {
                    return Ok(());
                } else {
                    return Err(ServerError::new_bind_error(port, err));
                }
            }
        };

        info!("Phoenix CMC running, accepting connections on port {port}.");

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Accepted TCP connection from {addr:?}");
                    let _client = Client::new(addr, stream);
                }
                Err(err) => warn!("Error accepting TCP connection: {err:?}"),
            }
        }
    }
}

#[derive(Debug)]
pub enum ServerError {
    BindError { port: u16, source: IoError },
}

impl ServerError {
    fn new_bind_error(port: u16, source: IoError) -> Self {
        Self::BindError { port, source }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BindError { source, .. } => source.source(),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BindError { port, source } => {
                write!(f, "Error binding to TCP port {port}: {source}")
            }
        }
    }
}

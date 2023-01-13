// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: crate root
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

mod client;

use crate::client::{setup_client, Client};
use async_backtrace::frame;
use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[derive(Debug, Parser)]
pub struct Options {
    /// Running from cron to restart server
    #[arg(long)]
    pub cron: bool,

    /// Enable debug mode
    #[arg(long)]
    pub debug: bool,

    /// Use IPv6 instead of IPv4
    #[arg(long)]
    pub ipv6: bool,

    /// Set listening port number
    #[arg(long, default_value = "9999")]
    pub port: u16,
}

/// Shared state between async tasks.
pub struct SharedState {
    clients: HashMap<SocketAddr, Arc<Mutex<Client>>>,
}

impl SharedState {
    /// Create a new instance of `SharedState`.
    fn new() -> Self {
        SharedState {
            clients: HashMap::new(),
        }
    }
}

#[tokio::main]
pub async fn run(options: Options) -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("phoenix_cmc=trace".parse()?))
        .with_span_events(FmtSpan::FULL)
        .init();

    let state = Arc::new(Mutex::new(SharedState::new()));
    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), options.port);

    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            if options.cron && e.kind() == ErrorKind::AddrInUse {
                return Ok(());
            } else {
                error!("Error binding to TCP port {}: {:?}", options.port, e);
                return Err(Box::new(e) as Box<dyn Error>);
            }
        }
    };

    info!(
        "Phoenix CMC running, accepting connections on port {}.",
        options.port
    );

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Accepted TCP connection from {:?}", addr);

                let state = Arc::clone(&state);

                tokio::spawn(frame!(async move {
                    if let Err(e) = setup_client(addr, stream, state).await {
                        warn!("Error processing TCP connection from {:?}: {:?}", addr, e);
                    }
                }));
            }
            Err(e) => warn!("Error accepting TCP connection: {:?}", e),
        }
    }
}

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

use crate::client::Client;
use async_backtrace::frame;
use clap::Parser;
use std::error::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tracing::{error, info, warn};

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

#[tokio::main]
pub async fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let port = opts.port;
    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            if opts.cron && e.kind() == ErrorKind::AddrInUse {
                return Ok(());
            } else {
                error!("Error binding to TCP port {port}: {e:?}");
                return Err(Box::new(e) as Box<dyn Error>);
            }
        }
    };

    info!("Phoenix CMC running, accepting connections on port {port}.");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Accepted TCP connection from {addr:?}");

                let mut client = Client::new(addr, stream).await;

                tokio::spawn(frame!(async move {
                    if let Err(e) = client.setup().await {
                        warn!("Error processing TCP connection from {addr:?}: {e:?}");
                    }
                }));
            }
            Err(e) => warn!("Error accepting TCP connection: {e:?}"),
        }
    }
}

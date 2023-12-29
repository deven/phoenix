// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: server module
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::Client;
use crate::Options;
use async_backtrace::{frame, framed};
use std::error::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

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
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let port = self.port;
        let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

        let listener = match TcpListener::bind(socket).await {
            Ok(listener) => listener,
            Err(e) => {
                if self.cron && e.kind() == ErrorKind::AddrInUse {
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

                    let (_sender, receiver) = mpsc::channel(8);
                    let mut client = Client::new(addr, stream, receiver);

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
}
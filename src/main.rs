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
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use structopt::StructOpt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{warn, info};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[derive(Debug, StructOpt)]
struct Opts {
    /// Running from cron to restart server
    #[structopt(long)]
    cron: bool,

    /// Enable debug mode
    #[structopt(long)]
    debug: bool,

    /// Use IPv6 instead of IPv4
    #[structopt(long)]
    ipv6: bool,

    /// Set listening port number
    #[structopt(long, default_value = "9999")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("phoenix_cmc=info".parse()?))
        .with_span_events(FmtSpan::FULL)
        .init();

    let opts     = Opts::from_args();
    let socket   = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), opts.port);
    let listener = TcpListener::bind(socket).await?;

    info!("Phoenix CMC running, accepting connections on port {}.", opts.port);

    loop {
        let (mut socket, _) = listener.accept().await?;

        info!("Accepted connection from {:?}", socket);

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            info!("Spawned a new async task.");

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        warn!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    warn!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

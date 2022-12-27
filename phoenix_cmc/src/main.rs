// -*- Rust -*-
//
// $Id$
//
// Main program.
//
// Copyright 2021-2022 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

use async_backtrace::{frame, framed, taskdump_tree};
use clap::Parser;
use futures::SinkExt;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, trace, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[derive(Debug, Parser)]
struct Opts {
    /// Running from cron to restart server
    #[arg(long)]
    cron: bool,

    /// Enable debug mode
    #[arg(long)]
    _debug: bool,

    /// Use IPv6 instead of IPv4
    #[arg(long)]
    _ipv6: bool,

    /// Set listening port number
    #[arg(long, default_value = "9999")]
    port: u16,
}

/// Shared state between async tasks.
struct SharedState {
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

struct Client {
    addr: SocketAddr,
    lines: Framed<TcpStream, LinesCodec>,
    sender: UnboundedSender<String>,
    receiver: UnboundedReceiver<String>,
}

impl Client {
    /// Create a new instance of `Client`.
    async fn new(
        addr: SocketAddr,
        lines: Framed<TcpStream, LinesCodec>,
        state: Arc<Mutex<SharedState>>,
    ) -> io::Result<Arc<Mutex<Client>>> {
        // Create a channel for sending events to this client.
        let (sender, receiver) = unbounded_channel();

        // Create the new `Client` instance.
        let client = Arc::new(Mutex::new(Client {
            addr,
            lines,
            sender,
            receiver,
        }));

        // Save the new instance in the server shared state HashMap.
        state.lock().await.clients.insert(addr, client.clone());

        // Return the new instance.
        Ok(client)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("phoenix_cmc=trace".parse()?))
        .with_span_events(FmtSpan::FULL)
        .init();

    let state = Arc::new(Mutex::new(SharedState::new()));
    let opts = Opts::parse();
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

/// Setup a new client connection.
#[framed]
async fn setup_client(
    _addr: SocketAddr,
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    client_loop(&mut lines, state).await?;
    Ok(())
}

/// Client main loop.
#[framed]
async fn client_loop(
    lines: &mut Framed<TcpStream, LinesCodec>,
    _state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn Error>> {
    trace!("{}", taskdump_tree(false));

    // In a loop, read lines from the socket and write them back.
    loop {
        let input = match lines.next().await {
            Some(Ok(line)) => line,
            _ => return Ok(()),
        };
        lines.send(input).await?;
    }
}

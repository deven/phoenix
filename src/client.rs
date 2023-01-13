// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: client module
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::SharedState;
use async_backtrace::{framed, taskdump_tree};
use futures::SinkExt;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, trace};

pub struct Client {
    username: String,
    addr: SocketAddr,
    lines: Framed<TcpStream, LinesCodec>,
    sender: UnboundedSender<String>,
    receiver: UnboundedReceiver<String>,
}

impl Client {
    /// Create a new instance of `Client`.
    async fn new(
        username: String,
        addr: SocketAddr,
        lines: Framed<TcpStream, LinesCodec>,
        state: Arc<Mutex<SharedState>>,
    ) -> io::Result<Arc<Mutex<Client>>> {
        // Create a channel for sending events to this client.
        let (sender, receiver) = unbounded_channel();

        // Create the new `Client` instance.
        let client = Arc::new(Mutex::new(Client {
            username,
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

/// Setup a new client connection.
#[framed]
pub async fn setup_client(
    addr: SocketAddr,
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    {
        let stream = lines.get_mut();
        stream.write_all(b"Enter username: ").await?;
    }

    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            info!(
                "Client disconnected from {} without sending a username.",
                addr
            );
            return Ok(());
        }
    };

    info!("User \"{}\" logged in from {}.", username, addr);

    client_loop(&mut lines, state).await?;

    info!("User \"{}\" disconnected from {}.", username, addr);

    Ok(())
}

/// Client main loop.
#[framed]
async fn client_loop(
    lines: &mut Framed<TcpStream, LinesCodec>,
    state: Arc<Mutex<SharedState>>,
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
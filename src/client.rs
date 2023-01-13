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

use async_backtrace::{framed, taskdump_tree};
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, trace};

pub struct Client {
    pub username: Option<String>,
    pub addr: SocketAddr,
    lines: Framed<TcpStream, LinesCodec>,
    _sender: UnboundedSender<String>,
    _receiver: UnboundedReceiver<String>,
}

impl Client {
    /// Create a new instance of `Client`.
    #[framed]
    pub async fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        // Create a LinesCodec to encode the stream as lines.
        let lines = Framed::new(stream, LinesCodec::new());

        // Create a channel for sending events to this client.
        let (_sender, _receiver) = unbounded_channel();

        // Create the new `Client` instance.
        Self {
            username: None,
            addr,
            lines,
            _sender,
            _receiver,
        }
    }

    /// Setup a new client connection.
    #[framed]
    pub async fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let stream = self.lines.get_mut();
            stream.write_all(b"Enter username: ").await?;
        }

        let addr = &self.addr;
        let username = match self.lines.next().await {
            Some(Ok(line)) => line,
            _ => {
                info!("Client disconnected from {addr} without sending a username.");
                return Ok(());
            }
        };

        info!("User \"{username}\" logged in from {addr}.");

        self.client_loop().await?;

        let addr = &self.addr;
        info!("User \"{username}\" disconnected from {addr}.");

        Ok(())
    }

    /// Client main loop.
    #[framed]
    async fn client_loop(&mut self) -> Result<(), Box<dyn Error>> {
        trace!("{}", taskdump_tree(false));

        // In a loop, read lines from the socket and write them back.
        loop {
            let input = match self.lines.next().await {
                Some(Ok(line)) => line,
                _ => return Ok(()),
            };
            self.lines.send(input).await?;
        }
    }
}

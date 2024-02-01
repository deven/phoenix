// -*- Rust -*-
//
// Phoenix CMC library: client module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::server::Server;
use crate::session::Session;
use async_backtrace::{frame, framed, taskdump_tree};
use futures::SinkExt;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{info, trace, warn};

/// Client handle.
#[derive(Debug, Clone)]
pub struct Client(Arc<RwLock<ClientInner>>);

#[derive(Debug, Clone)]
pub struct ClientInner {
    pub server: Server,
    pub addr: SocketAddr,
    pub session: Option<Session>,
    pub task: Option<Arc<JoinHandle<()>>>,
}

impl Client {
    /// Create a new instance of `Client`.
    pub fn new(server: Server, addr: SocketAddr) -> Self {
        let inner = ClientInner {
            server,
            addr,
            session: None,
            task: None,
        };

        Client(Arc::new(RwLock::new(inner)))
    }

    /// Run async task for `Client`.
    pub async fn run(&mut self, stream: TcpStream) -> Result<(), ClientError> {
        // Create a LinesCodec to encode the stream as lines.
        let lines = Framed::new(stream, LinesCodec::new());

        let mut client = self.clone();

        // Spawn separate async task to manage the TCP connection.
        let task = tokio::spawn(frame!(async move {
            if let Err(e) = client.setup(lines).await {
                let addr = client.addr().await;
                warn!("Error processing TCP connection from {addr:?}: {e:?}");
            }
        }));

        self.set_task(Some(Arc::new(task))).await;

        Ok(())
    }

    /// Setup a new client connection.
    #[framed]
    pub async fn setup(
        &mut self,
        mut lines: Framed<TcpStream, LinesCodec>,
    ) -> Result<(), ClientError> {
        let server = self.server().await;

        {
            let stream = lines.get_mut();
            stream.write_all(b"Enter username: ").await?;
        }

        let addr = self.addr().await;
        let username = match lines.next().await {
            Some(Ok(line)) => line,
            _ => {
                info!("Client disconnected from {addr} without sending a username.");
                return Ok(());
            }
        };

        info!("User \"{username}\" logged in from {addr}.");

        let session = Session::new(username);
        self.set_session(Some(session.clone())).await;
        server.push_session(session).await;

        self.client_loop(lines).await?;

        if let Some(session) = self.session().await {
            let username = session.username().await;
            info!("User \"{username}\" disconnected from {addr}.");
        } else {
            info!("Unknown client disconnected from {addr}.");
        }

        Ok(())
    }

    /// Client main loop.
    #[framed]
    pub async fn client_loop(
        &mut self,
        mut lines: Framed<TcpStream, LinesCodec>,
    ) -> Result<(), ClientError> {
        trace!("{}", taskdump_tree(false));

        // In a loop, read lines from the socket and write them back.
        loop {
            let input = match lines.next().await {
                Some(Ok(line)) => line,
                Some(Err(e)) => return Err(e.into()),
                None => return Ok(()),
            };

            if let Some(session) = self.session().await {
                let username = session.username().await;
                let msg = format!("{username}: {input}");
                lines.send(msg).await?;
            } else {
                lines.send(format!("Error: You are not logged in!")).await?;
            }
        }
    }

    /// Disconnect client after login timeout.
    #[framed]
    pub async fn login_timeout(&self) {
        // TODO: Send timeout message and disconnect.
        todo!("Login timeout not implemented.");
        //telnet->output("\nLogin timed out!\n");
        //telnet->Close();
    }

    /// Obtain read lock on the client data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, ClientInner> {
        self.0.read().await
    }

    /// Obtain write lock on the client data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, ClientInner> {
        self.0.write().await
    }

    /// Get server.
    #[framed]
    pub async fn server(&self) -> Server {
        self.read().await.server.clone()
    }

    /// Set server.
    #[framed]
    pub async fn set_server(&self, server: Server) {
        self.write().await.server = server;
    }

    /// Get socket address.
    #[framed]
    pub async fn addr(&self) -> SocketAddr {
        self.read().await.addr.clone()
    }

    /// Set socket address.
    #[framed]
    pub async fn set_addr(&self, addr: SocketAddr) {
        self.write().await.addr = addr;
    }

    /// Get session.
    #[framed]
    pub async fn session(&self) -> Option<Session> {
        self.read().await.session.clone()
    }

    /// Set session.
    #[framed]
    pub async fn set_session(&self, session: Option<Session>) {
        self.write().await.session = session;
    }

    /// Get task join handle.
    #[framed]
    pub async fn task(&self) -> Option<Arc<JoinHandle<()>>> {
        self.read().await.task.clone()
    }

    /// Set task join handle.
    #[framed]
    pub async fn set_task(&self, task: Option<Arc<JoinHandle<()>>>) {
        self.write().await.task = task;
    }
}

#[derive(Debug)]
pub enum ClientError {
    IoError(IoError),
    LinesCodecError(LinesCodecError),
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IoError(err) => err.source(),
            Self::LinesCodecError(err) => err.source(),
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "I/O error: {err}"),
            Self::LinesCodecError(err) => write!(f, "LinesCodec error: {err}"),
        }
    }
}

impl From<IoError> for ClientError {
    fn from(err: IoError) -> Self {
        Self::IoError(err)
    }
}

impl From<LinesCodecError> for ClientError {
    fn from(err: LinesCodecError) -> Self {
        Self::LinesCodecError(err)
    }
}

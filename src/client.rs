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

pub mod session;

use crate::actor::{Actor, ActorInner};
use crate::client::session::Session;
use async_backtrace::{frame, framed, taskdump_tree};
use futures::SinkExt;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, watch};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, trace, warn};

/// Client actor handle.
#[derive(Debug, Clone)]
pub struct Client {
    actor_tx: mpsc::Sender<ClientMsg>,
    state_rx: watch::Receiver<Arc<ClientState>>,
}

impl Client {
    /// Create a new instance of `Client`.
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        // Spawn actor task.
        let (inner, actor_tx, state_rx) = ClientInner::new(addr);
        tokio::spawn(frame!(async move { inner.run().await }));

        let client = Self { actor_tx, state_rx };

        let mut connection = ClientConnection::new(client.clone(), stream);

        // Spawn separate async task to manage the TCP connection.
        tokio::spawn(frame!(async move {
            if let Err(e) = connection.setup().await {
                let addr = connection.client.addr();
                warn!("Error processing TCP connection from {addr:?}: {e:?}");
            }
        }));

        client
    }

    /// Get session actor handle.
    pub fn session(&self) -> Session {
        self.state_rx.borrow().session.clone()
    }

    #[framed]
    /// Set session.
    pub async fn set_session(&self, session: Session) -> Result<Session, ClientError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.actor_tx
            .send(ClientMsg::SetSession(response_tx, session))
            .await?;
        response_rx.await?
    }

    /// Get socket address.
    pub fn addr(&self) -> SocketAddr {
        self.state_rx.borrow().addr.clone()
    }

    #[framed]
    /// Set socket address.
    pub async fn set_addr(&self, addr: SocketAddr) -> Result<SocketAddr, ClientError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.actor_tx
            .send(ClientMsg::SetAddr(response_tx, addr))
            .await?;
        response_rx.await?
    }
}

impl Actor for Client {
    type Error = ClientError;
}

/// Client actor state.
#[derive(Debug, Clone)]
pub struct ClientState {
    pub session: Session,
    pub addr: SocketAddr,
}

impl ClientState {
    /// Create a new instance of `ClientState`.
    pub fn new(addr: SocketAddr) -> Self {
        let session = Session::new();
        Self { session, addr }
    }
}

/// Client connection.
pub struct ClientConnection {
    pub client: Client,
    pub lines: Framed<TcpStream, LinesCodec>,
}

impl ClientConnection {
    /// Create a new instance of `ClientConnection`.
    pub fn new(client: Client, stream: TcpStream) -> Self {
        // Create a LinesCodec to encode the stream as lines.
        let lines = Framed::new(stream, LinesCodec::new());

        Self { client, lines }
    }

    /// Setup a new client connection.
    #[framed]
    pub async fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let stream = self.lines.get_mut();
            stream.write_all(b"Enter username: ").await?;
        }

        let session = self.client.session();
        let addr = self.client.addr();
        let username = match self.lines.next().await {
            Some(Ok(line)) => line,
            _ => {
                info!("Client disconnected from {addr} without sending a username.");
                return Ok(());
            }
        };

        info!("User \"{username}\" logged in from {addr}.");

        session.set_username(Some(username)).await?;

        self.client_loop().await?;

        let username = session.username()?;
        info!("User \"{username}\" disconnected from {addr}.");

        Ok(())
    }

    /// Client main loop.
    #[framed]
    async fn client_loop(&mut self) -> Result<(), Box<dyn Error>> {
        trace!("{}", taskdump_tree(false));

        let session = self.client.session();

        // In a loop, read lines from the socket and write them back.
        loop {
            let input = match self.lines.next().await {
                Some(Ok(line)) => line,
                Some(Err(e)) => return Err(Box::new(e)),
                None => return Ok(()),
            };
            let username = session.username()?;
            let msg = format!("{username}: {input}");
            self.lines.send(msg).await?;
        }
    }
}

/// Client actor implementation.
#[derive(Debug)]
struct ClientInner {
    actor_rx: mpsc::Receiver<ClientMsg>,
    state_tx: watch::Sender<Arc<ClientState>>,
    state: Arc<ClientState>,
}

impl ClientInner {
    /// Create a new instance of `ClientInner`.
    pub fn new(
        addr: SocketAddr,
    ) -> (
        Self,
        mpsc::Sender<ClientMsg>,
        watch::Receiver<Arc<ClientState>>,
    ) {
        let state = Arc::from(ClientState::new(addr.clone()));

        let (actor_tx, actor_rx) = mpsc::channel(8);
        let (state_tx, state_rx) = watch::channel(state.clone());

        let inner = Self {
            actor_rx,
            state_tx,
            state,
        };

        (inner, actor_tx, state_rx)
    }

    /// Handle a message sent from a `Client` handle.
    #[framed]
    async fn handle_message(&mut self, msg: ClientMsg) -> Result<(), ClientError> {
        match msg {
            ClientMsg::SetSession(respond_to, session) => {
                let _ = respond_to.send(self.update_session(session));
            }
            ClientMsg::SetAddr(respond_to, addr) => {
                let _ = respond_to.send(self.update_addr(addr));
            }
        };

        Ok(())
    }

    /// Update session.
    fn update_session(&mut self, new_session: Session) -> Result<Session, ClientError> {
        Arc::make_mut(&mut self.state).session = new_session.clone();
        self.state = self.state_tx.send_replace(self.state.clone());

        let old_session = self.state.session.clone();
        Arc::make_mut(&mut self.state).session = new_session;

        Ok(old_session)
    }

    /// Update addr.
    fn update_addr(&mut self, new_addr: SocketAddr) -> Result<SocketAddr, ClientError> {
        Arc::make_mut(&mut self.state).addr = new_addr.clone();
        self.state = self.state_tx.send_replace(self.state.clone());

        let old_addr = self.state.addr.clone();
        Arc::make_mut(&mut self.state).addr = new_addr;

        Ok(old_addr)
    }
}

impl ActorInner for ClientInner {
    type Error = ClientError;

    /// Run client actor task.
    #[framed]
    async fn run(mut self) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        while let Some(msg) = self.actor_rx.recv().await {
            let debug_msg = format!("{msg:?}");
            if let Err(e) = self.handle_message(msg).await {
                warn!("Error handling {debug_msg}: {e:?}");
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ClientMsg {
    SetSession(oneshot::Sender<Result<Session, ClientError>>, Session),
    SetAddr(oneshot::Sender<Result<SocketAddr, ClientError>>, SocketAddr),
}

type SendError = mpsc::error::SendError<ClientMsg>;
type RecvError = oneshot::error::RecvError;

#[derive(Debug)]
pub enum ClientError {
    TxError(SendError),
    RxError(RecvError),
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TxError(err) => err.source(),
            Self::RxError(err) => err.source(),
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TxError(err) => err.fmt(f),
            Self::RxError(err) => err.fmt(f),
        }
    }
}

impl From<SendError> for ClientError {
    fn from(err: SendError) -> Self {
        Self::TxError(err)
    }
}

impl From<RecvError> for ClientError {
    fn from(err: RecvError) -> Self {
        Self::RxError(err)
    }
}

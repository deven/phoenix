// -*- Rust -*-
//
// Phoenix CMC library: server module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::session::Session;
use crate::client::Client;
use crate::Options;
use async_backtrace::{frame, framed};
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::{info, warn};

/// Server handle.
#[derive(Debug, Clone)]
pub struct Server(Arc<RwLock<ServerInner>>);

#[derive(Debug, Clone)]
pub struct ServerInner {
    pub options: Options,
    pub clients: Vec<Client>,
    pub sessions: Vec<Session>,
}

impl Server {
    /// Create a new instance of `Server`.
    pub fn new(options: Options) -> Self {
        let clients = Vec::new();
        let sessions = Vec::new();

        let inner = ServerInner {
            options,
            clients,
            sessions,
        };

        Server(Arc::new(RwLock::new(inner)))
    }

    pub async fn run(&self) -> Result<(), ServerError> {
        let cron = self.cron().await;
        let port = self.port().await;
        let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

        let listener = match TcpListener::bind(socket).await {
            Ok(listener) => listener,
            Err(err) => {
                if cron && err.kind() == ErrorKind::AddrInUse {
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
                    let server = self.clone();
                    let mut client = Client::new(server, addr);
                    tokio::spawn(frame!(async move {
                        if let Err(err) = client.run(stream).await {
                            let addr = client.addr().await;
                            warn!("Error processing TCP connection from {addr:?}: {err:?}");
                        }
                    }));
                }
                Err(err) => warn!("Error accepting TCP connection: {err:?}"),
            }
        }
    }

    /// Obtain read lock on the server data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, ServerInner> {
        self.0.read().await
    }

    /// Obtain write lock on the server data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, ServerInner> {
        self.0.write().await
    }

    /// Get options.
    #[framed]
    pub async fn options(&self) -> Options {
        self.read().await.options.clone()
    }

    /// Set options.
    #[framed]
    pub async fn set_options(&self, options: Options) {
        self.write().await.options = options;
    }

    /// Get cron flag.
    #[framed]
    pub async fn cron(&self) -> bool {
        self.read().await.options.cron
    }

    /// Set cron flag.
    #[framed]
    pub async fn set_cron(&self, cron: bool) {
        self.write().await.options.cron = cron;
    }

    /// Get debug flag.
    #[framed]
    pub async fn debug(&self) -> bool {
        self.read().await.options.debug
    }

    /// Set debug flag.
    #[framed]
    pub async fn set_debug(&self, debug: bool) {
        self.write().await.options.debug = debug;
    }

    /// Get ipv6 flag.
    #[framed]
    pub async fn ipv6(&self) -> bool {
        self.read().await.options.ipv6
    }

    /// Set ipv6 flag.
    #[framed]
    pub async fn set_ipv6(&self, ipv6: bool) {
        self.write().await.options.ipv6 = ipv6;
    }

    /// Get TCP port number to listen on.
    #[framed]
    pub async fn port(&self) -> u16 {
        self.read().await.options.port
    }

    /// Set TCP port number to listen on.
    #[framed]
    pub async fn set_port(&self, port: u16) {
        self.write().await.options.port = port;
    }

    /// Get list of all clients.
    #[framed]
    pub async fn clients(&self) -> Vec<Client> {
        self.read().await.clients.clone()
    }

    /// Set list of all clients.
    #[framed]
    pub async fn set_clients(&self, clients: Vec<Client>) {
        self.write().await.clients = clients;
    }

    /// Push a new client onto the list of all clients.
    #[framed]
    pub async fn push_client(&self, client: Client) {
        self.write().await.clients.push(client);
    }

    /// Get list of all sessions.
    #[framed]
    pub async fn sessions(&self) -> Vec<Session> {
        self.read().await.sessions.clone()
    }

    /// Set list of all sessions.
    #[framed]
    pub async fn set_sessions(&self, sessions: Vec<Session>) {
        self.write().await.sessions = sessions;
    }

    /// Push a new session onto the list of all sessions.
    #[framed]
    pub async fn push_session(&self, session: Session) {
        self.write().await.sessions.push(session);
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

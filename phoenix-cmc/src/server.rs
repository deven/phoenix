use crate::session::Session;
use crate::telnet::Telnet;
use crate::text::Text;
use anyhow::Result;
use async_backtrace::framed;
use log::{error, info};
use std::mem;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::AbortHandle;
use tokio::time::Duration;

/// Server handle.
#[derive(Debug, Clone)]
pub struct Server(Arc<RwLock<ServerInner>>);

#[derive(Debug)]
pub struct ServerInner
where
    Self: Send + Sync + 'static,
{
    pub listener: Arc<TcpListener>,
    pub port: u16,
    pub debug: bool,
    pub shutdown_tx: broadcast::Sender<()>,
    pub shutdown_handle: Option<AbortHandle>,
    pub restarting: bool,
}

impl Server {
    /// Create a new `Server` object.
    #[framed]
    pub async fn new(port: u16, debug: bool) -> Result<Self> {
        let listener = Arc::new(TcpListener::bind(("0.0.0.0", port)).await?);
        let (shutdown_tx, _) = broadcast::channel(16);
        let shutdown_handle = None;
        let restarting = false;

        let inner = ServerInner { listener, port, debug, shutdown_tx, shutdown_handle, restarting };

        Ok(Self(Arc::new(RwLock::new(inner))))
    }

    /// Obtain read lock on the `ServerInner` data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, ServerInner> {
        self.0.read().await
    }

    /// Obtain write lock on the `ServerInner` data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, ServerInner> {
        self.0.write().await
    }

    /// Get the `TcpListener` object.
    #[framed]
    pub async fn listener(&self) -> Arc<TcpListener> {
        self.read().await.listener.clone()
    }

    /// Get the TCP listening port number.
    #[framed]
    pub async fn port(&self) -> u16 {
        self.read().await.port
    }

    /// Get the debugging flag.
    #[framed]
    pub async fn debug(&self) -> bool {
        self.read().await.debug
    }

    /// Set the debugging flag.
    #[framed]
    pub async fn set_debug(&self, value: bool) {
        self.write().await.debug = value;
    }

    /// Get the shutdown `broadcast::Sender`.
    #[framed]
    pub async fn shutdown_tx(&self) -> broadcast::Sender<()> {
        self.read().await.shutdown_tx.clone()
    }

    /// Get the shutdown `AbortHandle`, if any.
    #[framed]
    pub async fn shutdown_handle(&self) -> Option<AbortHandle> {
        self.read().await.shutdown_handle.clone()
    }

    /// Set the shutdown `AbortHandle`.
    #[framed]
    pub async fn set_shutdown_handle(&self, value: Option<AbortHandle>) {
        self.write().await.shutdown_handle = value;
    }

    /// Take the shutdown `AbortHandle`, if any.
    #[framed]
    pub async fn take_shutdown_handle(&self) -> Option<AbortHandle> {
        mem::take(&mut self.write().await.shutdown_handle)
    }

    /// Get the restarting flag.
    #[framed]
    pub async fn restarting(&self) -> bool {
        self.read().await.restarting
    }

    /// Set the restarting flag.
    #[framed]
    pub async fn set_restarting(&self, value: bool) {
        self.write().await.restarting = value;
    }

    /// Run the Phoenix server.
    pub async fn run(&self) -> Result<()> {
        // Accept loop
        loop {
            match self.listener().await.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {addr}");

                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Connection error: {e}");
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {e}");
                    continue;
                }
            }
        }
    }

    /// Handle a new TCP connection.
    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        // Set TCP options
        stream.set_nodelay(true)?;

        // Create telnet connection
        let telnet = Telnet::new(stream).await;

        // Create session
        let session = Session::new(self.clone(), telnet.clone()).await;

        // Initialize login sequence
        session.init_login_sequence().await;

        // Handle telnet I/O
        let mut shutdown_rx = self.shutdown_tx().await.subscribe();
        tokio::select! {
            _ = shutdown_rx.recv() => {
                telnet.output("\n\n*** Server is shutting down ***\n").await;
                telnet.close(true).await;
            }
            result = telnet.handle_input() => {
                if let Err(e) = result {
                    if e.kind() != std::io::ErrorKind::UnexpectedEof {
                        error!("Telnet error: {e}");
                    }
                }

                // Detach or close session
                if session.signed_on().await {
                    session.detach(&telnet, false).await;
                } else {
                    session.close(false).await;
                }
            }
        }

        Ok(())
    }

    /// Schedule a server restart.
    pub async fn schedule_restart(&self, who: Text, seconds: u64) {
        self.schedule_shutdown_or_restart(who, seconds, true).await;
    }

    /// Schedule a server shutdown.
    pub async fn schedule_shutdown(&self, who: Text, seconds: u64) {
        self.schedule_shutdown_or_restart(who, seconds, false).await;
    }

    /// Cancel a server restart/shutdown.
    pub async fn cancel_shutdown(&self) -> Option<bool> {
        let mut inner = self.write().await;
        let restarting = mem::take(&mut inner.restarting);

        if let Some(handle) = mem::take(&mut inner.shutdown_handle) {
            handle.abort();
            Some(restarting)
        } else {
            None
        }
    }

    /// Schedule a server shutdown or restart.
    pub async fn schedule_shutdown_or_restart(&self, who: Text, seconds: u64, restart: bool) {
        // Cancel any existing shutdown.
        self.cancel_shutdown().await;

        let action = if restart { "restart" } else { "shutdown" };
        info!("Server {action} scheduled by {who} in {seconds} seconds.");

        self.set_restarting(restart).await;

        let action = if restart { "restarting" } else { "shutting down" };

        let server = self.clone();
        let handle = tokio::spawn(async move {
            let mut remaining = seconds;
            while remaining > 0 {
                match remaining {
                    300 => Session::announce(&format!("*** Server {action} in 5 minutes ***\n")).await,
                    180 => Session::announce(&format!("*** Server {action} in 3 minutes ***\n")).await,
                    120 => Session::announce(&format!("*** Server {action} in 2 minutes ***\n")).await,
                    60 => Session::announce(&format!("*** Server {action} in 1 minute ***\n")).await,
                    30 | 10 | 2..=5 => Session::announce(&format!("*** Server {action} in {remaining} seconds ***\n")).await,
                    1 => Session::announce(&format!("*** Server {action} in 1 second ***\n")).await,
                    _ => {}
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
                remaining -= 1;
            }

            Session::announce(&format!("*** Server {action} NOW ***\n")).await;
            server.perform_shutdown_or_restart(restart).await;
        });

        self.set_shutdown_handle(Some(handle.abort_handle())).await;
    }

    /// Perform a server shutdown or restart.
    pub async fn perform_shutdown_or_restart(&self, restart: bool) {
        // Signal all connections to shut down gracefully.
        let _ = self.shutdown_tx().await.send(());

        // Give connections time to close.
        tokio::time::sleep(Duration::from_secs(2)).await;

        if restart {
            match std::env::current_exe() {
                Ok(exe) => match std::process::Command::new(exe).args(std::env::args().skip(1)).spawn() {
                    Ok(_) => info!("Server restart initiated"),
                    Err(e) => info!("Failed to restart server: {e}"),
                },
                Err(e) => info!("Failed to get executable path: {e}"),
            }
        }

        std::process::exit(0);
    }
}

/// Check if the specified TCP port number is busy.
pub async fn is_port_busy(port: u16) -> bool {
    match TcpListener::bind(("0.0.0.0", port)).await {
        Ok(_) => false, // Port is free
        Err(_) => true, // Port is busy
    }
}

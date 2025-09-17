use crate::session::Session;
use crate::telnet::Telnet;
use crate::text::Text;
use anyhow::Result;
use arc_swap::{ArcSwap, ArcSwapOption};
use async_backtrace::framed;
use log::{error, info};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::task::AbortHandle;
use tokio::time::Duration;

/// Server handle.
#[derive(Debug, Clone)]
pub struct Server(pub Arc<ServerInner>);

#[derive(Debug)]
pub struct ServerInner
{
    pub listener: ArcSwap<TcpListener>,
    pub port: AtomicU16,
    pub debug: AtomicBool,
    pub shutdown_tx: ArcSwap<broadcast::Sender<()>>,
    pub shutdown_handle: ArcSwapOption<AbortHandle>,
    pub restarting: AtomicBool,
}

impl Server {
    /// Create a new `Server` object.
    #[framed]
    pub async fn new(port: u16, debug: bool) -> Result<Self> {
        let listener = Arc::new(TcpListener::bind(("0.0.0.0", port)).await?);
        let (shutdown_tx, _) = broadcast::channel(16);
        let shutdown_handle = None;
        let restarting = false;

        let inner = ServerInner {
            listener: ArcSwap::new(listener),
            port: AtomicU16::new(port),
            debug: AtomicBool::new(debug),
            shutdown_tx: ArcSwap::new(Arc::new(shutdown_tx)),
            shutdown_handle: ArcSwapOption::new(shutdown_handle),
            restarting: AtomicBool::new(restarting),
        };

        Ok(Self(Arc::new(inner)))
    }

    /// Get the `TcpListener` object.
    pub fn listener(&self) -> Arc<TcpListener> {
        self.0.listener.load_full()
    }

    /// Set the `TcpListener` object.
    pub fn set_listener(&self, value: TcpListener) {
        self.0.listener.store(Arc::new(value));
    }

    /// Get the TCP listening port number.
    pub fn port(&self) -> u16 {
        self.0.port.load(Ordering::Relaxed)
    }

    /// Set the TCP listening port number.
    pub fn set_port(&self, value: u16) {
        self.0.port.store(value, Ordering::Relaxed);
    }

    /// Get the debugging flag.
    pub fn debug(&self) -> bool {
        self.0.debug.load(Ordering::Relaxed)
    }

    /// Set the debugging flag.
    pub fn set_debug(&self, value: bool) {
        self.0.debug.store(value, Ordering::Relaxed);
    }

    /// Get the shutdown `broadcast::Sender`.
    pub fn shutdown_tx(&self) -> broadcast::Sender<()> {
        (*self.0.shutdown_tx.load_full()).clone()
    }

    /// Set the shutdown `broadcast::Sender`.
    pub fn set_shutdown_tx(&self, value: broadcast::Sender<()>) {
        self.0.shutdown_tx.store(Arc::new(value));
    }

    /// Get the shutdown `AbortHandle`, if any.
    pub fn shutdown_handle(&self) -> Option<AbortHandle> {
        self.0.shutdown_handle.load_full().map(|arc| (*arc).clone())
    }

    /// Set the shutdown `AbortHandle`.
    pub fn set_shutdown_handle(&self, value: Option<AbortHandle>) {
        self.0.shutdown_handle.store(value.map(Arc::new));
    }

    /// Take the shutdown `AbortHandle`, if any.
    pub fn take_shutdown_handle(&self) -> Option<AbortHandle> {
        self.0.shutdown_handle.swap(None).map(|arc| Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone()))
    }

    /// Get the restarting flag.
    pub fn restarting(&self) -> bool {
        self.0.restarting.load(Ordering::Relaxed)
    }

    /// Set the restarting flag.
    pub fn set_restarting(&self, value: bool) {
        self.0.restarting.store(value, Ordering::Relaxed);
    }

    /// Run the Phoenix server.
    pub async fn run(&self) -> Result<()> {
        // Accept loop
        loop {
            match self.listener().accept().await {
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
        // Set TCP options.
        stream.set_nodelay(true)?;

        // Create `Telnet` with associated `LoginSession`.
        let telnet = Telnet::new(stream, self.clone());

        // Initiate TELNET protocol option negotiations and session login sequence.
        telnet.init_login_sequence().await;

        // Handle network I/O.
        let mut shutdown_rx = self.shutdown_tx().subscribe();
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
                if let Some(session) = telnet.session() {
                    if session.signed_on() {
                        session.detach(&telnet, false).await;
                    } else {
                        session.close(false).await;
                    }
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
        let restarting = self.restarting();
        self.set_restarting(false);

        if let Some(handle) = self.take_shutdown_handle() {
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

        self.set_restarting(restart);

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

        self.set_shutdown_handle(Some(handle.abort_handle()));
    }

    /// Perform a server shutdown or restart.
    pub async fn perform_shutdown_or_restart(&self, restart: bool) {
        // Signal all connections to shut down gracefully.
        let _ = self.shutdown_tx().send(());

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

//#[cfg(test)]
fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<Server>();
    assert_send_sync_static::<ServerInner>();
};

// -*- Rust -*-
//
// Phoenix CMC library: server module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::session::Session;
use crate::telnet::Telnet;
use crate::text::Text;
use crate::timestamp::{Timestamp, system_uptime};
use anyhow::Result;
use arc_swap::{ArcSwap, ArcSwapOption};
use async_backtrace::framed;
use log::{error, info};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::task::AbortHandle;
use tokio::time::Duration;

/// Server handle.
#[derive(Debug, Clone)]
pub struct Server(pub Arc<ServerInner>);

#[derive(Debug)]
pub struct ServerInner {
    pub listener: ArcSwap<TcpListener>,
    pub port: AtomicU16,
    pub debug: AtomicBool,
    pub shutdown_tx: ArcSwap<broadcast::Sender<()>>,
    pub shutdown_handle: ArcSwapOption<AbortHandle>,
    pub restarting: AtomicBool,
    pub server_start_time: i64,
    pub server_start_uptime: Option<i64>,
}

impl Server {
    /// Create a new `Server` object.
    #[framed]
    pub async fn new(port: u16, debug: bool) -> Result<Self> {
        println!("=== DEBUG: Server::new() called with port={port}, debug={debug} ===");
        let listener = Arc::new(TcpListener::bind(("0.0.0.0", port)).await?);
        println!("=== DEBUG: TcpListener bound successfully to 0.0.0.0:{port} ===");
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
            server_start_time: Timestamp::new().unix(),
            server_start_uptime: system_uptime().await,
        };

        println!("=== DEBUG: Server::new() completed successfully ===");
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

    /// Get the server start time.
    pub fn server_start_time(&self) -> i64 {
        self.0.server_start_time
    }

    /// Get the system uptime when server started.
    pub fn server_start_uptime(&self) -> Option<i64> {
        self.0.server_start_uptime
    }

    /// Run the Phoenix server.
    pub async fn run(&self) -> Result<()> {
        println!("=== DEBUG: Server::run() starting accept loop ===");
        // Accept loop
        loop {
            println!("=== DEBUG: Waiting for connection on listener.accept() ===");
            match self.listener().accept().await {
                Ok((stream, addr)) => {
                    println!("=== DEBUG: New connection accepted from {addr} ===");
                    info!("New connection from {addr}");

                    let server = self.clone();
                    tokio::spawn(async move {
                        println!("=== DEBUG: Spawned task to handle connection from {addr} ===");
                        if let Err(e) = server.handle_connection(stream).await {
                            println!("=== DEBUG: Connection error from {addr}: {e} ===");
                            error!("Connection error: {e}");
                        }
                    });
                }
                Err(e) => {
                    println!("=== DEBUG: Failed to accept connection: {e} ===");
                    error!("Failed to accept connection: {e}");
                    continue;
                }
            }
        }
    }

    /// Handle a new TCP connection.
    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        println!("=== DEBUG: Server::handle_connection() starting ===");
        // Set TCP options.
        stream.set_nodelay(true)?;
        println!("=== DEBUG: TCP nodelay set ===");

        // Create `Telnet` with associated `LoginSession`.
        println!("=== DEBUG: Creating Telnet instance ===");
        let telnet = Telnet::new(&stream, self.clone());
        println!("=== DEBUG: Telnet instance created ===");

        // Log connection details
        telnet.log_caller().await;

        // Initiate TELNET protocol option negotiations and session login sequence.
        println!("=== DEBUG: Starting telnet.init_login_sequence() ===");
        telnet.init_login_sequence().await?;
        println!("=== DEBUG: telnet.init_login_sequence() completed ===");

        // Handle network I/O.
        println!("=== DEBUG: Starting I/O handling loop ===");
        let shutdown_rx = self.shutdown_tx().subscribe();
        let result = telnet.handle_input(stream, shutdown_rx).await;
        println!("=== DEBUG: telnet.handle_input() returned: {result:?} ===");
        if let Err(e) = result {
            if e.kind() != std::io::ErrorKind::UnexpectedEof {
                error!("Telnet error: {e}");
            }
        }

        // Detach or close session
        let session = telnet.session();
        {
            if session.signed_on() {
                session.detach(&telnet, false).await?;
            } else {
                session.close(false).await?;
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
                    300 => Session::announce(&format!("*** Server {action} in 5 minutes ***\n")).await.unwrap_or(()),
                    180 => Session::announce(&format!("*** Server {action} in 3 minutes ***\n")).await.unwrap_or(()),
                    120 => Session::announce(&format!("*** Server {action} in 2 minutes ***\n")).await.unwrap_or(()),
                    60 => Session::announce(&format!("*** Server {action} in 1 minute ***\n")).await.unwrap_or(()),
                    30 | 10 | 2..=5 => Session::announce(&format!("*** Server {action} in {remaining} seconds ***\n")).await.unwrap_or(()),
                    1 => Session::announce(&format!("*** Server {action} in 1 second ***\n")).await.unwrap_or(()),
                    _ => {}
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
                remaining -= 1;
            }

            Session::announce(&format!("*** Server {action} NOW ***\n")).await.unwrap_or(());
            server.perform_shutdown_or_restart(restart).await;
        });

        self.set_shutdown_handle(Some(handle.abort_handle()));

        // Check immediately in case there are no active sessions.
        self.check_shutdown().await;
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

    /// Check if shutting down and no users are left.
    pub async fn check_shutdown(&self) {
        use crate::session::SESSIONS;

        // Only proceed if shutdown is scheduled
        if self.shutdown_handle().is_none() {
            return;
        }

        // Check if any sessions remain
        if !SESSIONS.is_empty() {
            return;
        }

        // All connections closed, proceed with shutdown/restart
        let restart = self.restarting();
        log::info!("All connections closed, {verb} now.", verb = if restart { "restarting" } else { "shutting down" });
        self.perform_shutdown_or_restart(restart).await;
    }
}

/// Check if the specified TCP port number is busy.
pub async fn is_port_busy(port: u16) -> bool {
    TcpListener::bind(("0.0.0.0", port)).await.is_err()
}

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<Server>();
    assert_send_sync_static::<ServerInner>();
};

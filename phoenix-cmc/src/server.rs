// -*- Rust -*-
//
// Phoenix CMC library: server module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::atomic::AtomicAbortHandleOption;
use crate::name::Name;
use crate::session::{Session, SessionMsg};
use crate::telnet::Telnet;
use crate::text::Text;
use crate::timestamp::{Timestamp, system_uptime};
use anyhow::Result;
use async_backtrace::framed;
use log::{error, info};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::task::AbortHandle;
use tokio::time::Duration;

/// Server handle.
#[derive(Debug, Clone)]
pub struct Server(pub Arc<ServerInner>);

/// Messages to the server actor (the accept loop is the registry serializer, as the C++ main loop was).
#[derive(Debug)]
pub enum ServerMsg {
    /// Claim a name to complete a login (second login handshake).  The claim is authoritative against simultaneous
    /// in-flight logins, which no scan of existing sessions can serialize; the outcome returns as
    /// SessionMsg::EnterResult.  (~ the atomicity the C++ single thread gave CheckNameAvailability and Session::Login
    /// implicitly.)
    Enter { session: Session, name: Name },
    /// Release a name claim (a signed-on session closed).
    Exit { name: Text },
}

/// Private server state, owned by the server actor task.
#[derive(Debug)]
pub struct ServerObj {
    pub server: Server,
    pub rx: mpsc::UnboundedReceiver<ServerMsg>,
    /// Names claimed by signed-on (or completing) sessions; Text folds case.
    pub names: HashSet<Text>,
}

/// Public server state, readable by any task.
#[derive(Debug)]
pub struct ServerInner {
    pub listener: TcpListener,
    pub tx: mpsc::UnboundedSender<ServerMsg>,
    pub port: AtomicU16,
    pub debug: AtomicBool,
    pub shutdown_tx: broadcast::Sender<()>,
    pub shutdown_handle: AtomicAbortHandleOption,
    pub restarting: AtomicBool,
    pub server_start_time: i64,
    pub server_start_uptime: Option<i64>,
}

impl Server {
    /// Create a new `Server` object.
    #[framed]
    pub async fn new(port: u16, debug: bool) -> Result<(Self, ServerObj)> {
        println!("=== DEBUG: Server::new() called with port={port}, debug={debug} ===");
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        println!("=== DEBUG: TcpListener bound successfully to 0.0.0.0:{port} ===");
        let (shutdown_tx, _) = broadcast::channel(16);
        let shutdown_handle = None;
        let restarting = false;

        let (tx, rx) = mpsc::unbounded_channel();
        let inner = ServerInner {
            listener,
            tx,
            port: AtomicU16::new(port),
            debug: AtomicBool::new(debug),
            shutdown_tx,
            shutdown_handle: AtomicAbortHandleOption::new(shutdown_handle),
            restarting: AtomicBool::new(restarting),
            server_start_time: Timestamp::new().unix(),
            server_start_uptime: system_uptime().await,
        };

        println!("=== DEBUG: Server::new() completed successfully ===");
        let server = Self(Arc::new(inner));
        let obj = ServerObj { server: server.clone(), rx, names: HashSet::new() };
        Ok((server, obj))
    }

    /// Get the `TcpListener` object.
    pub fn listener(&self) -> &TcpListener {
        &self.0.listener
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
        self.0.shutdown_tx.clone()
    }

    /// Get the shutdown `AbortHandle`, if any.
    pub fn shutdown_handle(&self) -> Option<AbortHandle> {
        self.0.shutdown_handle.snapshot()
    }

    /// Set the shutdown `AbortHandle`.
    pub fn set_shutdown_handle(&self, value: Option<AbortHandle>) {
        self.0.shutdown_handle.set(value);
    }

    /// Take the shutdown `AbortHandle`, if any.
    pub fn take_shutdown_handle(&self) -> Option<AbortHandle> {
        self.0.shutdown_handle.swap(None)
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

    /// Claim a name to complete a login (second login handshake); the outcome arrives as SessionMsg::EnterResult.
    pub fn enter(&self, session: Session, name: Name) {
        let _ = self.0.tx.send(ServerMsg::Enter { session, name });
    }

    /// Release a name claim (a signed-on session closed).
    pub fn release_name(&self, name: Text) {
        let _ = self.0.tx.send(ServerMsg::Exit { name });
    }

    /// Handle a new TCP connection.
    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        println!("=== DEBUG: Server::handle_connection() starting ===");
        // Set TCP options.
        stream.set_nodelay(true)?;
        println!("=== DEBUG: TCP nodelay set ===");

        // Create `Telnet` with associated `LoginSession`.
        println!("=== DEBUG: Creating Telnet instance ===");
        let (telnet, obj) = Telnet::new(stream, self.clone());
        println!("=== DEBUG: Telnet instance created ===");

        // Log connection details
        telnet.log_caller().await;

        // Initiate TELNET protocol option negotiations and session login sequence.
        println!("=== DEBUG: Starting telnet.init_login_sequence() ===");
        println!("=== DEBUG: telnet.init_login_sequence() completed ===");

        // Handle network I/O.
        println!("=== DEBUG: Starting I/O handling loop ===");
        let shutdown_rx = self.shutdown_tx().subscribe();
        let result = obj.run(shutdown_rx).await;
        println!("=== DEBUG: telnet.handle_input() returned: {result:?} ===");
        if let Err(e) = result {
            if e.kind() != std::io::ErrorKind::UnexpectedEof {
                error!("Telnet error: {e}");
            }
        }

        // Detach or close the session, decided on the session actor (~ Telnet::Closed()).
        telnet.closed();

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

impl ServerObj {
    /// Run the Phoenix server.  The server actor: the accept loop and the name-claim registry share one sequential
    /// context (~ the C++ main loop serializing accepts and session bookkeeping).
    #[framed]
    pub async fn run(mut self) -> Result<()> {
        println!("=== DEBUG: Server::run() starting accept loop ===");
        loop {
            tokio::select! {
                accepted = self.server.listener().accept() => match accepted {
                    Ok((stream, addr)) => {
                        println!("=== DEBUG: New connection accepted from {addr} ===");
                        info!("New connection from {addr}");

                        let server = self.server.clone();
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
                },
                msg = self.rx.recv() => match msg {
                    Some(ServerMsg::Enter { session, name }) => {
                        // The claim is the check and the insert as one sequential unit: simultaneous logins racing one
                        // name serialize here, and exactly one wins.
                        let ok = self.names.insert(name.name().clone());
                        let _ = session.0.tx.send(SessionMsg::EnterResult { ok, name });
                    }
                    Some(ServerMsg::Exit { name }) => {
                        self.names.remove(&name);
                    }
                    None => {}
                },
            }
        }
    }
}

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<Server>();
    assert_send_sync_static::<ServerInner>();
};

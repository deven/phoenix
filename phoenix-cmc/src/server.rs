use crate::event::{EventQueue, LoginTimeoutEvent};
use crate::session::Session;
use crate::telnet::Telnet;
use crate::VERSION;
use anyhow::Result;
use log::{error, info};
use std::sync::{Arc, LazyLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

pub struct PhoenixServer {
    pub listener: TcpListener,
    pub port: u16,
    pub debug: bool,
    pub shutdown_tx: broadcast::Sender<()>,
    pub shutdown_handle: RwLock<Option<JoinHandle<()>>>,
    pub restarting: Arc<AtomicBool>,
}

impl PhoenixServer {
    pub async fn new(port: u16, debug: bool) -> Result<Arc<Self>> {
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        let (shutdown_tx, _) = broadcast::channel(16);
        let shutdown_handle = RwLock::new(None);
        let restarting = Arc::new(AtomicBool::new(false));

        Ok(Arc::new(Self {
            listener,
            port,
            debug,
            shutdown_tx,
            shutdown_handle,
            restarting,
        }))
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        // Accept loop
        loop {
            let (stream, addr) = match listener.accept().await {
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
            };
        }

        Ok(())
    }

    pub async fn handle_connection(self: Arc<Self>, stream: TcpStream) -> Result<()> {
        // Set TCP options
        stream.set_nodelay(true)?;

        // Create telnet connection
        let telnet = Telnet::new(stream).await;

        // Create session
        let session = Session::new(self.clone(), telnet.clone()).await;

        // Set up login timeout
        let timeout_event = Box::new(LoginTimeoutEvent::new(
            Arc::new(RwLock::new(telnet.clone())),
            Telnet::LOGIN_TIMEOUT_TIME as i64,
        ));

        // Initialize login sequence
        session.init_login_sequence().await;

        // Handle telnet I/O
        let mut shutdown_rx = self.shutdown_tx.subscribe();
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

    pub async fn initiate_shutdown(self: &Arc<Self>, reason: &str, delay: u64) {
        info!("Initiating shutdown: {reason} (delay: {delay}s)");

        // Send shutdown signal after delay
        let shutdown_tx = self.shutdown_tx.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(delay)).await;
            let _ = shutdown_tx.send(());
        });
    }

    pub async fn schedule_restart(self: &Arc<Self>, who: ArcStr, seconds: u64) {
        self.schedule_shutdown_or_restart(who, seconds, true).await;
    }

    pub async fn schedule_shutdown(self: &Arc<Self>, who: ArcStr, seconds: u64) {
        self.schedule_shutdown_or_restart(who, seconds, false).await;
    }

    pub async fn cancel_shutdown(self: &Arc<Self>) -> Option<bool> {
        let restarting = self.restarting.load(Ordering::Relaxed);

        if let Some(handle) = self.shutdown_handle.write().await.take() {
            handle.abort();
            Some(restarting)
        } else {
            None
        }
    }

    pub async fn schedule_shutdown_or_restart(
        self: &Arc<Self>,
        who: ArcStr,
        seconds: u64,
        restart: bool,
    ) {
        // Cancel any existing shutdown
        let _ = self.cancel_shutdown().await;

        let action = if restart { "restart" } else { "shutdown" };
        info!("Server {action} scheduled by {who} in {seconds} seconds.");

        self.restarting.store(restart, Ordering::Relaxed);

        let action = if restart {
            "restarting"
        } else {
            "shutting down"
        };
        let handle = tokio::spawn(async move {
            let mut remaining = seconds;
            while remaining > 0 {
                match remaining {
                    300 => {
                        Session::announce(&format!("*** Server {action} in 5 minutes ***\n")).await
                    }
                    180 => {
                        Session::announce(&format!("*** Server {action} in 3 minutes ***\n")).await
                    }
                    120 => {
                        Session::announce(&format!("*** Server {action} in 2 minutes ***\n")).await
                    }
                    60 => {
                        Session::announce(&format!("*** Server {action} in 1 minute ***\n")).await
                    }
                    30 | 10 | 5..1 => {
                        Session::announce(&format!(
                            "*** Server {action} in {remaining} seconds ***\n"
                        ))
                        .await
                    }
                    1 => Session::announce(&format!("*** Server {action} in 1 second ***\n")).await,
                    _ => {}
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
                remaining -= 1;
            }

            Session::announce(&format!("*** Server {action} NOW ***\n")).await;
            self.perform_shutdown_or_restart(restart).await;
        });

        *shutdown_handle.write().await = Some(handle);
    }

    pub async fn perform_shutdown_or_restart(self: &Arc<Self>, restart: bool) {
        tokio::time::sleep(Duration::from_secs(2)).await;

        if restart {
            match std::env::current_exe() {
                Ok(exe) => {
                    match std::process::Command::new(exe)
                        .args(std::env::args().skip(1))
                        .spawn()
                    {
                        Ok(_) => info!("Server restart initiated"),
                        Err(e) => info!("Failed to restart server: {e}"),
                    }
                }
                Err(e) => info!("Failed to get executable path: {e}"),
            }
        }

        std::process::exit(0);
    }
}

pub async fn is_port_busy(port: u16) -> bool {
    match TcpListener::bind(("0.0.0.0", port)).await {
        Ok(_) => false, // Port is free
        Err(_) => true, // Port is busy
    }
}

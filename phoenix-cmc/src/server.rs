use crate::event::{EventQueue, LoginTimeoutEvent};
use crate::session::Session;
use crate::telnet::Telnet;
use crate::VERSION;
use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio::time::{sleep, Duration};

pub struct PhoenixServer {
    pub listener: TcpListener,
    pub port: u16,
    pub debug: bool,
    pub event_queue: Arc<EventQueue>,
    pub shutdown_tx: broadcast::Sender<()>,
    pub shutdown_rx: broadcast::Receiver<()>,
}

impl PhoenixServer {
    pub async fn new(port: u16, debug: bool) -> Result<Arc<Self>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        let (shutdown_tx, shutdown_rx) = broadcast::channel(16);

        Ok(Arc::new(Self {
            listener,
            port,
            debug,
            event_queue: Arc::new(EventQueue::new()),
            shutdown_tx,
            shutdown_rx,
        }))
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        // Spawn event processor
        let event_queue = self.event_queue.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => break,
                    wait_time = event_queue.execute() => {
                        if let Some(duration) = wait_time {
                            tokio::select! {
                                _ = shutdown_rx.recv() => break,
                                _ = sleep(duration) => continue,
                            }
                        } else {
                            // No events, wait for signal
                            tokio::select! {
                                _ = shutdown_rx.recv() => break,
                                _ = sleep(Duration::from_secs(60)) => continue,
                            }
                        }
                    }
                }
            }
        });

        // Accept loop
        let mut shutdown_rx = self.shutdown_rx.resubscribe();
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Server shutdown initiated");
                    break;
                }
                result = self.listener.accept() => {
                    match result {
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
                            error!("Accept error: {e}");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        // Set TCP options
        stream.set_nodelay(true)?;

        // Create telnet connection
        let telnet = Telnet::new(stream).await;

        // Create session
        let session = Session::new(telnet.clone()).await;

        // Set up login timeout
        let timeout_event = Box::new(LoginTimeoutEvent::new(
            Arc::new(RwLock::new(telnet.clone())),
            Telnet::LOGIN_TIMEOUT_TIME as i64,
        ));
        self.event_queue.enqueue(timeout_event).await;

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

    pub async fn initiate_shutdown(&self, reason: &str, delay: u64) {
        info!("Initiating shutdown: {reason} (delay: {delay}s)");

        // Send shutdown signal after delay
        let shutdown_tx = self.shutdown_tx.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(delay)).await;
            let _ = shutdown_tx.send(());
        });
    }
}

pub async fn is_port_busy(port: u16) -> bool {
    match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
        Ok(_) => false, // Port is free
        Err(_) => true, // Port is busy
    }
}

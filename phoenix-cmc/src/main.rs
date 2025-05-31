use anyhow::Result;
use log::info;
use phoenix_cmc::server;
use std::env;
use std::path::PathBuf;
use tokio::signal;

const VERSION: &str = "2.0.0";
const DEFAULT_PORT: u16 = 9999;
const LIBDIR: &str = "phoenix";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut port = DEFAULT_PORT;
    let mut cron = false;
    let mut debug = false;

    // Parse command-line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                println!(
                    "Usage: {} [--cron] [--debug] [--port {}]",
                    args[0], DEFAULT_PORT
                );
                return Ok(());
            }
            "--version" => {
                println!("Phoenix {} (Rust version)", VERSION);
                return Ok(());
            }
            "--cron" => cron = true,
            "--debug" => debug = true,
            "--port" => {
                i += 1;
                if i < args.len() {
                    port = args[i].parse()?;
                } else {
                    eprintln!(
                        "Usage: {} [--cron] [--debug] [--port {}]",
                        args[0], DEFAULT_PORT
                    );
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!(
                    "Usage: {} [--cron] [--debug] [--port {}]",
                    args[0], DEFAULT_PORT
                );
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // If --cron option was given, check if the listening port is busy
    if cron && server::is_port_busy(port).await {
        return Ok(());
    }

    // Change to LIBDIR (create if necessary)
    let libdir = PathBuf::from(LIBDIR);
    if !libdir.exists() {
        std::fs::create_dir(&libdir)?;
    }
    env::set_current_dir(&libdir)?;

    // Create logs subdirectory
    let logs_dir = PathBuf::from("logs");
    if !logs_dir.exists() {
        std::fs::create_dir(&logs_dir)?;
    }

    // Initialize server
    let server = server::PhoenixServer::new(port, debug).await?;
    info!("Started Phoenix server, version {}.", VERSION);
    info!(
        "Listening for connections on TCP port {}. (pid {})",
        port,
        std::process::id()
    );

    // Set up signal handlers
    let mut sigint = signal::ctrl_c();
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
    let mut sigquit = signal::unix::signal(signal::unix::SignalKind::quit())?;

    // Main event loop
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                log::error!("Server error: {}", e);
                return Err(e);
            }
        }
        _ = sigint => {
            info!("Received SIGINT, shutting down...");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down...");
        }
        _ = sigquit.recv() => {
            info!("Received SIGQUIT, initiating shutdown...");
            server.initiate_shutdown("signal", 5).await;
        }
    }

    Ok(())
}

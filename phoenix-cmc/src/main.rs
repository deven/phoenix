// -*- Rust -*-
//
// Phoenix CMC: main program
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use anyhow::Result;
use log::info;
use phoenix_cmc::{VERSION, server, server::Server};
use std::env;
use std::path::PathBuf;
use tokio::signal;

const DEFAULT_PORT: u16 = 9999;
const LIBDIR: &str = "phoenix";

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init(); // XXX Should logfile use non-blocking code instead?

    println!("=== DEBUG: Starting main() ===");

    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut port = DEFAULT_PORT;
    let mut cron = false;
    let mut debug = false;

    let usage = format!("Usage: {program} [--cron] [--debug] [--port {DEFAULT_PORT}]");

    // Parse command-line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                println!("{usage}");
                return Ok(());
            }
            "--version" => {
                println!("Phoenix {VERSION} (Rust version)");
                return Ok(());
            }
            "--cron" => cron = true,
            "--debug" => debug = true,
            "--port" => {
                i += 1;
                if i < args.len() {
                    port = args[i].parse()?;
                } else {
                    eprintln!("{usage}");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("{usage}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // If --cron option was given, check if the listening port is busy
    if cron && server::is_port_busy(port).await {
        println!("=== DEBUG: Port {port} is busy, exiting (cron mode) ===");
        return Ok(());
    }

    println!("=== DEBUG: Parsed args - port: {port}, cron: {cron}, debug: {debug} ===");

    // Change to LIBDIR (create if necessary)
    let libdir = PathBuf::from(LIBDIR);
    if !libdir.exists() {
        println!("=== DEBUG: Creating libdir: {:?} ===", libdir);
        std::fs::create_dir(&libdir)?;
    }
    println!("=== DEBUG: Changing to libdir: {:?} ===", libdir);
    env::set_current_dir(&libdir)?;

    // Create logs subdirectory
    let logs_dir = PathBuf::from("logs"); // XXX log file
    if !logs_dir.exists() {
        println!("=== DEBUG: Creating logs dir: {:?} ===", logs_dir);
        std::fs::create_dir(&logs_dir)?;
    }

    // Initialize server
    println!("=== DEBUG: Creating server on port {port} ===");
    let (server, server_obj) = Server::new(port, debug).await?;
    println!("=== DEBUG: Server created successfully ===");
    let pid = std::process::id();
    info!("Started Phoenix server, version {VERSION}.");
    info!("Listening for connections on TCP port {port}. (pid {pid})");

    // Set up signal handlers
    let sigint = signal::ctrl_c();
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
    let mut sigquit = signal::unix::signal(signal::unix::SignalKind::quit())?;

    // Main event loop
    println!("=== DEBUG: Starting main event loop ===");
    tokio::select! {
        result = server_obj.run() => {
            println!("=== DEBUG: Server.run() returned: {:?} ===", result);
            if let Err(e) = result {
                log::error!("Server error: {e}"); // XXX print error message
                return Err(e);
            }
        }
        _ = sigint => info!("Received SIGINT, shutting down..."),
        _ = sigterm.recv() => info!("Received SIGTERM, shutting down..."),
        _ = sigquit.recv() => {
            info!("Received SIGQUIT, initiating shutdown...");
            server.schedule_shutdown("signal".into(), 5).await;
        }
    }

    Ok(())
}

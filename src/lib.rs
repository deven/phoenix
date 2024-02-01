// -*- Rust -*-
//
// Phoenix CMC library: crate root
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

pub mod actor;
pub mod client;
pub mod error;
pub mod event;
pub mod file;
pub mod server;
pub mod session;
pub mod user;

use crate::error::PhoenixError;
use async_backtrace::taskdump_tree;
use clap::Parser;
use tracing::{trace, warn};

#[derive(Debug, Clone, Parser)]
pub struct Options {
    /// Running from cron to restart server
    #[arg(long)]
    pub cron: bool,

    /// Enable debug mode
    #[arg(long)]
    pub debug: bool,

    /// Use IPv6 instead of IPv4
    #[arg(long)]
    pub ipv6: bool,

    /// Set listening port number
    #[arg(long, default_value = "9999")]
    pub port: u16,
}

#[tokio::main]
pub async fn run(opts: Options) -> Result<(), PhoenixError> {
    trace!("phoenix_cmc::run()\n{}", taskdump_tree(true));

    let server = server::Server::new(opts);

    server.run().await?;

    trace!("phoenix_cmc::run()\n{}", taskdump_tree(true));

    Ok(())
}

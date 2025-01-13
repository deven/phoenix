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
pub mod config;
pub mod discussion;
pub mod error;
pub mod event;
pub mod file;
pub mod name;
pub mod sendlist;
pub mod server;
pub mod session;
pub mod user;

use crate::config::Options;
use crate::error::PhoenixError;
use async_backtrace::taskdump_tree;
use tracing::{trace, warn};

#[tokio::main]
pub async fn run(opts: Options) -> Result<(), PhoenixError> {
    trace!("phoenix_cmc::run()\n{}", taskdump_tree(true));

    let server = server::Server::new(opts);

    server.run().await?;

    trace!("phoenix_cmc::run()\n{}", taskdump_tree(true));

    Ok(())
}

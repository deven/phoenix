// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: crate root
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

mod client;
mod server;

use async_backtrace::taskdump_tree;
use clap::Parser;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use tracing::{trace, warn};

#[derive(Debug, Parser)]
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
pub async fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    trace!("phoenix_cmc::run()\n{}", taskdump_tree(true));

    let server = server::Server::new(opts);

    server.run().await?;

    trace!("phoenix_cmc::run()\n{}", taskdump_tree(true));

    Ok(())
}

#[derive(Debug)]
pub enum PhoenixError {
    //RequestError(reqwest::Error),
    FileIoError {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Error for PhoenixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            //PhoenixError::RequestError(err) => Some(err),
            PhoenixError::FileIoError { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for PhoenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //PhoenixError::RequestError(err) => write!(f, "Request error: {}", err),
            PhoenixError::FileIoError { path, source } => {
                write!(f, "File I/O error for path {}: {}", path.display(), source)
            }
        }
    }
}

//impl From<reqwest::Error> for PhoenixError {
//    fn from(error: reqwest::Error) -> Self {
//        PhoenixError::RequestError(error)
//    }
//}

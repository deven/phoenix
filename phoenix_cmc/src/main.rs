// -*- Rust -*-
//
// Phoenix CMC: main program
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

use clap::Parser;
use phoenix_cmc::Options;
use std::error::Error;
use std::process;
use tracing::trace;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

fn setup_tracing(directive: &str) -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(directive.parse()?))
        .with_span_events(FmtSpan::FULL)
        .init();
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    setup_tracing("phoenix_cmc=trace")?;

    let opts = Options::parse();
    trace!("{opts:?}");
    Ok(phoenix_cmc::run(opts)?)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

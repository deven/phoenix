// -*- Rust -*-
//
// $Id$
//
// Main program.
//
// Copyright 2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// Running from cron to restart server
    #[structopt(long)]
    cron: bool,

    /// Enable debug mode
    #[structopt(long)]
    debug: bool,

    /// Set listening port number
    #[structopt(long, default_value = "9999")]
    port: u16,
}

fn main() {
    let opts = Opts::from_args();
    println!("opts: {:?}", opts);
}

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

use std::net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// Running from cron to restart server
    #[structopt(long)]
    cron: bool,

    /// Enable debug mode
    #[structopt(long)]
    debug: bool,

    /// Use IPv6 instead of IPv4
    #[structopt(long)]
    ipv6: bool,

    /// Set listening port number
    #[structopt(long, default_value = "9999")]
    port: u16,
}

fn main() {
    let opts      = Opts::from_args();
    let socket    = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), opts.port);
    let _listener = TcpListener::bind(socket).unwrap();
    println!("opts: {:?}", opts);
}

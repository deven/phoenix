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

#[derive(Debug)]
struct Opts {
    /// Run from cron to restart server
    cron: bool,

    /// Enable debug mode
    debug: bool,

    /// Set listening port number
    port: u16,
}

fn main() {
    let opts = Opts {
        cron:  false,
        debug: false,
        port:  9999,
    };
    println!("opts: {:?}", opts);
}

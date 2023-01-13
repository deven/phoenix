// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC: main program
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![warn(rust_2018_idioms)]

use clap::Parser;
use phoenix_cmc::Options;
use std::process;

fn main() {
    let options = Options::parse();

    if let Err(e) = phoenix_cmc::run(options) {
        eprintln!("{e}");
        process::exit(1);
    }
}

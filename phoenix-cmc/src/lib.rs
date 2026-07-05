// -*- Rust -*-
//
// Phoenix CMC library: crate root
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

//#![recursion_limit = "256"]
#![deny(rust_2018_idioms, nonstandard_style, unused_must_use, clippy::all)]
//#![warn(future_incompatible, missing_docs, clippy::pedantic, clippy::cargo)]

pub mod atomic;
pub mod constants;
pub mod discussion;
pub mod name;
pub mod output;
pub mod sendlist;
pub mod server;
pub mod session;
pub mod telnet;
pub mod text;
pub mod timestamp;
pub mod user;

/// Phoenix server version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn getword(input: &str, separator: Option<char>) -> (&str, &str) {
    let input = input.trim_start();

    let end = if let Some(sep) = separator { input.find(|c: char| c.is_whitespace() || c == sep) } else { input.find(char::is_whitespace) };

    match end {
        Some(pos) => {
            let word = &input[..pos];
            let mut rest = input[pos..].trim_start();

            if let Some(sep) = separator {
                if let Some(stripped) = rest.strip_prefix(sep) {
                    rest = stripped.trim_start();
                }
            }

            (word, rest)
        }
        None => (input, ""),
    }
}

pub fn match_keyword<'a>(input: &'a str, keyword: &str, min: usize) -> Option<&'a str> {
    let (word, rest) = getword(input, None);
    let min = if min == 0 { keyword.len() } else { min };

    if word.len() >= min && word.len() <= keyword.len() {
        let keyword_prefix = &keyword[..word.len()];
        if word.eq_ignore_ascii_case(keyword_prefix) {
            return Some(rest);
        }
    }

    None
}

pub fn match_name(name: &str, sendlist: &str) -> Option<usize> {
    use crate::constants::*;

    if name.is_empty() || sendlist.is_empty() {
        return None;
    }

    let name_bytes = name.as_bytes();
    let sendlist_bytes = sendlist.as_bytes();

    for (start_pos, _) in name.char_indices() {
        let mut name_pos = start_pos;
        let mut send_pos = 0;

        while name_pos < name_bytes.len() && send_pos < sendlist_bytes.len() {
            let n = name_bytes[name_pos];
            let s = sendlist_bytes[send_pos];

            // Let an unquoted underscore match a space or an underscore
            if s == UNQUOTED_UNDERSCORE && (n == SPACE || n == UNDERSCORE) {
                name_pos += 1;
                send_pos += 1;
                continue;
            }

            if !n.eq_ignore_ascii_case(&s) {
                break;
            }

            name_pos += 1;
            send_pos += 1;
        }

        if send_pos == sendlist_bytes.len() {
            return Some(start_pos + 1);
        }
    }

    None
}

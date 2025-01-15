// -*- Rust -*-
//
// Phoenix CMC library: config module
//
// Copyright 2021-2025 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use clap::{parser::ValueSource, CommandFactory, FromArgMatches, Parser};
use config::Config;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

macro_rules! config {
    // Entry point: Transform public syntax into internal recursive form.
    ( $name:ident => { $($rest:tt)* } ) => {
        config!(@ ($name matches config partial) { $($rest)* } -> () () ());
    };

    // Use "default_value" for `=> "literal"` syntax.
    (@ ($($vars:tt)*) {
        $(#[$attr:meta])* $field:ident : $type:ty => $default:literal $(=> $env:ident)?, $($rest:tt)*
    } -> ($($fields:tt)*) ($($optional:tt)*) ($($overrides:tt)*)) => {
        config!(@ ($($vars)*) { $($rest)* } -> @ ($(#[$attr])*) $field ($type) (default_value = $default) $($env)? ($($fields)*) ($($optional)*) ($($overrides)*));
    };

    // Use "default_value_t" for `= expr` syntax.
    (@ ($($vars:tt)*) {
        $(#[$attr:meta])* $field:ident : $type:ty = $default:expr $(=> $env:ident)?, $($rest:tt)*
    } -> ($($fields:tt)*) ($($optional:tt)*) ($($overrides:tt)*)) => {
        config!(@ ($($vars)*) { $($rest)* } -> @ ($(#[$attr])*) $field ($type) (default_value_t = $default) $($env)? ($($fields)*) ($($optional)*) ($($overrides)*));
    };

    // Apply common transformations.
    (@ ($name:ident $matches:ident $config:ident $partial:ident) { $($rest:tt)* } -> @ ($(#[$attr:meta])*) $field:ident ($type:ty) ($($default:tt)*) $($env:ident)? ($($fields:tt)*) ($($optional:tt)*) ($($overrides:tt)*)) => {
        config!(@ ($name $matches $config $partial) { $($rest)* } -> (
            $($fields)*
            $(#[$attr])*
            #[arg(long, $(env = stringify!($env),)? $($default)*)]
            pub $field: $type,
        ) (
            $($optional)*
            pub $field: Option<$type>,
        ) (
            $($overrides)*
            if let Some(val) = $partial.$field {
                if $matches.value_source(stringify!($field)) == Some(ValueSource::DefaultValue) {
                    $config.$field = val;
                }
            }
        ));
    };

    // Terminal rule: Emit the final code.
    (@ ($name:ident $matches:ident $config:ident $partial:ident) { } -> ($($fields:tt)*) ($($optional:tt)*) ($($overrides:tt)*)) => {
        #[derive(Debug, Clone, Parser)]
        #[command(author, version, about, long_about = None)]
        pub struct $name
        where
            Self: Send + Sync + 'static,
        {
            $($fields)*
        }

        #[derive(Debug, Default, Deserialize)]
        pub struct PartialConfig {
            $($optional)*
        }

        impl $name {
            pub fn load() -> $name {
                // Parse command-line arguments and environment variables.
                let $matches = $name::command().get_matches();
                let mut $config = $name::from_arg_matches(&$matches).expect("Clap parse failed");

                // Load config file, if any.
                let $partial: PartialConfig = Config::builder()
                    .add_source(config::File::from($config.config_file.clone()).required(false))
                    .build()
                    .unwrap_or_default()
                    .try_deserialize()
                    .unwrap_or_default();

                // Override Clap default values with config file values.
                $($overrides)*

                $config
            }
        }
    };
}

config!(Options => {
    /// Running from cron to restart server
    cron: bool = false => PHOENIX_CRON_MODE,

    /// Enable debug mode
    debug: bool = false => PHOENIX_DEBUG,

    /// Use IPv6 instead of IPv4
    ipv6: bool = false => PHOENIX_USE_IPV6,

    /// Listening bind address
    bind_addr: SocketAddr => "0.0.0.0:9999" => PHOENIX_BIND_ADDR,

    /// Set listening port number
    port: u16 = 9999 => PHOENIX_TELNET_PORT,

    /// Library directory.
    lib_dir: PathBuf => "/var/lib/phoenix" => PHOENIX_LIB_DIR,

    /// Path to the configuration file
    config_file: PathBuf => "config.toml" => PHOENIX_CONFIG_FILE,

    //telnet_enabled: bool => true,
    //telnet_port: u16 => 9999,
    //telnet_login_timeout: u64 => 60,
    //terminal_width: u16 => 80,
    //terminal_height: u16 => 24,
    //terminal_min_width: u16 => 10,
    //guest_enabled: bool => false,
    //max_login_attempts: u32 => 3,
});

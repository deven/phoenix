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
use paste::paste;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

macro_rules! config {
    // Entry point: Transform public syntax into internal recursive form.
    ( $name:ident => { $($rest:tt)* } ) => {
        config!(@ ($name () matches config partial) { $($rest)* } -> {} () () (@ () {}));
    };

    // Use "default_value" for `=> "literal"` syntax.
    (@ ($($vars:tt)*) {
        $(#[$attr:meta])* $field:ident : $type:ty => $default:literal $(=> $env:ident)?, $($rest:tt)*
    } -> {
        $($output:tt)*
    } ($($fields:tt)*) ($($optional:tt)*) (@ ($($skip:tt)*) { $($output2:tt)* } $($overrides:tt)*)) => {
        config!(@ ($($vars)*) { $($rest)* } -> @ { ($(#[$attr])*) $field ($type) (default_value = $default) $($env)? } -> {
            $($output)*
            $($output2)*
        } ($($fields)*) ($($optional)*) ($($overrides)*));
    };

    // Use "default_value_t" for `= expr` syntax.
    (@ ($($vars:tt)*) {
        $(#[$attr:meta])* $field:ident : $type:ty = $default:expr $(=> $env:ident)?, $($rest:tt)*
    } -> {
        $($output:tt)*
    } ($($fields:tt)*) ($($optional:tt)*) (@ ($($skip:tt)*) { $($output2:tt)* } $($overrides:tt)*)) => {
        config!(@ ($($vars)*) { $($rest)* } -> @ { ($(#[$attr])*) $field ($type) (default_value_t = $default) $($env)? } -> {
            $($output)*
            $($output2)*
        } ($($fields)*) ($($optional)*) ($($overrides)*));
    };

    // Apply common transformations.
    (@ ($name:ident ($($path:tt)*) $matches:ident $config:ident $partial:ident) { $($rest:tt)* } -> @ {
        ($(#[$attr:meta])*) $field:ident ($type:ty) ($($default:tt)*) $($env:ident)?
    } -> {
        $($output:tt)*
    } ($($fields:tt)*) ($($optional:tt)*) ($($overrides:tt)*)) => {
        config!(@ ($name ($($path)*) $matches $config $partial) { $($rest)* } -> {
            $($output)*
        } (
            $($fields)*
            $(#[$attr])*
            #[arg(long = concat!($(stringify!($path), "-",)* stringify!($field)), $(env = stringify!($env),)? $($default)*)]
            pub $field: $type,
        ) (
            $($optional)*
            pub $field: Option<$type>,
        ) (
            @ () {}
            $($overrides)*
            if let Some(val) = $partial$(.$path)*.$field {
                if $matches.value_source(concat!($(stringify!($path), "-",)* stringify!($field))) == Some(ValueSource::DefaultValue) {
                    $config.$($path.)*$field = val;
                }
            }
        ));
    };

    // Handle nested sections.
    (@ ($name:ident ($($path:tt)*) $matches:ident $config:ident $partial:ident) {
        $(#[$attr:meta])* $section:ident => { $($nested:tt)* }, $($rest:tt)*
    } -> {
        $($output:tt)*
    } ($($fields:tt)*) ($($optional:tt)*) (@ ($($skip:tt)*) { $($output2:tt)* } $($overrides:tt)*)) => {
        paste! {
            config!(@ ($name ($($path)*) $matches $config $partial) { $($rest)* } -> {
                $($output)*
                $($output2)*
            } (
                $($fields)*
                $(#[$attr])*
                pub $section: [<$name $section:camel>],
            ) (
                $($optional)*
                pub $section: Option<[<Partial $name $section:camel>]>,
            ) (
                @ () {
                    // Generate nested section structs.
                    config!(@ ([<$name $section:camel>] ($($path)* $section) $matches $config $partial) { $($nested)* } -> {} () () ({}));
                }
                $($overrides)*
            ));
        }
    };

    // Generate completed structures.
    (@ ($name:ident $($vars:tt)*) { } -> {
        $($output:tt)*
    } ($($fields:tt)*) ($($optional:tt)*) (@ ($($skip:tt)*) { $($output2:tt)* } $($overrides:tt)*)) => {
        config!(@ ($name $($vars)*) -> {
            $($output)*
            $($output2)*
            paste! {
                #[derive(Debug, Clone, Parser)]
                #[command(author, version, about, long_about = None)]
                pub struct $name
                where
                    Self: Send + Sync + 'static,
                {
                    $($fields)*
                }

                #[derive(Debug, Default, Deserialize)]
                pub struct [<Partial $name>] {
                    $($optional)*
                }
            }
        } {
            $($overrides)*
        });
    };

    // Terminal rule: Emit the final code.
    (@ ($name:ident () $matches:ident $config:ident $partial:ident) -> {
        $($output:tt)*
    } {
        $($overrides:tt)*
    }) => {
        paste! {
            impl $name {
                pub fn load() -> $name {
                    // Parse command-line arguments and environment variables.
                    let $matches = $name::command().get_matches();
                    let mut $config = $name::from_arg_matches(&$matches).expect("Clap parse failed");

                    // Load config file, if any.
                    let $partial: [<Partial $name>] = Config::builder()
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
        }
    };
}

config!(Options => {
    debug: bool = false => PHOENIX_DEBUG,
    network => {
        telnet => {
            enabled: bool = true => PHOENIX_TELNET_ENABLED,
            port: u16 = 23,
            terminal => {
                width: 80,
                width: 24,
            },
        },
        ssh => {
            enabled: bool = false => PHOENIX_SSH_ENABLED,
            port: u16 = 22,
            terminal => {
                width: 135,
                width: 38,
            },
        },
    },
});

/*
config!(Options => {
    /// Running from cron to restart server
    cron: bool = false => PHOENIX_CRON_MODE,

    /// Enable debug mode
    debug: bool = false => PHOENIX_DEBUG,

    /// Enable guest login
    guest_enabled: bool => false,

    /// Maximum login attempts
    max_login_attempts: u16 => 3,

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

    /// TELNET protocol options
    telnet => {
        /// Enable TELNET protocol
        enabled: bool = true => PHOENIX_TELNET_ENABLED,

        /// Listening port for TELNET
        port: u16 = 23,

        /// Timeout for TELNET login
        login_timeout: u16 = 60,
    },

    /// Terminal options
    terminal => {
        /// Terminal width
        width: u16 => 80,

        /// Terminal height
        height: u16 => 24,

        /// Terminal minimum width
        min_width: u16 => 10,
    },
});
*/

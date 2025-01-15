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
        config!(@ $name matches config partial { $($rest)* } -> () () ());
    };

    // Nested section definition
    (@ $name:ident $matches:ident $config:ident $partial:ident {
        section $section:ident => { $($fields:tt)* }, $($rest:tt)*
    } -> ($($partial_structs:tt)*) ($($partial_assignments:tt)*) ($($fields_out:tt)*)) => {
        config!(@ $name $matches $config $partial { $($rest)* } -> (
            $($partial_structs)*
            #[derive(Debug, Default, Deserialize)]
            pub struct $section {
                $($fields)*
            }
        ) (
            $($partial_assignments)*
            if let Some(val) = $partial.$section {
                $config.$section = val;
            }
        ) (
            $($fields_out)*
            pub $section: $section,
        ));
    };

    // Case 1: `=> "literal"` or `=> "literal" => ENV`
    (@ $name:ident $matches:ident $config:ident $partial:ident {
        $( #[$attr:meta] )* $field:ident : $type:ty => $default:literal $(=> $env:ident)?, $($rest:tt)*
    } -> ($($partial_structs:tt)*) ($($partial_assignments:tt)*) ($($fields_out:tt)*)) => {
        config!(@ $name $matches $config $partial { $($rest)* } -> ($($partial_structs)*) ($($partial_assignments)*) (
            $($fields_out)*
            $( #[$attr] )*
            #[arg(long, $(env = stringify!($env),)? default_value = $default)]
            pub $field: $type,
        ));
    };


        section $section:ident => { $( $( #[$attr:meta] )* $field:ident : $type:ty = $default:expr $(=> $env:ident)?, )* },
        $($rest:tt)*
    } -> ($($partial_structs:tt)*) ($($partial_assignments:tt)*) ($($fields_out:tt)*)) => {
        config!(@ $name $matches $config $partial { $($rest)* } -> (
            $($partial_structs)*
            #[derive(Debug, Default, Deserialize)]
            pub struct $section {
                $(
                    $( #[$attr] )*
                    #[serde(default)]
                    pub $field: $type,
                )*
            }
        ) (
            $($partial_assignments)*
            if let Some(val) = $partial.$section {
                $config.$section = val;
            }
        ) (
            $($fields_out)*
            pub $section: $section,
        ));
    };


    // Case 2: `= expr` or `= expr => ENV`
    (@ $name:ident $matches:ident $config:ident $partial:ident {
        $( #[$attr:meta] )* $field:ident : $type:ty = $default:expr $(=> $env:ident)?, $($rest:tt)*
    } -> ($($partial_structs:tt)*) ($($partial_assignments:tt)*) ($($fields_out:tt)*)) => {
        config!(@ $name $matches $config $partial { $($rest)* } -> ($($partial_structs)*) ($($partial_assignments)*) (
            $($fields_out)*
            $( #[$attr] )*
            #[arg(long, $(env = stringify!($env),)? default_value_t = $default)]
            pub $field: $type,
        ));
    };

    // Terminal rule: Emit the final code.
    (@ $name:ident $matches:ident $config:ident $partial:ident { } -> ($($partial_structs:tt)*) ($($partial_assignments:tt)*) ($($fields_out:tt)*)) => {
        $($partial_structs)*

        #[derive(Debug, Default, Deserialize)]
        pub struct PartialConfig {
            $($fields_out)*
        }

        #[derive(Debug, Clone, Parser)]
        #[command(author, version, about, long_about = None)]
        pub struct $name {
            $($fields_out)*
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
                $($partial_assignments)*

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
    section telnet => {
        /// Enable TELNET protocol
        enabled: bool = true => PHOENIX_TELNET_ENABLED,

        /// Listening port for TELNET
        port: u16 = 23,

        /// Timeout for TELNET login
        login_timeout: u16 = 60,
    },

    /// Terminal options
    section terminal => {
        /// Terminal width
        width: u16 => 80,

        /// Terminal height
        height: u16 => 24,

        /// Terminal minimum width
        min_width: u16 => 10,
    },
});

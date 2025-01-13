// -*- Rust -*-
//
// Phoenix CMC library: config module
//
// Copyright 2021-2025 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use clap::{parser::ValueSource, ArgMatches, Parser};
use config::Config;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

macro_rules! define_config_field {
    ( $field:ident: $type:ty => $default:literal => $env_var:ident) => {
        #[arg(long, env = stringify!($env_var), default_value = $default)]
        pub $field: $type,
    };
    ( $field:ident: $type:ty => $default:literal) => {
        #[arg(long, default_value = $default)]
        pub $field: $type,
    };
    ( $field:ident: $type:ty = $default:expr => $env_var:ident) => {
        #[arg(long, env = stringify!($env_var), default_value_t = $default)]
        pub $field: $type,
    };
    ( $field:ident: $type:ty = $default:expr) => {
        #[arg(long, default_value_t = $default)]
        pub $field: $type,
    };
}

macro_rules! define_config {
    (
        $name:ident => {
            $(
                $field:ident: $type:tt $op:tt $default:expr $(=> $env_var:ident)?,
            )* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Parser)]
        #[command(author, version, about, long_about = None)]
        pub struct $name
        where
            Self: Send + Sync + 'static,
        {
            $(
                define_config_field!(
                    $field: $type $op $default $(=> $env_var)?
                ),
            )*
        }

        #[derive(Debug, Default, Deserialize)]
        pub struct PartialConfig {
            $(
                pub $field: Option<$type>,
            )*
        }

        impl $name {
            pub fn load() -> $name {
                // Parse command-line arguments and environment variables.
                let matches = $name::command().get_matches();
                let mut config = $name::from_arg_matches(&matches).expect("Clap parse failed");

                // Load config file, if any.
                let partial: PartialConfig = Config::builder()
                    .add_source(config::File::from(config.config_file.clone()).required(false))
                    .build()
                    .unwrap_or_default()
                    .try_deserialize()
                    .unwrap_or_default();

                // Override Clap default values with config file values.
                $(
                    if let Some(val) = partial.$field {
                        if matches.value_source(stringify!($field)) == Some(ValueSource::DefaultValue) {
                            config.$field = val;
                        }
                    }
                )*

                config
            }
        }
    };
}

define_config!(Options => {
    cron: bool = false => PHOENIX_CRON_MODE,
    debug: bool = false => PHOENIX_DEBUG,
    ipv6: bool = false => PHOENIX_USE_IPV6,
    bind_addr: SocketAddr => "0.0.0.0:9999" => PHOENIX_BIND_ADDR,
    port: u16 = 9999 => PHOENIX_TELNET_PORT,
    lib_dir: PathBuf => "/var/lib/phoenix" => PHOENIX_LIB_DIR,
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

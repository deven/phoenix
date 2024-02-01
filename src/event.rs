// -*- Rust -*-
//
// Phoenix CMC library: event module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::session::Session;
use crate::client::Client;
use crate::server::Server;
use async_backtrace::framed;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::time::sleep;
use tracing::{info, warn};

const BELL: char = '\x07';
const FINAL_WARNING_DELAY: Duration = Duration::from_secs(3);

// Use the macros defined in the "macros" module below.
use macros::*;

/// Event handle.
#[derive(Debug, Clone)]
pub struct EventRef(Arc<RwLock<Event>>);

#[derive(Debug)]
pub enum Event {
    Message {
        timestamp: DateTime<Utc>,
        sender: Session,
        message: Arc<str>,
    },
    EntryNotify {
        timestamp: DateTime<Utc>,
        name: Arc<str>,
    },
    ExitNotify {
        timestamp: DateTime<Utc>,
        name: Arc<str>,
    },
    Shutdown {
        timestamp: DateTime<Utc>,
        server: Server,
        by: Arc<str>,
        delay: Duration,
    },
    Restart {
        timestamp: DateTime<Utc>,
        server: Server,
        by: Arc<str>,
        delay: Duration,
    },
    LoginTimeout {
        timestamp: DateTime<Utc>,
        client: Client,
    },
}

constructor!(Message, sender: Session, message: Arc<str>);
constructor!(EntryNotify, name: Arc<str>);
constructor!(ExitNotify, name: Arc<str>);
constructor!(Shutdown, server: Server, by: Arc<str>, delay: Duration);
constructor!(Restart, server: Server, by: Arc<str>, delay: Duration);
constructor!(LoginTimeout, client: Client);

impl EventRef {
    /// Create a new event handle.
    pub fn new(event: Event) -> Self {
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Obtain read lock on the event data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, Event> {
        self.0.read().await
    }

    /// Obtain write lock on the event data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, Event> {
        self.0.write().await
    }

    attr!(client, set_client, Client, [LoginTimeout]);
    attr!(message, set_message, Into<Arc<str>>, [Message]);
    attr!(name, set_name, Into<Arc<str>>, [EntryNotify, ExitNotify]);
    attr!(delay, set_delay, Copy Duration, [Shutdown, Restart]);
    attr!(sender, set_sender, Session, [Message]);
    attr!(timestamp, set_timestamp, DateTime<Utc>, [*]);

    #[framed]
    pub async fn shutdown_or_restart(
        &self,
        server: Server,
        by: Arc<str>,
        delay: Duration,
        restart: bool,
    ) -> Result<(), EventError> {
        let operation = if restart { "restart" } else { "shutdown" };

        if delay.is_zero() {
            warn!("Immediate server {operation} requested by {by}.");
        } else {
            let secs = delay.as_secs();
            let nanos = delay.subsec_nanos();
            let secs = if nanos == 0 {
                format!("{secs}")
            } else {
                let nanos = format!("{nanos:0>9}");
                format!("{secs}.{}", nanos.trim_end_matches('0'))
            };

            warn!("Server {operation} requested by {by} in {secs} seconds.");
            server
                .announce(format!(
                    "{BELL}>>> This server will {operation} in {secs} seconds... <<<\n{BELL}"
                ))
                .await;
            sleep(delay).await;
        }

        info!("Final {operation} warning.");

        if restart {
            server
                .announce("{BELL}>>> Server restarting NOW!  Goodbye. <<<\n{BELL}")
                .await;
        } else {
            server
                .announce("{BELL}>>> Server shutting down NOW!  Goodbye. <<<\n{BELL}")
                .await;
        }

        sleep(FINAL_WARNING_DELAY).await;

        if restart {
            server.restart().await;
        } else {
            server.shutdown().await;
        }

        Ok(())
    }

    #[framed]
    pub async fn execute(&self) -> Result<(), EventError> {
        let event = self.read().await;
        match &*event {
            Event::Shutdown {
                server, by, delay, ..
            } => {
                self.shutdown_or_restart(server.clone(), by.clone(), *delay, false)
                    .await
            },
            Event::Restart {
                server, by, delay, ..
            } => {
                self.shutdown_or_restart(server.clone(), by.clone(), *delay, true)
                    .await
            },
            _ => Err(EventError::invalid_variant("execute", self.clone())),
        }
    }
}

#[derive(Debug)]
pub enum EventError {
    InvalidVariant {
        method: &'static str,
        event: EventRef,
    },
}

impl EventError {
    pub fn invalid_variant(method: &'static str, event: EventRef) -> Self {
        Self::InvalidVariant { method, event }
    }
}

impl Error for EventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let called = "() called on invalid event variant: ";
        match self {
            Self::InvalidVariant { method, event } => {
                write!(f, "Method {method}{called}{event:#?}")
            }
        }
    }
}

mod macros {
    macro_rules! add_ref_mut {
        ($variant:ident { $($fields:ident),* }) => {
            Event::$variant { $( $fields ),* }
        };
        ($variant:ident { $($fields:ident),*, .. }) => {
            Event::$variant { $( $fields ),*, .. }
        };
        (mut $variant:ident { $($fields:ident),* }) => {
            Event::$variant { $( ref mut $fields ),* }
        };
        (mut $variant:ident { $($fields:ident),*, .. }) => {
            Event::$variant { $( ref mut $fields ),*, .. }
        };
    }

    macro_rules! attr {
        ($field:ident, $setter:ident, Into<$type:ty>, $variants:tt) => {
            method!($field() -> $type { Ok($field.clone()) } => $variants { $field, .. });
            method!(mut $setter[<T: Into<$type>>]($setter: T) -> () { *$field = $setter.into(); Ok(()) } => $variants { $field, .. });
        };
        ($field:ident, $setter:ident, $type:ty, $variants:tt) => {
            method!($field() -> $type { Ok($field.clone()) } => $variants { $field, .. });
            method!(mut $setter($setter: $type) -> () { *$field = $setter; Ok(()) } => $variants { $field, .. });
        };
        ($field:ident, $setter:ident, Copy $type:ty, $variants:tt) => {
            method!($field() -> $type { Ok(*$field) } => $variants { $field, .. });
            method!(mut $setter($setter: $type) -> () { *$field = $setter; Ok(()) } => $variants { $field, .. });
        };
    }

    macro_rules! constructor {
        ($name:ident, $($field:ident: $type:ty),*) => {
            impl Event {
                #[allow(non_snake_case)]
                pub fn $name($($field: $type),*) -> Self {
                    Event::$name {
                        timestamp: Utc::now(),
                        $($field),*,
                    }
                }
            }
        };
    }

    macro_rules! fix_alternations {
        (| $($rest:tt)*) => { $($rest)* };
    }

    macro_rules! method {
        ($method:ident [$($generics:tt)*] ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!($method, [$($generics)*], ($($args)*), $return, $body, $variants $fields);
        };
        (mut $method:ident [$($generics:tt)*] ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!(mut $method, [$($generics)*], ($($args)*), $return, $body, $variants $fields);
        };
        ($method:ident ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!($method, [], ($($args)*), $return, $body, $variants $fields);
        };
        (mut $method:ident ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!(mut $method, [], ($($args)*), $return, $body, $variants $fields);
        };
    }

    macro_rules! method_impl {
        ($method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [*] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let event = self.read().await;
                match &*event {
                    add_ref_mut!(Message $fields)
                        | add_ref_mut!(EntryNotify $fields)
                        | add_ref_mut!(ExitNotify $fields)
                        | add_ref_mut!(Shutdown $fields)
                        | add_ref_mut!(Restart $fields)
                        | add_ref_mut!(LoginTimeout $fields) => $body,
                }
            }
        };
        ($method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [$($variant:ident),+] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let event = self.read().await;
                match &*event {
                    fix_alternations!($( | add_ref_mut!($variant $fields) )*) => $body,
                    _ => Err(EventError::invalid_variant(stringify!($method), self.clone())),
                }
            }
        };
        (mut $method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [*] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let mut event = self.write().await;
                match *event {
                    add_ref_mut!(mut Message $fields)
                        | add_ref_mut!(mut EntryNotify $fields)
                        | add_ref_mut!(mut ExitNotify $fields)
                        | add_ref_mut!(mut Shutdown $fields)
                        | add_ref_mut!(mut Restart $fields)
                        | add_ref_mut!(mut LoginTimeout $fields) => $body,
                }
            }
        };
        (mut $method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [$($variant:ident),+] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let mut event = self.write().await;
                match *event {
                    fix_alternations!($( | add_ref_mut!(mut $variant $fields) )*) => $body,
                    _ => Err(EventError::invalid_variant(stringify!($method), self.clone())),
                }
            }
        };
    }

    pub(crate) use add_ref_mut;
    pub(crate) use attr;
    pub(crate) use constructor;
    pub(crate) use fix_alternations;
    pub(crate) use method;
    pub(crate) use method_impl;
}

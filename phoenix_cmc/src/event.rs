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
use async_backtrace::framed;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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
        seconds: u16,
    },
    Restart {
        timestamp: DateTime<Utc>,
        seconds: u16,
    },
    LoginTimeout {
        timestamp: DateTime<Utc>,
        client: Client,
    },
}

constructor!(Message, sender: Session, message: Arc<str>);
constructor!(EntryNotify, name: Arc<str>);
constructor!(ExitNotify, name: Arc<str>);
constructor!(Shutdown, seconds: u16);
constructor!(Restart, seconds: u16);
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
    attr!(seconds, set_seconds, Copy u16, [Shutdown, Restart]);
    attr!(sender, set_sender, Session, [Message]);
    attr!(timestamp, set_timestamp, DateTime<Utc>, [*]);
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
            Self::InvalidVariant { method, event } => write!(f, "Method {method}{called}{event:#?}"),
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

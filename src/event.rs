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

event_constructor!(Message, sender: Session, message: Arc<str>);
event_constructor!(EntryNotify, name: Arc<str>);
event_constructor!(ExitNotify, name: Arc<str>);
event_constructor!(Shutdown, seconds: u16);
event_constructor!(Restart, seconds: u16);
event_constructor!(LoginTimeout, client: Client);

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

    attr!(client, set_client, Client, Clone, [LoginTimeout]);
    attr!(message, set_message, Arc<str>, Into, [Message]);
    attr!(name, set_name, Arc<str>, Into, [EntryNotify, ExitNotify]);
    attr!(seconds, set_seconds, u16, Copy, [Shutdown, Restart]);
    attr!(sender, set_sender, Session, Clone, [Message]);
    attr!(timestamp, set_timestamp, DateTime<Utc>, Copy, [*]);
}

#[derive(Debug)]
pub enum EventError {
    InvalidGetter {
        getter: &'static str,
        event: EventRef,
    },
    InvalidSetter {
        setter: &'static str,
        event: EventRef,
    },
}

impl EventError {
    pub fn invalid_getter(getter: &'static str, event: EventRef) -> Self {
        Self::InvalidGetter { getter, event }
    }

    pub fn invalid_setter(setter: &'static str, event: EventRef) -> Self {
        Self::InvalidSetter { setter, event }
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
            Self::InvalidGetter { getter, event } => write!(f, "Getter {getter}{called}{event:#?}"),
            Self::InvalidSetter { setter, event } => write!(f, "Setter {setter}{called}{event:#?}"),
        }
    }
}

mod macros {
    macro_rules! event_constructor {
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

    macro_rules! attr {
        ($getter:ident, $setter:ident, $type:ty, Into, $variants:tt) => {
            getter_impl!($getter, $type, ($getter.clone()), $variants);
            setter_impl!($getter, $setter, Into<$type>, $variants);
        };
        ($getter:ident, $setter:ident, $type:ty, Clone, $variants:tt) => {
            getter_impl!($getter, $type, ($getter.clone()), $variants);
            setter_impl!($getter, $setter, $type, $variants);
        };
        ($getter:ident, $setter:ident, $type:ty, Copy, $variants:tt) => {
            getter_impl!($getter, $type, (*$getter), $variants);
            setter_impl!($getter, $setter, $type, $variants);
        };
    }

    macro_rules! getter_impl {
        ($getter:ident, $type:ty, $clone:tt, [*]) => {
            #[framed]
            pub async fn $getter(&self) -> Result<$type, EventError> {
                let event = self.read().await;
                match &*event {
                    Event::Message { $getter, .. }
                    | Event::EntryNotify { $getter, .. }
                    | Event::ExitNotify { $getter, .. }
                    | Event::Shutdown { $getter, .. }
                    | Event::Restart { $getter, .. }
                    | Event::LoginTimeout { $getter, .. } => Ok($clone),
                }
            }
        };
        ($getter:ident, $type:ty, $clone:tt, [$($variant:ident),*]) => {
            #[framed]
            pub async fn $getter(&self) -> Result<$type, EventError> {
                let event = self.read().await;
                match &*event {
                    $(
                        Event::$variant { $getter, .. } => Ok($clone),
                    )*
                    _ => Err(EventError::invalid_getter(stringify!($getter), self.clone())),
                }
            }
        };
    }

    macro_rules! setter_impl {
        ($field:ident, $setter:ident, Into<$type:ty>, [*]) => {
            #[framed]
            pub async fn $setter<T: Into<$type>>(&self, $setter: T) -> Result<(), EventError> {
                let mut event = self.write().await;
                match *event {
                    Event::Message { ref mut $field, .. }
                    | Event::EntryNotify { ref mut $field, .. }
                    | Event::ExitNotify { ref mut $field, .. }
                    | Event::Shutdown { ref mut $field, .. }
                    | Event::Restart { ref mut $field, .. }
                    | Event::LoginTimeout { ref mut $field, .. } => {
                        *$field = $setter.into();
                        Ok(())
                    }
                }
            }
        };
        ($field:ident, $setter:ident, Into<$type:ty>, [$($variant:ident),*]) => {
            #[framed]
            pub async fn $setter<T: Into<$type>>(&self, $setter: T) -> Result<(), EventError> {
                let mut event = self.write().await;
                match *event {
                    $(
                        Event::$variant { ref mut $field, .. } => {
                            *$field = $setter.into();
                            Ok(())
                        }
                    ),*
                    _ => Err(EventError::invalid_setter(stringify!($setter), self.clone())),
                }
            }
        };
        ($field:ident, $setter:ident, $type:ty, [*]) => {
            #[framed]
            pub async fn $setter(&self, $setter: $type) -> Result<(), EventError> {
                let mut event = self.write().await;
                match *event {
                    Event::Message { ref mut $field, .. }
                    | Event::EntryNotify { ref mut $field, .. }
                    | Event::ExitNotify { ref mut $field, .. }
                    | Event::Shutdown { ref mut $field, .. }
                    | Event::Restart { ref mut $field, .. }
                    | Event::LoginTimeout { ref mut $field, .. } => {
                        *$field = $setter;
                        Ok(())
                    }
                }
            }
        };
        ($field:ident, $setter:ident, $type:ty, [$($variant:ident),*]) => {
            #[framed]
            pub async fn $setter(&self, $setter: $type) -> Result<(), EventError> {
                let mut event = self.write().await;
                match *event {
                    $(
                        Event::$variant { ref mut $field, .. } => {
                            *$field = $setter;
                            Ok(())
                        }
                    ),*
                    _ => Err(EventError::invalid_setter(stringify!($setter), self.clone())),
                }
            }
        };
    }

    pub(crate) use {attr, event_constructor, getter_impl, setter_impl};
}

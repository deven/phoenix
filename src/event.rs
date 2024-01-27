// -*- Rust -*-
//
// $Id$
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
    Message { sender: Session, message: Arc<str> },
    EntryNotify { name: Arc<str> },
    ExitNotify { name: Arc<str> },
    Shutdown { seconds: u16 },
    Restart { seconds: u16 },
    LoginTimeout { client: Client },
}

impl EventRef {
    /// Create a new message event.
    pub fn new_message<T: Into<Arc<str>>>(sender: Session, message: T) -> Self {
        let message = message.into();
        let event = Event::Message { sender, message };
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Create a new entry notification event.
    pub fn new_entry_notify<T: Into<Arc<str>>>(name: T) -> Self {
        let name = name.into();
        let event = Event::EntryNotify { name };
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Create a new exit notification event.
    pub fn new_exit_notify<T: Into<Arc<str>>>(name: T) -> Self {
        let name = name.into();
        let event = Event::ExitNotify { name };
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Create a new shutdown event.
    pub fn new_shutdown_event(seconds: u16) -> Self {
        let event = Event::Shutdown { seconds };
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Create a new restart event.
    pub fn new_restart_event(seconds: u16) -> Self {
        let event = Event::Restart { seconds };
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Create a new login timeout event.
    pub fn new_login_timeout_event(client: Client) -> Self {
        let event = Event::LoginTimeout { client };
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
    macro_rules! attr {
        ($getter:ident, $setter:ident, $type:ty, Into, [$($variant:ident),*]) => {
            getter_impl!($getter, $type, ($getter.clone()), [$($variant),*]);
            setter_impl!($getter, $setter, Into<$type>, [$($variant),*]);
        };
        ($getter:ident, $setter:ident, $type:ty, Clone, [$($variant:ident),*]) => {
            getter_impl!($getter, $type, ($getter.clone()), [$($variant),*]);
            setter_impl!($getter, $setter, $type, [$($variant),*]);
        };
        ($getter:ident, $setter:ident, $type:ty, Copy, [$($variant:ident),*]) => {
            getter_impl!($getter, $type, (*$getter), [$($variant),*]);
            setter_impl!($getter, $setter, $type, [$($variant),*]);
        };
    }

    macro_rules! getter_impl {
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

    pub(crate) use {attr, getter_impl, setter_impl};
}

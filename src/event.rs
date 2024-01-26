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

    #[framed]
    pub async fn client(&self) -> Result<Client, EventError> {
        let event = self.read().await;
        match &*event {
            Event::LoginTimeout { client } => Ok(client.clone()),
            _ => Err(EventError::new_invalid_getter("client", self.clone())),
        }
    }

    #[framed]
    pub async fn set_client(&self, new_client: Client) -> Result<(), EventError> {
        let mut event = self.write().await;
        match *event {
            Event::LoginTimeout { ref mut client } => {
                *client = new_client;
                Ok(())
            }
            _ => Err(EventError::new_invalid_setter("client", self.clone())),
        }
    }

    #[framed]
    pub async fn message(&self) -> Result<Arc<str>, EventError> {
        let event = self.read().await;
        match &*event {
            Event::Message { message, .. } => Ok(message.clone()),
            _ => Err(EventError::new_invalid_getter("message", self.clone())),
        }
    }

    #[framed]
    pub async fn set_message<T: Into<Arc<str>>>(&self, new_message: T) -> Result<(), EventError> {
        let mut event = self.write().await;
        match *event {
            Event::Message { ref mut message, .. } => {
                *message = new_message.into();
                Ok(())
            }
            _ => Err(EventError::new_invalid_setter("message", self.clone())),
        }
    }

    #[framed]
    pub async fn name(&self) -> Result<Arc<str>, EventError> {
        let event = self.read().await;
        match &*event {
            Event::EntryNotify { name } => Ok(name.clone()),
            Event::ExitNotify { name } => Ok(name.clone()),
            _ => Err(EventError::new_invalid_getter("name", self.clone())),
        }
    }

    #[framed]
    pub async fn set_name<T: Into<Arc<str>>>(&self, new_name: T) -> Result<(), EventError> {
        let mut event = self.write().await;
        match *event {
            Event::EntryNotify { ref mut name } => {
                *name = new_name.into();
                Ok(())
            }
            Event::ExitNotify { ref mut name } => {
                *name = new_name.into();
                Ok(())
            }
            _ => Err(EventError::new_invalid_setter("name", self.clone())),
        }
    }

    #[framed]
    pub async fn seconds(&self) -> Result<u16, EventError> {
        let event = self.read().await;
        match &*event {
            Event::Shutdown { seconds } => Ok(*seconds),
            Event::Restart { seconds } => Ok(*seconds),
            _ => Err(EventError::new_invalid_getter("seconds", self.clone())),
        }
    }

    #[framed]
    pub async fn set_seconds(&self, new_seconds: u16) -> Result<(), EventError> {
        let mut event = self.write().await;
        match *event {
            Event::Shutdown { ref mut seconds } => {
                *seconds = new_seconds;
                Ok(())
            }
            Event::Restart { ref mut seconds } => {
                *seconds = new_seconds;
                Ok(())
            }
            _ => Err(EventError::new_invalid_setter("seconds", self.clone())),
        }
    }

    #[framed]
    pub async fn sender(&self) -> Result<Session, EventError> {
        let event = self.read().await;
        match &*event {
            Event::Message { sender, .. } => Ok(sender.clone()),
            _ => Err(EventError::new_invalid_getter("sender", self.clone())),
        }
    }

    #[framed]
    pub async fn set_sender(&self, new_sender: Session) -> Result<(), EventError> {
        let mut event = self.write().await;
        match *event {
            Event::Message { ref mut sender, .. } => {
                *sender = new_sender;
                Ok(())
            }
            _ => Err(EventError::new_invalid_setter("sender", self.clone())),
        }
    }
}

#[derive(Debug)]
pub enum EventError {
    InvalidGetter { attr: &'static str, event: EventRef },
    InvalidSetter { attr: &'static str, event: EventRef },
}

impl EventError {
    pub fn new_invalid_getter(attr: &'static str, event: EventRef) -> Self {
        Self::InvalidGetter { attr, event }
    }

    pub fn new_invalid_setter(attr: &'static str, event: EventRef) -> Self {
        Self::InvalidSetter { attr, event }
    }
}

impl Error for EventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGetter { attr, event } => write!(
                f,
                "Getter {attr}() called on invalid event variant: {event:#?}"
            ),
            Self::InvalidSetter { attr, event } => write!(
                f,
                "Setter set_{attr}() called on invalid event variant: {event:#?}"
            ),
        }
    }
}

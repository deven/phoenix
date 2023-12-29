// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: client::session module
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::actor::{Actor, ActorInner};
use async_backtrace::{frame, framed};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::warn;

/// Session actor handle.
#[derive(Debug, Clone)]
pub struct Session {
    actor_tx: mpsc::Sender<SessionMsg>,
    state_rx: watch::Receiver<Arc<SessionState>>,
}

impl Session {
    /// Create a new instance of `Session`.
    pub fn new() -> Self {
        let (inner, actor_tx, state_rx) = SessionInner::new();
        tokio::spawn(frame!(async move { inner.run().await }));
        Self { actor_tx, state_rx }
    }

    /// Get username.
    pub fn username(&self) -> Result<Arc<str>, SessionError> {
        self.state_rx
            .borrow()
            .username
            .clone()
            .ok_or(SessionError::UsernameNotFound)
    }

    #[framed]
    /// Set username.
    pub async fn set_username(
        &self,
        username: Option<String>,
    ) -> Result<Option<Arc<str>>, SessionError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.actor_tx.send(SessionMsg::SetUsername(response_tx, username)).await?;
        response_rx.await?
    }
}

impl Actor for Session {
    type Error = SessionError;
}

/// Session actor state.
#[derive(Debug, Clone, Default)]
pub struct SessionState {
    pub username: Option<Arc<str>>,
}

impl SessionState {
    /// Create a new instance of `SessionState`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Session actor implementation.
#[derive(Debug)]
struct SessionInner {
    actor_rx: mpsc::Receiver<SessionMsg>,
    state_tx: watch::Sender<Arc<SessionState>>,
    state: Arc<SessionState>,
}

impl SessionInner {
    /// Create a new instance of `SessionInner` and associated channels.
    fn new() -> (Self, mpsc::Sender<SessionMsg>, watch::Receiver<Arc<SessionState>>) {
        let state = Arc::from(SessionState::new());

        let (actor_tx, actor_rx) = mpsc::channel(8);
        let (state_tx, state_rx) = watch::channel(state.clone());

        let inner = Self {
            actor_rx,
            state_tx,
            state,
        };

        (inner, actor_tx, state_rx)
    }

    /// Handle a message sent from a `Session` handle.
    #[framed]
    async fn handle_message(&mut self, msg: SessionMsg) -> Result<(), SessionError> {
        let _ = match msg {
            SessionMsg::SetUsername(respond_to, username) => {
                respond_to.send(self.update_username(username))
            }
        };
        Ok(())
    }

    /// Update username.
    fn update_username(
        &mut self,
        new_username: Option<String>,
    ) -> Result<Option<Arc<str>>, SessionError> {
        let new_username = new_username.map(|s| Arc::from(s.into_boxed_str()));

        Arc::make_mut(&mut self.state).username = new_username.clone();
        self.state = self.state_tx.send_replace(self.state.clone());

        let old_username = self.state.username.clone();
        Arc::make_mut(&mut self.state).username = new_username;

        Ok(old_username)
    }
}

impl ActorInner for SessionInner {
    type Error = SessionError;

    /// Run session actor task.
    #[framed]
    async fn run(mut self) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        while let Some(msg) = self.actor_rx.recv().await {
            let debug_msg = format!("{msg:?}");
            if let Err(e) = self.handle_message(msg).await {
                warn!("Error handling {debug_msg}: {e:?}");
            }
        }
        Ok(())
    }
}

/// Session actor message.
#[derive(Debug)]
pub enum SessionMsg {
    SetUsername(
        oneshot::Sender<Result<Option<Arc<str>>, SessionError>>,
        Option<String>,
    ),
}

type SendError = mpsc::error::SendError<SessionMsg>;
type RecvError = oneshot::error::RecvError;

/// Session actor error.
#[derive(Debug)]
pub enum SessionError {
    TxError(SendError),
    RxError(RecvError),
    UsernameNotFound,
}

impl Error for SessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TxError(err) => err.source(),
            Self::RxError(err) => err.source(),
            Self::UsernameNotFound => None,
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TxError(err) => err.fmt(f),
            Self::RxError(err) => err.fmt(f),
            Self::UsernameNotFound => write!(f, "Username not found!"),
        }
    }
}

impl From<SendError> for SessionError {
    fn from(err: SendError) -> Self {
        Self::TxError(err)
    }
}

impl From<RecvError> for SessionError {
    fn from(err: RecvError) -> Self {
        Self::RxError(err)
    }
}

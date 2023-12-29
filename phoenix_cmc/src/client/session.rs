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
use std::error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::warn;

#[derive(Debug, Clone)]
struct SessionState {
    username: Option<Arc<str>>,
}

impl SessionState {
    pub fn new() -> Self {
        let username = None;
        Self { username }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    tx: mpsc::Sender<SessionMsg>,
    state_rx: watch::Receiver<Arc<SessionState>>,
}

impl Session {
    pub fn username(&self) -> Option<Arc<str>> {
        self.state_rx.borrow().username.clone()
    }

    #[framed]
    pub async fn set_username(
        &self,
        username: Option<String>,
    ) -> Result<Option<Arc<str>>, SessionError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(SessionMsg::SetUsername(tx, username)).await?;
        rx.await?
    }
}

impl Actor for Session {
    type Error = SessionError;

    fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        let (inner, state_rx) = SessionInner::new(rx);
        tokio::spawn(frame!(async move { inner.run().await }));
        Self { tx, state_rx }
    }
}

#[derive(Debug)]
struct SessionInner {
    rx: mpsc::Receiver<SessionMsg>,
    state: Arc<SessionState>,
    state_tx: watch::Sender<Arc<SessionState>>,
}

impl SessionInner {
    fn new(rx: mpsc::Receiver<SessionMsg>) -> (Self, watch::Receiver<Arc<SessionState>>) {
        let state = Arc::from(SessionState::new());
        let (state_tx, state_rx) = watch::channel(state.clone());
        (Self { rx, state, state_tx }, state_rx)
    }

    #[framed]
    async fn handle_message(&mut self, msg: SessionMsg) -> Result<(), SessionError> {
        let _ = match msg {
            SessionMsg::SetUsername(respond_to, username) => respond_to.send(self.update_username(username)),
        };
        Ok(())
    }

    fn update_username(&mut self, new_username: Option<String>) -> Result<Option<Arc<str>>, SessionError> {
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

    #[framed]
    async fn run(mut self) -> Result<(), Self::Error> where Self: Sized {
        while let Some(msg) = self.rx.recv().await {
            let debug_msg = format!("{msg:?}");
            if let Err(e) = self.handle_message(msg).await {
                warn!("Error handling {debug_msg}: {e:?}");
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SessionMsg {
    SetUsername(oneshot::Sender<Result<Option<Arc<str>>, SessionError>>, Option<String>),
}

type SendError = mpsc::error::SendError<SessionMsg>;
type RecvError = oneshot::error::RecvError;

#[derive(Debug)]
pub enum SessionError {
    TxError(SendError),
    RxError(RecvError),
    UsernameNotFound,
}

impl error::Error for SessionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::TxError(err)     => err.source(),
            Self::RxError(err)     => err.source(),
            Self::UsernameNotFound => None,
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TxError(err)     => err.fmt(f),
            Self::RxError(err)     => err.fmt(f),
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

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
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct Session {
    tx: mpsc::Sender<InnerMsg>,
}

impl Session {
    pub async fn get_username(&self) -> Result<Option<String>, SessionError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(InnerMsg::GetUsername(tx)).await?;
        rx.await?
    }

    #[framed]
    pub async fn set_username(&self, username: String) -> Result<(), SessionError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(InnerMsg::SetUsername(tx, username)).await?;
        rx.await?
    }
}

impl Actor for Session {
    type Error = SessionError;

    fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        let inner = Inner::new(rx, None);
        tokio::spawn(frame!(async move { inner.run().await }));
        Self { tx }
    }
}

#[derive(Debug)]
struct Inner {
    rx: mpsc::Receiver<InnerMsg>,
    username: Option<String>,
}

impl Inner {
    fn new(rx: mpsc::Receiver<InnerMsg>, username: Option<String>) -> Self {
        Self { rx, username }
    }

    #[framed]
    async fn handle_message(&mut self, msg: InnerMsg) -> Result<(), SessionError> {
        match msg {
            InnerMsg::GetUsername(respond_to) => {
                let _ = respond_to.send(Ok(self.username.clone()));
            }
            InnerMsg::SetUsername(respond_to, username) => {
                self.username = Some(username);
                let _ = respond_to.send(Ok(()));
            }
        };
        Ok(())
    }
}

impl ActorInner for Inner {
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
pub enum InnerMsg {
    GetUsername(oneshot::Sender<Result<Option<String>, SessionError>>),
    SetUsername(oneshot::Sender<Result<(), SessionError>>, String),
}

type SendError = mpsc::error::SendError<InnerMsg>;
type RecvError = oneshot::error::RecvError;

#[derive(Debug)]
pub enum SessionError {
    TxError(SendError),
    RxError(RecvError),
}

impl error::Error for SessionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::TxError(err) => err.source(),
            Self::RxError(err) => err.source(),
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TxError(err) => err.fmt(f),
            Self::RxError(err) => err.fmt(f),
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

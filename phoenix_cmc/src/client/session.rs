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

use async_backtrace::{frame, framed};
use std::error::Error;
use std::fmt;
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

#[derive(Debug)]
struct SessionObj {
    rx: mpsc::Receiver<SessionMessage>,
    username: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Session {
    tx: mpsc::Sender<SessionMessage>,
}

#[derive(Debug)]
pub enum SessionMessage {
    GetUsername(oneshot::Sender<Result<Option<String>, SessionError>>),
    SetUsername(oneshot::Sender<Result<(), SessionError>>, String),
}

impl SessionObj {
    fn new(rx: mpsc::Receiver<SessionMessage>, username: Option<String>) -> Self {
        Self { rx, username }
    }

    #[framed]
    async fn handle_message(&mut self, msg: SessionMessage) -> Result<(), SessionError> {
        match msg {
            SessionMessage::GetUsername(respond_to) => {
                let _ = respond_to.send(Ok(self.username.clone()));
            }
            SessionMessage::SetUsername(respond_to, username) => {
                self.username = Some(username);
                let _ = respond_to.send(Ok(()));
            }
        };
        Ok(())
    }

    #[framed]
    async fn run(mut self) -> Result<(), SessionError> {
        while let Some(msg) = self.rx.recv().await {
            let debug_msg = format!("{msg:?}");
            if let Err(e) = self.handle_message(msg).await {
                warn!("Error handling {debug_msg}: {e:?}");
            }
        }
        Ok(())
    }
}

impl Session {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        let obj = SessionObj::new(rx, None);
        tokio::spawn(frame!(async move { obj.run().await }));

        Self { tx }
    }

    #[framed]
    pub async fn get_username(&self) -> Result<Option<String>, SessionError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(SessionMessage::GetUsername(tx)).await?;
        rx.await?
    }

    #[framed]
    pub async fn set_username(&self, username: String) -> Result<(), SessionError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(SessionMessage::SetUsername(tx, username)).await?;
        rx.await?
    }
}

type SendError = mpsc::error::SendError<SessionMessage>;
type RecvError = oneshot::error::RecvError;

#[derive(Debug)]
pub enum SessionError {
    TxError(SendError),
    RxError(RecvError),
}

impl Error for SessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
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

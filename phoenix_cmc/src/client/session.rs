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

use crate::error::PhoenixError;
use async_backtrace::{frame, framed};
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

pub type TxError = mpsc::error::SendError<SessionMessage>;
pub type RxError = oneshot::error::RecvError;

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
    GetUsername(oneshot::Sender<Result<Option<String>, PhoenixError>>),
    SetUsername(oneshot::Sender<Result<(), PhoenixError>>, String),
}

impl SessionObj {
    fn new(rx: mpsc::Receiver<SessionMessage>, username: Option<String>) -> Self {
        Self { rx, username }
    }

    #[framed]
    async fn handle_message(&mut self, msg: SessionMessage) -> Result<(), PhoenixError> {
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
    async fn run(&mut self) -> Result<(), PhoenixError> {
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
    pub async fn get_username(&self) -> Result<Option<String>, PhoenixError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(SessionMessage::GetUsername(tx)).await?;
        rx.await?
    }

    #[framed]
    pub async fn set_username(&self, username: String) -> Result<(), PhoenixError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(SessionMessage::SetUsername(tx, username)).await?;
        rx.await?
    }
}

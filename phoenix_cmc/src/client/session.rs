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

use crate::PhoenixError;
use async_backtrace::{frame, framed};
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

#[derive(Debug)]
struct SessionObj {
    username: Option<String>,
    receiver: mpsc::Receiver<SessionMessage>,
}

#[derive(Debug, Clone)]
pub struct Session {
    sender: mpsc::Sender<SessionMessage>,
}

#[derive(Debug)]
enum SessionMessage {
    GetUsername(oneshot::Sender<Result<Option<String>, PhoenixError>>),
    SetUsername(oneshot::Sender<Result<(), PhoenixError>>, String),
}

impl SessionObj {
    fn new(username: Option<String>, receiver: mpsc::Receiver<SessionMessage>) -> Self {
        Self { username, receiver }
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
        while let Some(msg) = self.receiver.recv().await {
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
        let (sender, receiver) = mpsc::channel(8);
        let obj = SessionObj::new(receiver);
        tokio::spawn(frame!(async move { obj.run().await }));

        Self { sender }
    }

    #[framed]
    pub async fn get_username(&self) -> Result<Option<String>, PhoenixError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(SessionMessage::GetUsername(sender))
            .await?;
        self.receiver.await
    }

    #[framed]
    pub async fn set_username(&self, username: String) -> Result<(), PhoenixError> {
        self.sender
            .send(SessionMessage::SetUsername(username))
            .await?;
        self.receiver.await
    }
}

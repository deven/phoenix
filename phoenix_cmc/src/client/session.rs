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
use tokio::sync::{mpsc, oneshot};

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
    GetUsername(oneshot::Sender<Result<Option<String>, Error>>),
}

impl SessionObj {
    fn new(username: Option<String>, receiver: mpsc::Receiver<SessionMessage>) -> Self {
        Self { username, receiver }
    }

    #[framed]
    async fn handle_message(&mut self, msg: &SessionMessage) -> Result<(), Error> {
        match msg {
            SessionMessage::GetUsername(respond_to) => respond_to.send(Ok(self.username.clone()))?,
        }
        Ok(())
    }

    #[framed]
    async fn run(&mut self) -> Result<(), Error> {
        while let Some(msg) = self.receiver.recv().await {
            if let Err(e) = self.handle_message(&msg).await {
                warn!("Error handling {msg:?}: {e:?}");
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
    pub async fn get_username(&self) -> Result<Option<String>, Error> {
        let (sender, receiver) = oneshot::channel();
        self.sender.send(SessionMessage::GetUsername(sender)).await?;
        receiver.await
    }
}

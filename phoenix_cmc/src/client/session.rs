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
    GetUsername(oneshot::Sender<Option<String>>),
}

impl SessionObj {
    fn new(username: Option<String>, receiver: mpsc::Receiver<SessionMessage>) -> Self {
        Self { username, receiver }
    }

    #[framed]
    async fn run(&mut self) -> Result<(), Error> {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                SessionMessage::GetUsername(respond_to) => {
                    let _ = respond_to.send(self.username.clone());
                }
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
    pub async fn get_username(&self) -> Option<String> {
        let (sender, receiver) = oneshot::channel();
        let msg = SessionMessage::GetUsername(sender);
        let _ = self.sender.send(msg).await;
        receiver.await.expect("SessionObj task has been killed")
    }
}

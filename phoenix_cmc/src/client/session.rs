// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: client::session module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use async_backtrace::framed;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Session handle.
#[derive(Debug, Clone)]
pub struct Session(Arc<RwLock<SessionInner>>);

#[derive(Debug)]
pub struct SessionInner {
    pub username: Arc<str>,
}

impl Session {
    /// Create a new instance of `Session`.
    pub fn new() -> Self {
        let inner = SessionInner {
            username: "<None>".into(),
        };

        Session(Arc::new(RwLock::new(inner)))
    }

    /// Obtain read lock on the session data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, SessionInner> {
        self.0.read().await
    }

    /// Obtain write lock on the session data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, SessionInner> {
        self.0.write().await
    }

    #[framed]
    pub async fn username(&self) -> Arc<str> {
        self.read().await.username.clone()
    }

    #[framed]
    pub async fn set_username<T: Into<Arc<str>>>(&mut self, value: T) {
        self.write().await.username = value.into();
    }
}

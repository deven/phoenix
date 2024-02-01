// -*- Rust -*-
//
// Phoenix CMC library: user module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use async_backtrace::framed;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// User handle.
#[derive(Debug, Clone)]
pub struct User(Arc<RwLock<UserInner>>);

#[derive(Debug)]
pub struct UserInner {
    pub username: Arc<str>,
}

impl User {
    /// Create a new instance of `User`.
    pub fn new(username: String) -> Self {
        let inner = UserInner {
            username: username.into(),
        };

        User(Arc::new(RwLock::new(inner)))
    }

    /// Obtain read lock on the user data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, UserInner> {
        self.0.read().await
    }

    /// Obtain write lock on the user data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, UserInner> {
        self.0.write().await
    }

    #[framed]
    pub async fn username(&self) -> Arc<str> {
        self.read().await.username.clone()
    }

    #[framed]
    pub async fn set_username<T: Into<Arc<str>>>(&self, value: T) {
        self.write().await.username = value.into();
    }
}

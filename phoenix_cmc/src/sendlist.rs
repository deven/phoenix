// -*- Rust -*-
//
// Phoenix CMC library: sendlist module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::discussion::Discussion;
use crate::session::Session;
use async_backtrace::framed;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Sendlist handle.
#[derive(Debug, Clone)]
pub struct Sendlist(Arc<RwLock<SendlistInner>>);

#[derive(Debug)]
pub struct SendlistInner {
    errors: Arc<str>,
    typed: Arc<str>,
    sessions: HashSet<Session>,
    discussions: HashSet<Discussion>,
}

impl Sendlist {
    /// Create a new instance of `Sendlist`.
    pub fn new<T, U>(
        errors: T,
        typed: U,
        sessions: HashSet<Session>,
        discussions: HashSet<Discussion>,
    ) -> Self
    where
        T: Into<Arc<str>>,
        U: Into<Arc<str>>,
    {
        let inner = SendlistInner {
            errors: errors.into(),
            typed: typed.into(),
            sessions,
            discussions,
        };

        Sendlist(Arc::new(RwLock::new(inner)))
    }

    /// Obtain read lock on the sendlist data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, SendlistInner> {
        self.0.read().await
    }

    /// Obtain write lock on the sendlist data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, SendlistInner> {
        self.0.write().await
    }

    #[framed]
    pub async fn errors(&self) -> Arc<str> {
        self.read().await.errors.clone()
    }

    #[framed]
    pub async fn set_errors<T: Into<Arc<str>>>(&self, value: T) {
        self.write().await.errors = value.into();
    }

    #[framed]
    pub async fn typed(&self) -> Arc<str> {
        self.read().await.typed.clone()
    }

    #[framed]
    pub async fn set_typed<T: Into<Arc<str>>>(&self, value: T) {
        self.write().await.typed = value.into();
    }

    #[framed]
    pub async fn sessions(&self) -> HashSet<Session> {
        self.read().await.sessions.clone()
    }

    #[framed]
    pub async fn set_sessions(&self, value: HashSet<Session>) {
        self.write().await.sessions = value;
    }

    #[framed]
    pub async fn discussions(&self) -> HashSet<Discussion> {
        self.read().await.discussions.clone()
    }

    #[framed]
    pub async fn set_discussions(&self, value: HashSet<Discussion>) {
        self.write().await.discussions = value;
    }
}

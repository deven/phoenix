// -*- Rust -*-
//
// Phoenix CMC library: name module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::session::Session;
use crate::user::User;
use async_backtrace::framed;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Name handle.
#[derive(Debug, Clone)]
pub struct Name(Arc<RwLock<NameInner>>);

#[derive(Debug)]
pub struct NameInner {
    /// Session this name refers to.
    pub session: Session,

    /// User owning this session.
    pub user: User,

    /// Current name (pseudo) for this session.
    pub name: Arc<str>,

    /// Current blurb for this session.
    pub blurb: Arc<str>,
}

impl Name {
    /// Create a new instance of `Name`.
    pub fn new<T, U>(session: Session, user: User, name: T, blurb: U) -> Self
    where
        T: Into<Arc<str>>,
        U: Into<Arc<str>>,
    {
        let inner = NameInner {
            session,
            user,
            name: name.into(),
            blurb: blurb.into(),
        };

        Name(Arc::new(RwLock::new(inner)))
    }

    /// Obtain read lock on the name data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, NameInner> {
        self.0.read().await
    }

    /// Obtain write lock on the name data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, NameInner> {
        self.0.write().await
    }

    #[framed]
    pub async fn session(&self) -> Session {
        self.read().await.session.clone()
    }

    #[framed]
    pub async fn set_session(&self, value: Session) {
        self.write().await.session = value;
    }

    #[framed]
    pub async fn user(&self) -> User {
        self.read().await.user.clone()
    }

    #[framed]
    pub async fn set_user(&self, value: User) {
        self.write().await.user = value;
    }

    #[framed]
    pub async fn name(&self) -> Arc<str> {
        self.read().await.name.clone()
    }

    #[framed]
    pub async fn set_name<T: Into<Arc<str>>>(&self, value: T) {
        self.write().await.name = value.into();
    }

    #[framed]
    pub async fn blurb(&self) -> Arc<str> {
        self.read().await.blurb.clone()
    }

    #[framed]
    pub async fn set_blurb<T: Into<Arc<str>>>(&self, value: T) {
        self.write().await.blurb = value.into();
    }
}

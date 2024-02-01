// -*- Rust -*-
//
// Phoenix CMC library: discussion module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::event::EventRef;
use crate::name::Name;
use crate::session::Session;
use async_backtrace::framed;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Discussion handle.
#[derive(Debug, Clone)]
pub struct Discussion(Arc<RwLock<DiscussionInner>>);

#[derive(Debug)]
pub struct DiscussionInner {
    name: Arc<str>,
    title: Arc<str>,
    public: bool,
    creator: Name,
    members: HashSet<Session>,
    moderators: HashSet<Name>,
    allowed: HashSet<Name>,
    denied: HashSet<Name>,
    creation_time: DateTime<Utc>,
    idle_since: DateTime<Utc>,
    output: BTreeMap<DateTime<Utc>, EventRef>,
}

impl Discussion {
    /// Create a new instance of `Discussion`.
    pub fn new<T, U>(
        name: T,
        title: U,
        public: bool,
        creator: Name,
        members: HashSet<Session>,
        moderators: HashSet<Name>,
        allowed: HashSet<Name>,
        denied: HashSet<Name>,
        creation_time: DateTime<Utc>,
        idle_since: DateTime<Utc>,
        output: BTreeMap<DateTime<Utc>, EventRef>,
    ) -> Self
    where
        T: Into<Arc<str>>,
        U: Into<Arc<str>>,
    {
        let inner = DiscussionInner {
            name: name.into(),
            title: title.into(),
            public,
            creator,
            members,
            moderators,
            allowed,
            denied,
            creation_time,
            idle_since,
            output,
        };

        Discussion(Arc::new(RwLock::new(inner)))
    }

    /// Obtain read lock on the discussion data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, DiscussionInner> {
        self.0.read().await
    }

    /// Obtain write lock on the discussion data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, DiscussionInner> {
        self.0.write().await
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
    pub async fn title(&self) -> Arc<str> {
        self.read().await.title.clone()
    }

    #[framed]
    pub async fn set_title<T: Into<Arc<str>>>(&self, value: T) {
        self.write().await.title = value.into();
    }

    #[framed]
    pub async fn public(&self) -> bool {
        self.read().await.public.clone()
    }

    #[framed]
    pub async fn set_public(&self, value: bool) {
        self.write().await.public = value;
    }

    #[framed]
    pub async fn creator(&self) -> Name {
        self.read().await.creator.clone()
    }

    #[framed]
    pub async fn set_creator(&self, value: Name) {
        self.write().await.creator = value;
    }

    #[framed]
    pub async fn members(&self) -> HashSet<Session> {
        self.read().await.members.clone()
    }

    #[framed]
    pub async fn set_members(&self, value: HashSet<Session>) {
        self.write().await.members = value;
    }

    #[framed]
    pub async fn moderators(&self) -> HashSet<Name> {
        self.read().await.moderators.clone()
    }

    #[framed]
    pub async fn set_moderators(&self, value: HashSet<Name>) {
        self.write().await.moderators = value;
    }

    #[framed]
    pub async fn allowed(&self) -> HashSet<Name> {
        self.read().await.allowed.clone()
    }

    #[framed]
    pub async fn set_allowed(&self, value: HashSet<Name>) {
        self.write().await.allowed = value;
    }

    #[framed]
    pub async fn denied(&self) -> HashSet<Name> {
        self.read().await.denied.clone()
    }

    #[framed]
    pub async fn set_denied(&self, value: HashSet<Name>) {
        self.write().await.denied = value;
    }

    #[framed]
    pub async fn creation_time(&self) -> DateTime<Utc> {
        self.read().await.creation_time.clone()
    }

    #[framed]
    pub async fn set_creation_time(&self, value: DateTime<Utc>) {
        self.write().await.creation_time = value;
    }

    #[framed]
    pub async fn idle_since(&self) -> DateTime<Utc> {
        self.read().await.idle_since.clone()
    }

    #[framed]
    pub async fn set_idle_since(&self, value: DateTime<Utc>) {
        self.write().await.idle_since = value;
    }

    #[framed]
    pub async fn output(&self) -> BTreeMap<DateTime<Utc>, EventRef> {
        self.read().await.output.clone()
    }

    #[framed]
    pub async fn set_output(&self, value: BTreeMap<DateTime<Utc>, EventRef>) {
        self.write().await.output = value;
    }
}

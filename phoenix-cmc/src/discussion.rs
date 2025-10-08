// -*- Rust -*-
//
// Phoenix CMC library: discussion module
//
// Copyright 1992-2025 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::atomic::{AtomicNameOption, AtomicOrdSet, AtomicText, AtomicTimestamp};
use crate::constants::COMMA;
use crate::name::Name;
use crate::output::*;
use crate::session::Session;
use crate::text::Text;
use crate::timestamp::Timestamp;
use crate::{getword, match_keyword};
use async_backtrace::framed;
use im::OrdSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static DISCUSSION_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Discussion handle.
#[derive(Debug, Clone)]
pub struct Discussion(pub Arc<DiscussionInner>);

#[derive(Debug)]
pub struct DiscussionInner {
    pub id: usize,
    pub name: AtomicText,
    pub title: AtomicText,
    pub is_public: AtomicBool,
    pub creator: AtomicNameOption,
    pub members: AtomicOrdSet<Session>,
    pub moderators: AtomicOrdSet<Name>,
    pub allowed: AtomicOrdSet<Name>,
    pub denied: AtomicOrdSet<Name>,
    pub creation_time: AtomicTimestamp,
    pub idle_since: AtomicTimestamp,
}

impl Discussion {
    /// Create a new `Discussion` object.
    #[framed]
    pub async fn new(creator_session: Option<Session>, name: impl Into<Text>, title: impl Into<Text>, is_public: bool) -> Self {
        let id = DISCUSSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = name.into();
        let title = title.into();
        let mut creator = None;
        let mut members = OrdSet::new();
        let mut moderators = OrdSet::new();
        let allowed = OrdSet::new();
        let denied = OrdSet::new();
        let creation_time = Timestamp::new();
        let idle_since = creation_time.clone();

        // Set creator and add initial member/moderator if provided.
        if let Some(session) = creator_session {
            let creator_name = session.name();
            creator = Some(creator_name.clone());
            members.insert(session);
            moderators.insert(creator_name);
        }

        let inner = DiscussionInner {
            id,
            name: AtomicText::new(name),
            title: AtomicText::new(title),
            is_public: AtomicBool::new(is_public),
            creator: creator.into(),
            members: AtomicOrdSet::new(members),
            moderators: AtomicOrdSet::new(moderators),
            allowed: AtomicOrdSet::new(allowed),
            denied: AtomicOrdSet::new(denied),
            creation_time: AtomicTimestamp::new(creation_time),
            idle_since: AtomicTimestamp::new(idle_since),
        };

        Self(Arc::new(inner))
    }

    /// Get the discussion ID.
    pub fn id(&self) -> usize {
        self.0.id
    }

    /// Get the discussion name.
    pub fn name(&self) -> Text {
        self.0.name.snapshot()
    }

    /// Set the discussion name.
    pub fn set_name(&self, value: Text) {
        self.0.name.set(value);
    }

    /// Get the discussion title.
    pub fn title(&self) -> Text {
        self.0.title.snapshot()
    }

    /// Set the discussion title.
    pub fn set_title(&self, value: Text) {
        self.0.title.set(value);
    }

    /// Get the is-public flag.
    pub fn is_public(&self) -> bool {
        self.0.is_public.load(Ordering::Relaxed)
    }

    /// Set the is-public flag.
    pub fn set_public(&self, value: bool) {
        self.0.is_public.store(value, Ordering::Relaxed);
    }

    /// Get the discussion creator, if any.
    pub fn creator(&self) -> Option<Name> {
        self.0.creator.snapshot()
    }

    /// Set the discussion creator.
    pub fn set_creator(&self, value: Option<Name>) {
        self.0.creator.set(value);
    }

    /// Check if the specified name is the creator of the discussion.
    pub fn is_creator(&self, name: &Name) -> bool {
        self.0.creator.borrow().as_ref().map_or(false, |creator| creator == name)
    }

    /// Check if the specified session is in the members list for the discussion.
    pub fn is_member(&self, session: &Session) -> bool {
        self.0.members.borrow().as_ref().contains(session)
    }

    /// Check if the specified name is in the moderators list for the discussion.
    pub fn is_moderator(&self, name: &Name) -> bool {
        self.0.moderators.borrow().as_ref().contains(name)
    }

    /// Check if the specified name is in the allowed list for the discussion.
    pub fn is_allowed(&self, name: &Name) -> bool {
        self.0.allowed.borrow().as_ref().contains(name)
    }

    /// Check if the specified name is in the denied list for the discussion.
    pub fn is_denied(&self, name: &Name) -> bool {
        self.0.denied.borrow().as_ref().contains(name)
    }

    /// Check if the specified session is permitted to the discussion.
    pub fn is_permitted(&self, name: &Name) -> bool {
        self.is_creator(name) || self.is_moderator(name) || ((self.is_public() || self.is_allowed(name)) && !self.is_denied(name))
    }

    /// Get the members of the discussion.
    pub fn members(&self) -> im::OrdSet<Session> {
        self.0.members.snapshot()
    }

    /// Get the discussion creation timestamp.
    pub fn creation_time(&self) -> Timestamp {
        self.0.creation_time.snapshot()
    }

    /// Set the discussion creation timestamp.
    pub fn set_creation_time(&self, value: Timestamp) {
        self.0.creation_time.set(value);
    }

    /// Get the idle-since timestamp.
    pub fn idle_since(&self) -> Timestamp {
        self.0.idle_since.snapshot()
    }

    /// Set the idle-since timestamp.
    pub fn set_idle_since(&self, value: Timestamp) {
        self.0.idle_since.set(value);
    }

    /// Reset the idle-since timestamp to now.
    pub fn reset_idle_since(&self) {
        self.0.idle_since.set(Timestamp::new());
    }

    /// Get the moderators.
    pub fn moderators(&self) -> im::OrdSet<Name> {
        self.0.moderators.snapshot()
    }

    /// Set the moderators.
    pub fn set_moderators(&self, value: im::OrdSet<Name>) {
        self.0.moderators.set(value);
    }

    /// Add a moderator.
    pub fn add_moderator(&self, name: Name) {
        let mut moderators = self.0.moderators.snapshot();
        moderators.insert(name);
        self.0.moderators.set(moderators);
    }

    /// Remove a moderator.
    pub fn remove_moderator(&self, name: &Name) {
        let mut moderators = self.0.moderators.snapshot();
        moderators.remove(name);
        self.0.moderators.set(moderators);
    }

    /// Get the allowed users.
    pub fn allowed(&self) -> im::OrdSet<Name> {
        self.0.allowed.snapshot()
    }

    /// Set the allowed users.
    pub fn set_allowed(&self, value: im::OrdSet<Name>) {
        self.0.allowed.set(value);
    }

    /// Add an allowed user.
    pub fn add_allowed(&self, name: Name) {
        let mut allowed = self.0.allowed.snapshot();
        allowed.insert(name);
        self.0.allowed.set(allowed);
    }

    /// Remove an allowed user.
    pub fn remove_allowed(&self, name: &Name) {
        let mut allowed = self.0.allowed.snapshot();
        allowed.remove(name);
        self.0.allowed.set(allowed);
    }

    /// Get the denied users.
    pub fn denied(&self) -> im::OrdSet<Name> {
        self.0.denied.snapshot()
    }

    /// Set the denied users.
    pub fn set_denied(&self, value: im::OrdSet<Name>) {
        self.0.denied.set(value);
    }

    /// Add a denied user.
    pub fn add_denied(&self, name: Name) {
        let mut denied = self.0.denied.snapshot();
        denied.insert(name);
        self.0.denied.set(denied);
    }

    /// Remove a denied user.
    pub fn remove_denied(&self, name: &Name) {
        let mut denied = self.0.denied.snapshot();
        denied.remove(name);
        self.0.denied.set(denied);
    }

    /// Set the member list.
    pub fn set_members(&self, value: im::OrdSet<Session>) {
        self.0.members.set(value);
    }

    /// Add a member.
    pub fn add_member(&self, session: Session) {
        let mut members = self.0.members.snapshot();
        members.insert(session);
        self.0.members.set(members);
    }

    /// Remove a member.
    pub fn remove_member(&self, session: &Session) {
        let mut members = self.0.members.snapshot();
        members.remove(session);
        self.0.members.set(members);
    }

    /// Enqueue an `Output` to all members of the discussion except the sender.
    #[framed]
    pub async fn enqueue_others(&self, out: Output, sender: &Session) -> tokio::io::Result<()> {
        let mut result = Ok(());

        let members = self.0.members.borrow();
        for member in members.as_ref().iter() {
            if member != sender {
                if let Err(e) = member.enqueue(out.clone()).await {
                    println!("=== DEBUG: Error in enqueue() during enqueue_others(): {e} ===");
                    if result.is_ok() {
                        result = Err(e);
                    }
                }
            }
        }

        result
    }

    /// Destroy the discussion.
    #[framed]
    pub async fn destroy(&self, session: &Session) -> tokio::io::Result<()> {
        let session_name = session.name();
        let disc = self.name();

        if self.is_creator(&session_name) || self.is_moderator(&session_name) {
            Session::remove_discussion(disc.clone()).await;
            self.enqueue_others(DestroyNotify::new(disc.clone(), session_name), &session).await?;
            session.output(&format!("You have destroyed discussion {disc}.\n")).await;
        } else {
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
        }

        Ok(())
    }

    /// Join the discussion.
    #[framed]
    pub async fn join(&self, session: &Session) -> tokio::io::Result<()> {
        let session_name = session.name();
        let disc = self.name();

        if self.is_member(session) {
            session.output(&format!("You are already a member of discussion {disc}.\n")).await;
        } else {
            if self.is_permitted(&session_name) {
                self.enqueue_others(JoinNotify::new(disc.clone(), session_name), &session).await?;

                let mut members = self.0.members.snapshot();
                members.insert(session.clone());
                self.0.members.set(members);
                session.output(&format!("You are now a member of discussion {disc}.\n")).await;
            } else {
                session.output(&format!("You are not permitted to join discussion {disc}.\n")).await;
            }
        }

        Ok(())
    }

    /// Quit the discussion.
    #[framed]
    pub async fn quit(&self, session: &Session) -> tokio::io::Result<()> {
        let disc = self.name();

        if self.is_member(session) {
            let mut members = self.0.members.snapshot();
            members.remove(session);
            self.0.members.set(members);

            if session.signed_on() {
                self.enqueue_others(QuitNotify::new(disc.clone(), session.name()), &session).await?;
                session.output(&format!("You are no longer a member of discussion {disc}.\n")).await;
            }
        } else {
            session.output(&format!("You are not a member of discussion {disc}.\n")).await;
        }

        Ok(())
    }

    /// Permit someone to the discussion.
    #[framed]
    pub async fn permit(&self, session: &Session, args: &str) -> tokio::io::Result<()> {
        let session_name = session.name();
        let disc = self.name();

        if !(self.is_creator(&session_name) || self.is_moderator(&session_name)) {
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if self.is_public() {
                    session.output(&format!("Discussion {disc} is already public.\n")).await;
                } else {
                    self.set_public(true);
                    self.enqueue_others(PublicNotify::new(disc.clone(), session_name.clone()), &session).await?;
                    session.output(&format!("You have made discussion {disc} public.\n")).await;
                }
            } else {
                let (found_session, matches) = session.find_session(user).await;

                if let Some(s) = found_session {
                    let name = s.name();

                    if self.is_public() {
                        if self.is_denied(&name) {
                            let denied = self.0.denied.snapshot();
                            let denied = denied.without(&name);
                            self.0.denied.set(denied);
                            self.enqueue_others(PermitNotify::new(disc.clone(), true, name.clone(), true), &session).await?;
                            session.output(&format!("You have repermitted {name} to discussion {disc}.\n")).await;
                        } else if self.is_allowed(&name) {
                            session.output(&format!("{name} is already explicitly permitted to public discussion {disc}.\n")).await;
                        } else {
                            let mut allowed = self.0.allowed.snapshot();
                            allowed.insert(name.clone());
                            self.0.allowed.set(allowed);
                            self.enqueue_others(PermitNotify::new(disc.clone(), true, name.clone(), false), &session).await?;
                            session.output(&format!("You have explicitly permitted {name} to public discussion {disc}.\n")).await;
                        }
                    } else {
                        if self.is_denied(&name) {
                            let denied = self.0.denied.snapshot();
                            let denied = denied.without(&name);
                            self.0.denied.set(denied);
                            let mut allowed = self.0.allowed.snapshot();
                            allowed.insert(name.clone());
                            self.0.allowed.set(allowed);
                            self.enqueue_others(PermitNotify::new(disc.clone(), false, name.clone(), true), &session).await?;
                            session.output(&format!("You have repermitted {name} to discussion {disc}.\n")).await;
                        } else if self.is_allowed(&name) {
                            session.output(&format!("{name} is already permitted to discussion {disc}.\n")).await;
                        } else {
                            let mut allowed = self.0.allowed.snapshot();
                            allowed.insert(name.clone());
                            self.0.allowed.set(allowed);
                            self.enqueue_others(PermitNotify::new(disc.clone(), false, name.clone(), false), &session).await?;
                            session.output(&format!("You have permitted {name} to discussion {disc}.\n")).await;
                        }
                    }
                } else {
                    session.session_matches(user, &matches).await;
                }
            }
        }

        Ok(())
    }

    /// Depermit someone from the discussion.
    #[framed]
    pub async fn depermit(&self, session: &Session, args: &str) -> tokio::io::Result<()> {
        let session_name = session.name();
        let disc = self.name();

        if !(self.is_creator(&session_name) || self.is_moderator(&session_name)) {
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if self.is_public() {
                    self.set_public(false);

                    // Add current members to allowed list.
                    let mut new_allowed = Vec::new();
                    let members = self.0.members.snapshot();
                    for member in members.iter() {
                        let member_name = member.name();
                        if !self.is_allowed(&member_name) {
                            new_allowed.push(member_name)
                        }
                    }

                    let mut allowed = self.0.allowed.snapshot();
                    for name in new_allowed {
                        allowed.insert(name);
                    }
                    self.0.allowed.set(allowed);

                    self.enqueue_others(PrivateNotify::new(disc.clone(), session_name.clone()), &session).await?;
                    session.output(&format!("You have made discussion {disc} private.\n")).await;
                } else {
                    session.output(&format!("Discussion {disc} is already private.\n")).await;
                }
            } else {
                let (found_session, matches) = session.find_session(user).await;

                if let Some(s) = found_session {
                    let name = s.name();

                    if self.is_public() {
                        let allowed = self.0.allowed.snapshot();
                        let allowed = allowed.without(&name);
                        self.0.allowed.set(allowed);

                        if self.is_denied(&name) {
                            session.output(&format!("{name} is already depermitted from discussion {disc}.\n")).await;
                        } else {
                            let mut denied = self.0.denied.snapshot();
                            denied.insert(name.clone());
                            self.0.denied.set(denied);

                            if self.is_member(&s) {
                                let mut members = self.0.members.snapshot();
                                members.remove(&s);
                                self.0.members.set(members);
                                self.enqueue_others(DepermitNotify::new(disc.clone(), true, name.clone(), true, Some(name.clone())), &session).await?;
                                session.output(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                self.enqueue_others(DepermitNotify::new(disc.clone(), true, name.clone(), true, None), &session).await?;
                                session.output(&format!("You have depermitted {name} from discussion {disc}.\n")).await;
                            }
                        }
                    } else {
                        if self.is_allowed(&name) {
                            let allowed = self.0.allowed.snapshot();
                            let allowed = allowed.without(&name);
                            self.0.allowed.set(allowed);

                            if self.is_member(&s) {
                                let mut members = self.0.members.snapshot();
                                members.remove(&s);
                                self.0.members.set(members);
                                self.enqueue_others(DepermitNotify::new(disc.clone(), false, name.clone(), false, Some(name.clone())), &session).await?;
                                session.output(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                self.enqueue_others(DepermitNotify::new(disc.clone(), false, name.clone(), false, None), &session).await?;
                                session.output(&format!("You have depermitted {name} from discussion {disc}.\n")).await;
                            }
                        } else if self.is_denied(&name) {
                            session.output(&format!("{name} is already explicitly depermitted from private discussion {disc}.\n")).await;
                        } else {
                            let mut denied = self.0.denied.snapshot();
                            denied.insert(name.clone());
                            self.0.denied.set(denied);
                            self.enqueue_others(DepermitNotify::new(disc.clone(), false, name.clone(), true, None), &session).await?;
                            session.output(&format!("You have explicitly depermitted {name} from discussion {disc}.\n")).await;
                        }
                    }
                } else {
                    session.session_matches(user, &matches).await;
                }
            }
        }

        Ok(())
    }

    /// Appoint a new moderator for the discussion.
    #[framed]
    pub async fn appoint(&self, session: &Session, args: &str) -> tokio::io::Result<()> {
        let session_name = session.name();
        let disc = self.name();

        if !(self.is_creator(&session_name) || self.is_moderator(&session_name) || session.priv_level() >= 50) {
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            let (found_session, matches) = session.find_session(user).await;

            if let Some(s) = found_session {
                let name = s.name();

                if self.is_moderator(&name) {
                    session.output(&format!("{name} is already a moderator of discussion {disc}.\n")).await;
                } else {
                    let mut moderators = self.0.moderators.snapshot();
                    moderators.insert(name.clone());
                    self.0.moderators.set(moderators);

                    self.enqueue_others(Output::AppointNotify(AppointNotify::new(disc.clone(), session_name.clone(), name.clone())), &session).await?;
                    session.output(&format!("You have appointed {name} as a moderator of discussion {disc}.\n")).await;
                }
            } else {
                session.session_matches(user, &matches).await;
            }
        }

        Ok(())
    }

    /// Unappoint an existing moderator from the discussion.
    #[framed]
    pub async fn unappoint(&self, session: &Session, args: &str) -> tokio::io::Result<()> {
        let session_name = session.name();
        let disc = self.name();

        if !(self.is_creator(&session_name) || self.is_moderator(&session_name)) {
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            let (found_session, matches) = session.find_session(user).await;

            if let Some(s) = found_session {
                let name = s.name();

                if self.is_creator(&name) {
                    session.output(&format!("{name} is the creator of discussion {disc} and may not be unappointed.\n")).await;
                } else if !self.is_moderator(&name) {
                    session.output(&format!("{name} is not a moderator of discussion {disc}.\n")).await;
                } else {
                    let mut moderators = self.0.moderators.snapshot();
                    moderators.remove(&name);
                    self.0.moderators.set(moderators);

                    self.enqueue_others(Output::UnappointNotify(UnappointNotify::new(disc.clone(), session_name.clone(), name.clone())), &session).await?;
                    session.output(&format!("You have unappointed {name} as a moderator of discussion {disc}.\n")).await;
                }
            } else {
                session.session_matches(user, &matches).await;
            }
        }

        Ok(())
    }
}

impl PartialEq for Discussion {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for Discussion {}

impl PartialOrd for Discussion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Discussion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

impl std::hash::Hash for Discussion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state);
    }
}

//#[cfg(test)]
const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<Discussion>();
    assert_send_sync_static::<DiscussionInner>();
};

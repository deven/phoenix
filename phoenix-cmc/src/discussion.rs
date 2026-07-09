// -*- Rust -*-
//
// Phoenix CMC library: discussion module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
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
use tokio::sync::mpsc;

static DISCUSSION_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Discussion handle.
#[derive(Debug, Clone)]
pub struct Discussion(pub Arc<DiscussionInner>);

/// Messages to the discussion actor: one mailbox creates a total order over the union of membership changes and
/// deliveries, so every member sees the same message sequence, and a session receives exactly the messages processed
/// during its membership -- nothing before its Join, nothing after its Quit.  (~ the C++ single thread's implicit
/// per-discussion serialization, restored as structure.)
#[derive(Debug)]
pub enum DiscussionMsg {
    /// Add a member (~ Discussion::Join).
    Join(Session),
    /// Remove a member (~ Discussion::Quit).
    Quit(Session),
    /// Fan an output out to the members, excluding the sender if given.
    Deliver {
        out: Arc<Output>,
        sender: Option<Session>,
    },
    /// Moderation commands (~ the discussion.cc bodies), serialized with membership and delivery: the permit/depermit
    /// interleaving that could violate allowed/denied disjointness cannot occur.
    Permit {
        session: Session,
        args: Text,
    },
    Depermit {
        session: Session,
        args: Text,
    },
    Appoint {
        session: Session,
        args: Text,
    },
    Unappoint {
        session: Session,
        args: Text,
    },
    Destroy(Session),
}

/// Private discussion state, owned by the discussion actor task.
#[derive(Debug)]
pub struct DiscussionObj {
    pub discussion: Discussion,
    pub rx: mpsc::UnboundedReceiver<DiscussionMsg>,
}

impl DiscussionObj {
    /// The discussion actor: membership and delivery share one sequential context.
    #[framed]
    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                DiscussionMsg::Join(session) => {
                    if let Err(e) = self.discussion.join(&session).await {
                        log::error!("discussion join: {e}");
                    }
                }
                DiscussionMsg::Quit(session) => {
                    if let Err(e) = self.discussion.quit(&session).await {
                        log::error!("discussion quit: {e}");
                    }
                }
                DiscussionMsg::Permit { session, args } => {
                    if let Err(e) = self.discussion.permit(&session, args.as_str()).await {
                        log::error!("discussion permit: {e}");
                    }
                }
                DiscussionMsg::Depermit { session, args } => {
                    if let Err(e) = self.discussion.depermit(&session, args.as_str()).await {
                        log::error!("discussion depermit: {e}");
                    }
                }
                DiscussionMsg::Appoint { session, args } => {
                    if let Err(e) = self.discussion.appoint(&session, args.as_str()).await {
                        log::error!("discussion appoint: {e}");
                    }
                }
                DiscussionMsg::Unappoint { session, args } => {
                    if let Err(e) = self.discussion.unappoint(&session, args.as_str()).await {
                        log::error!("discussion unappoint: {e}");
                    }
                }
                DiscussionMsg::Destroy(session) => {
                    if let Err(e) = self.discussion.destroy(&session).await {
                        log::error!("discussion destroy: {e}");
                    }
                    // A successful destroy removed the discussion from the registry: the actor exits, and later sends
                    // fail silently (messages to a destroyed discussion are no-ops).
                    if crate::session::DISCUSSIONS.get(&self.discussion.name()).is_none() {
                        return;
                    }
                }
                DiscussionMsg::Deliver { out, sender } => {
                    let result = match &sender {
                        Some(sender) => self.discussion.enqueue_others(Arc::clone(&out), sender).await,
                        None => {
                            for member in &self.discussion.members() {
                                member.enqueue(Arc::clone(&out)).await.ok();
                            }
                            Ok(())
                        }
                    };
                    if let Err(e) = result {
                        log::error!("discussion deliver: {e}");
                    }
                }
            }
        }
    }
}

/// Discussion shared state.  SINGLE WRITER: every mutation happens on the discussion actor (stage 2a-3); other tasks
/// only read.  The RCU element operations on the sets below are correct but superfluous under one writer and retire
/// with the stage 3 cleanup.
#[derive(Debug)]
pub struct DiscussionInner {
    pub tx: mpsc::UnboundedSender<DiscussionMsg>,
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

        let (tx, rx) = mpsc::unbounded_channel();
        let inner = DiscussionInner {
            tx,
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

        let discussion = Self(Arc::new(inner));
        tokio::spawn(DiscussionObj { discussion: discussion.clone(), rx }.run());
        discussion
    }

    /// Ask the discussion actor to add a member (ordered with deliveries).
    pub fn send_join(&self, session: Session) {
        let _ = self.0.tx.send(DiscussionMsg::Join(session));
    }

    /// Ask the discussion actor to remove a member (ordered with deliveries).
    pub fn send_quit(&self, session: Session) {
        let _ = self.0.tx.send(DiscussionMsg::Quit(session));
    }

    /// Ask the discussion actor to run a moderation command (ordered with membership and delivery).
    pub fn send_permit(&self, session: Session, args: Text) {
        let _ = self.0.tx.send(DiscussionMsg::Permit { session, args });
    }

    pub fn send_depermit(&self, session: Session, args: Text) {
        let _ = self.0.tx.send(DiscussionMsg::Depermit { session, args });
    }

    pub fn send_appoint(&self, session: Session, args: Text) {
        let _ = self.0.tx.send(DiscussionMsg::Appoint { session, args });
    }

    pub fn send_unappoint(&self, session: Session, args: Text) {
        let _ = self.0.tx.send(DiscussionMsg::Unappoint { session, args });
    }

    pub fn send_destroy(&self, session: Session) {
        let _ = self.0.tx.send(DiscussionMsg::Destroy(session));
    }

    /// Deliver an output to the members through the discussion actor, excluding the sender if given.
    pub fn deliver(&self, out: Arc<Output>, sender: Option<Session>) {
        let _ = self.0.tx.send(DiscussionMsg::Deliver { out, sender });
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
        self.0.creator.borrow().as_ref().is_some_and(|creator| creator == name)
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
        self.0.moderators.insert(name);
    }

    /// Remove a moderator.
    pub fn remove_moderator(&self, name: &Name) {
        self.0.moderators.remove(name);
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
        self.0.allowed.insert(name);
    }

    /// Remove an allowed user.
    pub fn remove_allowed(&self, name: &Name) {
        self.0.allowed.remove(name);
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
        self.0.denied.insert(name);
    }

    /// Remove a denied user.
    pub fn remove_denied(&self, name: &Name) {
        self.0.denied.remove(name);
    }

    /// Set the member list.
    pub fn set_members(&self, value: im::OrdSet<Session>) {
        self.0.members.set(value);
    }

    /// Add a member.
    pub fn add_member(&self, session: Session) {
        self.0.members.insert(session);
    }

    /// Remove a member.
    pub fn remove_member(&self, session: &Session) {
        self.0.members.remove(session);
    }

    /// Enqueue an `Output` to all members of the discussion except the sender.
    #[framed]
    pub async fn enqueue_others(&self, out: impl Into<Arc<Output>>, sender: &Session) -> tokio::io::Result<()> {
        let out: Arc<Output> = out.into();
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
            self.enqueue_others(DestroyNotify::new(disc.clone(), session_name), session).await?;
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
                self.enqueue_others(JoinNotify::new(disc.clone(), session_name), session).await?;

                self.0.members.insert(session.clone());
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
            self.0.members.remove(session);

            if session.signed_on() {
                self.enqueue_others(QuitNotify::new(disc.clone(), session.name()), session).await?;
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

            if match_keyword(user, "others", 6).is_some() {
                if self.is_public() {
                    session.output(&format!("Discussion {disc} is already public.\n")).await;
                } else {
                    self.set_public(true);
                    self.enqueue_others(PublicNotify::new(disc.clone(), session_name.clone()), session).await?;
                    session.output(&format!("You have made discussion {disc} public.\n")).await;
                }
            } else {
                let (found_session, matches) = session.find_session(user).await;

                if let Some(s) = found_session {
                    let name = s.name();

                    if self.is_public() {
                        if self.is_denied(&name) {
                            self.0.denied.remove(&name);
                            self.enqueue_others(PermitNotify::new(disc.clone(), true, name.clone(), true), session).await?;
                            session.output(&format!("You have repermitted {name} to discussion {disc}.\n")).await;
                        } else if self.is_allowed(&name) {
                            session.output(&format!("{name} is already explicitly permitted to public discussion {disc}.\n")).await;
                        } else {
                            self.0.allowed.insert(name.clone());
                            self.enqueue_others(PermitNotify::new(disc.clone(), true, name.clone(), false), session).await?;
                            session.output(&format!("You have explicitly permitted {name} to public discussion {disc}.\n")).await;
                        }
                    } else {
                        if self.is_denied(&name) {
                            self.0.denied.remove(&name);
                            self.0.allowed.insert(name.clone());
                            self.enqueue_others(PermitNotify::new(disc.clone(), false, name.clone(), true), session).await?;
                            session.output(&format!("You have repermitted {name} to discussion {disc}.\n")).await;
                        } else if self.is_allowed(&name) {
                            session.output(&format!("{name} is already permitted to discussion {disc}.\n")).await;
                        } else {
                            self.0.allowed.insert(name.clone());
                            self.enqueue_others(PermitNotify::new(disc.clone(), false, name.clone(), false), session).await?;
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

            if match_keyword(user, "others", 6).is_some() {
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

                    for name in new_allowed {
                        self.0.allowed.insert(name);
                    }

                    self.enqueue_others(PrivateNotify::new(disc.clone(), session_name.clone()), session).await?;
                    session.output(&format!("You have made discussion {disc} private.\n")).await;
                } else {
                    session.output(&format!("Discussion {disc} is already private.\n")).await;
                }
            } else {
                let (found_session, matches) = session.find_session(user).await;

                if let Some(s) = found_session {
                    let name = s.name();

                    if self.is_public() {
                        self.0.allowed.remove(&name);

                        if self.is_denied(&name) {
                            session.output(&format!("{name} is already depermitted from discussion {disc}.\n")).await;
                        } else {
                            self.0.denied.insert(name.clone());

                            if self.is_member(&s) {
                                self.0.members.remove(&s);
                                self.enqueue_others(DepermitNotify::new(disc.clone(), true, name.clone(), true, Some(name.clone())), session).await?;
                                session.output(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                self.enqueue_others(DepermitNotify::new(disc.clone(), true, name.clone(), true, None), session).await?;
                                session.output(&format!("You have depermitted {name} from discussion {disc}.\n")).await;
                            }
                        }
                    } else {
                        if self.is_allowed(&name) {
                            self.0.allowed.remove(&name);

                            if self.is_member(&s) {
                                self.0.members.remove(&s);
                                self.enqueue_others(DepermitNotify::new(disc.clone(), false, name.clone(), false, Some(name.clone())), session).await?;
                                session.output(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                self.enqueue_others(DepermitNotify::new(disc.clone(), false, name.clone(), false, None), session).await?;
                                session.output(&format!("You have depermitted {name} from discussion {disc}.\n")).await;
                            }
                        } else if self.is_denied(&name) {
                            session.output(&format!("{name} is already explicitly depermitted from private discussion {disc}.\n")).await;
                        } else {
                            self.0.denied.insert(name.clone());
                            self.enqueue_others(DepermitNotify::new(disc.clone(), false, name.clone(), true, None), session).await?;
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
                    self.0.moderators.insert(name.clone());

                    self.enqueue_others(AppointNotify::new(disc.clone(), session_name.clone(), name.clone()), session).await?;
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
                    self.0.moderators.remove(&name);

                    self.enqueue_others(UnappointNotify::new(disc.clone(), session_name.clone(), name.clone()), session).await?;
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

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<Discussion>();
    assert_send_sync_static::<DiscussionInner>();
};

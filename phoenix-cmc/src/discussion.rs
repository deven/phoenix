use crate::constants::COMMA;
use crate::name::Name;
use crate::output::*;
use crate::session::Session;
use crate::text::Text;
use crate::timestamp::Timestamp;
use crate::types::{getword, match_keyword, OrderedSet};
use async_backtrace::framed;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

static DISCUSSION_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Discussion handle.
#[derive(Debug, Clone)]
pub struct Discussion
where
    Self: Send + Sync + 'static,
{
    pub id: usize,
    pub inner: Arc<RwLock<DiscussionInner>>,
}

#[derive(Debug)]
pub struct DiscussionInner
where
    Self: Send + Sync + 'static,
{
    pub id: usize,
    pub name: Text,
    pub title: Text,
    pub is_public: bool,
    pub creator: Option<Name>,
    pub members: OrderedSet<Session>,
    pub moderators: OrderedSet<Name>,
    pub allowed: OrderedSet<Name>,
    pub denied: OrderedSet<Name>,
    pub creation_time: Timestamp,
    pub idle_since: Timestamp,
}

impl Discussion {
    /// Create a new `Discussion` object.
    #[framed]
    pub async fn new(creator_session: Option<Session>, name: impl Into<Text>, title: impl Into<Text>, is_public: bool) -> Self {
        let id = DISCUSSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = name.into();
        let title = title.into();
        let mut creator = None;
        let mut members = OrderedSet::new();
        let mut moderators = OrderedSet::new();
        let allowed = OrderedSet::new();
        let denied = OrderedSet::new();
        let creation_time = Timestamp::new();
        let idle_since = creation_time.clone();

        // Set creator and add initial member/moderator if provided.
        if let Some(session) = creator_session {
            let creator_name = session.name().await;
            creator = Some(creator_name.clone());
            members.insert(session);
            moderators.insert(creator_name);
        }

        let inner = DiscussionInner {
            id,
            name,
            title,
            is_public,
            creator,
            members,
            moderators,
            allowed,
            denied,
            creation_time,
            idle_since,
        };

        Self { id, inner: Arc::new(RwLock::new(inner)) }
    }

    /// Get the discussion ID.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Obtain read lock on the `DiscussionInner` data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, DiscussionInner> {
        self.inner.read().await
    }

    /// Obtain write lock on the `DiscussionInner` data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, DiscussionInner> {
        self.inner.write().await
    }

    /// Get the discussion name.
    #[framed]
    pub async fn name(&self) -> Text {
        self.read().await.name.clone()
    }

    /// Set the discussion name.
    #[framed]
    pub async fn set_name(&self, value: Text) {
        self.write().await.name = value;
    }

    /// Get the discussion title.
    #[framed]
    pub async fn title(&self) -> Text {
        self.read().await.title.clone()
    }

    /// Set the discussion title.
    #[framed]
    pub async fn set_title(&self, value: Text) {
        self.write().await.title = value;
    }

    /// Get the is-public flag.
    #[framed]
    pub async fn is_public(&self) -> bool {
        self.read().await.is_public
    }

    /// Set the is-public flag.
    #[framed]
    pub async fn set_is_public(&self, value: bool) {
        self.write().await.is_public = value;
    }

    /// Get the discussion creator, if any.
    #[framed]
    pub async fn creator(&self) -> Option<Name> {
        self.read().await.creator.clone()
    }

    /// Set the discussion creator.
    #[framed]
    pub async fn set_creator(&self, value: Option<Name>) {
        self.write().await.creator = value;
    }

    /// Check if the specified name is the creator of the discussion.
    #[framed]
    pub async fn is_creator(&self, name: &Name) -> bool {
        self.read().await.is_creator(name)
    }

    /// Check if the specified session is in the members list for the discussion.
    #[framed]
    pub async fn is_member(&self, session: &Session) -> bool {
        self.read().await.is_member(session)
    }

    /// Check if the specified name is in the moderators list for the discussion.
    #[framed]
    pub async fn is_moderator(&self, name: &Name) -> bool {
        self.read().await.is_moderator(name)
    }

    /// Check if the specified name is in the allowed list for the discussion.
    #[framed]
    pub async fn is_allowed(&self, name: &Name) -> bool {
        self.read().await.is_allowed(name)
    }

    /// Check if the specified name is in the denied list for the discussion.
    #[framed]
    pub async fn is_denied(&self, name: &Name) -> bool {
        self.read().await.is_denied(name)
    }

    /// Check if the specified session is permitted to the discussion.
    #[framed]
    pub async fn is_permitted(&self, name: &Name) -> bool {
        self.read().await.is_permitted(name)
    }

    /// Get the discussion creation timestamp.
    #[framed]
    pub async fn creation_time(&self) -> Timestamp {
        self.read().await.creation_time.clone()
    }

    /// Set the discussion creation timestamp.
    #[framed]
    pub async fn set_creation_time(&self, value: Timestamp) {
        self.write().await.creation_time = value;
    }

    /// Get the discussion idle-since timestamp.
    #[framed]
    pub async fn idle_since(&self) -> Timestamp {
        self.read().await.idle_since.clone()
    }

    /// Set the discussion idle-since timestamp.
    #[framed]
    pub async fn set_idle_since(&self, value: Timestamp) {
        self.write().await.idle_since = value;
    }

    /// Reset the discussion idle-since timestamp.
    #[framed]
    pub async fn reset_idle_since(&self) {
        self.write().await.idle_since = Timestamp::new();
    }

    /// Enqueue an `OutputObj` to all members of the discussion except the sender.
    #[framed]
    pub async fn enqueue_others(&self, out: Arc<dyn OutputObj>, sender: &Session) {
        let inner = self.read().await;
        for member in inner.members.iter() {
            if member != sender {
                member.enqueue(out.clone()).await;
            }
        }
    }

    /// Destroy the discussion.
    #[framed]
    pub async fn destroy(&self, session: &Session) {
        let inner = self.read().await;
        let session_name = session.name().await;

        if inner.is_creator(&session_name) || inner.is_moderator(&session_name) {
            Session::remove_discussion(inner.name.clone()).await;
            let notification = Arc::new(DestroyNotify::new(inner.name.clone(), session_name));
            inner.enqueue_others(notification, &session).await;
            let disc = &inner.name;
            session.output(&format!("You have destroyed discussion {disc}.\n")).await;
        } else {
            let disc = &inner.name;
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
        }
    }

    /// Join the discussion.
    #[framed]
    pub async fn join(&self, session: &Session) {
        let mut inner = self.write().await;
        let session_name = session.name().await;

        if inner.is_member(session) {
            let disc = &inner.name;
            session.output(&format!("You are already a member of discussion {disc}.\n")).await;
        } else {
            if inner.is_permitted(&session_name) {
                let notification = Arc::new(JoinNotify::new(inner.name.clone(), session_name));
                inner.enqueue_others(notification, &session).await;
                inner.members.insert(session.clone());
                let disc = &inner.name;
                session.output(&format!("You are now a member of discussion {disc}.\n")).await;
            } else {
                let disc = &inner.name;
                session.output(&format!("You are not permitted to join discussion {disc}.\n")).await;
            }
        }
    }

    /// Quit the discussion.
    #[framed]
    pub async fn quit(&self, session: &Session) {
        let mut inner = self.write().await;

        if inner.is_member(session) {
            inner.members.shift_remove(session);
            if session.signed_on().await {
                let notification = Arc::new(QuitNotify::new(inner.name.clone(), session.name().await));
                inner.enqueue_others(notification, &session).await;
                let disc = &inner.name;
                session.output(&format!("You are no longer a member of discussion {disc}.\n")).await;
            }
        } else {
            let disc = &inner.name;
            session.output(&format!("You are not a member of discussion {disc}.\n")).await;
        }
    }

    /// Permit someone to the discussion.
    #[framed]
    pub async fn permit(&self, session: &Session, args: &str) {
        let mut inner = self.write().await;
        let session_name = session.name().await;

        if !(inner.is_creator(&session_name) || inner.is_moderator(&session_name)) {
            let disc = &inner.name;
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if inner.is_public {
                    let disc = &inner.name;
                    session.output(&format!("Discussion {disc} is already public.\n")).await;
                } else {
                    inner.is_public = true;
                    let notification = Arc::new(PublicNotify::new(inner.name.clone(), session_name.clone()));
                    inner.enqueue_others(notification, &session).await;
                    let disc = &inner.name;
                    session.output(&format!("You have made discussion {disc} public.\n")).await;
                }
            } else {
                let (found_session, matches) = session.find_session(user).await;

                if let Some(s) = found_session {
                    let name = &s.name().await;

                    if inner.is_public {
                        if inner.is_denied(name) {
                            inner.denied.retain(|n| n != name);
                            let notification = Arc::new(PermitNotify::new(inner.name.clone(), true, name.clone(), true));
                            inner.enqueue_others(notification, &session).await;
                            let disc = &inner.name;
                            session.output(&format!("You have repermitted {name} to discussion {disc}.\n")).await;
                        } else if inner.is_allowed(name) {
                            let disc = &inner.name;
                            session.output(&format!("{name} is already explicitly permitted to public discussion {disc}.\n")).await;
                        } else {
                            inner.allowed.insert(name.clone());
                            let notification = Arc::new(PermitNotify::new(inner.name.clone(), true, name.clone(), false));
                            inner.enqueue_others(notification, &session).await;
                            let disc = &inner.name;
                            session.output(&format!("You have explicitly permitted {name} to public discussion {disc}.\n")).await;
                        }
                    } else {
                        if inner.is_denied(name) {
                            inner.denied.retain(|n| n != name);
                            inner.allowed.insert(name.clone());
                            let notification = Arc::new(PermitNotify::new(inner.name.clone(), false, name.clone(), true));
                            inner.enqueue_others(notification, &session).await;
                            let disc = &inner.name;
                            session.output(&format!("You have repermitted {name} to discussion {disc}.\n")).await;
                        } else if inner.is_allowed(name) {
                            let disc = &inner.name;
                            session.output(&format!("{name} is already permitted to discussion {disc}.\n")).await;
                        } else {
                            inner.allowed.insert(name.clone());
                            let notification = Arc::new(PermitNotify::new(inner.name.clone(), false, name.clone(), false));
                            inner.enqueue_others(notification, &session).await;
                            let disc = &inner.name;
                            session.output(&format!("You have permitted {name} to discussion {disc}.\n")).await;
                        }
                    }
                } else {
                    session.session_matches(user, &matches).await;
                }
            }
        }
    }

    /// Depermit someone from the discussion.
    #[framed]
    pub async fn depermit(&self, session: &Session, args: &str) {
        let mut inner = self.write().await;
        let session_name = session.name().await;

        if !(inner.is_creator(&session_name) || inner.is_moderator(&session_name)) {
            let disc = &inner.name;
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if inner.is_public {
                    inner.is_public = false;

                    // Add current members to allowed list.
                    let mut new_allowed = Vec::new();
                    for member in inner.members.iter() {
                        let member_name = member.name().await;
                        if !inner.is_allowed(&member_name) {
                            new_allowed.push(member_name)
                        }
                    }

                    for name in new_allowed {
                        inner.allowed.insert(name);
                    }

                    let notification = Arc::new(PrivateNotify::new(inner.name.clone(), session_name.clone()));
                    inner.enqueue_others(notification, &session).await;
                    let disc = &inner.name;
                    session.output(&format!("You have made discussion {disc} private.\n")).await;
                } else {
                    let disc = &inner.name;
                    session.output(&format!("Discussion {disc} is already private.\n")).await;
                }
            } else {
                let (found_session, matches) = session.find_session(user).await;

                if let Some(s) = found_session {
                    let name = &s.name().await;

                    if inner.is_public {
                        inner.allowed.retain(|n| n != name);

                        if inner.is_denied(name) {
                            let disc = &inner.name;
                            session.output(&format!("{name} is already depermitted from discussion {disc}.\n")).await;
                        } else {
                            inner.denied.insert(name.clone());
                            if inner.is_member(&s) {
                                inner.members.shift_remove(&s);
                                let notification =
                                    Arc::new(DepermitNotify::new(inner.name.clone(), true, name.clone(), true, Some(name.clone())));
                                inner.enqueue_others(notification, &session).await;
                                let disc = &inner.name;
                                session.output(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                let notification =
                                    Arc::new(DepermitNotify::new(inner.name.clone(), true, name.clone(), true, None));
                                inner.enqueue_others(notification, &session).await;
                                let disc = &inner.name;
                                session.output(&format!("You have depermitted {name} from discussion {disc}.\n")).await;
                            }
                        }
                    } else {
                        if inner.is_allowed(name) {
                            inner.allowed.retain(|n| n != name);
                            if inner.is_member(&s) {
                                inner.members.shift_remove(&s);
                                let notification = Arc::new(DepermitNotify::new(
                                    inner.name.clone(),
                                    false,
                                    name.clone(),
                                    false,
                                    Some(name.clone()),
                                ));
                                inner.enqueue_others(notification, &session).await;
                                let disc = &inner.name;
                                session.output(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                let notification =
                                    Arc::new(DepermitNotify::new(inner.name.clone(), false, name.clone(), false, None));
                                inner.enqueue_others(notification, &session).await;
                                let disc = &inner.name;
                                session.output(&format!("You have depermitted {name} from discussion {disc}.\n")).await;
                            }
                        } else if inner.is_denied(name) {
                            let disc = &inner.name;
                            session
                                .output(&format!("{name} is already explicitly depermitted from private discussion {disc}.\n"))
                                .await;
                        } else {
                            inner.denied.insert(name.clone());
                            let notification = Arc::new(DepermitNotify::new(inner.name.clone(), false, name.clone(), true, None));
                            inner.enqueue_others(notification, &session).await;
                            let disc = &inner.name;
                            session.output(&format!("You have explicitly depermitted {name} from discussion {disc}.\n")).await;
                        }
                    }
                } else {
                    session.session_matches(user, &matches).await;
                }
            }
        }
    }

    /// Appoint a new moderator for the discussion.
    #[framed]
    pub async fn appoint(&self, session: &Session, args: &str) {
        let inner = self.read().await;
        let session_name = session.name().await;

        if !(inner.is_creator(&session_name) || inner.is_moderator(&session_name) || session.priv_level().await >= 50) {
            let disc = &inner.name;
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            // Handle appointments - would need FindSession implementation
            session.output(&format!("Appointment handling for '{user}' not yet implemented.\n")).await;
        }
    }

    /// Unappoint an existing moderator from the discussion.
    #[framed]
    pub async fn unappoint(&self, session: &Session, args: &str) {
        let inner = self.read().await;
        let session_name = session.name().await;

        if !(inner.is_creator(&session_name) || inner.is_moderator(&session_name)) {
            let disc = &inner.name;
            session.output(&format!("You are not a moderator of discussion {disc}.\n")).await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            // Handle unappointments - would need FindSession implementation
            session.output(&format!("Unappointment handling for '{user}' not yet implemented.\n")).await;
        }
    }
}

impl DiscussionInner {
    /// Check if the specified name is the creator of the discussion.
    pub fn is_creator(&self, name: &Name) -> bool {
        self.creator.as_ref() == Some(name)
    }

    /// Check if the specified session is in the members list for the discussion.
    pub fn is_member(&self, session: &Session) -> bool {
        self.members.contains(session)
    }

    /// Check if the specified name is in the moderators list for the discussion.
    pub fn is_moderator(&self, name: &Name) -> bool {
        self.moderators.contains(name)
    }

    /// Check if the specified name is in the allowed list for the discussion.
    pub fn is_allowed(&self, name: &Name) -> bool {
        self.allowed.contains(name)
    }

    /// Check if the specified name is in the denied list for the discussion.
    pub fn is_denied(&self, name: &Name) -> bool {
        self.denied.contains(name)
    }

    /// Check if the specified session is permitted to the discussion.
    pub fn is_permitted(&self, name: &Name) -> bool {
        self.is_creator(name) || self.is_moderator(name) || ((self.is_public || self.is_allowed(name)) && !self.is_denied(name))
    }

    /// Enqueue an `OutputObj` to all members of the discussion except the sender.
    pub async fn enqueue_others(&self, out: Arc<dyn OutputObj>, sender: &Session) {
        for member in self.members.iter() {
            if member != sender {
                member.enqueue(out.clone()).await;
            }
        }
    }
}

impl PartialEq for Discussion {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Discussion {}

impl std::hash::Hash for Discussion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

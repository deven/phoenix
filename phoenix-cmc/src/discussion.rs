use crate::name::Name;
use crate::output::*;
use crate::session::Session;
use crate::timestamp::Timestamp;
use crate::types::{ArcStr, OrderedSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Discussion {
    pub name: ArcStr,
    pub title: ArcStr,
    pub is_public: bool,
    pub creator: Option<Arc<Name>>,
    pub members: Arc<RwLock<OrderedSet<Arc<Session>>>>,
    pub moderators: Arc<RwLock<OrderedSet<Arc<Name>>>>,
    pub allowed: Arc<RwLock<OrderedSet<Arc<Name>>>>,
    pub denied: Arc<RwLock<OrderedSet<Arc<Name>>>>,
    pub creation_time: Timestamp,
    pub idle_since: Arc<RwLock<Timestamp>>,
}

impl Discussion {
    pub async fn new(
        creator_session: Option<Arc<Session>>,
        name: impl Into<ArcStr>,
        title: impl Into<ArcStr>,
        is_public: bool,
    ) -> Arc<Self> {
        let name = name.into();
        let title = title.into();
        let creation_time = Timestamp::new();
        let idle_since = Arc::new(RwLock::new(creation_time));

        let discussion = Arc::new(Self {
            name,
            title,
            is_public,
            creator: None,
            members: Arc::new(RwLock::new(OrderedSet::new())),
            moderators: Arc::new(RwLock::new(OrderedSet::new())),
            allowed: Arc::new(RwLock::new(OrderedSet::new())),
            denied: Arc::new(RwLock::new(OrderedSet::new())),
            creation_time,
            idle_since,
        });

        // Set creator and add initial member/moderator if provided
        if let Some(session) = creator_session {
            let creator_name = session.name_obj().await;
            Arc::get_mut(&mut discussion.clone()).unwrap().creator = Some(creator_name.clone());

            tokio::spawn({
                let discussion = discussion.clone();
                let session = session.clone();
                let creator_name = creator_name.clone();
                async move {
                    discussion.members.write().await.insert(session);
                    discussion.moderators.write().await.insert(creator_name);
                }
            });
        }

        discussion
    }

    pub async fn allowed(&self, session: &Arc<Session>) -> Option<Arc<Name>> {
        let allowed = self.allowed.read().await;
        let session_name = session.name();
        allowed
            .iter()
            .find(|name| name.name.eq_ignore_ascii_case(&session_name))
            .cloned()
    }

    pub async fn denied(&self, session: &Arc<Session>) -> Option<Arc<Name>> {
        let denied = self.denied.read().await;
        let session_name = session.name();
        denied
            .iter()
            .find(|name| name.name.eq_ignore_ascii_case(&session_name))
            .cloned()
    }

    pub async fn is_creator(&self, session: &Arc<Session>) -> bool {
        if let Some(creator) = &self.creator {
            creator.name.eq_ignore_ascii_case(&session.name())
        } else {
            false
        }
    }

    pub async fn is_moderator(&self, session: &Arc<Session>) -> Option<Arc<Name>> {
        let moderators = self.moderators.read().await;
        let session_name = session.name();
        moderators
            .iter()
            .find(|name| name.name.eq_ignore_ascii_case(&session_name))
            .cloned()
    }

    pub async fn permitted(&self, session: &Arc<Session>) -> bool {
        if self.is_creator(session).await || self.is_moderator(session).await.is_some() {
            return true;
        }
        if !self.is_public && self.allowed(session).await.is_none() {
            return false;
        }
        if self.denied(session).await.is_some() {
            return false;
        }
        true
    }

    pub async fn enqueue_others(&self, out: Arc<dyn OutputObj>, sender: &Arc<Session>) {
        let members = self.members.read().await;
        for member in members.iter() {
            if !Arc::ptr_eq(member, sender) {
                member.enqueue(out.clone()).await;
            }
        }
    }

    pub async fn destroy(&self, session: Arc<Session>) {
        if self.is_creator(&session).await || self.is_moderator(&session).await.is_some() {
            Session::remove_discussion(self.name.clone()).await;
            self.enqueue_others(
                Arc::new(DestroyNotify::new(self.name.clone(), session.name_obj().await)),
                &session,
            )
            .await;
            session
                .print(&format!("You have destroyed discussion {}.\n", self.name))
                .await;
        } else {
            session
                .print(&format!(
                    "You are not a moderator of discussion {}.\n",
                    self.name
                ))
                .await;
        }
    }

    pub async fn join(&self, session: Arc<Session>) {
        let mut members = self.members.write().await;
        if members.contains(&session) {
            session
                .print(&format!(
                    "You are already a member of discussion {}.\n",
                    self.name
                ))
                .await;
        } else {
            if self.permitted(&session).await {
                self.enqueue_others(
                    Arc::new(JoinNotify::new(self.name.clone(), session.name_obj().await)),
                    &session,
                )
                .await;
                members.insert(session.clone());
                session
                    .print(&format!(
                        "You are now a member of discussion {}.\n",
                        self.name
                    ))
                    .await;
            } else {
                session
                    .print(&format!(
                        "You are not permitted to join discussion {}.\n",
                        self.name
                    ))
                    .await;
            }
        }
    }

    pub async fn quit(&self, session: Arc<Session>) {
        let mut members = self.members.write().await;
        if members.contains(&session) {
            members.shift_remove(&session);
            if session.signed_on().await {
                self.enqueue_others(
                    Arc::new(QuitNotify::new(self.name.clone(), session.name_obj().await)),
                    &session,
                )
                .await;
                session
                    .print(&format!(
                        "You are no longer a member of discussion {}.\n",
                        self.name
                    ))
                    .await;
            }
        } else {
            session
                .print(&format!(
                    "You are not a member of discussion {}.\n",
                    self.name
                ))
                .await;
        }
    }

    pub async fn permit(&self, session: Arc<Session>, args: &str) {
        use crate::constants::COMMA;
        use crate::types::{getword, match_keyword};

        if !(self.is_creator(&session).await || self.is_moderator(&session).await.is_some()) {
            session
                .print(&format!(
                    "You are not a moderator of discussion {}.\n",
                    self.name
                ))
                .await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if self.is_public {
                    session
                        .print(&format!("Discussion {} is already public.\n", self.name))
                        .await;
                } else {
                    let disc = Arc::get_mut(&mut session.clone()).unwrap();
                    disc.is_public = true;
                    session
                        .enqueue_others(Arc::new(PublicNotify::new(
                            self.name.clone(),
                            session.name_obj().await,
                        )))
                        .await;
                    session
                        .print(&format!("You have made discussion {} public.\n", self.name))
                        .await;
                }
            } else {
                // Handle individual user permissions - would need FindSession implementation
                session
                    .print(&format!(
                        "Permission handling for '{}' not yet implemented.\n",
                        user
                    ))
                    .await;
            }
        }
    }

    pub async fn depermit(&self, session: Arc<Session>, args: &str) {
        use crate::constants::COMMA;
        use crate::types::{getword, match_keyword};

        if !(self.is_creator(&session).await || self.is_moderator(&session).await.is_some()) {
            session
                .print(&format!(
                    "You are not a moderator of discussion {}.\n",
                    self.name
                ))
                .await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if self.is_public {
                    let disc = Arc::get_mut(&mut session.clone()).unwrap();
                    disc.is_public = false;

                    // Add current members to allowed list
                    let members = self.members.read().await;
                    let mut allowed = self.allowed.write().await;
                    for member in members.iter() {
                        if self.allowed(&member).await.is_none() {
                            allowed.insert(member.name_obj().await);
                        }
                    }

                    session
                        .enqueue_others(Arc::new(PrivateNotify::new(
                            self.name.clone(),
                            session.name_obj().await,
                        )))
                        .await;
                    session
                        .print(&format!(
                            "You have made discussion {} private.\n",
                            self.name
                        ))
                        .await;
                } else {
                    session
                        .print(&format!("Discussion {} is already private.\n", self.name))
                        .await;
                }
            } else {
                // Handle individual user depermissions - would need FindSession implementation
                session
                    .print(&format!(
                        "Depermission handling for '{}' not yet implemented.\n",
                        user
                    ))
                    .await;
            }
        }
    }

    pub async fn appoint(&self, session: Arc<Session>, args: &str) {
        use crate::constants::COMMA;
        use crate::types::getword;

        if !(self.is_creator(&session).await
            || self.is_moderator(&session).await.is_some()
            || session.priv_level().await >= 50)
        {
            session
                .print(&format!(
                    "You are not a moderator of discussion {}.\n",
                    self.name
                ))
                .await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            // Handle appointments - would need FindSession implementation
            session
                .print(&format!(
                    "Appointment handling for '{}' not yet implemented.\n",
                    user
                ))
                .await;
        }
    }

    pub async fn unappoint(&self, session: Arc<Session>, args: &str) {
        use crate::constants::COMMA;
        use crate::types::getword;

        if !(self.is_creator(&session).await || self.is_moderator(&session).await.is_some()) {
            session
                .print(&format!(
                    "You are not a moderator of discussion {}.\n",
                    self.name
                ))
                .await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            // Handle unappointments - would need FindSession implementation
            session
                .print(&format!(
                    "Unappointment handling for '{}' not yet implemented.\n",
                    user
                ))
                .await;
        }
    }
}

impl PartialEq for Discussion {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}

impl Eq for Discussion {}

impl std::hash::Hash for Discussion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.to_lowercase().hash(state);
    }
}

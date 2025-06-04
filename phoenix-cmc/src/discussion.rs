use crate::constants::COMMA;
use crate::name::Name;
use crate::output::*;
use crate::session::Session;
use crate::timestamp::Timestamp;
use crate::types::{getword, match_keyword, ArcStr, OrderedSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Discussion {
    pub name: ArcStr,
    pub title: ArcStr,
    pub is_public: Arc<AtomicBool>,
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
        let is_public = Arc::new(AtomicBool::new(is_public));
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
        let session_name = session.name().await;
        allowed
            .iter()
            .find(|name| name.name.eq_ignore_ascii_case(&session_name))
            .cloned()
    }

    pub async fn denied(&self, session: &Arc<Session>) -> Option<Arc<Name>> {
        let denied = self.denied.read().await;
        let session_name = session.name().await;
        denied
            .iter()
            .find(|name| name.name.eq_ignore_ascii_case(&session_name))
            .cloned()
    }

    pub async fn is_creator(&self, session: &Arc<Session>) -> bool {
        let session_name = session.name().await;
        if let Some(creator) = &self.creator {
            creator.name == session_name
        } else {
            false
        }
    }

    pub async fn is_moderator(&self, session: &Arc<Session>) -> Option<Arc<Name>> {
        let moderators = self.moderators.read().await;
        let session_name = session.name().await;
        moderators
            .iter()
            .find(|name| name.name.eq_ignore_ascii_case(&session_name))
            .cloned()
    }

    pub async fn permitted(&self, session: &Arc<Session>) -> bool {
        if self.is_creator(session).await || self.is_moderator(session).await.is_some() {
            return true;
        }
        if !self.is_public.load(Ordering::Relaxed) && self.allowed(session).await.is_none() {
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
            if member.id != sender.id {
                member.enqueue(out.clone()).await;
            }
        }
    }

    pub async fn destroy(&self, session: Arc<Session>) {
        let name = &self.name;

        if self.is_creator(&session).await || self.is_moderator(&session).await.is_some() {
            Session::remove_discussion(self.name.clone()).await;
            let notification = Arc::new(DestroyNotify::new(
                self.name.clone(),
                session.name_obj().await,
            ));
            self.enqueue_others(notification, &session).await;
            session
                .print(&format!("You have destroyed discussion {name}.\n"))
                .await;
        } else {
            session
                .print(&format!("You are not a moderator of discussion {name}.\n"))
                .await;
        }
    }

    pub async fn join(&self, session: Arc<Session>) {
        let name = &self.name;

        let mut members = self.members.write().await;
        if members.contains(&session) {
            session
                .print(&format!("You are already a member of discussion {name}.\n"))
                .await;
        } else {
            if self.permitted(&session).await {
                let notification =
                    Arc::new(JoinNotify::new(self.name.clone(), session.name_obj().await));
                self.enqueue_others(notification, &session).await;
                members.insert(session.clone());
                session
                    .print(&format!("You are now a member of discussion {name}.\n"))
                    .await;
            } else {
                session
                    .print(&format!(
                        "You are not permitted to join discussion {name}.\n"
                    ))
                    .await;
            }
        }
    }

    pub async fn quit(&self, session: Arc<Session>) {
        let name = &self.name;

        let mut members = self.members.write().await;
        if members.contains(&session) {
            members.shift_remove(&session);
            if session.signed_on().await {
                let notification =
                    Arc::new(QuitNotify::new(self.name.clone(), session.name_obj().await));
                self.enqueue_others(notification, &session).await;
                session
                    .print(&format!(
                        "You are no longer a member of discussion {name}.\n"
                    ))
                    .await;
            }
        } else {
            session
                .print(&format!("You are not a member of discussion {name}.\n"))
                .await;
        }
    }

    pub async fn permit(&self, session: Arc<Session>, args: &str) {
        let name = &self.name;

        if !(self.is_creator(&session).await || self.is_moderator(&session).await.is_some()) {
            session
                .print(&format!("You are not a moderator of discussion {name}.\n"))
                .await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if self.is_public.load(Ordering::Relaxed) {
                    session
                        .print(&format!("Discussion {name} is already public.\n"))
                        .await;
                } else {
                    self.is_public.store(true, Ordering::Relaxed);
                    let notification = Arc::new(PublicNotify::new(
                        self.name.clone(),
                        session.name_obj().await,
                    ));
                    self.enqueue_others(notification, &session).await;
                    session
                        .print(&format!("You have made discussion {name} public.\n"))
                        .await;
                }
            } else {
                let (session, matches) = session.find_session(user);
                let name = session.name().await;

                if let Some(s) = session {
                    let mut denied = self.denied.write().await;
                    let mut allowed = self.allowed.write().await;
                    let name_obj = s.name_obj().await;
                    let name = s.name().await;

                    if self.is_public.load(Ordering::Relaxed) {
                        if denied.contains(&name) {
                            denied.shift_remove(&name);
                            let notification = Arc::new(PermitNotify::new(
                                self.name.clone(),
                                true,
                                name_obj,
                                true,
                            ));
                            self.enqueue_others(notification, &session).await;
                            session
                                .print(&format!(
                                    "You have repermitted {name} to discussion {disc}.\n"
                                ))
                                .await;
                        } else if allowed.contains(&name) {
                            session.print(&format!("{name} is already explicitly permitted to public discussion {disc}.\n")).await;
                        } else {
                            allowed.insert(name_obj.clone());
                            let notification = Arc::new(PermitNotify::new(
                                self.name.clone(),
                                true,
                                name_obj,
                                false,
                            ));
                            self.enqueue_others(notification, &session).await;
                            session.print(&format!("You have explicitly permitted {name} to public discussion {disc}.\n")).await;
                        }
                    } else {
                        if denied.contains(&name) {
                            denied.shift_remove(&name);
                            allowed.insert(name_obj.clone());
                            let notification = Arc::new(PermitNotify::new(
                                self.name.clone(),
                                false,
                                name_obj,
                                true,
                            ));
                            self.enqueue_others(notification, &session).await;
                            session
                                .print(&format!(
                                    "You have repermitted {name} to discussion {disc}.\n"
                                ))
                                .await;
                        } else if allowed.contains(&name) {
                            session
                                .print(&format!(
                                    "{name} is already permitted to discussion {disc}.\n"
                                ))
                                .await;
                        } else {
                            allowed.insert(name_obj.clone());
                            let notification = Arc::new(PermitNotify::new(
                                self.name.clone(),
                                false,
                                name_obj,
                                false,
                            ));
                            self.enqueue_others(notification, &session).await;
                            session
                                .print(&format!(
                                    "You have permitted {name} to discussion {disc}.\n"
                                ))
                                .await;
                        }
                    }
                } else {
                    session.session_matches(user, matches).await;
                }
            }
        }
    }

    pub async fn depermit(&self, session: Arc<Session>, args: &str) {
        let name = &self.name;

        if !(self.is_creator(&session).await || self.is_moderator(&session).await.is_some()) {
            session
                .print(&format!("You are not a moderator of discussion {name}.\n"))
                .await;
            return;
        }

        let mut members = self.members.write().await;
        let mut denied = self.denied.write().await;
        let mut allowed = self.allowed.write().await;

        let mut remaining = args;
        while !remaining.is_empty() {
            let (user, rest) = getword(remaining, Some(COMMA as char));
            remaining = rest;

            if let Some(_) = match_keyword(user, "others", 6) {
                if self.is_public.load(Ordering::Relaxed) {
                    self.is_public.store(false, Ordering::Relaxed);

                    // Add current members to allowed list
                    let members = self.members.read().await;
                    for member in members.iter() {
                        if self.allowed(&member).await.is_none() {
                            allowed.insert(member.name_obj().await);
                        }
                    }

                    let notification = Arc::new(PrivateNotify::new(
                        self.name.clone(),
                        session.name_obj().await,
                    ));
                    self.enqueue_others(notification, &session).await;
                    session
                        .print(&format!("You have made discussion {name} private.\n"))
                        .await;
                } else {
                    session
                        .print(&format!("Discussion {name} is already private.\n"))
                        .await;
                }
            } else {
                let (session, matches) = session.find_session(user);

                if let Some(s) = session {
                    let mut members = self.members.write().await;
                    let mut denied = self.denied.write().await;
                    let mut allowed = self.allowed.write().await;
                    let name_obj = s.name_obj().await;
                    let name = s.name().await;

                    if self.is_public.load(Ordering::Relaxed) {
                        let name = self.allowed(&s);
                        if let Some(n) = name {
                            allowed.shift_remove(n);
                        }
                        if self.denied(&s).is_some() {
                            session
                                .print(&format!(
                                    "{name} is already depermitted from discussion {disc}.\n"
                                ))
                                .await;
                        } else {
                            denied.insert(name_obj.clone());
                            if self.members.contains(&name) {
                                let removed = s;
                                members.shift_remove(&s);
                                let notification = Arc::new(DepermitNotify::new(
                                    self.name.clone(),
                                    true,
                                    name_obj,
                                    true,
                                    Some(name_obj),
                                ));
                                self.enqueue_others(notification, &session).await;
                                session.print(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                let notification = Arc::new(DepermitNotify::new(
                                    self.name.clone(),
                                    true,
                                    name_obj,
                                    true,
                                    None,
                                ));
                                self.enqueue_others(notification, &session).await;
                                session
                                    .print(&format!(
                                        "You have depermitted {name} from discussion {disc}.\n"
                                    ))
                                    .await;
                            }
                        }
                    } else {
                        let name = self.allowed(&s);
                        if let Some(n) = name {
                            allowed.shift_remove(n);
                            if members.contains(&s) {
                                members.shift_remove(s);
                                let notification = Arc::new(DepermitNotify::new(
                                    self.name.clone(),
                                    false,
                                    name_obj,
                                    false,
                                    Some(name_obj),
                                ));
                                self.enqueue_others(notification, &session).await;
                                session.print(&format!("You have depermitted and removed {name} from discussion {disc}.\n")).await;
                            } else {
                                let notification = Arc::new(DepermitNotify::new(
                                    self.name.clone(),
                                    false,
                                    name_obj,
                                    false,
                                    None,
                                ));
                                self.enqueue_others(notification, &session).await;
                                session
                                    .print(&format!(
                                        "You have depermitted {name} from discussion {disc}.\n"
                                    ))
                                    .await;
                            }
                        } else if self.denied(&s).is_some() {
                            session.print(&format!("{name} is already explicitly depermitted from private discussion {disc}.\n")).await;
                        } else {
                            denied.insert(name_obj.clone());
                            let notification = Arc::new(DepermitNotify::new(
                                self.name.clone(),
                                false,
                                name_obj,
                                true,
                                None,
                            ));
                            self.enqueue_others(notification, &session).await;
                            session.print(&format!("You have explicitly depermitted {name} from discussion {disc}.\n")).await;
                        }
                    }
                } else {
                    session.session_matches(user, matches).await;
                }
            }
        }
    }

    pub async fn appoint(&self, session: Arc<Session>, args: &str) {
        let name = &self.name;

        if !(self.is_creator(&session).await
            || self.is_moderator(&session).await.is_some()
            || session.priv_level().await >= 50)
        {
            session
                .print(&format!("You are not a moderator of discussion {name}.\n"))
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
                    "Appointment handling for '{user}' not yet implemented.\n"
                ))
                .await;
        }
    }

    pub async fn unappoint(&self, session: Arc<Session>, args: &str) {
        let name = &self.name;

        if !(self.is_creator(&session).await || self.is_moderator(&session).await.is_some()) {
            session
                .print(&format!("You are not a moderator of discussion {name}.\n"))
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
                    "Unappointment handling for '{user}' not yet implemented.\n"
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

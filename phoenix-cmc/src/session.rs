use crate::constants::*;
use crate::discussion::Discussion;
use crate::event::{EventQueue, RestartEvent, ShutdownEvent};
use crate::name::Name;
use crate::output::*;
use crate::sendlist::{message_start, Sendlist};
use crate::telnet::{Telnet, TELNET_ENABLED};
use crate::timestamp::{system_uptime, Timestamp};
use crate::types::*;
use crate::user::{hash_password, verify_password, User, UserManager};
use dashmap::DashMap;
use log::info;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub type InputFunc =
    for<'a> fn(&'a Arc<Session>, &'a str) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

lazy_static::lazy_static! {
    static ref INITS: DashMap<usize, Arc<Session>> = DashMap::new();
    static ref SESSIONS: DashMap<usize, Arc<Session>> = DashMap::new();
    static ref DISCUSSIONS: DashMap<String, Arc<Discussion>> = DashMap::new();
    static ref SESSION_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static ref DEFAULTS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref SHUTDOWN_EVENT: RwLock<Option<Arc<Box<dyn crate::event::Event + Send + Sync>>>> = RwLock::new(None);
}

pub struct Session {
    id: usize,
    user: Arc<RwLock<Option<Arc<RwLock<User>>>>>,
    telnet: Arc<RwLock<Option<Arc<Telnet>>>>,
    input_func: Arc<RwLock<Option<InputFunc>>>,
    lines: Arc<Mutex<Vec<String>>>,
    output_buffer: Arc<Mutex<String>>,
    pending: Arc<OutputStream>,
    user_vars: Arc<RwLock<HashMap<String, String>>>,
    sys_vars: Arc<RwLock<HashMap<String, String>>>,
    login_time: Arc<RwLock<Timestamp>>,
    idle_since: Arc<RwLock<Timestamp>>,
    away: Arc<RwLock<AwayState>>,
    signal_public: Arc<AtomicBool>,
    signal_private: Arc<AtomicBool>,
    signed_on: Arc<AtomicBool>,
    closing: Arc<AtomicBool>,
    attempts: Arc<AtomicI32>,
    priv_level: Arc<AtomicI32>,
    name: Arc<RwLock<ArcStr>>,
    blurb: Arc<RwLock<ArcStr>>,
    name_obj: Arc<RwLock<Arc<Name>>>,
    last_message: Arc<RwLock<Option<Arc<Message>>>>,
    default_sendlist: Arc<RwLock<Option<Arc<Sendlist>>>>,
    last_sendlist: Arc<RwLock<Option<Arc<Sendlist>>>>,
    last_explicit: Arc<RwLock<String>>,
    reply_sendlist: Arc<RwLock<String>>,
    oops_text: Arc<RwLock<String>>,
}

impl Session {
    pub const MAX_LOGIN_ATTEMPTS: i32 = 3;

    pub fn new(telnet: Arc<Telnet>) -> Arc<Self> {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let now = Timestamp::new();

        let session = Arc::new(Self {
            id,
            user: Arc::new(RwLock::new(None)),
            telnet: Arc::new(RwLock::new(Some(telnet.clone()))),
            input_func: Arc::new(RwLock::new(None)),
            lines: Arc::new(Mutex::new(Vec::new())),
            output_buffer: Arc::new(Mutex::new(String::new())),
            pending: Arc::new(OutputStream::new()),
            user_vars: Arc::new(RwLock::new(HashMap::new())),
            sys_vars: Arc::new(RwLock::new(HashMap::new())),
            login_time: Arc::new(RwLock::new(now)),
            idle_since: Arc::new(RwLock::new(now)),
            away: Arc::new(RwLock::new(AwayState::Here)),
            signal_public: Arc::new(AtomicBool::new(true)),
            signal_private: Arc::new(AtomicBool::new(true)),
            signed_on: Arc::new(AtomicBool::new(false)),
            closing: Arc::new(AtomicBool::new(false)),
            attempts: Arc::new(AtomicI32::new(0)),
            priv_level: Arc::new(AtomicI32::new(0)),
            name: Arc::new(RwLock::new(ArcStr::new(""))),
            blurb: Arc::new(RwLock::new(ArcStr::new(""))),
            name_obj: Arc::new(RwLock::new(Name::with_name_only(""))),
            last_message: Arc::new(RwLock::new(None)),
            default_sendlist: Arc::new(RwLock::new(None)),
            last_sendlist: Arc::new(RwLock::new(None)),
            last_explicit: Arc::new(RwLock::new(String::new())),
            reply_sendlist: Arc::new(RwLock::new(String::new())),
            oops_text: Arc::new(RwLock::new(
                "Oops!  Sorry, that last message was intended for someone else...".to_string(),
            )),
        });

        // Initialize defaults if needed
        tokio::spawn(async {
            let mut defaults = DEFAULTS.write().await;
            if defaults.is_empty() {
                defaults.insert("time_format".to_string(), "verbose".to_string());
            }
        });

        // Add to initializing sessions
        INITS.insert(id, session.clone());

        // Set telnet session
        telnet.set_session(Some(session.clone())).await;

        session
    }

    pub fn name(&self) -> ArcStr {
        self.name.read().await.clone()
    }

    pub fn name_obj(&self) -> Arc<Name> {
        self.name_obj.read().await.clone()
    }

    pub fn blurb(&self) -> ArcStr {
        self.blurb.read().await.clone()
    }

    pub async fn signed_on(&self) -> bool {
        self.signed_on.load(Ordering::Relaxed)
    }

    pub async fn priv_level(&self) -> i32 {
        self.priv_level.load(Ordering::Relaxed)
    }

    pub async fn signal_public(&self) -> bool {
        self.signal_public.load(Ordering::Relaxed)
    }

    pub async fn signal_private(&self) -> bool {
        self.signal_private.load(Ordering::Relaxed)
    }

    pub async fn last_explicit(&self) -> String {
        self.last_explicit.read().await.clone()
    }

    pub async fn reply_sendlist(&self) -> String {
        self.reply_sendlist.read().await.clone()
    }

    pub async fn set_reply_sendlist(&self, sendlist: &str) {
        let mut reply = self.reply_sendlist.write().await;
        *reply = sendlist.to_string();

        // Quote if necessary
        if sendlist
            .chars()
            .any(|c| c == ' ' || c == ',' || c == ':' || c == ';' || c == '_')
        {
            *reply = format!("\"{}\"", sendlist);
        }
    }

    pub async fn close(&self, drain: bool) {
        INITS.remove(&self.id);
        SESSIONS.remove(&self.id);

        if self.signed_on().await {
            self.notify_exit().await;
        }
        self.signed_on.store(false, Ordering::Relaxed);

        // Quit all discussions silently
        let disc_keys: Vec<_> = DISCUSSIONS.iter().map(|r| r.key().clone()).collect();
        for key in disc_keys {
            if let Some(disc) = DISCUSSIONS.get(&key) {
                disc.quit(self.clone()).await;
            }
        }

        // Close telnet connection if attached
        if let Some(telnet) = &*self.telnet.read().await {
            telnet.close(drain).await;
        }
        *self.telnet.write().await = None;

        // Disassociate from user
        if let Some(user_lock) = &*self.user.read().await {
            let mut user = user_lock.write().await;
            user.remove_session(&self.clone());
        }
        *self.user.write().await = None;
    }

    pub async fn transfer(&self, new_telnet: Arc<Telnet>) {
        let old_telnet = self.telnet.read().await.clone();
        *self.telnet.write().await = Some(new_telnet.clone());
        new_telnet.set_session(Some(self.clone())).await;

        if let Some(old) = old_telnet {
            info!(
                "Transfer: {} ({}) from fd to new connection",
                self.name().await,
                self.user
                    .read()
                    .await
                    .as_ref()
                    .map(|u| u.read().await.user.clone())
                    .unwrap_or_default()
            );
            old.output("*** This session has been transferred to a new connection. ***\n")
                .await;
            old.close(true).await;
        }

        self.enqueue_others(Arc::new(TransferNotify::new(self.name_obj())))
            .await;
        self.pending.attach(new_telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn attach(&self, telnet: Arc<Telnet>) {
        *self.telnet.write().await = Some(telnet.clone());
        telnet.set_session(Some(self.clone())).await;

        info!(
            "Attach: {} ({}) on new connection",
            self.name().await,
            self.user
                .read()
                .await
                .as_ref()
                .map(|u| u.read().await.user.clone())
                .unwrap_or_default()
        );

        self.enqueue_others(Arc::new(AttachNotify::new(self.name_obj())))
            .await;
        self.pending.attach(telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn detach(&self, telnet: &Arc<Telnet>, intentional: bool) {
        if self.signed_on().await && self.priv_level().await > 0 {
            let current_telnet = self.telnet.read().await;
            if let Some(t) = &*current_telnet {
                if Arc::ptr_eq(t, telnet) {
                    if intentional {
                        info!(
                            "Detach: {} ({}) (intentional)",
                            self.name().await,
                            self.user
                                .read()
                                .await
                                .as_ref()
                                .map(|u| u.read().await.user.clone())
                                .unwrap_or_default()
                        );
                    } else {
                        info!(
                            "Detach: {} ({}) (accidental)",
                            self.name().await,
                            self.user
                                .read()
                                .await
                                .as_ref()
                                .map(|u| u.read().await.user.clone())
                                .unwrap_or_default()
                        );
                    }
                    drop(current_telnet);
                    self.enqueue_others(Arc::new(DetachNotify::new(self.name_obj(), intentional)))
                        .await;
                    *self.telnet.write().await = None;
                }
            }
        } else {
            self.close(true).await;
        }
    }

    pub async fn save_input_line(&self, line: &str) {
        self.lines.lock().await.push(line.to_string());
    }

    pub async fn set_input_function(&self, input: Option<InputFunc>, prompt: Option<&str>) {
        *self.input_func.write().await = input;

        if let Some(p) = prompt {
            if let Some(telnet) = &*self.telnet.read().await {
                telnet.prompt(p).await;
            }
        }

        // Process any pending lines
        while let Some(func) = *self.input_func.read().await {
            let line = {
                let mut lines = self.lines.lock().await;
                if lines.is_empty() {
                    break;
                }
                lines.remove(0)
            };

            func(&self.clone(), &line).await;
            self.enqueue_output().await;
        }
    }

    pub async fn init_input_function(&self) {
        self.set_input_function(Some(Self::login_input), Some("login: "))
            .await;
    }

    pub async fn input(&self, line: &str) {
        self.pending.dequeue().await;

        if let Some(func) = *self.input_func.read().await {
            func(&self.clone(), line).await;
            self.enqueue_output().await;
        } else {
            self.save_input_line(line).await;
        }
    }

    pub async fn output(&self, text: &str) {
        self.output_buffer.lock().await.push_str(text);
    }

    pub async fn print(&self, format: &str) {
        self.output(format).await;
    }

    pub async fn announce(message: &str) {
        for session in SESSIONS.iter() {
            session.output(message).await;
            session.enqueue_output().await;
        }

        for session in INITS.iter() {
            session.output(message).await;
            session.enqueue_output().await;
        }
    }

    pub async fn remove_discussion(name: ArcStr) {
        DISCUSSIONS.remove(&name.to_string());
    }

    pub async fn enqueue(&self, out: Arc<dyn OutputObj>) {
        self.enqueue_output().await;
        if let Some(telnet) = &*self.telnet.read().await {
            self.pending.enqueue(Some(telnet), out).await;
        } else {
            self.pending.enqueue(None, out).await;
        }
    }

    pub async fn enqueue_output(&self) {
        let text = {
            let mut buf = self.output_buffer.lock().await;
            if buf.is_empty() {
                return;
            }
            std::mem::take(&mut *buf)
        };

        if let Some(telnet) = &*self.telnet.read().await {
            self.pending
                .enqueue(Some(telnet), Arc::new(Text::new(text)))
                .await;
        } else {
            self.pending.enqueue(None, Arc::new(Text::new(text))).await;
        }
    }

    pub async fn enqueue_others(&self, out: Arc<dyn OutputObj>) {
        for session in SESSIONS.iter() {
            if session.id != self.id {
                session.enqueue(out.clone()).await;
            }
        }
    }

    pub async fn acknowledge_output(&self) {
        self.pending.acknowledge().await;
    }

    pub async fn output_next(&self, telnet: &Telnet) -> bool {
        self.pending.send_next(telnet).await
    }

    pub fn find_sendable(
        &self,
        sendlist: &str,
        exact: bool,
        member: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) -> (
        Option<Arc<Session>>,
        OrderedSet<Session>,
        Option<Arc<Discussion>>,
        OrderedSet<Discussion>,
    ) {
        let mut session = None;
        let mut session_matches = OrderedSet::new();
        let mut discussion = None;
        let mut discussion_matches = OrderedSet::new();

        if do_sessions {
            if sendlist.eq_ignore_ascii_case("me") {
                session = Some(self.clone());
                session_matches.insert(self.clone());
                return (session, session_matches, discussion, discussion_matches);
            }

            for s in SESSIONS.iter() {
                let s_name = s.name().await;
                if s_name.eq_ignore_ascii_case(sendlist) {
                    session = Some(s.clone());
                    session_matches.insert(s.clone());
                } else if !exact {
                    if let Some(pos) = match_name(&s_name, sendlist) {
                        if pos == 1 {
                            session = Some(s.clone());
                        }
                        session_matches.insert(s.clone());
                    }
                }
            }
        }

        if do_discussions {
            for d in DISCUSSIONS.iter() {
                if member {
                    let members = d.members.read().await;
                    if !members.contains(&self.clone()) {
                        continue;
                    }
                }

                if d.name.eq_ignore_ascii_case(sendlist) {
                    discussion = Some(d.clone());
                    discussion_matches.insert(d.clone());
                } else if !exact {
                    if let Some(pos) = match_name(&d.name, sendlist) {
                        if pos == 1 {
                            discussion = Some(d.clone());
                        }
                        discussion_matches.insert(d.clone());
                    }
                }
            }
        }

        // If we found an exact match, return it
        if session.is_some() || discussion.is_some() {
            return (session, session_matches, discussion, discussion_matches);
        }

        // If we have exactly one match total, use it
        if session_matches.len() + discussion_matches.len() == 1 {
            if session_matches.len() == 1 {
                session = session_matches.first().cloned();
            }
            if discussion_matches.len() == 1 {
                discussion = discussion_matches.first().cloned();
            }
        }

        (session, session_matches, discussion, discussion_matches)
    }

    pub async fn notify_entry(&self) {
        if let Some(telnet) = &*self.telnet.read().await {
            info!(
                "Enter: {} ({}) on connection",
                self.name().await,
                self.user
                    .read()
                    .await
                    .as_ref()
                    .map(|u| u.read().await.user.clone())
                    .unwrap_or_default()
            );
        } else {
            info!(
                "Enter: {} ({}), detached",
                self.name().await,
                self.user
                    .read()
                    .await
                    .as_ref()
                    .map(|u| u.read().await.user.clone())
                    .unwrap_or_default()
            );
        }

        let now = Timestamp::new();
        *self.idle_since.write().await = now;
        *self.login_time.write().await = now;

        self.enqueue_others(Arc::new(EntryNotify::new(self.name_obj())))
            .await;
    }

    pub async fn notify_exit(&self) {
        if let Some(telnet) = &*self.telnet.read().await {
            info!(
                "Exit: {} ({}) on connection",
                self.name().await,
                self.user
                    .read()
                    .await
                    .as_ref()
                    .map(|u| u.read().await.user.clone())
                    .unwrap_or_default()
            );
        } else {
            info!(
                "Exit: {} ({}), detached",
                self.name().await,
                self.user
                    .read()
                    .await
                    .as_ref()
                    .map(|u| u.read().await.user.clone())
                    .unwrap_or_default()
            );
        }

        self.enqueue_others(Arc::new(ExitNotify::new(self.name_obj())))
            .await;
    }

    // Input handler functions
    async fn login_input(session: &Arc<Session>, line: &str) {
        let line = line.trim();

        if let Some(args) = match_keyword(line, "/bye", 4) {
            session.do_bye(args).await;
            return;
        }

        if line.is_empty() {
            if let Some(telnet) = &*session.telnet.read().await {
                telnet.prompt("login: ").await;
            }
            return;
        }

        let user_manager = UserManager::new();
        let user = user_manager.get_user(line).await;
        *session.user.write().await = user.clone();

        if user.is_none() || user.as_ref().unwrap().read().await.password.is_some() {
            // Need password
            if let Some(telnet) = &*session.telnet.read().await {
                let echo = telnet.get_echo().await;
                if !echo {
                    telnet
                        .output("\n\x07Sorry, password probably WILL echo.\n\n")
                        .await;
                } else if echo != TELNET_ENABLED {
                    telnet.output("\nWarning: password may echo.\n\n").await;
                }

                telnet.set_do_echo(false).await;
                session
                    .set_input_function(Some(Session::password_input), Some("Password: "))
                    .await;
            }
        } else {
            // No password required (guest account)
            session.print_reserved_names().await;
            session
                .set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
                .await;
        }
    }

    async fn password_input(session: &Arc<Session>, line: &str) {
        if let Some(telnet) = &*session.telnet.read().await {
            telnet.output("\n").await;
            telnet.set_do_echo(true).await;
        }

        let user_manager = UserManager::new();
        user_manager.update_all().await.ok();

        let valid = if let Some(user_lock) = &*session.user.read().await {
            let user = user_lock.read().await;
            if let Some(password) = &user.password {
                verify_password(line, password)
            } else {
                false
            }
        } else {
            false
        };

        if !valid {
            session.output("Login incorrect.\n").await;
            let attempts = session.attempts.fetch_add(1, Ordering::Relaxed) + 1;
            if attempts >= Session::MAX_LOGIN_ATTEMPTS {
                session.close(true).await;
                return;
            }

            session
                .set_input_function(Some(Session::login_input), Some("login: "))
                .await;
            *session.user.write().await = None;
            return;
        }

        session.print_reserved_names().await;
        session
            .set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
            .await;
    }

    async fn entered_name_input(session: &Arc<Session>, line: &str) {
        let line = line.trim();
        let name = if line.is_empty() {
            if let Some(user_lock) = &*session.user.read().await {
                let user = user_lock.read().await;
                if let Some(reserved) = user.reserved.first() {
                    reserved.clone()
                } else {
                    if let Some(telnet) = &*session.telnet.read().await {
                        telnet.prompt("Enter name: ").await;
                    }
                    return;
                }
            } else {
                return;
            }
        } else {
            ArcStr::new(line)
        };

        *session.name.write().await = name.clone();

        if session.check_name_availability(&name, false, false).await {
            session
                .set_input_function(Some(Session::entered_blurb_input), Some("Enter blurb: "))
                .await;
        }
    }

    async fn entered_blurb_input(session: &Arc<Session>, line: &str) {
        if !session
            .check_name_availability(&session.name().await, true, false)
            .await
        {
            return;
        }

        let line = if line.is_empty() {
            if let Some(user_lock) = &*session.user.read().await {
                let user = user_lock.read().await;
                user.blurb.as_ref().map(|b| b.as_str()).unwrap_or("")
            } else {
                ""
            }
        } else {
            line
        };

        session.do_blurb(line, true).await;

        if let Some(telnet) = &*session.telnet.read().await {
            telnet.login_sequence_finished().await;
        }

        session.signed_on.store(true, Ordering::Relaxed);

        if let Some(user_lock) = &*session.user.read().await {
            let mut user = user_lock.write().await;
            session.priv_level.store(user.priv_level, Ordering::Relaxed);
            user.add_session(session.clone());
        }

        SESSIONS.insert(session.id, session.clone());
        INITS.remove(&session.id);

        session.notify_entry().await;

        // Welcome message and automatic commands
        session
            .output("\n\nWelcome to Phoenix.  Type \"/help\" for a list of commands.\n\n")
            .await;

        // Make sure discussion A exists
        let (_, _, discussion, _) = session.find_sendable("A", true, false, false, true);
        if discussion.is_none() {
            let disc = Discussion::new(None, "A", "General Discussion", true);
            DISCUSSIONS.insert("A".to_string(), disc);
        }

        // Automatic commands
        session.do_join("A").await;
        session.do_send("A").await;
        session.do_who("").await;
        session.do_howmany("").await;

        if let Some(telnet) = &*session.telnet.read().await {
            telnet.reset_history().await;
        }

        session
            .set_input_function(Some(Session::process_input), None)
            .await;
    }

    async fn process_input(session: &Arc<Session>, line: &str) {
        let line = line.trim_end();

        if line.starts_with('!') {
            let line = line[1..].trim();
            if session.priv_level().await < 50 {
                session
                    .output("Sorry, all !commands are privileged.\n")
                    .await;
                return;
            }

            if let Some(args) = match_keyword(line, "!restart", 8) {
                session.do_restart(args).await;
            } else if let Some(args) = match_keyword(line, "!down", 5) {
                session.do_down(args).await;
            } else if let Some(args) = match_keyword(line, "!nuke", 5) {
                session.do_nuke(args).await;
            } else {
                session.output("Unknown !command.\n").await;
            }
        } else if line.starts_with('/') {
            let line = line[1..].trim();
            if let Some(args) = match_keyword(line, "/who", 2) {
                session.do_who(args).await;
            } else if let Some(args) = match_keyword(line, "/idle", 2) {
                session.do_idle(args).await;
            } else if let Some(args) = match_keyword(line, "/blurb", 3) {
                session.do_blurb(args, false).await;
            } else if let Some(args) = match_keyword(line, "/here", 2) {
                session.do_here(args).await;
            } else if let Some(args) = match_keyword(line, "/away", 2) {
                session.do_away(args).await;
            } else if let Some(args) = match_keyword(line, "/busy", 2) {
                session.do_busy(args).await;
            } else if let Some(args) = match_keyword(line, "/gone", 2) {
                session.do_gone(args).await;
            } else if let Some(args) = match_keyword(line, "/help", 2) {
                session.do_help(args).await;
            } else if let Some(args) = match_keyword(line, "/send", 2) {
                session.do_send(args).await;
            } else if let Some(args) = match_keyword(line, "/bye", 4) {
                session.do_bye(args).await;
            } else if let Some(args) = match_keyword(line, "/what", 3) {
                session.do_what(args).await;
            } else if let Some(args) = match_keyword(line, "/join", 2) {
                session.do_join(args).await;
            } else if let Some(args) = match_keyword(line, "/quit", 2) {
                session.do_quit(args).await;
            } else if let Some(args) = match_keyword(line, "/create", 3) {
                session.do_create(args).await;
            } else if let Some(args) = match_keyword(line, "/destroy", 4) {
                session.do_destroy(args).await;
            } else if let Some(args) = match_keyword(line, "/permit", 4) {
                session.do_permit(args).await;
            } else if let Some(args) = match_keyword(line, "/depermit", 4) {
                session.do_depermit(args).await;
            } else if let Some(args) = match_keyword(line, "/appoint", 4) {
                session.do_appoint(args).await;
            } else if let Some(args) = match_keyword(line, "/unappoint", 10) {
                session.do_unappoint(args).await;
            } else if let Some(args) = match_keyword(line, "/rename", 7) {
                session.do_rename(args).await;
            } else if let Some(args) = match_keyword(line, "/clear", 3) {
                session.do_clear(args).await;
            } else if let Some(args) = match_keyword(line, "/unidle", 7) {
                session.do_unidle(args).await;
            } else if let Some(args) = match_keyword(line, "/detach", 4) {
                session.do_detach(args).await;
            } else if let Some(args) = match_keyword(line, "/howmany", 3) {
                session.do_howmany(args).await;
            } else if let Some(args) = match_keyword(line, "/why", 4) {
                session.do_why(args).await;
            } else if let Some(args) = match_keyword(line, "/date", 3) {
                session.do_date(args).await;
            } else if let Some(args) = match_keyword(line, "/signal", 3) {
                session.do_signal(args).await;
            } else if let Some(args) = match_keyword(line, "/set", 4) {
                session.do_set(args).await;
            } else if let Some(args) = match_keyword(line, "/display", 2) {
                session.do_display(args).await;
            } else if let Some(args) = match_keyword(line, "/also", 3) {
                session.do_also(args).await;
            } else if let Some(args) = match_keyword(line, "/oops", 3) {
                session.do_oops(args).await;
            } else {
                session
                    .output("Unknown /command.  Type /help for help.\n")
                    .await;
            }
        } else if line == " " {
            session.do_reset().await;
        } else if !line.is_empty() {
            session.do_message(line).await;
        }
    }

    async fn print_reserved_names(&self) {
        if let Some(user_lock) = &*self.user.read().await {
            let user = user_lock.read().await;

            if let Some(first) = user.reserved.first() {
                self.print(&format!(
                    "\nYour default (reserved) name is \"{}\".\n",
                    first
                ))
                .await;

                let count = user.reserved.len();
                if count > 1 {
                    self.output("\nYou also have \"").await;
                    self.output(&user.reserved[1]).await;

                    for i in 2..count - 1 {
                        self.output("\", \"").await;
                        self.output(&user.reserved[i]).await;
                    }

                    if count > 2 {
                        self.output("\" and \"").await;
                        self.output(&user.reserved[count - 1]).await;
                    }

                    self.output("\" reserved.\n").await;
                }
            }

            self.output("\n").await;
        }
    }

    async fn check_name_availability(
        &self,
        name: &str,
        double_check: bool,
        transferring: bool,
    ) -> bool {
        let user_manager = UserManager::new();

        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  Choose another name.\n")
                .await;
            self.set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
                .await;
            return false;
        }

        if let Some((reserved, found_user)) = user_manager.find_reserved(name).await {
            let is_same_user = if let Some(my_user) = &*self.user.read().await {
                Arc::ptr_eq(my_user, &found_user)
            } else {
                false
            };

            if !is_same_user {
                self.print(&format!(
                    "\"{}\" is{} a reserved name.  Choose another.\n",
                    reserved,
                    if double_check { " now" } else { "" }
                ))
                .await;
                self.set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
                    .await;
                return false;
            }
        }

        let (session, _, discussion, _) = self.find_sendable(name, true, false, true, true);

        if let Some(found_session) = session {
            let same_user = if let (Some(my_user), Some(their_user)) =
                (&*self.user.read().await, &*found_session.user.read().await)
            {
                Arc::ptr_eq(my_user, their_user)
            } else {
                false
            };

            if same_user && found_session.priv_level().await > 0 {
                if let Some(their_telnet) = &*found_session.telnet.read().await {
                    if transferring {
                        self.output("Transferring active session...\n").await;
                        found_session
                            .transfer(self.telnet.read().await.as_ref().unwrap().clone())
                            .await;
                        *self.telnet.write().await = None;
                        self.close(true).await;
                    } else {
                        self.print(&format!(
                            "You are{} attached elsewhere under that name.\n",
                            if double_check { " now" } else { "" }
                        ))
                        .await;
                        self.set_input_function(
                            Some(Session::transfer_session_input),
                            Some("Transfer active session? [no] "),
                        )
                        .await;
                    }
                    return false;
                } else {
                    self.output("Attaching to detached session...\n").await;
                    found_session
                        .attach(self.telnet.read().await.as_ref().unwrap().clone())
                        .await;
                    *self.telnet.write().await = None;
                    self.close(true).await;
                    return false;
                }
            } else {
                self.print(&format!(
                    "The name \"{}\" is {} in use.  Choose another.\n",
                    found_session.name().await,
                    if double_check { "now" } else { "already" }
                ))
                .await;
                self.set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
                    .await;
                return false;
            }
        }

        if let Some(found_discussion) = discussion {
            self.print(&format!(
                "There is {} a discussion named \"{}\".  Choose another name.\n",
                if double_check { "now" } else { "already" },
                found_discussion.name
            ))
            .await;
            self.set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
                .await;
            return false;
        }

        true
    }

    async fn transfer_session_input(session: &Arc<Session>, line: &str) {
        if match_keyword(line, "yes", 1).is_none() {
            session.output("Session not transferred.\n").await;
            session
                .set_input_function(Some(Session::entered_name_input), Some("Enter name: "))
                .await;
            return;
        }

        if session
            .check_name_availability(&session.name().await, true, true)
            .await
        {
            session.output("(That session is now gone.)\n").await;
            session
                .set_input_function(Some(Session::entered_blurb_input), Some("Enter blurb: "))
                .await;
        }
    }

    // Command implementations
    async fn reset_idle(&self, min: usize) -> i64 {
        let now = Timestamp::new();
        let idle = (now - *self.idle_since.read().await) / 60;

        if min > 0 && idle >= min as i64 {
            self.output("[You were idle for").await;
            self.print_time_long(idle as i32).await;
            self.output(".]\n").await;
        }

        *self.idle_since.write().await = now;
        idle
    }

    async fn set_blurb(&self, new_blurb: Option<&str>) {
        self.reset_idle(10).await;

        let blurb = if let Some(text) = new_blurb {
            format!(" [{}]", text)
        } else {
            String::new()
        };

        *self.blurb.write().await = ArcStr::new(&blurb);
        *self.name_obj.write().await = Name::new(self.name().await, &blurb);
    }

    async fn print_time_long(&self, minutes: i32) {
        let format = if let Some(fmt) = self.sys_vars.read().await.get("time_format") {
            fmt.clone()
        } else {
            DEFAULTS
                .read()
                .await
                .get("time_format")
                .cloned()
                .unwrap_or_else(|| "verbose".to_string())
        };

        let hours = minutes / 60;
        let days = hours / 24;
        let minutes = minutes % 60;
        let hours = hours % 24;

        match format.as_str() {
            "verbose" => {
                if days > 0 || hours > 0 || minutes > 0 {
                    if minutes == 0 {
                        self.output(" exactly").await;
                    }
                    if days > 0 {
                        self.print(&format!(
                            " {} day{}{}",
                            days,
                            if days == 1 { "" } else { "s" },
                            if hours > 0 && minutes > 0 {
                                ","
                            } else if hours > 0 || minutes > 0 {
                                " and"
                            } else {
                                ""
                            }
                        ))
                        .await;
                    }
                    if hours > 0 {
                        self.print(&format!(
                            " {} hour{}{}",
                            hours,
                            if hours == 1 { "" } else { "s" },
                            if minutes > 0 { " and" } else { "" }
                        ))
                        .await;
                    }
                    if minutes > 0 {
                        self.print(&format!(
                            " {} minute{}",
                            minutes,
                            if minutes == 1 { "" } else { "s" }
                        ))
                        .await;
                    }
                } else {
                    self.output(" under a minute").await;
                }
            }
            "both" => {
                self.print_time_long_verbose(days, hours, minutes).await;
                self.output(" ").await;
                self.output("(").await;
                self.print_time_long_terse(days, hours, minutes).await;
                self.output(")").await;
            }
            "terse" | _ => {
                self.print_time_long_terse(days, hours, minutes).await;
            }
        }
    }

    async fn print_time_long_verbose(&self, days: i32, hours: i32, minutes: i32) {
        if days > 0 || hours > 0 || minutes > 0 {
            if minutes == 0 {
                self.output(" exactly").await;
            }
            if days > 0 {
                self.print(&format!(
                    " {} day{}{}",
                    days,
                    if days == 1 { "" } else { "s" },
                    if hours > 0 && minutes > 0 {
                        ","
                    } else if hours > 0 || minutes > 0 {
                        " and"
                    } else {
                        ""
                    }
                ))
                .await;
            }
            if hours > 0 {
                self.print(&format!(
                    " {} hour{}{}",
                    hours,
                    if hours == 1 { "" } else { "s" },
                    if minutes > 0 { " and" } else { "" }
                ))
                .await;
            }
            if minutes > 0 {
                self.print(&format!(
                    " {} minute{}",
                    minutes,
                    if minutes == 1 { "" } else { "s" }
                ))
                .await;
            }
        } else {
            self.output(" under a minute").await;
        }
    }

    async fn print_time_long_terse(&self, days: i32, hours: i32, minutes: i32) {
        if days > 0 {
            self.print(&format!("{}d{:02}:{:02}", days, hours, minutes))
                .await;
        } else {
            self.print(&format!("{}:{:02}", hours, minutes)).await;
        }
    }

    async fn do_restart(&self, args: &str) {
        let who = format!(
            "{} ({})",
            self.name().await,
            self.user
                .read()
                .await
                .as_ref()
                .map(|u| u.read().await.user.clone())
                .unwrap_or_default()
        );

        if args == "!" {
            if let Some(shutdown) = &*SHUTDOWN_EVENT.read().await {
                // Cancel existing shutdown
            }
            Self::announce(&format!(
                "*** {}{} has restarted Phoenix! ***\n",
                self.name().await,
                self.blurb().await
            ))
            .await;

            let event = Box::new(RestartEvent::immediate(who));
            *SHUTDOWN_EVENT.write().await = Some(Arc::new(event));
        } else if match_keyword(args, "cancel", 6).is_some() {
            if let Some(shutdown) = &*SHUTDOWN_EVENT.read().await {
                info!(
                    "Restart cancelled by {} ({})",
                    self.name().await,
                    self.user
                        .read()
                        .await
                        .as_ref()
                        .map(|u| u.read().await.user.clone())
                        .unwrap_or_default()
                );
                Self::announce(&format!(
                    "*** {}{} has cancelled the server restart. ***\n",
                    self.name().await,
                    self.blurb().await
                ))
                .await;
                *SHUTDOWN_EVENT.write().await = None;
            } else {
                self.output("The server was not about to shut down.\n")
                    .await;
            }
        } else {
            let seconds = args.parse::<i64>().unwrap_or(30);
            if let Some(shutdown) = &*SHUTDOWN_EVENT.read().await {
                // Cancel existing shutdown
            }
            Self::announce(&format!(
                "*** {}{} has restarted Phoenix! ***\n",
                self.name().await,
                self.blurb().await
            ))
            .await;

            let event = Box::new(RestartEvent::new(who, seconds));
            *SHUTDOWN_EVENT.write().await = Some(Arc::new(event));
        }
    }

    async fn do_down(&self, args: &str) {
        let who = format!(
            "{} ({})",
            self.name().await,
            self.user
                .read()
                .await
                .as_ref()
                .map(|u| u.read().await.user.clone())
                .unwrap_or_default()
        );

        if args == "!" {
            if let Some(shutdown) = &*SHUTDOWN_EVENT.read().await {
                // Cancel existing shutdown
            }
            Self::announce(&format!(
                "*** {}{} has shut down Phoenix! ***\n",
                self.name().await,
                self.blurb().await
            ))
            .await;

            let event = Box::new(ShutdownEvent::immediate(who));
            *SHUTDOWN_EVENT.write().await = Some(Arc::new(event));
        } else if match_keyword(args, "cancel", 6).is_some() {
            if let Some(shutdown) = &*SHUTDOWN_EVENT.read().await {
                info!(
                    "Shutdown cancelled by {} ({})",
                    self.name().await,
                    self.user
                        .read()
                        .await
                        .as_ref()
                        .map(|u| u.read().await.user.clone())
                        .unwrap_or_default()
                );
                Self::announce(&format!(
                    "*** {}{} has cancelled the server shutdown. ***\n",
                    self.name().await,
                    self.blurb().await
                ))
                .await;
                *SHUTDOWN_EVENT.write().await = None;
            } else {
                self.output("The server was not about to shut down.\n")
                    .await;
            }
        } else {
            let seconds = args.parse::<i64>().unwrap_or(30);
            if let Some(shutdown) = &*SHUTDOWN_EVENT.read().await {
                // Cancel existing shutdown
            }
            Self::announce(&format!(
                "*** {}{} has shut down Phoenix! ***\n",
                self.name().await,
                self.blurb().await
            ))
            .await;

            let event = Box::new(ShutdownEvent::new(who, seconds));
            *SHUTDOWN_EVENT.write().await = Some(Arc::new(event));
        }
    }

    async fn do_nuke(&self, args: &str) {
        let drain = !args.starts_with('!');
        let args = if drain { args } else { &args[1..] };

        let (session, matches, _, _) = self.find_sendable(args, false, false, true, false);

        if let Some(target) = session {
            if drain {
                self.print(&format!("\"{}\" has been nuked.\n", target.name().await))
                    .await;
            } else {
                self.print(&format!(
                    "\"{}\" has been nuked immediately.\n",
                    target.name().await
                ))
                .await;
            }

            if let Some(telnet) = &*target.telnet.read().await {
                *target.telnet.write().await = None;
                info!(
                    "{} ({}) has been nuked by {} ({})",
                    target.name().await,
                    target
                        .user
                        .read()
                        .await
                        .as_ref()
                        .map(|u| u.read().await.user.clone())
                        .unwrap_or_default(),
                    self.name().await,
                    self.user
                        .read()
                        .await
                        .as_ref()
                        .map(|u| u.read().await.user.clone())
                        .unwrap_or_default()
                );
                telnet.undraw_input().await;
                telnet
                    .print(&format!(
                        "\x07\x07\x07*** You have been nuked by {}{}. ***\n",
                        self.name().await,
                        self.blurb().await
                    ))
                    .await;
                telnet.redraw_input().await;
                telnet.close(drain).await;
            } else {
                info!(
                    "{} ({}), detached, has been nuked by {} ({})",
                    target.name().await,
                    target
                        .user
                        .read()
                        .await
                        .as_ref()
                        .map(|u| u.read().await.user.clone())
                        .unwrap_or_default(),
                    self.name().await,
                    self.user
                        .read()
                        .await
                        .as_ref()
                        .map(|u| u.read().await.user.clone())
                        .unwrap_or_default()
                );
                target.close(true).await;
            }
        } else {
            self.output("\x07\x07").await;
            self.session_matches(args, &matches).await;
        }
    }

    async fn do_bye(&self, _args: &str) {
        self.close(true).await;
    }

    async fn do_who(&self, args: &str) {
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return;
        }

        // TODO: Implement full /who display
        self.output("\n Name                              On Since  Idle  Away\n")
            .await;
        self.output(" ----                              --------  ----  ----\n")
            .await;

        let now = Timestamp::new();
        for session in who {
            if session.telnet.read().await.is_some() {
                self.output(" ").await;
            } else {
                self.output("~").await;
            }

            let name_blurb = format!("{}{}", session.name().await, session.blurb().await);
            if name_blurb.len() > 33 {
                self.print(&format!("{:<32.32}+ ", name_blurb)).await;
            } else {
                self.print(&format!("{:<33} ", name_blurb)).await;
            }

            // Login time
            let login_time = session.login_time.read().await;
            if (now - *login_time) < 86400 {
                self.output(&login_time.date(11, 8)).await;
            } else if (now - *login_time) < 31536000 {
                self.output(" ").await;
                self.output(&login_time.date(4, 6)).await;
                self.output(" ").await;
            } else {
                self.output(&login_time.date(4, 4)).await;
                self.output(&login_time.date(20, 4)).await;
            }

            // Idle time
            let idle = (now - *session.idle_since.read().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.print(&format!(" {:>2}d{:02}:{:02}  ", days, hours, minutes))
                        .await;
                } else if hours > 0 {
                    self.print(&format!("    {:>2}:{:02}  ", hours, minutes))
                        .await;
                } else {
                    self.print(&format!("       {:>2}  ", minutes)).await;
                }
            } else {
                self.output("           ").await;
            }

            // Away state
            match *session.away.read().await {
                AwayState::Here => self.output("Here\n").await,
                AwayState::Away => self.output("Away\n").await,
                AwayState::Busy => self.output("Busy\n").await,
                AwayState::Gone => self.output("Gone\n").await,
            }
        }

        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }
    }

    async fn do_idle(&self, args: &str) {
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return;
        }

        if who.len() == 1 {
            self.output("\n Name                              Idle\n")
                .await;
            self.output(" ----                              ----\n")
                .await;
        } else {
            self.output("\n Name                              Idle  Name                              Idle\n").await;
            self.output(
                " ----                              ----  ----                              ----\n",
            )
            .await;
        }

        let now = Timestamp::new();
        let mut col = 0;

        for session in who {
            if session.telnet.read().await.is_some() {
                self.output(" ").await;
            } else {
                self.output("~").await;
            }

            let name_blurb = format!("{}{}", session.name().await, session.blurb().await);
            self.print(&format!(
                "{:<32.32}{} ",
                name_blurb,
                if name_blurb.len() > 32 { "+" } else { " " }
            ))
            .await;

            let idle = (now - *session.idle_since.read().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 9 {
                    self.print(&format!("{:2}d{:02}", days, hours)).await;
                } else if days > 0 {
                    self.print(&format!("{}d{:02}h", days, hours)).await;
                } else if hours > 0 {
                    self.print(&format!("{:2}:{:02}", hours, minutes)).await;
                } else {
                    self.print(&format!("   {:2}", minutes)).await;
                }
            } else {
                self.output("     ").await;
            }

            if col == 1 {
                self.output("\n").await;
            } else {
                self.output(" ").await;
            }
            col = 1 - col;
        }

        if col == 1 {
            self.output("\n").await;
        }

        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }
    }

    async fn do_why(&self, args: &str) {
        if self.priv_level().await < 50 {
            self.output("Why not?\n").await;
            return;
        }

        // TODO: Implement privileged /why command
        self.do_who(args).await;
    }

    async fn do_blurb(&self, args: &str, entry: bool) {
        let args = args.trim();

        if !args.is_empty() {
            let mut start = 0;
            let mut end = args.len();

            if args.len() == 3 && args.eq_ignore_ascii_case("off") {
                if entry || !self.blurb().await.is_empty() {
                    self.set_blurb(None).await;
                    if !entry {
                        self.output("Your blurb has been turned off.\n").await;
                    }
                } else if !entry {
                    self.output("Your blurb was already turned off.\n").await;
                }
            } else {
                if (args.starts_with('"') && args.ends_with('"') && args.len() > 2)
                    || (args.starts_with('[') && args.ends_with(']'))
                {
                    start = 1;
                    end = args.len() - 1;
                }

                let blurb = &args[start..end];
                self.set_blurb(Some(blurb)).await;
                if !entry {
                    self.print(&format!(
                        "Your blurb has been set to{}.\n",
                        self.blurb().await
                    ))
                    .await;
                }
            }
        } else if entry {
            self.set_blurb(None).await;
        } else if !self.blurb().await.is_empty() {
            self.print(&format!(
                "Your blurb is currently set to{}.\n",
                self.blurb().await
            ))
            .await;
        } else {
            self.output("You do not currently have a blurb set.\n")
                .await;
        }
    }

    async fn do_here(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"here\".\n").await;
        *self.away.write().await = AwayState::Here;
        self.enqueue_others(Arc::new(HereNotify::new(self.name_obj())))
            .await;
    }

    async fn do_away(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"away\".\n").await;
        *self.away.write().await = AwayState::Away;
        self.enqueue_others(Arc::new(AwayNotify::new(self.name_obj())))
            .await;
    }

    async fn do_busy(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"busy\".\n").await;
        *self.away.write().await = AwayState::Busy;
        self.enqueue_others(Arc::new(BusyNotify::new(self.name_obj())))
            .await;
    }

    async fn do_gone(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"gone\".\n").await;
        *self.away.write().await = AwayState::Gone;
        self.enqueue_others(Arc::new(GoneNotify::new(self.name_obj())))
            .await;
    }

    async fn do_clear(&self, _args: &str) {
        self.output("\x1b[H\x1b[J").await;
    }

    async fn do_unidle(&self, _args: &str) {
        let idle = self.reset_idle(1).await;
        if idle == 0 {
            self.output("Your idle time has been reset.\n").await;
        }
    }

    async fn do_detach(&self, _args: &str) {
        if self.priv_level().await > 0 {
            self.reset_idle(10).await;
            self.output("You have been detached.\n").await;
            self.enqueue_output().await;
            if let Some(telnet) = &*self.telnet.read().await {
                telnet.close(true).await;
            }
        } else {
            self.output(
                "Guest users are not allowed to detach from the system.  Use /bye to sign off.\n",
            )
            .await;
        }
    }

    async fn do_howmany(&self, _args: &str) {
        let mut here = 0;
        let mut away = 0;
        let mut busy = 0;
        let mut gone = 0;
        let mut attached = 0;
        let mut detached = 0;
        let mut total = 0;

        for session in SESSIONS.iter() {
            match *session.away.read().await {
                AwayState::Here => here += 1,
                AwayState::Away => away += 1,
                AwayState::Busy => busy += 1,
                AwayState::Gone => gone += 1,
            }
            if session.telnet.read().await.is_some() {
                attached += 1;
            } else {
                detached += 1;
            }
            total += 1;
        }

        self.output("\nActive Users:\n\n").await;
        self.output(
            "  \"Here\"     \"Away\"     \"Busy\"     \"Gone\"    Attached   Detached    Total\n",
        )
        .await;
        self.print(&format!(" {:3} {:3}%   {:3} {:3}%   {:3} {:3}%   {:3} {:3}%   {:3} {:3}%   {:3} {:3}%   {:3} 100%\n",
            here, (here * 100 + total / 2) / total.max(1),
            away, (away * 100 + total / 2) / total.max(1),
            busy, (busy * 100 + total / 2) / total.max(1),
            gone, (gone * 100 + total / 2) / total.max(1),
            attached, (attached * 100 + total / 2) / total.max(1),
            detached, (detached * 100 + total / 2) / total.max(1),
            total
        )).await;

        self.print(&format!("\nDiscussions in use: {}\n\n", DISCUSSIONS.len()))
            .await;
    }

    async fn do_what(&self, args: &str) {
        if DISCUSSIONS.is_empty() {
            self.output("No discussions currently exist.\n").await;
            return;
        }

        let sendlist = Sendlist::new(&self.clone(), args, true, false, true);

        if !args.is_empty() && sendlist.discussions.is_empty() {
            self.output(&sendlist.errors).await;
            return;
        }

        let discussions = if args.is_empty() {
            DISCUSSIONS.iter().map(|r| r.value().clone()).collect()
        } else {
            sendlist.discussions.clone()
        };

        self.output("\n Name            Users  Idle  Title\n").await;
        self.output(" ----            -----  ----  -----\n").await;

        let now = Timestamp::new();

        for disc in discussions {
            self.output(" ").await;
            let name = if disc.name.len() > 15 {
                format!("{:<14.14}+", disc.name)
            } else {
                format!("{:<15}", disc.name)
            };
            self.output(&name).await;

            let members = disc.members.read().await;
            let is_member = members.contains(&self.clone());
            self.print(&format!(
                "{:>3}{} ",
                members.len(),
                if is_member { '*' } else { ' ' }
            ))
            .await;

            let idle = (now - *disc.idle_since.read().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.print(&format!(" {}d{:02}:{:02}  ", days, hours, minutes))
                        .await;
                } else if hours > 0 {
                    self.print(&format!("    {}:{:02}  ", hours, minutes)).await;
                } else {
                    self.print(&format!("      {:>2}  ", minutes)).await;
                }
            } else {
                self.output("         ").await;
            }

            if disc.permitted(&self.clone()).await {
                if disc.title.len() > 49 {
                    self.print(&format!("{:<48.48}+\n", disc.title)).await;
                } else {
                    self.print(&format!("{}\n", disc.title)).await;
                }
            } else {
                self.output("<Private>\n").await;
            }
        }

        if !sendlist.errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors).await;
        }
    }

    async fn do_date(&self, _args: &str) {
        let t = Timestamp::new();
        self.print(&format!("{}\n", t.date(0, 0))).await;
    }

    async fn do_signal(&self, args: &str) {
        let mut args = args;

        if let Some(rest) = match_keyword(args, "on", 2) {
            self.signal_public.store(true, Ordering::Relaxed);
            self.signal_private.store(true, Ordering::Relaxed);
            self.output("All signals are now on.\n").await;
        } else if let Some(rest) = match_keyword(args, "off", 2) {
            self.signal_public.store(false, Ordering::Relaxed);
            self.signal_private.store(false, Ordering::Relaxed);
            self.output("All signals are now off.\n").await;
        } else if let Some(rest) = match_keyword(args, "public", 2) {
            args = rest;
            if let Some(_) = match_keyword(args, "on", 2) {
                self.signal_public.store(true, Ordering::Relaxed);
                self.output("Signals for public messages are now on.\n")
                    .await;
            } else if let Some(_) = match_keyword(args, "off", 2) {
                self.signal_public.store(false, Ordering::Relaxed);
                self.output("Signals for public messages are now off.\n")
                    .await;
            } else if args.is_empty() {
                self.print(&format!(
                    "Signals are {} for public messages.\n",
                    if self.signal_public.load(Ordering::Relaxed) {
                        "on"
                    } else {
                        "off"
                    }
                ))
                .await;
            } else {
                self.output("Usage: /signal public [on|off]\n").await;
            }
        } else if let Some(rest) = match_keyword(args, "private", 2) {
            args = rest;
            if let Some(_) = match_keyword(args, "on", 2) {
                self.signal_private.store(true, Ordering::Relaxed);
                self.output("Signals for private messages are now on.\n")
                    .await;
            } else if let Some(_) = match_keyword(args, "off", 2) {
                self.signal_private.store(false, Ordering::Relaxed);
                self.output("Signals for private messages are now off.\n")
                    .await;
            } else if args.is_empty() {
                self.print(&format!(
                    "Signals are {} for private messages.\n",
                    if self.signal_private.load(Ordering::Relaxed) {
                        "on"
                    } else {
                        "off"
                    }
                ))
                .await;
            } else {
                self.output("Usage: /signal private [on|off]\n").await;
            }
        } else if args.is_empty() {
            let pub_sig = self.signal_public.load(Ordering::Relaxed);
            let priv_sig = self.signal_private.load(Ordering::Relaxed);

            if pub_sig == priv_sig {
                self.print(&format!(
                    "Signals are {} for both public and private messages.\n",
                    if pub_sig { "on" } else { "off" }
                ))
                .await;
            } else {
                self.print(&format!(
                    "Signals are {} for public messages and {} for private messages.\n",
                    if pub_sig { "on" } else { "off" },
                    if priv_sig { "on" } else { "off" }
                ))
                .await;
            }
        } else {
            self.output("Usage: /signal [public|private] [on|off]\n")
                .await;
        }
    }

    async fn do_send(&self, args: &str) {
        if args.is_empty() {
            if let Some(sendlist) = &*self.default_sendlist.read().await {
                self.output("You are sending to ").await;
                self.print_sendlist(sendlist).await;
                self.output(".\n").await;
            } else {
                self.output("Your default sendlist is turned off.\n").await;
            }
            return;
        }

        if args.eq_ignore_ascii_case("off") {
            *self.default_sendlist.write().await = None;
            self.output("Your default sendlist has been turned off.\n")
                .await;
            return;
        }

        // Parse the sendlist
        let mut slist = String::new();
        for ch in args.chars() {
            match ch {
                '\\' => {
                    // Handle escape
                }
                '"' => {
                    // Handle quoted section
                }
                '_' => slist.push(UNQUOTED_UNDERSCORE as char),
                ',' => slist.push(SEPARATOR as char),
                _ => slist.push(ch),
            }
        }

        let sendlist = Sendlist::new(&self.clone(), &slist, false, true, true);

        if !sendlist.errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors).await;
        }

        if sendlist.sessions.is_empty() && sendlist.discussions.is_empty() {
            self.output("Your default sendlist is unchanged.\n").await;
            return;
        }

        *self.default_sendlist.write().await = Some(sendlist.clone());
        self.output("You are now sending to ").await;
        self.print_sendlist(&sendlist).await;
        self.output(".\n").await;
    }

    async fn print_sendlist(&self, sendlist: &Sendlist) {
        if !sendlist.sessions.is_empty() {
            let mut first = true;
            for session in &sendlist.sessions {
                if first {
                    first = false;
                } else {
                    self.output(", ").await;
                }
                self.output(&session.name().await).await;
            }

            if !sendlist.discussions.is_empty() {
                self.print(&format!(
                    " and discussion{} ",
                    if sendlist.discussions.len() == 1 {
                        ""
                    } else {
                        "s"
                    }
                ))
                .await;

                first = true;
                for discussion in &sendlist.discussions {
                    if first {
                        first = false;
                    } else {
                        self.output(", ").await;
                    }
                    self.output(&discussion.name).await;
                }
            }
        } else {
            let mut first = true;
            for discussion in &sendlist.discussions {
                if first {
                    first = false;
                } else {
                    self.output(", ").await;
                }
                self.output(&discussion.name).await;
            }
        }
    }

    async fn do_join(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /join <disc>[,<disc>...]\n").await;
            return;
        }

        let (name, _) = getword(args, Some(','));
        let (_, _, discussion, matches) = self.find_sendable(name, false, false, false, true);

        if let Some(disc) = discussion {
            disc.join(self.clone()).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_quit(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /quit <disc>[,<disc>...]\n").await;
            return;
        }

        let (name, _) = getword(args, Some(','));
        let (_, _, discussion, matches) = self.find_sendable(name, false, true, false, true);

        if let Some(disc) = discussion {
            disc.quit(self.clone()).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_create(&self, args: &str) {
        let mut args = args;
        let mut is_public = true;

        if let Some(rest) = match_keyword(args, "-public", 3) {
            is_public = true;
            args = rest;
        } else if let Some(rest) = match_keyword(args, "-private", 3) {
            is_public = false;
            args = rest;
        } else if let Some(rest) = match_keyword(args, "public", 6) {
            is_public = true;
            args = rest;
        } else if let Some(rest) = match_keyword(args, "private", 7) {
            is_public = false;
            args = rest;
        }

        let (name, title) = getword(args, None);
        if name.is_empty() || title.is_empty() {
            self.output("Usage: /create [public|private] <name> <title>\n")
                .await;
            return;
        }

        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (not created)\n")
                .await;
            return;
        }

        let user_manager = UserManager::new();
        if let Some((reserved, found_user)) = user_manager.find_reserved(name).await {
            let is_same_user = if let Some(my_user) = &*self.user.read().await {
                Arc::ptr_eq(my_user, &found_user)
            } else {
                false
            };

            self.print(&format!(
                "\"{}\" is {} reserved name. (not created)\n",
                reserved,
                if is_same_user { "your" } else { "a" }
            ))
            .await;
            return;
        }

        let (session, _, discussion, _) = self.find_sendable(name, true, false, true, true);

        if let Some(s) = session {
            self.print(&format!(
                "There is already someone named \"{}\". (not created)\n",
                s.name().await
            ))
            .await;
            return;
        }

        if let Some(d) = discussion {
            self.print(&format!(
                "There is already a discussion named \"{}\". (not created)\n",
                d.name
            ))
            .await;
            return;
        }

        let disc = Discussion::new(Some(self.clone()), name, title, is_public);
        DISCUSSIONS.insert(name.to_string(), disc.clone());

        self.enqueue_others(Arc::new(CreateNotify::new(
            disc.name.clone(),
            disc.title.clone(),
            disc.is_public,
            self.name_obj(),
        )))
        .await;

        self.print(&format!(
            "You have created discussion {}, \"{}\".\n",
            disc.name, disc.title
        ))
        .await;
    }

    async fn do_destroy(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /destroy <disc>[,<disc>...]\n").await;
            return;
        }

        let (name, _) = getword(args, Some(','));
        let (_, _, discussion, matches) = self.find_sendable(name, false, true, false, true);

        if let Some(disc) = discussion {
            disc.destroy(self.clone()).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_permit(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /permit <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (_, _, discussion, matches) = self.find_sendable(name, false, true, false, true);

        if let Some(disc) = discussion {
            disc.permit(self.clone(), rest).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_depermit(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /depermit <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (_, _, discussion, matches) = self.find_sendable(name, false, true, false, true);

        if let Some(disc) = discussion {
            disc.depermit(self.clone(), rest).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_appoint(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /appoint <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (_, _, discussion, matches) = self.find_sendable(name, false, true, false, true);

        if let Some(disc) = discussion {
            disc.appoint(self.clone(), rest).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_unappoint(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /unappoint <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (_, _, discussion, matches) = self.find_sendable(name, false, true, false, true);

        if let Some(disc) = discussion {
            disc.unappoint(self.clone(), rest).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    async fn do_rename(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /rename <name>\n").await;
            return;
        }

        if args.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (name unchanged)\n")
                .await;
            return;
        }

        let user_manager = UserManager::new();
        if let Some((reserved, found_user)) = user_manager.find_reserved(args).await {
            let is_same_user = if let Some(my_user) = &*self.user.read().await {
                Arc::ptr_eq(my_user, &found_user)
            } else {
                false
            };

            if !is_same_user {
                self.print(&format!(
                    "\"{}\" is a reserved name.  (name unchanged)\n",
                    reserved
                ))
                .await;
                return;
            }
        }

        let (session, _, discussion, _) = self.find_sendable(args, true, false, true, true);

        if let Some(s) = session {
            if s.id != self.id {
                self.output("That name is already in use.  (name unchanged)\n")
                    .await;
                return;
            }
        }

        if let Some(d) = discussion {
            self.print(&format!(
                "There is already a discussion named \"{}\". (name unchanged)\n",
                d.name
            ))
            .await;
            return;
        }

        self.enqueue_others(Arc::new(RenameNotify::new(self.name().await, args)))
            .await;

        self.print(&format!("You have changed your name to \"{}\".\n", args))
            .await;
        *self.name.write().await = ArcStr::new(args);
        *self.name_obj.write().await = Name::new(args, self.blurb().await);
    }

    async fn do_set(&self, args: &str) {
        let (var, value) = getword(args, Some('='));
        if var.is_empty() || value.is_empty() {
            self.output("Usage: /set <variable>=<value>\n").await;
            return;
        }

        if var.starts_with('$') {
            self.user_vars
                .write()
                .await
                .insert(var.to_string(), value.to_string());
        } else if let Some(_) = match_keyword(var, "echo", 4) {
            if let Some(telnet) = &*self.telnet.read().await {
                let (val, _) = getword(value, None);
                if let Some(_) = match_keyword(val, "on", 2) {
                    telnet.set_echo(true).await;
                    self.output("Remote echoing is now enabled.\n").await;
                } else if let Some(_) = match_keyword(val, "off", 3) {
                    telnet.set_echo(false).await;
                    self.output("Remote echoing is now disabled.\n").await;
                } else {
                    self.output("Usage: /set echo=[on|off]\n").await;
                }
            }
        } else if let Some(_) = match_keyword(var, "height", 6) {
            if let Ok(height) = value.parse::<usize>() {
                if height > 0 {
                    if let Some(telnet) = &*self.telnet.read().await {
                        let h = telnet.set_height(height).await;
                        self.print(&format!("Terminal height is now set to {}.\n", h))
                            .await;
                    }
                } else {
                    self.output("Usage: /set height=<number of rows>\n").await;
                }
            } else {
                self.output("Usage: /set height=<number of rows>\n").await;
            }
        } else if let Some(_) = match_keyword(var, "idle", 4) {
            self.set_idle(value).await;
        } else if let Some(_) = match_keyword(var, "time_format", 11) {
            if let Some(_) = match_keyword(value, "verbose", 7) {
                self.sys_vars
                    .write()
                    .await
                    .insert("time_format".to_string(), "verbose".to_string());
            } else if let Some(_) = match_keyword(value, "both", 4) {
                self.sys_vars
                    .write()
                    .await
                    .insert("time_format".to_string(), "both".to_string());
            } else if let Some(_) = match_keyword(value, "terse", 5) {
                self.sys_vars
                    .write()
                    .await
                    .insert("time_format".to_string(), "terse".to_string());
            } else if let Some(_) = match_keyword(value, "default", 7) {
                self.sys_vars.write().await.remove("time_format");
            } else {
                self.output("Usage: /set time_format [terse|verbose|both|default]\n")
                    .await;
            }
        } else if let Some(_) = match_keyword(var, "uptime", 6) {
            self.output("Server uptime is a readonly variable.\n").await;
        } else if let Some(_) = match_keyword(var, "width", 5) {
            if let Ok(width) = value.parse::<usize>() {
                if width > 0 {
                    if let Some(telnet) = &*self.telnet.read().await {
                        let w = telnet.set_width(width).await;
                        self.print(&format!("Terminal width is now set to {}.\n", w))
                            .await;
                    }
                } else {
                    self.output("Usage: /set width=<number of columns>\n").await;
                }
            } else {
                self.output("Usage: /set width=<number of columns>\n").await;
            }
        } else {
            self.print(&format!("Unknown system variable: \"{}\"\n", var))
                .await;
        }
    }

    async fn set_idle(&self, args: &str) {
        // TODO: Implement idle time parsing and setting
        self.output("Idle time setting not yet implemented.\n")
            .await;
    }

    async fn do_display(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /display <variable>[,<variable>...]\n")
                .await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (var, rest) = getword(remaining, Some(','));
            remaining = rest;

            if var.starts_with('$') {
                if let Some(value) = self.user_vars.read().await.get(var) {
                    self.print(&format!("{} = \"{}\"\n", var, value)).await;
                } else {
                    self.print(&format!("Unknown user variable: \"{}\"\n", var))
                        .await;
                }
            } else if let Some(_) = match_keyword(var, "echo", 4) {
                if let Some(telnet) = &*self.telnet.read().await {
                    if telnet.get_echo().await {
                        self.output("Remote echoing is currently enabled.\n").await;
                    } else {
                        self.output("Remote echoing is currently disabled.\n").await;
                    }
                }
            } else if let Some(_) = match_keyword(var, "height", 6) {
                if let Some(telnet) = &*self.telnet.read().await {
                    let height = telnet.set_height(0).await;
                    self.print(&format!(
                        "Terminal height is currently set to {}.\n",
                        height
                    ))
                    .await;
                }
            } else if let Some(_) = match_keyword(var, "idle", 4) {
                let now = Timestamp::new();
                self.output("Your idle time is").await;
                self.print_time_long(((now - *self.idle_since.read().await) / 60) as i32)
                    .await;
                self.output(".\n").await;
            } else if let Some(_) = match_keyword(var, "time_format", 11) {
                self.output("Your time format is ").await;
                if let Some(format) = self.sys_vars.read().await.get("time_format") {
                    match format.as_str() {
                        "verbose" => self.output("verbose.\n").await,
                        "both" => self.output("both verbose and terse.\n").await,
                        "terse" => self.output("terse.\n").await,
                        _ => self.output("unknown.\n").await,
                    }
                } else {
                    self.output("the default: ").await;
                    let default = DEFAULTS
                        .read()
                        .await
                        .get("time_format")
                        .cloned()
                        .unwrap_or_else(|| "verbose".to_string());
                    match default.as_str() {
                        "verbose" => self.output("verbose.\n").await,
                        "both" => self.output("both verbose and terse.\n").await,
                        "terse" => self.output("terse.\n").await,
                        _ => self.output("unknown.\n").await,
                    }
                }
            } else if let Some(_) = match_keyword(var, "uptime", 6) {
                let uptime = if let Some(system) = system_uptime() {
                    // TODO: Use actual server start time
                    system / 60
                } else {
                    let now = Timestamp::new();
                    // TODO: Use actual server start time
                    (now.unix() / 60) as i64
                };

                self.output("This server has been running for").await;
                self.print_time_long(uptime as i32).await;
                self.output(".\n").await;

                if let Some(system) = system_uptime() {
                    let system = system / 60;
                    self.output("(This machine has been running for").await;
                    self.print_time_long(system as i32).await;
                    self.output(".)\n").await;
                }
            } else if let Some(_) = match_keyword(var, "version", 7) {
                self.print(&format!("Phoenix server version: {}\n", crate::VERSION))
                    .await;
            } else if let Some(_) = match_keyword(var, "width", 5) {
                if let Some(telnet) = &*self.telnet.read().await {
                    let width = telnet.set_width(0).await;
                    self.print(&format!("Terminal width is currently set to {}.\n", width))
                        .await;
                }
            } else {
                self.print(&format!("Unknown system variable: \"{}\"\n", var))
                    .await;
            }
        }
    }

    async fn do_also(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /also <sendlist>\n").await;
            return;
        }

        if let Some(last_msg) = &*self.last_message.read().await {
            let sendlist = Sendlist::new(&self.clone(), args, false, true, true);
            self.send_message(&sendlist, &last_msg.text).await;
        } else {
            self.output("You have no previous message to resend.\n")
                .await;
        }
    }

    async fn do_oops(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /oops <sendlist> OR /oops text [<message>]\n")
                .await;
            return;
        }

        if let Some(text_args) = match_keyword(args, "text", 4) {
            let text = text_args.trim();
            if !text.is_empty() {
                *self.oops_text.write().await = text.to_string();
                self.print(&format!("Your /oops text is now \"{}\".\n", text))
                    .await;
            } else {
                self.print(&format!(
                    "Your /oops text is currently \"{}\".\n",
                    self.oops_text.read().await
                ))
                .await;
            }
        } else {
            if let Some(last_msg) = &*self.last_message.read().await {
                let sendlist = Sendlist::new(&self.clone(), args, false, true, true);
                let text = last_msg.text.clone();
                let to = last_msg.to.clone();

                self.send_message(&to, &self.oops_text.read().await).await;
                self.send_message(&sendlist, &text).await;
                *self.last_sendlist.write().await = Some(sendlist.clone());
            } else {
                self.output("You have no previous message to resend.\n")
                    .await;
            }
        }
    }

    async fn do_help(&self, args: &str) {
        // TODO: Load help from external file
        self.output("Help system not yet implemented. Known commands:\n\n")
            .await;
        self.output("   /who     /blurb    /create    /permit     /clear     /howmany\n")
            .await;
        self.output("   /what    /here     /destroy   /depermit   /unidle    /detach\n")
            .await;
        self.output("   /why     /away     /join      /appoint    /date      /bye\n")
            .await;
        self.output("   /idle    /busy     /quit      /unappoint  /set\n")
            .await;
        self.output("   /help    /gone     /send      /rename     /signal\n\n")
            .await;
        self.output("Type \"/help <command>\" for more information about a particular command.\n")
            .await;
    }

    async fn do_reset(&self) {
        self.reset_idle(1).await;
    }

    async fn do_message(&self, line: &str) {
        let (msg_start, sendlist_str, mut last_explicit, is_explicit) = message_start(line);
        let msg_start = msg_start.trim();

        if is_explicit {
            *self.last_explicit.write().await = last_explicit;
        }

        let sendlist = if sendlist_str.is_empty() {
            if let Some(last) = &*self.last_sendlist.read().await {
                last.clone()
            } else {
                self.output("\x07\x07You have no previous sendlist. (message not sent)\n")
                    .await;
                return;
            }
        } else if sendlist_str.eq_ignore_ascii_case("default") {
            if let Some(default) = &*self.default_sendlist.read().await {
                default.clone()
            } else {
                self.output("\x07\x07You have no default sendlist. (message not sent)\n")
                    .await;
                return;
            }
        } else {
            Arc::new(Sendlist::new(
                &self.clone(),
                &sendlist_str,
                false,
                true,
                true,
            ))
        };

        *self.last_sendlist.write().await = Some(sendlist.clone());

        if msg_start.is_empty() {
            if sendlist_str == "default" {
                self.output("\x07\x07There is no message after \"default\". (message not sent)\n")
                    .await;
            } else if is_explicit {
                self.print(&format!(
                    "\x07\x07There is no message after \"{}:\". (message not sent)\n",
                    sendlist.typed
                ))
                .await;
            } else {
                self.print(&format!(
                    "\x07\x07There is no message after \"{};\". (message not sent)\n",
                    sendlist.typed
                ))
                .await;
            }
            return;
        }

        self.send_message(&sendlist, msg_start).await;
    }

    async fn send_message(&self, sendlist: &Arc<Sendlist>, text: &str) {
        if !sendlist.errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors).await;
        }

        if sendlist.sessions.is_empty() && sendlist.discussions.is_empty() {
            self.output("Your message is unchanged.\n").await;
            return;
        }

        let idle = self.reset_idle(30).await;

        let mut who = OrderedSet::new();
        let count = sendlist.expand(&mut who, Some(&self.clone())).await;

        let output_type = if count > 1 || !sendlist.discussions.is_empty() {
            OutputType::PublicMessage
        } else {
            OutputType::PrivateMessage
        };

        let msg = Arc::new(Message::new(
            output_type,
            self.name_obj(),
            sendlist.clone(),
            text,
        ));
        *self.last_message.write().await = Some(msg.clone());

        for session in &who {
            session.enqueue(msg.clone()).await;
        }

        for disc in &sendlist.discussions {
            *disc.idle_since.write().await = Timestamp::new();
        }

        self.output("(message sent to ").await;
        self.print_sendlist(sendlist).await;
        self.output(")");

        if idle >= 30 {
            self.output(&format!(" [idle {}]", idle)).await;
        }
        self.output("\n").await;
    }

    async fn get_who_set(&self, args: &str) -> (OrderedSet<Session>, String, String) {
        let mut who = OrderedSet::new();
        let mut errors = String::new();
        let mut msg = String::new();

        if args.is_empty() {
            // Show all sessions
            for session in SESSIONS.iter() {
                who.insert(session.value().clone());
            }
            msg = format!("\n{} signed on.\n", who.len());
        } else {
            let sendlist = Sendlist::new(&self.clone(), args, true, true, true);

            let mut total = sendlist.expand(&mut who, None).await;

            if !sendlist.errors.is_empty() {
                errors = sendlist.errors.clone();
            }

            if who.is_empty() {
                if errors.is_empty() {
                    errors = "No one matched your request.\n".to_string();
                }
            } else {
                msg = format!(
                    "\n{} user{} matched.\n",
                    who.len(),
                    if who.len() == 1 { "" } else { "s" }
                );
            }
        }

        (who, errors, msg)
    }

    async fn session_matches(&self, name: &str, matches: &OrderedSet<Session>) {
        if matches.is_empty() {
            self.print(&format!("No names matched \"{}\".\n", name))
                .await;
        } else if matches.len() == 1 {
            self.print(&format!(
                "\"{}\" matches one name: {}.\n",
                name,
                matches.iter().next().unwrap().name().await
            ))
            .await;
        } else {
            self.print(&format!("\"{}\" matches {} names: ", name, matches.len()))
                .await;

            let names: Vec<String> = matches.iter().map(|s| s.name().to_string()).collect();
            self.output(&names.join(", ")).await;
            self.output(".\n").await;
        }
    }

    async fn discussion_matches(&self, name: &str, matches: &OrderedSet<Discussion>) {
        if matches.is_empty() {
            self.print(&format!("No discussions matched \"{}\".\n", name))
                .await;
        } else if matches.len() == 1 {
            self.print(&format!(
                "\"{}\" matches one discussion: {}.\n",
                name,
                matches.iter().next().unwrap().name
            ))
            .await;
        } else {
            self.print(&format!(
                "\"{}\" matches {} discussions: ",
                name,
                matches.len()
            ))
            .await;

            let names: Vec<&str> = matches.iter().map(|d| d.name.as_ref()).collect();
            self.output(&names.join(", ")).await;
            self.output(".\n").await;
        }
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Session {}

impl std::hash::Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

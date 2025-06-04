use crate::constants::*;
use crate::discussion::Discussion;
use crate::name::Name;
use crate::output::*;
use crate::sendlist::{message_start, Sendlist};
use crate::server::PhoenixServer;
use crate::telnet::{Telnet, TELNET_ENABLED};
use crate::timestamp::{system_uptime, Timestamp};
use crate::types::*;
use crate::user::{verify_password, User, UserManager};
use crate::VERSION;
use dashmap::DashMap;
use log::info;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::task::AbortHandle;

const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

static INITS: LazyLock<DashMap<usize, Arc<Session>>> = LazyLock::new(DashMap::new);
static SESSIONS: LazyLock<DashMap<usize, Arc<Session>>> = LazyLock::new(DashMap::new);
static DISCUSSIONS: LazyLock<DashMap<String, Arc<Discussion>>> = LazyLock::new(DashMap::new);
static SESSION_COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));
static DEFAULTS: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("time_format".to_string(), "verbose".to_string());
    RwLock::new(map)
});
static USER_MANAGER: LazyLock<UserManager> = LazyLock::new(UserManager::new);

pub struct Session {
    pub id: usize,
    pub server: Arc<PhoenixServer>,
    pub user: Arc<RwLock<Option<Arc<RwLock<User>>>>>,
    pub telnet: Arc<RwLock<Option<Arc<Telnet>>>>,
    pub login_timeout: Arc<RwLock<Option<AbortHandle>>>,
    pub login_state: Arc<RwLock<LoginState>>,
    pub lines: Arc<Mutex<Vec<String>>>,
    pub output_buffer: Arc<Mutex<String>>,
    pub pending: Arc<OutputStream>,
    pub user_vars: Arc<RwLock<HashMap<String, String>>>,
    pub sys_vars: Arc<RwLock<HashMap<String, String>>>,
    pub login_time: Arc<RwLock<Timestamp>>,
    pub idle_since: Arc<RwLock<Timestamp>>,
    pub away: Arc<RwLock<AwayState>>,
    pub signal_public: Arc<AtomicBool>,
    pub signal_private: Arc<AtomicBool>,
    pub signed_on: Arc<AtomicBool>,
    pub closing: Arc<AtomicBool>,
    pub attempts: Arc<AtomicI32>,
    pub priv_level: Arc<AtomicI32>,
    pub name: Arc<RwLock<ArcStr>>,
    pub blurb: Arc<RwLock<ArcStr>>,
    pub name_obj: Arc<RwLock<Arc<Name>>>,
    pub last_message: Arc<RwLock<Option<Arc<Message>>>>,
    pub default_sendlist: Arc<RwLock<Option<Arc<Sendlist>>>>,
    pub last_sendlist: Arc<RwLock<Option<Arc<Sendlist>>>>,
    pub last_explicit: Arc<RwLock<String>>,
    pub reply_sendlist: Arc<RwLock<String>>,
    pub oops_text: Arc<RwLock<String>>,
}

#[derive(Debug, Clone)]
pub enum LoginState {
    PreLogin,
    AwaitingLogin,
    AwaitingPassword,
    AwaitingName,
    AwaitingBlurb,
    AwaitingTransferConfirmation,
    LoggedIn,
}

impl Session {
    pub const MAX_LOGIN_ATTEMPTS: i32 = 3;

    pub async fn new(server: Arc<PhoenixServer>, telnet: Arc<Telnet>) -> Arc<Self> {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let now = Timestamp::new();

        let session = Arc::new(Self {
            id,
            server,
            user: Arc::new(RwLock::new(None)),
            telnet: Arc::new(RwLock::new(Some(telnet.clone()))),
            login_timeout: Arc::new(RwLock::new(None)),
            login_state: Arc::new(RwLock::new(LoginState::PreLogin)),
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

        // Add to initializing sessions
        INITS.insert(id, session.clone());

        // Set telnet session
        telnet.set_session(Some(session.clone())).await;

        session
    }

    pub async fn name(self: &Arc<Self>) -> ArcStr {
        self.name.read().await.clone()
    }

    pub async fn user_name(self: &Arc<Self>) -> ArcStr {
        let guard = self.user.read().await;
        match &*guard {
            Some(u_arc) => u_arc.read().await.user.clone(),
            None => ArcStr::from(""),
        }
    }

    pub async fn name_user(self: &Arc<Self>) -> ArcStr {
        let name = self.name().await;
        let user_name = self.user_name().await;
        ArcStr::from(format!("{name} ({user_name})"))
    }

    pub async fn name_obj(self: &Arc<Self>) -> Arc<Name> {
        self.name_obj.read().await.clone()
    }

    pub async fn blurb(self: &Arc<Self>) -> ArcStr {
        self.blurb.read().await.clone()
    }

    pub async fn name_blurb(self: &Arc<Self>) -> ArcStr {
        let name = self.name().await;
        let blurb = self.blurb().await;
        &name + &blurb
    }

    pub async fn signed_on(self: &Arc<Self>) -> bool {
        self.signed_on.load(Ordering::Relaxed)
    }

    pub async fn priv_level(self: &Arc<Self>) -> i32 {
        self.priv_level.load(Ordering::Relaxed)
    }

    pub async fn signal_public(self: &Arc<Self>) -> bool {
        self.signal_public.load(Ordering::Relaxed)
    }

    pub async fn signal_private(self: &Arc<Self>) -> bool {
        self.signal_private.load(Ordering::Relaxed)
    }

    pub async fn last_explicit(self: &Arc<Self>) -> String {
        self.last_explicit.read().await.clone()
    }

    pub async fn reply_sendlist(self: &Arc<Self>) -> String {
        self.reply_sendlist.read().await.clone()
    }

    pub async fn set_reply_sendlist(self: &Arc<Self>, sendlist: &str) {
        let mut reply = self.reply_sendlist.write().await;
        *reply = sendlist.to_string();

        // Quote if necessary
        if sendlist
            .chars()
            .any(|c| c == ' ' || c == ',' || c == ':' || c == ';' || c == '_')
        {
            *reply = format!("\"{sendlist}\"");
        }
    }

    pub async fn close(self: &Arc<Self>, drain: bool) {
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

    pub async fn transfer(self: &Arc<Self>, new_telnet: Arc<Telnet>) {
        let old_telnet = self.telnet.read().await.clone();
        *self.telnet.write().await = Some(new_telnet.clone());
        new_telnet.set_session(Some(self.clone())).await;

        if let Some(old) = old_telnet {
            let who = self.name_user().await;
            info!("Transfer: {who} from fd to new connection");
            old.output("*** This session has been transferred to a new connection. ***\n")
                .await;
            old.close(true).await;
        }

        self.enqueue_others(Arc::new(TransferNotify::new(self.name_obj().await)))
            .await;
        self.pending.attach(new_telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn attach(self: &Arc<Self>, telnet: Arc<Telnet>) {
        *self.telnet.write().await = Some(telnet.clone());
        telnet.set_session(Some(self.clone())).await;

        let who = self.name_user().await;
        info!("Attach: {who} on new connection");

        self.enqueue_others(Arc::new(AttachNotify::new(self.name_obj().await)))
            .await;
        self.pending.attach(telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn detach(self: &Arc<Self>, telnet: &Arc<Telnet>, intentional: bool) {
        if self.signed_on().await && self.priv_level().await > 0 {
            let current_telnet = self.telnet.read().await;
            if let Some(t) = &*current_telnet {
                if Arc::ptr_eq(t, telnet) {
                    let who = self.name_user().await;
                    if intentional {
                        info!("Detach: {who} (intentional)");
                    } else {
                        info!("Detach: {who} (accidental)");
                    };

                    self.enqueue_others(Arc::new(DetachNotify::new(
                        self.name_obj().await,
                        intentional,
                    )))
                    .await;
                    *self.telnet.write().await = None;
                }
            }
        } else {
            self.close(true).await;
        }
    }

    pub async fn output(self: &Arc<Self>, text: &str) {
        self.output_buffer.lock().await.push_str(text);
    }

    pub async fn print(self: &Arc<Self>, format: &str) {
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

    pub async fn enqueue(self: &Arc<Self>, out: Arc<dyn OutputObj>) {
        self.enqueue_output().await;
        if let Some(telnet) = &*self.telnet.read().await {
            self.pending.enqueue(Some(telnet), out).await;
        } else {
            self.pending.enqueue(None, out).await;
        }
    }

    pub async fn enqueue_output(self: &Arc<Self>) {
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

    pub async fn enqueue_others(self: &Arc<Self>, out: Arc<dyn OutputObj>) {
        for session in SESSIONS.iter() {
            if session.id != self.id {
                session.enqueue(out.clone()).await;
            }
        }
    }

    pub async fn acknowledge_output(self: &Arc<Self>) {
        self.pending.acknowledge().await;
    }

    pub async fn output_next(self: &Arc<Self>, telnet: &Telnet) -> bool {
        self.pending.send_next(telnet).await
    }

    pub async fn find_sendable(
        &self,
        sendlist: &str,
        member: bool,
        exact: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) -> (
        Option<Arc<Session>>,
        OrderedSet<Arc<Session>>,
        Option<Arc<Discussion>>,
        OrderedSet<Arc<Discussion>>,
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

    pub async fn find_session(
        &self,
        sendlist: &str,
    ) -> (Option<Arc<Session>>, OrderedSet<Arc<Session>>) {
        let (session, matches, _, _) = self
            .find_sendable(sendlist, false, false, true, false)
            .await;
        (session, matches)
    }

    pub async fn find_discussion(
        &self,
        sendlist: &str,
        member: bool,
    ) -> (Option<Arc<Discussion>>, OrderedSet<Arc<Discussion>>) {
        let (_, _, discussion, matches) = self
            .find_sendable(sendlist, member, false, false, true)
            .await;
        (discussion, matches)
    }

    pub async fn notify_entry(self: &Arc<Self>) {
        let who = self.name_user().await;
        if let Some(_telnet) = &*self.telnet.read().await {
            info!("Enter: {who} on connection");
        } else {
            info!("Enter: {who}, detached");
        }

        let now = Timestamp::new();
        *self.idle_since.write().await = now;
        *self.login_time.write().await = now;

        self.enqueue_others(Arc::new(EntryNotify::new(self.name_obj().await)))
            .await;
    }

    pub async fn notify_exit(self: &Arc<Self>) {
        let who = self.name_user().await;
        if let Some(_telnet) = &*self.telnet.read().await {
            info!("Exit: {who} on connection");
        } else {
            info!("Exit: {who}, detached");
        }

        self.enqueue_others(Arc::new(ExitNotify::new(self.name_obj().await)))
            .await;
    }

    pub async fn init_login_sequence(self: &Arc<Self>) {
        self.start_login_timeout().await;
        self.set_login_state(LoginState::AwaitingLogin, Some("login: "))
            .await;
    }

    pub async fn start_login_timeout(self: &Arc<Self>) {
        let weak_self = Arc::downgrade(&self.clone());

        let handle = tokio::spawn(async move {
            tokio::time::sleep(LOGIN_TIMEOUT).await;

            if let Some(session) = weak_self.upgrade() {
                session.output("\nLogin timeout.\n").await;
                session.enqueue_output().await;
                session.close(true).await;
            }
        });

        *self.login_timeout.write().await = Some(handle.abort_handle());
    }

    pub async fn cancel_login_timeout(self: &Arc<Self>) {
        if let Some(handle) = self.login_timeout.write().await.take() {
            handle.abort();
        }
    }

    pub async fn set_login_state(self: &Arc<Self>, state: LoginState, prompt: Option<&str>) {
        // Cancel the login timeout if login is complete.
        if state == LoginState::LoggedIn {
            self.cancel_login_timeout().await;
        }

        *self.login_state.write().await = state;

        if let Some(p) = prompt {
            if let Some(telnet) = &*self.telnet.read().await {
                telnet.prompt(p).await;
            }
        }

        // Process any pending lines
        self.process_pending_lines().await;
    }

    pub async fn process_pending_lines(self: &Arc<Self>) {
        loop {
            let line = {
                let mut lines = self.lines.lock().await;
                if lines.is_empty() {
                    break;
                }
                lines.remove(0)
            };

            self.handle_input(&line).await;
        }
    }

    pub async fn handle_input(self: &Arc<Self>, line: &str) {
        self.pending.dequeue().await;

        match *self.login_state.read().await {
            LoginState::PreLogin => self.save_input_line(line).await,
            LoginState::AwaitingLogin => self.handle_login_input(line).await,
            LoginState::AwaitingPassword => self.handle_password_input(line).await,
            LoginState::AwaitingName => self.handle_name_input(line).await,
            LoginState::AwaitingBlurb => self.handle_blurb_input(line).await,
            LoginState::AwaitingTransferConfirmation => self.handle_transfer_input(line).await,
            LoginState::LoggedIn => self.process_input(line).await,
        }

        self.enqueue_output().await;
    }

    pub async fn save_input_line(self: &Arc<Self>, line: &str) {
        self.lines.lock().await.push(line.to_string());
    }

    pub async fn handle_login_input(self: &Arc<Self>, line: &str) {
        let line = line.trim();

        if let Some(args) = match_keyword(line, "/bye", 4) {
            self.do_bye(args).await;
            return;
        }

        if line.is_empty() {
            if let Some(telnet) = &*self.telnet.read().await {
                telnet.prompt("login: ").await;
            }
            return;
        }

        let user = USER_MANAGER.get_user(line).await;
        *self.user.write().await = user.clone();

        if user.is_none() || user.as_ref().unwrap().read().await.password.is_some() {
            // Need password
            if let Some(telnet) = &*self.telnet.read().await {
                let echo = telnet.get_echo().await;
                if !echo {
                    telnet
                        .output("\n\x07Sorry, password probably WILL echo.\n\n")
                        .await;
                } else if echo != TELNET_ENABLED {
                    telnet.output("\nWarning: password may echo.\n\n").await;
                }

                telnet.set_do_echo(false).await;
                self.set_login_state(LoginState::AwaitingPassword, Some("Password: "))
                    .await;
            }
        } else {
            // No password required (guest account)
            self.print_reserved_names().await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
                .await;
        }
    }

    pub async fn handle_password_input(self: &Arc<Self>, line: &str) {
        if let Some(telnet) = &*self.telnet.read().await {
            telnet.output("\n").await;
            telnet.set_do_echo(true).await;
        }

        USER_MANAGER.update_all().await.ok();

        let valid = if let Some(user_lock) = &*self.user.read().await {
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
            self.output("Login incorrect.\n").await;
            let attempts = self.attempts.fetch_add(1, Ordering::Relaxed) + 1;
            if attempts >= Session::MAX_LOGIN_ATTEMPTS {
                self.close(true).await;
                return;
            }

            self.set_login_state(LoginState::AwaitingLogin, Some("login: "))
                .await;
            *self.user.write().await = None;
            return;
        }

        self.print_reserved_names().await;
        self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
            .await;
    }

    pub async fn print_reserved_names(self: &Arc<Self>) {
        if let Some(user_lock) = &*self.user.read().await {
            let user = user_lock.read().await;

            if let Some(first) = user.reserved.first() {
                self.print(&format!("\nYour default (reserved) name is \"{first}\".\n"))
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

    pub async fn handle_name_input(self: &Arc<Self>, line: &str) {
        let line = line.trim();
        let name = if line.is_empty() {
            if let Some(user_lock) = &*self.user.read().await {
                let user = user_lock.read().await;
                if let Some(reserved) = user.reserved.first() {
                    reserved.clone()
                } else {
                    if let Some(telnet) = &*self.telnet.read().await {
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

        *self.name.write().await = name.clone();

        if self.check_name_availability(&name, false, false).await {
            self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: "))
                .await;
        }
    }

    pub async fn handle_blurb_input(self: &Arc<Self>, line: &str) {
        if !self
            .check_name_availability(&self.name().await, true, false)
            .await
        {
            return;
        }

        let line = if line.is_empty() {
            if let Some(user_lock) = &*self.user.read().await {
                let user = user_lock.read().await;
                user.blurb.as_ref().map(|b| b.as_str()).unwrap_or("")
            } else {
                ""
            }
        } else {
            line
        };

        self.do_blurb(line, true).await;

        self.set_login_state(LoginState::LoggedIn, None);

        self.signed_on.store(true, Ordering::Relaxed);

        if let Some(user_lock) = &*self.user.read().await {
            let mut user = user_lock.write().await;
            self.priv_level.store(user.priv_level, Ordering::Relaxed);
            user.add_session(self.clone());
        }

        SESSIONS.insert(self.id, self.clone());
        INITS.remove(&self.id);

        self.notify_entry().await;

        // Welcome message and automatic commands
        self.output("\n\nWelcome to Phoenix.  Type \"/help\" for a list of commands.\n\n")
            .await;

        // Make sure discussion A exists
        let (_, _, discussion, _) = self.find_sendable("A", false, true, true, true).await;
        if discussion.is_none() {
            let disc = Discussion::new(None, "A", "General Discussion", true);
            DISCUSSIONS.insert("A".to_string(), disc);
        }

        // Automatic commands
        self.do_join("A").await;
        self.do_send("A").await;
        self.do_who("").await;
        self.do_howmany("").await;

        if let Some(telnet) = &*self.telnet.read().await {
            telnet.reset_history().await;
        }

        self.set_login_state(LoginState::LoggedIn, None).await;
    }

    pub async fn handle_transfer_input(self: &Arc<Self>, line: &str) {
        if match_keyword(line, "yes", 1).is_none() {
            self.output("Session not transferred.\n").await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
                .await;
            return;
        }

        if self
            .check_name_availability(&self.name().await, true, true)
            .await
        {
            self.output("(That session is now gone.)\n").await;
            self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: "))
                .await;
        }
    }

    pub async fn process_input(self: &Arc<Self>, line: &str) {
        let line = line.trim_end();

        if line.starts_with('!') {
            let line = line[1..].trim();
            if self.priv_level().await < 50 {
                self.output("Sorry, all !commands are privileged.\n").await;
                return;
            }

            if let Some(args) = match_keyword(line, "!restart", 8) {
                self.do_restart(args).await;
            } else if let Some(args) = match_keyword(line, "!down", 5) {
                self.do_down(args).await;
            } else if let Some(args) = match_keyword(line, "!nuke", 5) {
                self.do_nuke(args).await;
            } else {
                self.output("Unknown !command.\n").await;
            }
        } else if line.starts_with('/') {
            let line = line[1..].trim();
            if let Some(args) = match_keyword(line, "/who", 2) {
                self.do_who(args).await;
            } else if let Some(args) = match_keyword(line, "/idle", 2) {
                self.do_idle(args).await;
            } else if let Some(args) = match_keyword(line, "/blurb", 3) {
                self.do_blurb(args, false).await;
            } else if let Some(args) = match_keyword(line, "/here", 2) {
                self.do_here(args).await;
            } else if let Some(args) = match_keyword(line, "/away", 2) {
                self.do_away(args).await;
            } else if let Some(args) = match_keyword(line, "/busy", 2) {
                self.do_busy(args).await;
            } else if let Some(args) = match_keyword(line, "/gone", 2) {
                self.do_gone(args).await;
            } else if let Some(args) = match_keyword(line, "/help", 2) {
                self.do_help(args).await;
            } else if let Some(args) = match_keyword(line, "/send", 2) {
                self.do_send(args).await;
            } else if let Some(args) = match_keyword(line, "/bye", 4) {
                self.do_bye(args).await;
            } else if let Some(args) = match_keyword(line, "/what", 3) {
                self.do_what(args).await;
            } else if let Some(args) = match_keyword(line, "/join", 2) {
                self.do_join(args).await;
            } else if let Some(args) = match_keyword(line, "/quit", 2) {
                self.do_quit(args).await;
            } else if let Some(args) = match_keyword(line, "/create", 3) {
                self.do_create(args).await;
            } else if let Some(args) = match_keyword(line, "/destroy", 4) {
                self.do_destroy(args).await;
            } else if let Some(args) = match_keyword(line, "/permit", 4) {
                self.do_permit(args).await;
            } else if let Some(args) = match_keyword(line, "/depermit", 4) {
                self.do_depermit(args).await;
            } else if let Some(args) = match_keyword(line, "/appoint", 4) {
                self.do_appoint(args).await;
            } else if let Some(args) = match_keyword(line, "/unappoint", 10) {
                self.do_unappoint(args).await;
            } else if let Some(args) = match_keyword(line, "/rename", 7) {
                self.do_rename(args).await;
            } else if let Some(args) = match_keyword(line, "/clear", 3) {
                self.do_clear(args).await;
            } else if let Some(args) = match_keyword(line, "/unidle", 7) {
                self.do_unidle(args).await;
            } else if let Some(args) = match_keyword(line, "/detach", 4) {
                self.do_detach(args).await;
            } else if let Some(args) = match_keyword(line, "/howmany", 3) {
                self.do_howmany(args).await;
            } else if let Some(args) = match_keyword(line, "/why", 4) {
                self.do_why(args).await;
            } else if let Some(args) = match_keyword(line, "/date", 3) {
                self.do_date(args).await;
            } else if let Some(args) = match_keyword(line, "/signal", 3) {
                self.do_signal(args).await;
            } else if let Some(args) = match_keyword(line, "/set", 4) {
                self.do_set(args).await;
            } else if let Some(args) = match_keyword(line, "/display", 2) {
                self.do_display(args).await;
            } else if let Some(args) = match_keyword(line, "/also", 3) {
                self.do_also(args).await;
            } else if let Some(args) = match_keyword(line, "/oops", 3) {
                self.do_oops(args).await;
            } else {
                self.output("Unknown /command.  Type /help for help.\n")
                    .await;
            }
        } else if line == " " {
            self.do_reset().await;
        } else if !line.is_empty() {
            self.do_message(line).await;
        }
    }

    pub async fn check_name_availability(
        &self,
        name: &str,
        double_check: bool,
        transferring: bool,
    ) -> bool {
        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  Choose another name.\n")
                .await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
                .await;
            return false;
        }

        if let Some((reserved, found_user)) = USER_MANAGER.find_reserved(name).await {
            let is_same_user = if let Some(my_user) = &*self.user.read().await {
                Arc::ptr_eq(my_user, &found_user)
            } else {
                false
            };

            if !is_same_user {
                let now = if double_check { " now" } else { "" };
                self.print(&format!(
                    "\"{reserved}\" is{now} a reserved name.  Choose another.\n"
                ))
                .await;
                self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
                    .await;
                return false;
            }
        }

        let (session, _, discussion, _) = self.find_sendable(name, false, true, true, true).await;
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
                        let now = if double_check { " now" } else { "" };
                        self.print(&format!(
                            "You are{now} attached elsewhere under that name.\n"
                        ))
                        .await;
                        self.set_login_state(
                            LoginState::AwaitingTransferConfirmation,
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
                let found_name = found_session.name().await;
                let already = if double_check { "now" } else { "already" };
                self.print(&format!(
                    "The name \"{found_name}\" is {already} in use.  Choose another.\n"
                ))
                .await;
                self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
                    .await;
                return false;
            }
        }

        if let Some(found_discussion) = discussion {
            let found_name = &found_discussion.name;
            let already = if double_check { "now" } else { "already" };
            self.print(&format!(
                "There is {already} a discussion named \"{found_name}\".  Choose another name.\n"
            ))
            .await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: "))
                .await;
            return false;
        }

        true
    }

    // Command implementations
    pub async fn reset_idle(self: &Arc<Self>, min: usize) -> i64 {
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

    pub async fn set_blurb(self: &Arc<Self>, new_blurb: Option<&str>) {
        self.reset_idle(10).await;

        let blurb = if let Some(text) = new_blurb {
            format!(" [{text}]")
        } else {
            String::new()
        };

        *self.blurb.write().await = ArcStr::new(&blurb);
        *self.name_obj.write().await = Name::new(self.name().await, &blurb);
    }

    pub async fn print_time_long(self: &Arc<Self>, minutes: i32) {
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
                        let s = if days == 1 { "" } else { "s" };
                        let and = if hours > 0 && minutes > 0 {
                            ","
                        } else if hours > 0 || minutes > 0 {
                            " and"
                        } else {
                            ""
                        };
                        self.print(&format!(" {days} day{s}{and}")).await;
                    }
                    if hours > 0 {
                        let s = if hours == 1 { "" } else { "s" };
                        let and = if minutes > 0 { " and" } else { "" };
                        self.print(&format!(" {hours} hour{s}{and}")).await;
                    }
                    if minutes > 0 {
                        let s = if minutes == 1 { "" } else { "s" };
                        self.print(&format!(" {minutes} minute{s}")).await;
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

    pub async fn print_time_long_verbose(self: &Arc<Self>, days: i32, hours: i32, minutes: i32) {
        if days > 0 || hours > 0 || minutes > 0 {
            if minutes == 0 {
                self.output(" exactly").await;
            }
            if days > 0 {
                let s = if days == 1 { "" } else { "s" };
                let and = if hours > 0 && minutes > 0 {
                    ","
                } else if hours > 0 || minutes > 0 {
                    " and"
                } else {
                    ""
                };
                self.print(&format!(" {days} day{s}{and}")).await;
            }
            if hours > 0 {
                let s = if hours == 1 { "" } else { "s" };
                let and = if minutes > 0 { " and" } else { "" };
                self.print(&format!(" {hours} hour{s}{and}")).await;
            }
            if minutes > 0 {
                let s = if minutes == 1 { "" } else { "s" };
                self.print(&format!(" {minutes} minute{s}")).await;
            }
        } else {
            self.output(" under a minute").await;
        }
    }

    pub async fn print_time_long_terse(self: &Arc<Self>, days: i32, hours: i32, minutes: i32) {
        if days > 0 {
            self.print(&format!("{days}d{hours:02}:{minutes:02}")).await;
        } else {
            self.print(&format!("{hours}:{minutes:02}")).await;
        }
    }

    pub async fn do_restart(self: &Arc<Self>, args: &str) {
        let who = self.name_user().await;
        let name = self.name_blurb().await;

        if args == "!" {
            // Immediate restart
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await;
            self.server.schedule_restart(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel restart
            match self.server.cancel_shutdown().await {
                Some(true) => {
                    info!("Restart cancelled by {who}.");
                    Self::announce(&format!(
                        "*** {name} has cancelled the server restart. ***\n"
                    ))
                    .await;
                }
                Some(false) => {
                    info!("Shutdown cancelled by {who}.");
                    Self::announce(&format!(
                        "*** {name} has cancelled the server shutdown. ***\n"
                    ))
                    .await;
                }
                None => {
                    self.output("The server was not about to shut down or restart.\n")
                        .await
                }
            }
        } else {
            // Delayed restart
            let seconds = args.parse::<u64>().unwrap_or(30);
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await;
            self.server.schedule_restart(who.clone(), seconds).await;
        }
    }

    pub async fn do_down(self: &Arc<Self>, args: &str) {
        let who = self.name_user().await;
        let name = self.name_blurb().await;

        if args == "!" {
            // Immediate shutdown
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await;
            self.server.schedule_shutdown(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel shutdown
            match self.server.cancel_shutdown().await {
                Some(true) => {
                    info!("Restart cancelled by {who}.");
                    Self::announce(&format!(
                        "*** {name} has cancelled the server restart. ***\n"
                    ))
                    .await;
                }
                Some(false) => {
                    info!("Shutdown cancelled by {who}.");
                    Self::announce(&format!(
                        "*** {name} has cancelled the server shutdown. ***\n"
                    ))
                    .await;
                }
                None => {
                    self.output("The server was not about to shut down or restart.\n")
                        .await
                }
            }
        } else {
            // Delayed shutdown
            let seconds = args.parse::<u64>().unwrap_or(30);
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await;
            self.server.schedule_shutdown(who.clone(), seconds).await;
        }
    }

    pub async fn do_nuke(self: &Arc<Self>, args: &str) {
        let drain = !args.starts_with('!');
        let args = if drain { args } else { &args[1..] };

        let (session, matches) = self.find_session(args).await;

        if let Some(target) = session {
            let who = target.name_user().await;
            let name = target.name().await;
            let by_who = self.name_user().await;
            let by_name = self.name_blurb().await;

            if drain {
                self.print(&format!("\"{name}\" has been nuked.\n")).await;
            } else {
                self.print(&format!("\"{name}\" has been nuked immediately.\n"))
                    .await;
            }

            if let Some(telnet) = &*target.telnet.read().await {
                *target.telnet.write().await = None;
                info!("{who} has been nuked by {by_who}");
                telnet.undraw_input().await;
                telnet
                    .print(&format!(
                        "\x07\x07\x07*** You have been nuked by {by_name}. ***\n"
                    ))
                    .await;
                telnet.redraw_input().await;
                telnet.close(drain).await;
            } else {
                info!("{who}, detached, has been nuked by {by_who}");
                target.close(true).await;
            }
        } else {
            self.output("\x07\x07").await;
            self.session_matches(args, &matches).await;
        }
    }

    pub async fn do_bye(self: &Arc<Self>, _args: &str) {
        self.close(true).await;
    }

    pub async fn do_who(self: &Arc<Self>, args: &str) {
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

            let name = session.name_blurb().await;
            if name.len() > 33 {
                self.print(&format!("{name:<32.32}+ ")).await;
            } else {
                self.print(&format!("{name:<33} ")).await;
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
                    self.print(&format!(" {days:>2}d{hours:02}:{minutes:02}  "))
                        .await;
                } else if hours > 0 {
                    self.print(&format!("    {hours:>2}:{minutes:02}  ")).await;
                } else {
                    self.print(&format!("       {minutes:>2}  ")).await;
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

    pub async fn do_idle(self: &Arc<Self>, args: &str) {
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

            let name = session.name_blurb().await;
            let plus = if name.len() > 32 { "+" } else { " " };
            self.print(&format!("{name:<32.32}{plus} ")).await;

            let idle = (now - *session.idle_since.read().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 9 {
                    self.print(&format!("{days:2}d{hours:02}")).await;
                } else if days > 0 {
                    self.print(&format!("{days}d{hours:02}h")).await;
                } else if hours > 0 {
                    self.print(&format!("{hours:2}:{minutes:02}")).await;
                } else {
                    self.print(&format!("   {minutes:2}")).await;
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

    pub async fn do_why(self: &Arc<Self>, args: &str) {
        if self.priv_level().await < 50 {
            self.output("Why not?\n").await;
            return;
        }

        // TODO: Implement privileged /why command
        self.do_who(args).await;
    }

    pub async fn do_blurb(self: &Arc<Self>, args: &str, entry: bool) {
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
                    let blurb = self.blurb().await;
                    self.print(&format!("Your blurb has been set to{blurb}.\n"))
                        .await;
                }
            }
        } else if entry {
            self.set_blurb(None).await;
        } else if !self.blurb().await.is_empty() {
            let blurb = self.blurb().await;
            self.print(&format!("Your blurb is currently set to{blurb}.\n"))
                .await;
        } else {
            self.output("You do not currently have a blurb set.\n")
                .await;
        }
    }

    pub async fn do_here(self: &Arc<Self>, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"here\".\n").await;
        *self.away.write().await = AwayState::Here;
        self.enqueue_others(Arc::new(HereNotify::new(self.name_obj().await)))
            .await;
    }

    pub async fn do_away(self: &Arc<Self>, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"away\".\n").await;
        *self.away.write().await = AwayState::Away;
        self.enqueue_others(Arc::new(AwayNotify::new(self.name_obj().await)))
            .await;
    }

    pub async fn do_busy(self: &Arc<Self>, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"busy\".\n").await;
        *self.away.write().await = AwayState::Busy;
        self.enqueue_others(Arc::new(BusyNotify::new(self.name_obj().await)))
            .await;
    }

    pub async fn do_gone(self: &Arc<Self>, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"gone\".\n").await;
        *self.away.write().await = AwayState::Gone;
        self.enqueue_others(Arc::new(GoneNotify::new(self.name_obj().await)))
            .await;
    }

    pub async fn do_clear(self: &Arc<Self>, _args: &str) {
        self.output("\x1b[H\x1b[J").await;
    }

    pub async fn do_unidle(self: &Arc<Self>, _args: &str) {
        let idle = self.reset_idle(1).await;
        if idle == 0 {
            self.output("Your idle time has been reset.\n").await;
        }
    }

    pub async fn do_detach(self: &Arc<Self>, _args: &str) {
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

    pub async fn do_howmany(self: &Arc<Self>, _args: &str) {
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
        let here_pct = (here * 100 + total / 2) / total.max(1);
        let away_pct = (away * 100 + total / 2) / total.max(1);
        let busy_pct = (busy * 100 + total / 2) / total.max(1);
        let gone_pct = (gone * 100 + total / 2) / total.max(1);
        let attached_pct = (attached * 100 + total / 2) / total.max(1);
        let detached_pct = (detached * 100 + total / 2) / total.max(1);
        self.print(&format!(" {here:3} {here_pct:3}%   {away:3} {away_pct:3}%   {busy:3} {busy_pct:3}%   {gone:3} {gone_pct:3}%   {attached:3} {attached_pct:3}%   {detached:3} {detached_pct:3}%   {total:3} 100%\n")).await;

        let disc_count = DISCUSSIONS.len();
        self.print(&format!("\nDiscussions in use: {disc_count}\n\n"))
            .await;
    }

    pub async fn do_what(self: &Arc<Self>, args: &str) {
        if DISCUSSIONS.is_empty() {
            self.output("No discussions currently exist.\n").await;
            return;
        }

        let sendlist = Sendlist::new(&self.clone(), args, true, false, true).await;

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
            let disc_name = &disc.name;
            self.output(" ").await;
            let name = if disc.name.len() > 15 {
                format!("{disc_name:<14.14}+")
            } else {
                format!("{disc_name:<15}")
            };
            self.output(&name).await;

            let members = disc.members.read().await;
            let member_count = members.len();
            let is_member = if members.contains(&self.clone()) {
                '*'
            } else {
                ' '
            };
            self.print(&format!("{member_count:>3}{is_member} ")).await;

            let idle = (now - *disc.idle_since.read().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.print(&format!(" {days}d{hours:02}:{minutes:02}  "))
                        .await;
                } else if hours > 0 {
                    self.print(&format!("    {hours}:{minutes:02}  ")).await;
                } else {
                    self.print(&format!("      {minutes:>2}  ")).await;
                }
            } else {
                self.output("         ").await;
            }

            if disc.permitted(&self.clone()).await {
                let title = &disc.title;
                if disc.title.len() > 49 {
                    self.print(&format!("{title:<48.48}+\n")).await;
                } else {
                    self.print(&format!("{title}\n")).await;
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

    pub async fn do_date(self: &Arc<Self>, _args: &str) {
        let t = Timestamp::new();
        let date = t.date(0, 0);
        self.print(&format!("{date}\n")).await;
    }

    pub async fn do_signal(self: &Arc<Self>, args: &str) {
        let mut args = args;

        if let Some(_rest) = match_keyword(args, "on", 2) {
            self.signal_public.store(true, Ordering::Relaxed);
            self.signal_private.store(true, Ordering::Relaxed);
            self.output("All signals are now on.\n").await;
        } else if let Some(_rest) = match_keyword(args, "off", 2) {
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
                let on = if self.signal_public.load(Ordering::Relaxed) {
                    "on"
                } else {
                    "off"
                };
                self.print(&format!("Signals are {on} for public messages.\n"))
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
                let on = if self.signal_private.load(Ordering::Relaxed) {
                    "on"
                } else {
                    "off"
                };
                self.print(&format!("Signals are {on} for private messages.\n"))
                    .await;
            } else {
                self.output("Usage: /signal private [on|off]\n").await;
            }
        } else if args.is_empty() {
            let pub_sig = self.signal_public.load(Ordering::Relaxed);
            let priv_sig = self.signal_private.load(Ordering::Relaxed);

            if pub_sig == priv_sig {
                let on = if pub_sig { "on" } else { "off" };
                self.print(&format!(
                    "Signals are {on} for both public and private messages.\n"
                ))
                .await;
            } else {
                let pub_on = if pub_sig { "on" } else { "off" };
                let priv_on = if priv_sig { "on" } else { "off" };
                self.print(&format!("Signals are {pub_on} for public messages and {priv_on} for private messages.\n")).await;
            }
        } else {
            self.output("Usage: /signal [public|private] [on|off]\n")
                .await;
        }
    }

    pub async fn do_send(self: &Arc<Self>, args: &str) {
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

        let sendlist = Sendlist::new(&self.clone(), &slist, false, true, true).await;

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

    pub async fn print_sendlist(self: &Arc<Self>, sendlist: &Sendlist) {
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
                let s = if sendlist.discussions.len() == 1 {
                    ""
                } else {
                    "s"
                };
                self.print(&format!(" and discussion{s} ")).await;

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

    pub async fn do_join(self: &Arc<Self>, args: &str) {
        if args.is_empty() {
            self.output("Usage: /join <disc>[,<disc>...]\n").await;
            return;
        }

        let (name, _) = getword(args, Some(','));
        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.join(self.clone()).await;
        } else {
            self.discussion_matches(name, &matches).await;
        }
    }

    pub async fn do_quit(self: &Arc<Self>, args: &str) {
        if args.is_empty() {
            self.output("Usage: /quit <disc>[,<disc>...]\n").await;
            return;
        }

        let (name, _) = getword(args, Some(','));
        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.quit(self.clone()).await;
        } else {
            let (discussion, matches) = self.find_discussion(name, true).await;

            if let Some(disc) = discussion {
                disc.quit(self.clone()).await;
            } else {
                self.discussion_matches(name, &matches).await;
            }
        }
    }

    pub async fn do_create(self: &Arc<Self>, args: &str) {
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

        if let Some((reserved, found_user)) = USER_MANAGER.find_reserved(name).await {
            let is_same_user = if let Some(my_user) = &*self.user.read().await {
                my_user.user == found_user.user
            } else {
                false
            };

            let a = if is_same_user { "your" } else { "a" };
            self.print(&format!(
                "\"{reserved}\" is {a} reserved name. (not created)\n"
            ))
            .await;
            return;
        }

        let (session, _, discussion, _) = self.find_sendable(name, false, true, true, true).await;

        if let Some(s) = session {
            let name = s.name().await;
            self.print(&format!(
                "There is already someone named \"{name}\". (not created)\n"
            ))
            .await;
            return;
        }

        if let Some(d) = discussion {
            let name = &d.name;
            self.print(&format!(
                "There is already a discussion named \"{name}\". (not created)\n"
            ))
            .await;
            return;
        }

        let disc = Arc::new(Discussion::new(Some(self.clone()), name, title, is_public));
        DISCUSSIONS.insert(name.to_string(), disc.clone());

        self.enqueue_others(Arc::new(CreateNotify::new(
            disc.name.clone(),
            disc.title.clone(),
            disc.is_public,
            self.name_obj().await,
        )))
        .await;

        let name = &disc.name;
        let title = &disc.title;
        self.print(&format!(
            "You have created discussion {name}, \"{title}\".\n"
        ))
        .await;
    }

    pub async fn do_destroy(self: &Arc<Self>, args: &str) {
        if args.is_empty() {
            self.output("Usage: /destroy <disc>[,<disc>...]\n").await;
            return;
        }

        let (name, _) = getword(args, Some(','));
        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.destroy(self.clone()).await;
        } else {
            let (discussion, matches) = self.find_discussion(name, true).await;

            if let Some(disc) = discussion {
                disc.destroy(self.clone()).await;
            } else {
                self.discussion_matches(name, &matches).await;
            }
        }
    }

    pub async fn do_permit(self: &Arc<Self>, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /permit <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.permit(self.clone(), rest).await;
        } else {
            let (discussion, matches) = self.find_discussion(name, true).await;

            if let Some(disc) = discussion {
                disc.permit(self.clone(), rest).await;
            } else {
                self.discussion_matches(name, &matches).await;
            }
        }
    }

    pub async fn do_depermit(self: &Arc<Self>, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /depermit <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.depermit(self.clone(), rest).await;
        } else {
            let (discussion, matches) = self.find_discussion(name, true).await;

            if let Some(disc) = discussion {
                disc.depermit(self.clone(), rest).await;
            } else {
                self.discussion_matches(name, &matches).await;
            }
        }
    }

    pub async fn do_appoint(self: &Arc<Self>, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /appoint <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.appoint(self.clone(), rest).await;
        } else {
            let (discussion, matches) = self.find_discussion(name, true).await;

            if let Some(disc) = discussion {
                disc.appoint(self.clone(), rest).await;
            } else {
                self.discussion_matches(name, &matches).await;
            }
        }
    }

    pub async fn do_unappoint(self: &Arc<Self>, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /unappoint <disc> <person>[,<person>...]\n")
                .await;
            return;
        }

        let (discussion, matches) = self.find_discussion(name, false).await;

        if let Some(disc) = discussion {
            disc.unappoint(self.clone(), rest).await;
        } else {
            let (discussion, matches) = self.find_discussion(name, true).await;

            if let Some(disc) = discussion {
                disc.unappoint(self.clone(), rest).await;
            } else {
                self.discussion_matches(name, &matches).await;
            }
        }
    }

    pub async fn do_rename(self: &Arc<Self>, args: &str) {
        if args.is_empty() {
            self.output("Usage: /rename <name>\n").await;
            return;
        }

        if args.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (name unchanged)\n")
                .await;
            return;
        }

        if let Some((reserved, found_user)) = USER_MANAGER.find_reserved(args).await {
            let is_same_user = if let Some(my_user) = &*self.user.read().await {
                my_user.user == found_user.user
            } else {
                false
            };

            if !is_same_user {
                self.print(&format!(
                    "\"{reserved}\" is a reserved name.  (name unchanged)\n"
                ))
                .await;
                return;
            }
        }

        let (session, _, discussion, _) = self.find_sendable(args, false, true, true, true).await;

        if let Some(s) = session {
            if s.id != self.id {
                self.output("That name is already in use.  (name unchanged)\n")
                    .await;
                return;
            }
        }

        if let Some(d) = discussion {
            let name = &d.name;
            self.print(&format!(
                "There is already a discussion named \"{name}\". (name unchanged)\n"
            ))
            .await;
            return;
        }

        self.enqueue_others(Arc::new(RenameNotify::new(self.name().await, args)))
            .await;

        self.print(&format!("You have changed your name to \"{args}\".\n"))
            .await;
        *self.name.write().await = ArcStr::new(args);
        *self.name_obj.write().await = Name::new(args, self.blurb().await);
    }

    pub async fn do_set(self: &Arc<Self>, args: &str) {
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
                        self.print(&format!("Terminal height is now set to {h}.\n"))
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
                        self.print(&format!("Terminal width is now set to {w}.\n"))
                            .await;
                    }
                } else {
                    self.output("Usage: /set width=<number of columns>\n").await;
                }
            } else {
                self.output("Usage: /set width=<number of columns>\n").await;
            }
        } else {
            self.print(&format!("Unknown system variable: \"{var}\"\n"))
                .await;
        }
    }

    pub async fn set_idle(self: &Arc<Self>, args: &str) {
        // TODO: Implement idle time parsing and setting
        self.output("Idle time setting not yet implemented.\n")
            .await;
    }

    pub async fn do_display(self: &Arc<Self>, args: &str) {
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
                    self.print(&format!("{var} = \"{value}\"\n")).await;
                } else {
                    self.print(&format!("Unknown user variable: \"{var}\"\n"))
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
                    self.print(&format!("Terminal height is currently set to {height}.\n"))
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
                self.print(&format!("Phoenix server version: {VERSION}\n"))
                    .await;
            } else if let Some(_) = match_keyword(var, "width", 5) {
                if let Some(telnet) = &*self.telnet.read().await {
                    let width = telnet.set_width(0).await;
                    self.print(&format!("Terminal width is currently set to {width}.\n"))
                        .await;
                }
            } else {
                self.print(&format!("Unknown system variable: \"{var}\"\n"))
                    .await;
            }
        }
    }

    pub async fn do_also(self: &Arc<Self>, args: &str) {
        if args.is_empty() {
            self.output("Usage: /also <sendlist>\n").await;
            return;
        }

        if let Some(last_msg) = &*self.last_message.read().await {
            let sendlist = Sendlist::new(&self.clone(), args, false, true, true).await;
            self.send_message(&sendlist, &last_msg.text).await;
        } else {
            self.output("You have no previous message to resend.\n")
                .await;
        }
    }

    pub async fn do_oops(self: &Arc<Self>, args: &str) {
        if args.is_empty() {
            self.output("Usage: /oops <sendlist> OR /oops text [<message>]\n")
                .await;
            return;
        }

        if let Some(text_args) = match_keyword(args, "text", 4) {
            let text = text_args.trim();
            if !text.is_empty() {
                *self.oops_text.write().await = text.to_string();
                self.print(&format!("Your /oops text is now \"{text}\".\n"))
                    .await;
            } else {
                let oops_text = self.oops_text.read().await;
                self.print(&format!("Your /oops text is currently \"{oops_text}\".\n"))
                    .await;
            }
        } else {
            if let Some(last_msg) = &*self.last_message.read().await {
                let sendlist = Sendlist::new(&self.clone(), args, false, true, true).await;
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

    pub async fn do_help(self: &Arc<Self>, args: &str) {
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

    pub async fn do_reset(self: &Arc<Self>) {
        self.reset_idle(1).await;
    }

    pub async fn do_message(self: &Arc<Self>, line: &str) {
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
            Arc::new(Sendlist::new(&self.clone(), &sendlist_str, false, true, true).await)
        };

        *self.last_sendlist.write().await = Some(sendlist.clone());

        if msg_start.is_empty() {
            let sendlist_typed = &sendlist.typed;
            if sendlist_str == "default" {
                self.output("\x07\x07There is no message after \"default\". (message not sent)\n")
                    .await;
            } else if is_explicit {
                self.print(&format!(
                    "\x07\x07There is no message after \"{sendlist_typed}:\". (message not sent)\n"
                ))
                .await;
            } else {
                self.print(&format!(
                    "\x07\x07There is no message after \"{sendlist_typed};\". (message not sent)\n"
                ))
                .await;
            }
            return;
        }

        self.send_message(&sendlist, msg_start).await;
    }

    pub async fn send_message(self: &Arc<Self>, sendlist: &Arc<Sendlist>, text: &str) {
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
            self.name_obj().await,
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
            self.output(&format!(" [idle {idle}]")).await;
        }
        self.output("\n").await;
    }

    pub async fn get_who_set(
        self: &Arc<Self>,
        args: &str,
    ) -> (OrderedSet<Arc<Session>>, String, String) {
        let mut who = OrderedSet::new();
        let mut errors = String::new();
        let mut msg = String::new();

        if args.is_empty() {
            // Show all sessions
            for session in SESSIONS.iter() {
                who.insert(session.value().clone());
            }

            let count = who.len();
            let s = if count == 1 { "" } else { "s" };
            msg = format!("\n{count} user{s} signed on.\n");
        } else {
            let sendlist = Sendlist::new(&self.clone(), args, true, true, true).await;

            let mut total = sendlist.expand(&mut who, None).await;

            if !sendlist.errors.is_empty() {
                errors = sendlist.errors.clone();
            }

            if who.is_empty() {
                if errors.is_empty() {
                    errors = "No one matched your request.\n".to_string();
                }
            } else {
                let count = who.len();
                let s = if count == 1 { "" } else { "s" };
                msg = format!("\n{count} user{s} matched.\n",);
            }
        }

        (who, errors, msg)
    }

    pub async fn session_matches(self: &Arc<Self>, name: &str, matches: &OrderedSet<Arc<Session>>) {
        if matches.is_empty() {
            self.print(&format!("No names matched \"{name}\".\n")).await;
        } else if matches.len() == 1 {
            let matched_name = matches.iter().next().unwrap().name().await;
            self.print(&format!("\"{name}\" matches one name: {matched_name}.\n"))
                .await;
        } else {
            let count = matches.len();
            self.print(&format!("\"{name}\" matches {count} names: "))
                .await;

            //let names: Vec<String> = matches.iter().map(|s| s.name().to_string()).collect();
            let names: Vec<ArcStr> = matches.iter().map(|s| s.name()).collect::<Vec<_>>().await;
            self.output(&names.join(", ")).await;
            self.output(".\n").await;
        }
    }

    pub async fn discussion_matches(
        self: &Arc<Self>,
        name: &str,
        matches: &OrderedSet<Arc<Discussion>>,
    ) {
        if matches.is_empty() {
            self.print(&format!("No discussions matched \"{name}\".\n"))
                .await;
        } else if matches.len() == 1 {
            let matched_name = matches.iter().next().unwrap().name;
            self.print(&format!(
                "\"{name}\" matches one discussion: {matched_name}.\n"
            ))
            .await;
        } else {
            let count = matches.len();
            self.print(&format!("\"{name}\" matches {count} discussions: "))
                .await;

            let names: Vec<&str> = matches.iter().map(|d| d.name.as_ref()).collect();
            self.output(&names.join(", ")).await;
            self.output(".\n").await;
        }
    }
}

impl PartialEq for Session {
    fn eq(self: &Arc<Self>, other: &Arc<Self>) -> bool {
        self.id == other.id
    }
}

impl Eq for Session {}

impl std::hash::Hash for Session {
    fn hash<H: std::hash::Hasher>(self: &Arc<Self>, state: &mut H) {
        self.id.hash(state);
    }
}

use crate::constants::*;
use crate::discussion::Discussion;
use crate::name::{CurrentName, Name};
use crate::output::*;
use crate::sendlist::{message_start, Sendlist};
use crate::server::Server;
use crate::telnet::{Telnet, TELNET_ENABLED};
use crate::text::Text;
use crate::timestamp::{system_uptime, Timestamp};
use crate::user::{verify_password, User, UserManager};
use crate::{getword, match_keyword, match_name, OrderedSet, VERSION};
use async_backtrace::framed;
use dashmap::DashMap;
use log::info;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::AbortHandle;

const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

static INITS: LazyLock<DashMap<usize, Session>> = LazyLock::new(DashMap::new);
static SESSIONS: LazyLock<DashMap<usize, Session>> = LazyLock::new(DashMap::new);
static DISCUSSIONS: LazyLock<DashMap<String, Discussion>> = LazyLock::new(DashMap::new);
static SESSION_COUNTER: AtomicUsize = AtomicUsize::new(1);
static DEFAULTS: LazyLock<RwLock<HashMap<Text, Text>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(Text::from("time_format"), Text::from("verbose"));
    RwLock::new(map)
});
static USER_MANAGER: LazyLock<UserManager> = LazyLock::new(UserManager::new);

/// Session handle.
#[derive(Debug, Clone)]
pub struct Session(Arc<SessionInner>);

#[derive(Debug)]
pub struct SessionInner
where
    Self: Send + Sync + 'static,
{
    // Immutable fields
    pub id: usize,
    pub server: Server,

    // User and connection state
    pub user: Option<User>,
    pub telnet: Option<Telnet>,

    // I/O handling
    pub output_buffer: String,
    pub pending: OutputStream,

    // User preferences and variables
    pub user_vars: HashMap<Text, Text>,
    pub sys_vars: HashMap<Text, Text>,
    pub signal_public: bool,
    pub signal_private: bool,

    // Session state
    pub login_time: Timestamp,
    pub idle_since: Timestamp,
    pub away: AtomicAwayState,
    pub priv_level: i32,
    pub name: CurrentName,

    // Message handling
    pub last_message: Option<Arc<Message>>,
    pub default_sendlist: Option<Arc<Sendlist>>,
    pub last_sendlist: Option<Arc<Sendlist>>,
    pub last_explicit: Text,
    pub reply_sendlist: Text,
    pub oops_text: Text,
}

/// Trait for session objects that can be associated with a Telnet connection.
/// Implemented by both LoginSession (pre-login) and Session (post-login).
pub trait ConnectionSession: Send + Sync {
    fn name_opt(&self) -> Option<&Name> {
        None
    }
    async fn acknowledge_output(&self);
    async fn last_explicit(&self) -> Text {
        Text::default()
    }
    async fn reply_sendlist(&self) -> Text {
        Text::default()
    }
    async fn output_next(&self, telnet: &Telnet) -> bool;
    async fn output(&mut self, text: impl AsRef<str>);
    async fn handle_input(&mut self, line: String);

    async fn print_message(&self, _telnet: &mut Telnet) {}
}

/// Pre-login session for managing connections before authentication.
#[derive(Debug)]
pub struct LoginSession {
    pub id: usize,
    pub server: Server,
    pub user: Option<User>,
    pub telnet: Telnet,
    pub login_state: LoginState,
    pub login_timeout: Option<AbortHandle>,
    pub attempts: i32,
    pub lines: VecDeque<String>,
    pub output_buffer: String,
    pub pending: OutputStream,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginState {
    PreLogin,
    AwaitingLogin,
    AwaitingPassword,
    AwaitingName,
    AwaitingBlurb,
    AwaitingTransferConfirmation,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AwayState {
    Here = 0,
    Away = 1,
    Busy = 2,
    Gone = 3,
}

impl Default for AwayState {
    fn default() -> Self {
        AwayState::Here
    }
}

impl From<AwayState> for u8 {
    #[inline]
    fn from(state: AwayState) -> u8 {
        state as u8
    }
}

impl From<u8> for AwayState {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0 => AwayState::Here,
            1 => AwayState::Away,
            2 => AwayState::Busy,
            3 => AwayState::Gone,
            _ => AwayState::default(),
        }
    }
}

pub struct AtomicAwayState(AtomicU8);

impl AtomicAwayState {
    pub fn new(state: AwayState) -> Self {
        Self(AtomicU8::new(state.into()))
    }

    pub fn get(&self) -> AwayState {
        AwayState::from(self.0.load(Ordering::Acquire))
    }

    pub fn set(&self, state: AwayState) {
        self.0.store(state.into(), Ordering::Release)
    }
}

impl Default for AtomicAwayState {
    fn default() -> Self {
        Self::new(AwayState::default())
    }
}

impl LoginSession {
    pub fn new(id: usize, server: Server, telnet: Telnet) -> Self {
        Self {
            id,
            server,
            user: None,
            telnet,
            login_state: LoginState::PreLogin,
            login_timeout: None,
            attempts: 0,
            lines: VecDeque::new(),
            output_buffer: String::new(),
            pending: OutputStream::new(),
        }
    }

    pub async fn handle_login_input(&mut self, line: String) {
        let line = line.trim();
        if let Some(_args) = match_keyword(line, "/bye", 4) {
            self.do_bye().await;
            return;
        }
        if line.is_empty() {
            self.telnet.prompt("login: ").await;
            return;
        }
        let user = (*USER_MANAGER).get_user(&line).await;
        self.user = user.clone();
        if let Some(_user_lock) = &user {
            self.telnet.set_do_echo(false).await;
            self.set_login_state(LoginState::AwaitingPassword, Some("password: "));
        } else {
            self.output("Invalid login.\n").await;
            self.attempts += 1;
            if self.attempts >= Session::MAX_LOGIN_ATTEMPTS {
                self.close(true).await;
                return;
            }
            self.telnet.prompt("login: ").await;
        }
    }

    pub async fn handle_password_input(&mut self, line: String) {
        self.telnet.output("\n").await;
        self.telnet.set_do_echo(true).await;
        (*USER_MANAGER).update_all().await.ok();

        let valid = if let Some(user_lock) = &self.user {
            let user = user_lock.read().await;
            if let Some(password) = &user.password {
                verify_password(&line, password)
            } else {
                false
            }
        } else {
            false
        };

        if !valid {
            self.output("Login incorrect.\n").await;
            self.attempts += 1;
            if self.attempts >= Session::MAX_LOGIN_ATTEMPTS {
                self.close(true).await;
                return;
            }
            self.set_login_state(LoginState::AwaitingLogin, Some("login: "));
            self.user = None;
            return;
        }

        if let Some(user_lock) = &self.user {
            let user = user_lock.read().await;
            if user.reserved.is_empty() {
                self.output("You don't have any reserved names.\n").await;
                self.close(true).await;
                return;
            }

            if user.reserved.len() == 1 {
                let name = user.reserved[0].clone();
                if self.check_name_availability(&name, false, false).await {
                    self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: "));
                }
                return;
            }
        }

        self.print_reserved_names().await;
        self.set_login_state(LoginState::AwaitingName, Some("Enter name: "));
    }

    pub async fn handle_name_input(&mut self, line: String) {
        let line = line.trim();
        let name = if line.is_empty() {
            if let Some(user_lock) = &self.user {
                let user = user_lock.read().await;
                if let Some(reserved) = user.reserved.first() {
                    reserved.clone()
                } else {
                    self.telnet.prompt("Enter name: ").await;
                    return;
                }
            } else {
                return;
            }
        } else {
            Text::new(line)
        };

        if self.check_name_availability(&name, false, false).await {
            self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: "));
        }
    }

    pub async fn handle_blurb_input(&mut self, line: String) {
        let name = if let Some(user_lock) = &self.user {
            let user = user_lock.read().await;
            if let Some(reserved) = user.reserved.first() {
                reserved.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        if !self.check_name_availability(&name, true, false).await {
            return;
        }

        let line = if line.is_empty() {
            if let Some(user_lock) = &self.user {
                let user = user_lock.read().await;
                user.blurb.as_ref().map(|b| b.as_str()).unwrap_or("").to_string()
            } else {
                String::new()
            }
        } else {
            line
        };

        // At this point we would transition to a full Session
        // This is where we'd create the Session object and add to SESSIONS
        // For now, just mark as logged in
        self.set_login_state(LoginState::LoggedIn, None);
    }

    pub async fn handle_transfer_input(&mut self, line: String) {
        let line = line.trim();
        if match_keyword(&line, "yes", 1).is_none() {
            self.output("Session not transferred.\n").await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: "));
            return;
        }

        let name = if let Some(user_lock) = &self.user {
            let user = user_lock.read().await;
            if let Some(reserved) = user.reserved.first() {
                reserved.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        if self.check_name_availability(&name, true, true).await {
            self.output("(That session is now gone.)\n").await;
            self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: "));
        }
    }

    pub async fn print_reserved_names(&self) {
        self.output("\nYour reserved names are:\n\n").await;
        if let Some(user_lock) = &self.user {
            let user = user_lock.read().await;
            for (i, name) in user.reserved.iter().enumerate() {
                self.output(&format!("    {}: {}\n", i + 1, name)).await;
            }
        }
        self.output("\n").await;
    }

    pub async fn check_name_availability(&self, name: &Text, blurb: bool, transfer: bool) -> bool {
        // Placeholder - would need to implement full logic
        true
    }

    pub async fn set_login_state(&mut self, state: LoginState, prompt: Option<&str>) {
        self.login_state = state;
        if let Some(prompt) = prompt {
            self.telnet.prompt(prompt).await;
        }
    }

    pub async fn enqueue_output(&mut self) {
        if !self.output_buffer.is_empty() {
            self.pending.enqueue(self.output_buffer.clone()).await;
            self.output_buffer.clear();
        }
    }

    pub async fn do_bye(&self) {
        self.close(true).await;
    }

    pub async fn close(&self, drain: bool) {
        self.telnet.close(drain).await;
    }

    pub async fn save_input_line(&mut self, line: String) {
        self.lines.push_back(line);
    }
}

impl ConnectionSession for LoginSession {
    async fn acknowledge_output(&self) {
        self.pending.acknowledge().await;
    }

    async fn output_next(&self, telnet: &Telnet) -> bool {
        self.pending.send_next(telnet).await
    }

    async fn output(&mut self, text: impl AsRef<str>) {
        self.output_buffer.push_str(text.as_ref());
    }

    async fn handle_input(&mut self, line: String) {
        self.pending.dequeue().await;

        match self.login_state {
            LoginState::PreLogin => self.save_input_line(line).await,
            LoginState::AwaitingLogin => self.handle_login_input(line).await,
            LoginState::AwaitingPassword => self.handle_password_input(line).await,
            LoginState::AwaitingName => self.handle_name_input(line).await,
            LoginState::AwaitingBlurb => self.handle_blurb_input(line).await,
            LoginState::AwaitingTransferConfirmation => self.handle_transfer_input(line).await,
        }

        self.enqueue_output().await;
    }
}

impl Session {
    pub const MAX_LOGIN_ATTEMPTS: i32 = 3;

    #[framed]
    pub async fn new(server: Server, telnet: Telnet) -> Self {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let now = Timestamp::new();

        let inner = SessionInner {
            id,
            server,
            user: None,
            telnet: Some(telnet.clone()),
            login_timeout: None,
            login_state: LoginState::PreLogin,
            lines: VecDeque::new(),
            output_buffer: String::new(),
            pending: OutputStream::new(),
            user_vars: HashMap::new(),
            sys_vars: HashMap::new(),
            login_time: now,
            idle_since: now,
            away: Default::default(),
            signal_public: true,
            signal_private: true,
            signed_on: false,
            closing: false,
            attempts: 0,
            priv_level: 0,
            name: Name::new("", None),
            last_message: None,
            default_sendlist: None,
            last_sendlist: None,
            last_explicit: Text::default(),
            reply_sendlist: Text::default(),
            oops_text: Text::from("Oops!  Sorry, that last message was intended for someone else..."),
        };

        let session = Session { id, inner: Arc::new(RwLock::new(inner)) };

        // Add to initializing sessions
        INITS.insert(id, session.clone());

        // Set telnet session
        telnet.set_session(Some(session.clone())).await;

        session
    }

    /// Obtain read lock on the `SessionInner` data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, SessionInner> {
        self.inner.read().await
    }

    /// Obtain write lock on the `SessionInner` data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, SessionInner> {
        self.inner.write().await
    }

    /// Get the session ID.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the `Server` object.
    #[framed]
    pub async fn server(&self) -> Server {
        self.read().await.server.clone()
    }

    /// Get the `User` object, if any.
    #[framed]
    pub async fn user(&self) -> Option<User> {
        self.read().await.user.clone()
    }

    /// Set the `User` object, if any.
    #[framed]
    pub async fn set_user(&self, value: Option<User>) {
        self.write().await.user = value;
    }

    /// Get the `Telnet` object, if any.
    #[framed]
    pub async fn telnet(&self) -> Option<Telnet> {
        self.read().await.telnet.clone()
    }

    /// Set the `Telnet` object, if any.
    #[framed]
    pub async fn set_telnet(&self, value: Option<Telnet>) {
        self.write().await.telnet = value;
    }

    /// Return a single-character detached indicator.
    pub async fn detached_indicator(&self) -> &str {
        if self.read().await.telnet.is_some() {
            " "
        } else {
            "~"
        }
    }

    /// Get the login timeout Tokio task `AbortHandle`, if any.
    #[framed]
    pub async fn login_timeout(&self) -> Option<AbortHandle> {
        self.read().await.login_timeout.clone()
    }

    /// Set the login timeout Tokio task `AbortHandle`, if any.
    #[framed]
    pub async fn set_login_timeout(&self, value: Option<AbortHandle>) {
        self.write().await.login_timeout = value;
    }

    /// Get the `LoginState`.
    #[framed]
    pub async fn login_state(&self) -> LoginState {
        self.read().await.login_state.clone()
    }

    /// Set the `LoginState`.
    #[framed]
    pub async fn set_login_state(&self, value: LoginState) {
        self.write().await.login_state = value;
    }

    /// Add a line to the pending input line queue.
    #[framed]
    pub async fn add_pending_line(&self, line: String) {
        self.write().await.lines.push_back(line);
    }

    /// Take the next pending input line (FIFO).
    #[framed]
    pub async fn take_pending_line(&self) -> Option<String> {
        self.write().await.lines.pop_front()
    }

    /// Check if there are pending input lines.
    #[framed]
    pub async fn has_pending_lines(&self) -> bool {
        !self.read().await.lines.is_empty()
    }

    /// Get count of pending input lines.
    #[framed]
    pub async fn pending_line_count(&self) -> usize {
        self.read().await.lines.len()
    }

    /// Clear all pending input lines.
    #[framed]
    pub async fn clear_pending_lines(&self) {
        self.write().await.lines.clear();
    }

    /// Take all pending input lines at once.
    #[framed]
    pub async fn take_all_pending_lines(&self) -> VecDeque<String> {
        let mut inner = self.write().await;
        std::mem::take(&mut inner.lines)
    }

    /// Append text to output buffer.
    pub async fn output(&self, text: impl AsRef<str>) {
        self.write().await.output_buffer.push_str(text.as_ref());
    }

    /// Get the `OutputStream`.
    #[framed]
    pub async fn pending(&self) -> OutputStream {
        self.read().await.pending.clone()
    }

    /// Set the `OutputStream`.
    #[framed]
    pub async fn set_pending(&self, value: OutputStream) {
        self.write().await.pending = value;
    }

    /// Get a user variable.
    #[framed]
    pub async fn get_user_var(&self, key: impl AsRef<str>) -> Option<Text> {
        self.read().await.user_vars.get(key.as_ref()).clone()
    }

    /// Set a user variable.
    #[framed]
    pub async fn set_user_var(&self, key: impl Into<Arc<str>>, value: impl Into<Arc<str>>) {
        let key: Arc<str> = key.into();
        let value: Arc<str> = value.into();
        self.write().await.user_vars.insert(Text(key), Text(value));
    }

    /// Remove a user variable.
    #[framed]
    pub async fn remove_user_var(&self, key: impl AsRef<str>) -> Option<Text> {
        self.write().await.user_vars.remove(key.as_ref())
    }

    /// Clear all user variables.
    #[framed]
    pub async fn clear_user_vars(&self) {
        self.write().await.user_vars.clear();
    }

    /// Get a system variable.
    #[framed]
    pub async fn get_sys_var(&self, key: impl AsRef<str>) -> Option<Text> {
        self.read().await.sys_vars.get(key.as_ref()).clone()
    }

    /// Set a system variable.
    #[framed]
    pub async fn set_sys_var(&self, key: impl Into<Arc<str>>, value: impl Into<Arc<str>>) {
        let key: Arc<str> = key.into();
        let value: Arc<str> = value.into();
        self.write().await.sys_vars.insert(Text(key), Text(value));
    }

    /// Remove a system variable.
    #[framed]
    pub async fn remove_sys_var(&self, key: impl AsRef<str>) -> Option<Text> {
        self.write().await.sys_vars.remove(key)
    }

    /// Clear all system variables.
    #[framed]
    pub async fn clear_sys_vars(&self) {
        self.write().await.sys_vars.clear();
    }

    /// Get the login time.
    #[framed]
    pub async fn login_time(&self) -> Timestamp {
        self.read().await.login_time
    }

    /// Set the login time.
    #[framed]
    pub async fn set_login_time(&self, value: Timestamp) {
        self.write().await.login_time = value;
    }

    /// Get the idle-since timestamp.
    #[framed]
    pub async fn idle_since(&self) -> Timestamp {
        self.read().await.idle_since
    }

    /// Set the idle-since timestamp.
    #[framed]
    pub async fn set_idle_since(&self, value: Timestamp) {
        self.write().await.idle_since = value;
    }

    /// Reset idle time to now.
    #[framed]
    pub async fn reset_idle(&self) {
        self.write().await.idle_since = Timestamp::new();
    }

    /// Get the away state.
    pub fn away(&self) -> AwayState {
        self.0.away.get()
    }

    /// Set the away state.
    pub fn set_away(&self, value: AwayState) {
        self.0.away.set(value);
    }

    /// Get the public signal flag.
    #[framed]
    pub async fn signal_public(&self) -> bool {
        self.read().await.signal_public
    }

    /// Set the public signal flag.
    #[framed]
    pub async fn set_signal_public(&self, value: bool) {
        self.write().await.signal_public = value;
    }

    /// Get the private signal flag.
    #[framed]
    pub async fn signal_private(&self) -> bool {
        self.read().await.signal_private
    }

    /// Set the private signal flag.
    #[framed]
    pub async fn set_signal_private(&self, value: bool) {
        self.write().await.signal_private = value;
    }

    /// Get the signed-on flag.
    #[framed]
    pub async fn signed_on(&self) -> bool {
        self.read().await.signed_on
    }

    /// Set the signed-on flag.
    #[framed]
    pub async fn set_signed_on(&self, value: bool) {
        self.write().await.signed_on = value;
    }

    /// Get the closing flag.
    #[framed]
    pub async fn closing(&self) -> bool {
        self.read().await.closing
    }

    /// Set the closing flag.
    #[framed]
    pub async fn set_closing(&self, value: bool) {
        self.write().await.closing = value;
    }

    /// Get the login attempts count.
    #[framed]
    pub async fn attempts(&self) -> i32 {
        self.read().await.attempts
    }

    /// Set the login attempts count.
    #[framed]
    pub async fn set_attempts(&self, value: i32) {
        self.write().await.attempts = value;
    }

    /// Increment the login attempts count.
    #[framed]
    pub async fn increment_attempts(&self) -> i32 {
        let mut inner = self.write().await;
        inner.attempts += 1;
        inner.attempts
    }

    /// Get the privilege level.
    #[framed]
    pub async fn priv_level(&self) -> i32 {
        self.read().await.priv_level
    }

    /// Set the privilege level.
    #[framed]
    pub async fn set_priv_level(&self, value: i32) {
        self.write().await.priv_level = value;
    }

    /// Get the `Name` object.
    pub fn name(&self) -> Name {
        self.0.name.snapshot()
    }

    /// Get only the name from the `Name` object.
    pub fn name_only(&self) -> &Text {
        self.0.name.name()
    }

    /// Set the name.
    pub fn set_name(&self, value: impl AsRef<str>) {
        let current = self.0.name.borrow();
        self.0.name.set(Name::new(value.as_ref(), current.blurb()));
    }

    /// Check if a blurb is set.
    pub fn has_blurb(&self) -> bool {
        self.0.name.has_blurb()
    }

    /// Get the blurb, if any.
    pub fn blurb(&self) -> Option<&Text> {
        self.0.name.blurb()
    }

    /// Set the blurb.
    pub fn set_blurb(&self, value: Option<impl AsRef<str>>) {
        self.0.name = Name::new(self.0.name.name(), value);
    }

    /// Remove the blurb.
    pub fn remove_blurb(&self) {
        if self.0.name.has_blurb() {
            self.0.name = Name::new(self.0.name.name(), None);
        }
    }

    /// Set both name and blurb atomically.
    pub fn set_name_and_blurb(&self, name: impl AsRef<str>, blurb: Option<impl AsRef<str>>) {
        self.0.name = Name::new(name.as_ref(), blurb);
    }

    /// Get the last message.
    #[framed]
    pub async fn last_message(&self) -> Option<Arc<Message>> {
        self.read().await.last_message.clone()
    }

    /// Set the last message.
    #[framed]
    pub async fn set_last_message(&self, value: Option<Arc<Message>>) {
        self.write().await.last_message = value;
    }

    /// Get the default sendlist.
    #[framed]
    pub async fn default_sendlist(&self) -> Option<Arc<Sendlist>> {
        self.read().await.default_sendlist.clone()
    }

    /// Set the default sendlist.
    #[framed]
    pub async fn set_default_sendlist(&self, value: Option<Arc<Sendlist>>) {
        self.write().await.default_sendlist = value;
    }

    /// Get the last sendlist.
    #[framed]
    pub async fn last_sendlist(&self) -> Option<Arc<Sendlist>> {
        self.read().await.last_sendlist.clone()
    }

    /// Set the last sendlist.
    #[framed]
    pub async fn set_last_sendlist(&self, value: Option<Arc<Sendlist>>) {
        self.write().await.last_sendlist = value;
    }

    /// Get the last explicit sendlist.
    #[framed]
    pub async fn last_explicit(&self) -> Text {
        self.read().await.last_explicit.clone()
    }

    /// Set the last explicit sendlist.
    #[framed]
    pub async fn set_last_explicit(&self, value: impl Into<Arc<str>>) {
        let value: Arc<str> = value.into();
        self.write().await.last_explicit = Text(value);
    }

    /// Get the reply sendlist.
    #[framed]
    pub async fn reply_sendlist(&self) -> Text {
        self.read().await.reply_sendlist.clone()
    }

    /// Set the reply sendlist.
    #[framed]
    pub async fn set_reply_sendlist(&self, sendlist: impl Into<Arc<str>>) {
        let sendlist: Arc<str> = sendlist.into();
        let mut inner = self.write().await;

        // Quote if necessary
        if sendlist.chars().any(|c| c == ' ' || c == ',' || c == ':' || c == ';' || c == '_') {
            inner.reply_sendlist = Text::from(format!("\"{sendlist}\""));
        } else {
            inner.reply_sendlist = Text(sendlist);
        }
    }

    /// Get the oops text.
    #[framed]
    pub async fn oops_text(&self) -> Text {
        self.read().await.oops_text.clone()
    }

    /// Set the oops text.
    #[framed]
    pub async fn set_oops_text(&self, value: impl Into<Arc<str>>) {
        let value: Arc<str> = value.into();
        self.write().await.oops_text = Text(value);
    }

    pub async fn name_user(&self) -> Text {
        let name = self.name();
        let user_name = self.user_name().await;
        Text::from(format!("{name} ({user_name})"))
    }

    pub async fn close(&self, drain: bool) {
        let id = self.id;
        INITS.remove(&id);
        SESSIONS.remove(&id);

        if self.signed_on().await {
            self.notify_exit().await;
        }
        self.signed_on.store(false, Ordering::Relaxed);

        // Quit all discussions silently
        let disc_keys: Vec<_> = DISCUSSIONS.iter().map(|r| r.key().clone()).collect();
        for key in &disc_keys {
            if let Some(disc) = DISCUSSIONS.get(&key) {
                disc.quit(&self).await;
            }
        }

        // Close telnet connection if attached
        if let Some(telnet) = &*self.telnet.read().await {
            telnet.close(drain).await;
        }
        *self.telnet.write().await = None;

        // Disassociate from user
        if let Some(user) = &*self.user.read().await {
            user.remove_session(&self).await;
            *self.user.write().await = None;
        }
    }

    pub async fn transfer(&self, new_telnet: Telnet) {
        let old_telnet = self.telnet.read().await.clone();
        *self.telnet.write().await = Some(new_telnet.clone());
        new_telnet.set_session(Some(self.clone())).await;

        if let Some(old) = old_telnet {
            let who = self.name_user().await;
            info!("Transfer: {who} from fd to new connection");
            old.output("*** This session has been transferred to a new connection. ***\n").await;
            old.close(true).await;
        }

        self.enqueue_others(Arc::new(TransferNotify::new(self.name()))).await;
        self.pending.attach(new_telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn attach(&self, telnet: Telnet) {
        *self.telnet.write().await = Some(telnet.clone());
        telnet.set_session(Some(self.clone())).await;

        let who = self.name_user().await;
        info!("Attach: {who} on new connection");

        self.enqueue_others(Arc::new(AttachNotify::new(self.name()))).await;
        self.pending.attach(telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn detach(&self, telnet: &Telnet, intentional: bool) {
        if self.signed_on().await && self.priv_level().await > 0 {
            let current_telnet = self.telnet.read().await;
            if let Some(t) = &*current_telnet {
                let who = self.name_user().await;
                if intentional {
                    info!("Detach: {who} (intentional)");
                } else {
                    info!("Detach: {who} (accidental)");
                };

                self.enqueue_others(Arc::new(DetachNotify::new(self.name(), intentional))).await;
                *self.telnet.write().await = None;
            }
        } else {
            self.close(true).await;
        }
    }

    pub async fn announce(message: &str) {
        for session in &SESSIONS {
            session.output(message).await;
            session.enqueue_output().await;
        }

        for session in &INITS {
            session.output(message).await;
            session.enqueue_output().await;
        }
    }

    pub async fn remove_discussion(name: Text) {
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
            self.pending.enqueue(Some(telnet), Arc::new(Text::new(text))).await;
        } else {
            self.pending.enqueue(None, Arc::new(Text::new(text))).await;
        }
    }

    pub async fn enqueue_others(&self, out: Arc<dyn OutputObj>) {
        for session in &SESSIONS {
            if session != self {
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

    pub async fn find_sendable(
        &self,
        sendlist: &str,
        member: bool,
        exact: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) -> (Option<Session>, OrderedSet<Session>, Option<Discussion>, OrderedSet<Discussion>) {
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

            for s in &SESSIONS {
                let s_name = s.name();
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
            for d in &DISCUSSIONS {
                if member {
                    let inner = d.read().await;
                    if !inner.members.contains(&self) {
                        continue;
                    }
                }

                let d_name = d.name();
                if d_name.eq_ignore_ascii_case(sendlist) {
                    discussion = Some(d.clone());
                    discussion_matches.insert(d.clone());
                } else if !exact {
                    if let Some(pos) = match_name(&d_name, sendlist) {
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

    pub async fn find_session(&self, sendlist: &str) -> (Option<Session>, OrderedSet<Session>) {
        let (session, matches, _, _) = self.find_sendable(sendlist, false, false, true, false).await;
        (session, matches)
    }

    pub async fn find_discussion(&self, sendlist: &str, member: bool) -> (Option<Discussion>, OrderedSet<Discussion>) {
        let (_, _, discussion, matches) = self.find_sendable(sendlist, member, false, false, true).await;
        (discussion, matches)
    }

    pub async fn notify_entry(&self) {
        let who = self.name_user().await;
        if let Some(_telnet) = &*self.telnet.read().await {
            info!("Enter: {who} on connection");
        } else {
            info!("Enter: {who}, detached");
        }

        let now = Timestamp::new();
        *self.idle_since.write().await = now;
        *self.login_time.write().await = now;

        self.enqueue_others(Arc::new(EntryNotify::new(self.name()))).await;
    }

    pub async fn notify_exit(&self) {
        let who = self.name_user().await;
        if let Some(_telnet) = &*self.telnet.read().await {
            info!("Exit: {who} on connection");
        } else {
            info!("Exit: {who}, detached");
        }

        self.enqueue_others(Arc::new(ExitNotify::new(self.name()))).await;
    }

    pub async fn init_login_sequence(&self) {
        let mut inner = self.inner.write().await;
        inner.start_login_timeout().await;
        //        inner.set_login_state(LoginState::AwaitingLogin, Some("login: ")).await;
    }
}

impl SessionInner {
    pub async fn start_login_timeout(&mut self) {
        let session_id = self.id;

        let handle = tokio::spawn(async move {
            tokio::time::sleep(LOGIN_TIMEOUT).await;

            if let Some(session) = SESSIONS.get(&session_id).map(|e| e.value().clone()).or_else(|| INITS.get(&session_id).map(|e| e.value().clone())) {
                session.enqueue_output().await;
                session.close(true).await;
            }
        });

        *self.login_timeout.write().await = Some(handle.abort_handle());
    }
}

impl Session {
    pub async fn cancel_login_timeout(&self) {
        if let Some(handle) = self.login_timeout.write().await.take() {
            handle.abort();
        }
    }

    pub async fn set_login_state(&self, state: LoginState, prompt: Option<&str>) {
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

    pub async fn process_pending_lines(&self) {
        loop {
            let line = {
                let mut lines = self.lines.lock().await;
                if lines.is_empty() {
                    break;
                }
                lines.remove(0)
            };

            self.handle_input(line).await;
        }
    }

    pub async fn handle_input(&self, line: String) {
        self.pending.dequeue().await;

        match *self.login_state.read().await {
            LoginState::PreLogin => self.save_input_line(line).await,
            LoginState::AwaitingLogin => self.handle_login_input(line).await,
            LoginState::AwaitingPassword => self.handle_password_input(&line).await,
            LoginState::AwaitingName => self.handle_name_input(&line).await,
            LoginState::AwaitingBlurb => self.handle_blurb_input(&line).await,
            LoginState::AwaitingTransferConfirmation => self.handle_transfer_input(&line).await,
            LoginState::LoggedIn => self.process_input(&line).await,
        }

        self.enqueue_output().await;
    }

    pub async fn save_input_line(&self, line: String) {
        self.lines.lock().await.push(line);
    }

    pub async fn handle_login_input(&self, line: String) {
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

        let user = (*USER_MANAGER).get_user(&line).await;
        *self.user.write().await = user.clone();

        if user.is_none() || user.as_ref().unwrap().read().await.password.is_some() {
            // Need password
            if let Some(telnet) = &*self.telnet.read().await {
                let echo = telnet.get_echo().await;
                if echo == 0 {
                    telnet.output("\n\x07Sorry, password probably WILL echo.\n\n").await;
                } else if echo != TELNET_ENABLED {
                    telnet.output("\nWarning: password may echo.\n\n").await;
                }

                telnet.set_do_echo(false).await;
                self.set_login_state(LoginState::AwaitingPassword, Some("Password: ")).await;
            }
        } else {
            // No password required (guest account)
            self.print_reserved_names().await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
        }
    }

    pub async fn handle_password_input(&self, line: &str) {
        if let Some(telnet) = &*self.telnet.read().await {
            telnet.output("\n").await;
            telnet.set_do_echo(true).await;
        }

        (*USER_MANAGER).update_all().await.ok();

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

            self.set_login_state(LoginState::AwaitingLogin, Some("login: ")).await;
            *self.user.write().await = None;
            return;
        }

        self.print_reserved_names().await;
        self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
    }

    pub async fn print_reserved_names(&self) {
        if let Some(user_lock) = &*self.user.read().await {
            let user = user_lock.read().await;

            if let Some(first) = user.reserved.first() {
                self.output(&format!("\nYour default (reserved) name is \"{first}\".\n")).await;

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

    pub async fn handle_name_input(&self, line: &str) {
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
            Text::new(line)
        };

        *self.name.write().await = name.clone();

        if self.check_name_availability(&name, false, false).await {
            self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await;
        }
    }

    pub async fn handle_blurb_input(&self, line: &str) {
        if !self.check_name_availability(&self.name(), true, false).await {
            return;
        }

        let line = if line.is_empty() {
            let user_guard = self.user.read().await;
            if let Some(user_lock) = &*user_guard {
                let user = user_lock.read().await;
                user.blurb.as_ref().map(|b| b.as_str()).unwrap_or("").to_string()
            } else {
                String::new()
            }
        } else {
            line.to_string()
        };

        self.do_blurb(&line, true).await;

        self.set_login_state(LoginState::LoggedIn, None);

        self.signed_on.store(true, Ordering::Relaxed);

        if let Some(user_lock) = &*self.user.read().await {
            let mut user = user_lock.write().await;
            self.priv_level.store(user.priv_level, Ordering::Relaxed);
            user.add_session(self.clone()).await;
        }

        SESSIONS.insert(self.id, self.clone());
        INITS.remove(&self.id);

        self.notify_entry().await;

        // Welcome message and automatic commands
        self.output("\n\nWelcome to Phoenix.  Type \"/help\" for a list of commands.\n\n").await;

        // Make sure discussion A exists
        match self.find_sendable("A", false, true, true, true).await {
            (_, _, None, _) => {
                let disc = Discussion::new(None, "A", "General Discussion", true).await;
                DISCUSSIONS.insert("A".to_string(), disc);
            }
            _ => {}
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

    pub async fn handle_transfer_input(&self, line: &str) {
        if match_keyword(line, "yes", 1).is_none() {
            self.output("Session not transferred.\n").await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
            return;
        }

        if self.check_name_availability(&self.name(), true, true).await {
            self.output("(That session is now gone.)\n").await;
            self.set_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await;
        }
    }

    pub async fn process_input(&self, line: &str) {
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
                self.output("Unknown /command.  Type /help for help.\n").await;
            }
        } else if line == " " {
            self.do_reset().await;
        } else if !line.is_empty() {
            self.do_message(line).await;
        }
    }

    pub async fn check_name_availability(&self, name: &str, double_check: bool, transferring: bool) -> bool {
        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  Choose another name.\n").await;
            self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
            return false;
        }

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(name).await {
            match (&*self.user.read().await, &*found_user.read().await) {
                (Some(my_user), Some(found_user)) if my_user.user == found_user.user => {
                    let now = if double_check { " now" } else { "" };
                    self.output(&format!("\"{reserved}\" is{now} a reserved name.  Choose another.\n")).await;
                    self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
                    return false;
                }
                _ => {}
            }
        }

        match self.find_sendable(name, false, true, true, true).await {
            (Some(found_session), _, _, _) => {
                match (&*self.user.read().await, &*found_session.user.read().await) {
                    (Some(my_user), Some(their_user)) if my_user.user == their_user.user && found_session.priv_level().await > 0 => {
                        if let Some(their_telnet) = &*found_session.telnet.read().await {
                            if transferring {
                                self.output("Transferring active session...\n").await;
                                found_session.transfer(self.telnet.read().await.as_ref().unwrap().clone()).await;
                                *self.telnet.write().await = None;
                                self.close(true).await;
                            } else {
                                let now = if double_check { " now" } else { "" };
                                self.output(&format!("You are{now} attached elsewhere under that name.\n")).await;
                                self.set_login_state(LoginState::AwaitingTransferConfirmation, Some("Transfer active session? [no] ")).await;
                            }
                        } else {
                            self.output("Attaching to detached session...\n").await;
                            found_session.attach(self.telnet.read().await.as_ref().unwrap().clone()).await;
                            *self.telnet.write().await = None;
                            self.close(true).await;
                        }
                    }
                    _ => {
                        let found_name = found_session.name();
                        let already = if double_check { "now" } else { "already" };
                        self.output(&format!("The name \"{found_name}\" is {already} in use.  Choose another.\n")).await;
                        self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
                    }
                }

                false
            }
            (_, _, Some(found_discussion), _) => {
                let found_name = found_discussion.name().await;
                let already = if double_check { "now" } else { "already" };
                self.output(&format!("There is {already} a discussion named \"{found_name}\".  Choose another name.\n")).await;
                self.set_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;

                false
            }
            _ => true,
        }
    }

    // Command implementations
    pub async fn reset_idle(&self, min: usize) -> i64 {
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

    pub async fn print_time_long(&self, minutes: i32) {
        // Determine time format (0 = verbose, 1 = both, 2 = terse)
        let format = if let Some(fmt) = self.get_sys_var("time_format").await {
            match fmt.as_str() {
                "verbose" => 0,
                "both" => 1,
                "terse" => 2,
                _ => 0,
            }
        } else {
            match DEFAULTS.read().await.get("time_format").map(|s| s.as_str()) {
                Some("verbose") => 0,
                Some("both") => 1,
                Some("terse") => 2,
                _ => 0,
            }
        };

        // Calculate time components
        let hours = minutes / 60;
        let days = hours / 24;
        let minutes = minutes % 60;
        let hours = hours % 24;

        // Print verbose format if format <= 1
        if format <= 1 {
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
                    self.output(&format!(" {days} day{s}{and}")).await;
                }
                if hours > 0 {
                    let s = if hours == 1 { "" } else { "s" };
                    let and = if minutes > 0 { " and" } else { "" };
                    self.output(&format!(" {hours} hour{s}{and}")).await;
                }
                if minutes > 0 {
                    let s = if minutes == 1 { "" } else { "s" };
                    self.output(&format!(" {minutes} minute{s}")).await;
                }
            } else {
                self.output(" under a minute").await;
            }
        }

        // Print separator and/or terse format if format >= 1
        if format >= 1 {
            self.output(" ").await;
        }
        if format == 1 {
            self.output("(").await;
        }
        if format >= 1 {
            if days > 0 {
                self.output(&format!("{days}d{hours:02}:{minutes:02}")).await;
            } else {
                self.output(&format!("{hours}:{minutes:02}")).await;
            }
        }
        if format == 1 {
            self.output(")").await;
        }
    }

    pub async fn do_restart(&self, args: &str) {
        let who = self.name_user().await;
        let name = self.name();

        if args == "!" {
            // Immediate restart
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await;
            self.server.schedule_restart(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel restart
            match self.server.cancel_shutdown().await {
                Some(true) => {
                    info!("Restart cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server restart. ***\n")).await;
                }
                Some(false) => {
                    info!("Shutdown cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server shutdown. ***\n")).await;
                }
                None => self.output("The server was not about to shut down or restart.\n").await,
            }
        } else {
            // Delayed restart
            let seconds = args.parse::<u64>().unwrap_or(30);
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await;
            self.server.schedule_restart(who.clone(), seconds).await;
        }
    }

    pub async fn do_down(&self, args: &str) {
        let who = self.name_user().await;
        let name = self.name();

        if args == "!" {
            // Immediate shutdown
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await;
            self.server.schedule_shutdown(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel shutdown
            match self.server.cancel_shutdown().await {
                Some(true) => {
                    info!("Restart cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server restart. ***\n")).await;
                }
                Some(false) => {
                    info!("Shutdown cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server shutdown. ***\n")).await;
                }
                None => self.output("The server was not about to shut down or restart.\n").await,
            }
        } else {
            // Delayed shutdown
            let seconds = args.parse::<u64>().unwrap_or(30);
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await;
            self.server.schedule_shutdown(who.clone(), seconds).await;
        }
    }

    pub async fn do_nuke(&self, args: &str) {
        let drain = !args.starts_with('!');
        let args = if drain { args } else { &args[1..] };

        match self.find_session(args).await {
            (Some(target), _) => {
                let who = target.name_user().await;
                let name = target.name();
                let by_who = self.name_user().await;
                let by_name = self.name();

                if drain {
                    self.output(&format!("\"{name}\" has been nuked.\n")).await;
                } else {
                    self.output(&format!("\"{name}\" has been nuked immediately.\n")).await;
                }

                if let Some(telnet) = &*target.telnet.read().await {
                    *target.telnet.write().await = None;
                    info!("{who} has been nuked by {by_who}");
                    telnet.undraw_input().await;
                    telnet.output(&format!("\x07\x07\x07*** You have been nuked by {by_name}. ***\n")).await;
                    telnet.redraw_input().await;
                    telnet.close(drain).await;
                } else {
                    info!("{who}, detached, has been nuked by {by_who}");
                    target.close(true).await;
                }
            }
            (_, matches) => {
                self.output("\x07\x07").await;
                self.session_matches(args, &matches).await;
            }
        }
    }

    pub async fn do_bye(&self, _args: &str) {
        self.close(true).await;
    }

    pub async fn do_who(&self, args: &str) {
        // Get set of users to display.
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return;
        }

        let now = Timestamp::new();
        let mut extend = 0;

        // Find longest idle time for formatting.
        for session in &who {
            let days = (now - session.idle_since().await) / 86400;
            if days == 0 {
                continue;
            }

            let mut width = days.to_string().len();
            if session.telnet().await.is_none() || (now - session.login_time().await) >= 31536000 {
                width += 1;
            }
            if width > extend {
                extend = width;
            }
        }

        // Output header.
        let spaces = " ".repeat(extend);
        let head1 = " Name                              On Since";
        let line1 = " ----                              --------";
        let head2 = "  Idle  Away";
        let line2 = "  ----  ----";
        self.output(&format!("\n{head1}{spaces}{head2}\n{line1}{spaces}{line2}\n")).await;

        // Output each user.
        for session in &who {
            // Detached indicator.
            let detached = session.detached_indicator().await;
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Login time or "detached".
            if session.telnet().await.is_some() {
                let login_time = session.login_time().await;
                if (now - login_time) < 86400 {
                    self.output(&login_time.date(11, 8)).await;
                } else if (now - login_time) < 31536000 {
                    self.output(" ").await;
                    self.output(&login_time.date(4, 6)).await;
                    self.output(" ").await;
                } else {
                    self.output(&login_time.date(4, 4)).await;
                    self.output(&login_time.date(20, 4)).await;
                }
            } else {
                self.output("detached").await;
            }

            // Idle time.
            let idle = (now - session.idle_since().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.output(&format!("{days:>extend$}d{hours:02}:{minutes:02}  ")).await;
                } else if hours > 0 {
                    let width = extend + 3;
                    self.output(&format!("{hours:>width$}:{minutes:02}  ")).await;
                } else {
                    let width = extend + 6;
                    self.output(&format!("{minutes:>width$}  ")).await;
                }
            } else {
                self.output(" ".repeat(extend + 8)).await;
            }

            // Away state.
            match session.away_state().await {
                AwayState::Here => self.output("Here\n").await,
                AwayState::Away => self.output("Away\n").await,
                AwayState::Busy => self.output("Busy\n").await,
                AwayState::Gone => self.output("Gone\n").await,
            }

            // Show continuation of long name if only one user.
            if name.len() > 33 && who.len() == 1 {
                self.output(&format!(">{}\n", &name.as_str()[32..])).await;
            }
        }

        // Output message and errors from get_who_set().
        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }
    }

    pub async fn do_idle(&self, args: &str) {
        // Get set of users to display.
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return;
        }

        let now = Timestamp::new();
        let mut col = 0;

        // Output header.
        let head = " Name                              Idle";
        let line = " ----                              ----";
        if who.len() == 1 {
            self.output(&format!("\n{head}\n{line}\n")).await;
        } else {
            self.output(&format!("\n{head} {head}\n{line} {line}\n")).await;
        }

        // Output each user.
        for session in &who {
            // Detached indicator.
            let detached = session.detached_indicator().await;
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Idle time.
            let idle = (now - session.idle_since().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 9 {
                    self.output(&format!("{days:2}d{hours:02}")).await;
                } else if days > 0 {
                    self.output(&format!("{days}d{hours:02}h")).await;
                } else if hours > 0 {
                    self.output(&format!("{hours:2}:{minutes:02}")).await;
                } else {
                    self.output(&format!("   {minutes:2}")).await;
                }
            } else {
                self.output("     ").await;
            }

            // Column handling.
            if col == 0 && who.len() > 1 {
                self.output("  ").await;
            } else {
                self.output("\n").await;
            }
            col = 1 - col;
        }

        // Output newline if last line has only one column.
        if col == 1 {
            self.output("\n").await;
        }

        // Output message and errors from get_who_set().
        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }
    }

    pub async fn do_why(&self, args: &str) {
        // This is a privileged command.
        if self.priv_level().await < 50 {
            self.output("Why not?\n").await;
            return;
        }

        // Get set of users to display.
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return;
        }

        let now = Timestamp::new();
        let mut extend = 0;

        // Find longest idle time for formatting.
        for session in &who {
            let days = (now - session.idle_since().await) / 86400;
            if days == 0 {
                continue;
            }

            let mut width = days.to_string().len();
            if (now - session.login_time().await) >= 31536000 {
                width += 1;
            }
            if width > extend {
                extend = width;
            }
        }

        // Output header.
        let spaces = " ".repeat(extend);
        let head1 = " Name                              On Since";
        let line1 = " ----                              --------";
        let head2 = "  Idle  Away  User      FD  Priv";
        let line2 = "  ----  ----  ----      --  ----";
        self.output(&format!("\n{head1}{spaces}{head2}\n{line1}{spaces}{line2}\n")).await;

        // Output each user.
        for session in &who {
            // Detached indicator.
            let detached = session.detached_indicator().await;
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Login time.
            let login_time = session.login_time().await;
            if (now - login_time) < 86400 {
                self.output(&login_time.date(11, 8)).await;
            } else if (now - login_time) < 31536000 {
                self.output(" ").await;
                self.output(&login_time.date(4, 6)).await;
                self.output(" ").await;
            } else {
                self.output(&login_time.date(4, 4)).await;
                self.output(&login_time.date(20, 4)).await;
            }

            // Idle time.
            let idle = (now - session.idle_since().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.output(&format!("{days:>extend$}d{hours:02}:{minutes:02}  ")).await;
                } else if hours > 0 {
                    let width = extend + 3;
                    self.output(&format!("{hours:>width$}:{minutes:02}  ")).await;
                } else {
                    let width = extend + 6;
                    self.output(&format!("{minutes:>width$}  ")).await;
                }
            } else {
                self.output(" ".repeat(extend + 8)).await;
            }

            // Away state.
            match session.away_state().await {
                AwayState::Here => self.output("Here  ").await,
                AwayState::Away => self.output("Away  ").await,
                AwayState::Busy => self.output("Busy  ").await,
                AwayState::Gone => self.output("Gone  ").await,
            }

            // Username.
            if let Some(user) = session.user().await {
                let username = user.username().await;
                self.output(&format!("{username:<8}  ")).await;
            } else {
                self.output("guest     ").await;
            }

            // File descriptor.
            if let Some(telnet) = session.telnet().await {
                let fd = telnet.fd().await;
                self.output(&format!("{fd:2} ")).await;
            } else {
                self.output("-- ").await;
            }

            // Privilege level.
            let session_priv = session.priv_level().await;
            let user_priv = if let Some(user) = session.user().await { user.priv_level().await } else { 0 };
            let indicator = if session_priv == user_priv { " " } else { "*" };
            self.output(&format!("{indicator}{session_priv:4}\n")).await;
        }

        // Output message and errors from get_who_set().
        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }
    }

    pub async fn do_blurb(&self, args: &str, entry: bool) {
        let args = args.trim();

        if !args.is_empty() {
            let mut start = 0;
            let mut end = args.len();

            if args.len() == 3 && args.eq_ignore_ascii_case("off") {
                if entry || !self.blurb().await.is_empty() {
                    self.reset_idle(10).await;
                    self.remove_blurb().await;
                    if !entry {
                        self.output("Your blurb has been turned off.\n").await;
                    }
                } else if !entry {
                    self.output("Your blurb was already turned off.\n").await;
                }
            } else {
                if (args.starts_with('"') && args.ends_with('"') && args.len() > 2) || (args.starts_with('[') && args.ends_with(']')) {
                    start = 1;
                    end = args.len() - 1;
                }

                self.reset_idle(10).await;

                let blurb = &args[start..end];
                self.set_blurb(blurb).await;
                if !entry {
                    self.output(&format!("Your blurb has been set to [{blurb}].\n")).await;
                }
            }
        } else if entry {
            self.remove_blurb().await;
        } else if self.has_blurb().await {
            let blurb = self.blurb().await;
            self.output(&format!("Your blurb is currently set to [{blurb}].\n")).await;
        } else {
            self.output("You do not currently have a blurb set.\n").await;
        }
    }

    pub async fn do_here(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"here\".\n").await;
        self.set_away(AwayState::Here);
        self.enqueue_others(Arc::new(HereNotify::new(self.name()))).await;
    }

    pub async fn do_away(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"away\".\n").await;
        self.set_away(AwayState::Away);
        self.enqueue_others(Arc::new(AwayNotify::new(self.name()))).await;
    }

    pub async fn do_busy(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"busy\".\n").await;
        self.set_away(AwayState::Busy);
        self.enqueue_others(Arc::new(BusyNotify::new(self.name()))).await;
    }

    pub async fn do_gone(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"gone\".\n").await;
        self.set_away(AwayState::Gone);
        self.enqueue_others(Arc::new(GoneNotify::new(self.name()))).await;
    }

    pub async fn do_clear(&self, _args: &str) {
        self.output("\x1b[H\x1b[J").await;
    }

    pub async fn do_unidle(&self, _args: &str) {
        let idle = self.reset_idle(1).await;
        if idle == 0 {
            self.output("Your idle time has been reset.\n").await;
        }
    }

    pub async fn do_detach(&self, _args: &str) {
        if self.priv_level().await > 0 {
            self.reset_idle(10).await;
            self.output("You have been detached.\n").await;
            self.enqueue_output().await;
            if let Some(telnet) = &*self.telnet.read().await {
                telnet.close(true).await;
            }
        } else {
            self.output("Guest users are not allowed to detach from the system.  Use /bye to sign off.\n").await;
        }
    }

    pub async fn do_howmany(&self, _args: &str) {
        let mut here = 0;
        let mut away = 0;
        let mut busy = 0;
        let mut gone = 0;
        let mut attached = 0;
        let mut detached = 0;
        let mut total = 0;

        for session in &SESSIONS {
            match session.away() {
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
        self.output("  \"Here\"     \"Away\"     \"Busy\"     \"Gone\"    Attached   Detached    Total\n").await;
        let here_pct = (here * 100 + total / 2) / total.max(1);
        let away_pct = (away * 100 + total / 2) / total.max(1);
        let busy_pct = (busy * 100 + total / 2) / total.max(1);
        let gone_pct = (gone * 100 + total / 2) / total.max(1);
        let attached_pct = (attached * 100 + total / 2) / total.max(1);
        let detached_pct = (detached * 100 + total / 2) / total.max(1);
        self.output(&format!(" {here:3} {here_pct:3}%   {away:3} {away_pct:3}%   {busy:3} {busy_pct:3}%   {gone:3} {gone_pct:3}%   {attached:3} {attached_pct:3}%   {detached:3} {detached_pct:3}%   {total:3} 100%\n")).await;

        let disc_count = DISCUSSIONS.len();
        self.output(&format!("\nDiscussions in use: {disc_count}\n\n")).await;
    }

    pub async fn do_what(&self, args: &str) {
        if DISCUSSIONS.is_empty() {
            self.output("No discussions currently exist.\n").await;
            return;
        }

        let sendlist = Sendlist::new(&self, args, true, false, true).await;

        if !args.is_empty() && sendlist.discussions.is_empty() {
            self.output(&sendlist.errors).await;
            return;
        }

        let discussions = if args.is_empty() { DISCUSSIONS.iter().map(|r| r.value().clone()).collect() } else { sendlist.discussions.clone() };

        self.output("\n Name            Users  Idle  Title\n").await;
        self.output(" ----            -----  ----  -----\n").await;

        let now = Timestamp::new();

        for disc in &discussions {
            let disc_name = disc.name().await;
            self.output(" ").await;
            let name = if disc_name.len() > 15 { format!("{disc_name:<14.14}+") } else { format!("{disc_name:<15}") };
            self.output(&name).await;

            let inner = disc.read().await;
            let member_count = inner.members.len();
            let is_member = if inner.members.contains(&self) { '*' } else { ' ' };
            self.output(&format!("{member_count:>3}{is_member} ")).await;

            let idle = (now - disc.idle_since().await) / 60;
            if idle > 0 {
                let hours = idle / 60;
                let minutes = idle % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.output(&format!(" {days}d{hours:02}:{minutes:02}  ")).await;
                } else if hours > 0 {
                    self.output(&format!("    {hours}:{minutes:02}  ")).await;
                } else {
                    self.output(&format!("      {minutes:>2}  ")).await;
                }
            } else {
                self.output("         ").await;
            }

            if disc.is_permitted(&self.name()).await {
                let title = &disc.title().await;
                if title.len() > 49 {
                    self.output(&format!("{title:<48.48}+\n")).await;
                } else {
                    self.output(&format!("{title}\n")).await;
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

    pub async fn do_date(&self, _args: &str) {
        let t = Timestamp::new();
        let date = t.date(0, 0);
        self.output(&format!("{date}\n")).await;
    }

    pub async fn do_signal(&self, args: &str) {
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
                self.output("Signals for public messages are now on.\n").await;
            } else if let Some(_) = match_keyword(args, "off", 2) {
                self.signal_public.store(false, Ordering::Relaxed);
                self.output("Signals for public messages are now off.\n").await;
            } else if args.is_empty() {
                let on = if self.signal_public.load(Ordering::Relaxed) { "on" } else { "off" };
                self.output(&format!("Signals are {on} for public messages.\n")).await;
            } else {
                self.output("Usage: /signal public [on|off]\n").await;
            }
        } else if let Some(rest) = match_keyword(args, "private", 2) {
            args = rest;
            if let Some(_) = match_keyword(args, "on", 2) {
                self.signal_private.store(true, Ordering::Relaxed);
                self.output("Signals for private messages are now on.\n").await;
            } else if let Some(_) = match_keyword(args, "off", 2) {
                self.signal_private.store(false, Ordering::Relaxed);
                self.output("Signals for private messages are now off.\n").await;
            } else if args.is_empty() {
                let on = if self.signal_private.load(Ordering::Relaxed) { "on" } else { "off" };
                self.output(&format!("Signals are {on} for private messages.\n")).await;
            } else {
                self.output("Usage: /signal private [on|off]\n").await;
            }
        } else if args.is_empty() {
            let pub_sig = self.signal_public.load(Ordering::Relaxed);
            let priv_sig = self.signal_private.load(Ordering::Relaxed);

            if pub_sig == priv_sig {
                let on = if pub_sig { "on" } else { "off" };
                self.output(&format!("Signals are {on} for both public and private messages.\n")).await;
            } else {
                let pub_on = if pub_sig { "on" } else { "off" };
                let priv_on = if priv_sig { "on" } else { "off" };
                self.output(&format!("Signals are {pub_on} for public messages and {priv_on} for private messages.\n")).await;
            }
        } else {
            self.output("Usage: /signal [public|private] [on|off]\n").await;
        }
    }

    pub async fn do_send(&self, args: &str) {
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
            self.output("Your default sendlist has been turned off.\n").await;
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

        let sendlist = Sendlist::new(&self, &slist, false, true, true).await;

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

    pub async fn print_sendlist(&self, sendlist: &Sendlist) {
        if !sendlist.sessions.is_empty() {
            let mut first = true;
            for session in &sendlist.sessions {
                if first {
                    first = false;
                } else {
                    self.output(", ").await;
                }
                self.output(&session.name()).await;
            }

            if !sendlist.discussions.is_empty() {
                let s = if sendlist.discussions.len() == 1 { "" } else { "s" };
                self.output(&format!(" and discussion{s} ")).await;

                first = true;
                for discussion in &sendlist.discussions {
                    if first {
                        first = false;
                    } else {
                        self.output(", ").await;
                    }
                    self.output(discussion.name().await).await;
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
                self.output(discussion.name().await).await;
            }
        }
    }

    pub async fn do_join(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /join <disc>[,<disc>...]\n").await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (name, rest) = getword(remaining, Some(','));
            remaining = rest;

            match self.find_discussion(name, false).await {
                (Some(discussion), _) => discussion.join(&self).await,
                (_, matches) => self.discussion_matches(name, &matches).await,
            }
        }
    }

    pub async fn do_quit(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /quit <disc>[,<disc>...]\n").await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (name, rest) = getword(remaining, Some(','));
            remaining = rest;

            match self.find_discussion(name, false).await {
                (Some(discussion), _) => discussion.quit(&self).await,
                _ => match self.find_discussion(name, true).await {
                    (Some(discussion), _) => discussion.quit(&self).await,
                    (_, matches) => self.discussion_matches(name, &matches).await,
                },
            }
        }
    }

    pub async fn do_create(&self, args: &str) {
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
            self.output("Usage: /create [public|private] <name> <title>\n").await;
            return;
        }

        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (not created)\n").await;
            return;
        }

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(name).await {
            let is_same_user = match (&*self.user.read().await, &*found_user.read().await) {
                (Some(my_user), Some(found_user)) if my_user.user == found_user.user => true,
                _ => false,
            };

            let a = if is_same_user { "your" } else { "a" };
            self.output(&format!("\"{reserved}\" is {a} reserved name. (not created)\n")).await;
            return;
        }

        match self.find_sendable(name, false, true, true, true).await {
            (Some(session), _, _, _) => {
                let name = session.name();
                self.output(&format!("There is already someone named \"{name}\". (not created)\n")).await;
                return;
            }
            (_, _, Some(discussion), _) => {
                let name = discussion.name().await;
                self.output(&format!("There is already a discussion named \"{name}\". (not created)\n")).await;
                return;
            }
            _ => {}
        }

        let disc = Discussion::new(Some(self.clone()), name, title, is_public).await;
        DISCUSSIONS.insert(name.to_string(), disc.clone());

        self.enqueue_others(Arc::new(CreateNotify::new(disc.name().await, disc.title().await, disc.is_public().await, self.name()))).await;

        let name = disc.name().await;
        let title = disc.title().await;
        self.output(&format!("You have created discussion {name}, \"{title}\".\n")).await;
    }

    pub async fn do_destroy(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /destroy <disc>[,<disc>...]\n").await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (name, rest) = getword(remaining, Some(','));
            remaining = rest;

            match self.find_discussion(name, false).await {
                (Some(discussion), _) => discussion.destroy(&self).await,
                _ => match self.find_discussion(name, true).await {
                    (Some(discussion), _) => discussion.destroy(&self).await,
                    (_, matches) => self.discussion_matches(name, &matches).await,
                },
            }
        }
    }

    pub async fn do_permit(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /permit <disc> <person>[,<person>...]\n").await;
            return;
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.permit(&self, rest).await,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.permit(&self, rest).await,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }
    }

    pub async fn do_depermit(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /depermit <disc> <person>[,<person>...]\n").await;
            return;
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.depermit(&self, rest).await,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.depermit(&self, rest).await,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }
    }

    pub async fn do_appoint(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /appoint <disc> <person>[,<person>...]\n").await;
            return;
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.appoint(&self, rest).await,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.appoint(&self, rest).await,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }
    }

    pub async fn do_unappoint(&self, args: &str) {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /unappoint <disc> <person>[,<person>...]\n").await;
            return;
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.unappoint(&self, rest).await,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.unappoint(&self, rest).await,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }
    }

    pub async fn do_rename(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /rename <name>\n").await;
            return;
        }

        if args.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (name unchanged)\n").await;
            return;
        }

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(args).await {
            match (&*self.user.read().await, &*found_user.read().await) {
                (Some(my_user), Some(found_user)) if my_user.user == found_user.user => {
                    self.output(&format!("\"{reserved}\" is a reserved name.  (name unchanged)\n")).await;
                    return;
                }
                _ => {}
            }
        }

        match self.find_sendable(args, false, true, true, true).await {
            (Some(found_session), _, _, _) if found_session != self => {
                let found_name = found_session.name();
                self.output(&format!("The name \"{found_name}\" is already in use.  (name unchanged)\n")).await;
            }
            (_, _, Some(found_discussion), _) => {
                let found_name = found_discussion.name().await;
                self.output(&format!("There is already a discussion named \"{found_name}\".  (name unchanged)\n")).await;
            }
            _ => {}
        }

        self.enqueue_others(Arc::new(RenameNotify::new(self.name(), args))).await;

        self.output(&format!("You have changed your name to \"{args}\".\n")).await;
        *self.name.write().await = Name::new(args, self.blurb().await);
    }

    pub async fn do_set(&self, args: &str) {
        let (var, value) = getword(args, Some('='));
        if var.is_empty() || value.is_empty() {
            self.output("Usage: /set <variable>=<value>\n").await;
            return;
        }

        if var.starts_with('$') {
            self.user_vars.write().await.insert(var.to_string(), value.to_string());
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
                        self.output(&format!("Terminal height is now set to {h}.\n")).await;
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
                self.sys_vars.write().await.insert("time_format".to_string(), "verbose".to_string());
            } else if let Some(_) = match_keyword(value, "both", 4) {
                self.sys_vars.write().await.insert("time_format".to_string(), "both".to_string());
            } else if let Some(_) = match_keyword(value, "terse", 5) {
                self.sys_vars.write().await.insert("time_format".to_string(), "terse".to_string());
            } else if let Some(_) = match_keyword(value, "default", 7) {
                self.sys_vars.write().await.remove("time_format");
            } else {
                self.output("Usage: /set time_format [terse|verbose|both|default]\n").await;
            }
        } else if let Some(_) = match_keyword(var, "uptime", 6) {
            self.output("Server uptime is a readonly variable.\n").await;
        } else if let Some(_) = match_keyword(var, "width", 5) {
            if let Ok(width) = value.parse::<usize>() {
                if width > 0 {
                    if let Some(telnet) = &*self.telnet.read().await {
                        let w = telnet.set_width(width).await;
                        self.output(&format!("Terminal width is now set to {w}.\n")).await;
                    }
                } else {
                    self.output("Usage: /set width=<number of columns>\n").await;
                }
            } else {
                self.output("Usage: /set width=<number of columns>\n").await;
            }
        } else {
            self.output(&format!("Unknown system variable: \"{var}\"\n")).await;
        }
    }

    pub async fn set_idle(&self, args: &str) {
        let now = Timestamp::new();
        let current_idle = (now - self.idle_since().await) / 60;

        // Parse time specification: <d>d<hh>:<mm>
        let mut chars = args.trim().chars().peekable();
        let mut days = 0i64;
        let mut hours = 0i64;
        let mut minutes = 0i64;

        // Parse first number
        let mut num = 0i64;
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                num = num * 10 + (ch as i64 - '0' as i64);
                chars.next();
            } else {
                break;
            }
        }

        // Skip whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }

        // Check for 'd' or 'D' (days)
        if chars.peek() == Some(&'d') || chars.peek() == Some(&'D') {
            days = num;
            chars.next();

            // Skip whitespace and parse next number
            while chars.peek() == Some(&' ') {
                chars.next();
            }

            num = 0;
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() {
                    num = num * 10 + (ch as i64 - '0' as i64);
                    chars.next();
                } else {
                    break;
                }
            }

            // Skip whitespace
            while chars.peek() == Some(&' ') {
                chars.next();
            }
        }

        // Check for ':' (hours)
        if chars.peek() == Some(&':') {
            hours = num;
            chars.next();

            // Skip whitespace and parse minutes
            while chars.peek() == Some(&' ') {
                chars.next();
            }

            num = 0;
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() {
                    num = num * 10 + (ch as i64 - '0' as i64);
                    chars.next();
                } else {
                    break;
                }
            }
        }

        minutes = num;

        // Skip trailing whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }

        // If there are remaining characters, it's a syntax error
        if chars.peek().is_some() {
            self.output("Syntax error in time specification.  Format: <d>d<hh>:<mm>\n").await;
            return;
        }

        // Calculate new idle_since timestamp
        let total_minutes = days * 24 * 60 + hours * 60 + minutes;
        let new_idle_since = Timestamp::from_unix(now.unix() - total_minutes * 60);

        // Check permissions
        if new_idle_since < self.login_time().await && self.priv_level().await < 50 {
            self.output("Sorry, you can't be idle longer than you've been signed on.\n").await;
            return;
        }

        // Set the new idle time
        self.set_idle_since(new_idle_since).await;
        if self.idle_since().await < self.login_time().await {
            self.set_login_time(self.idle_since().await).await;
        }

        // Output results
        let new_idle = (now - self.idle_since().await) / 60;

        if current_idle > 0 && current_idle != new_idle {
            self.output("[You were idle for").await;
            self.print_time_long(current_idle as i32).await;
            self.output(".]\n").await;
        }

        if current_idle == new_idle {
            self.output("Your idle time is still").await;
            self.print_time_long(current_idle as i32).await;
            self.output(".\n").await;
        } else if new_idle > 0 {
            self.output("Your idle time has been set to").await;
            self.print_time_long(new_idle as i32).await;
            self.output(".\n").await;
        } else {
            self.output("Your idle time has been reset.\n").await;
            self.set_idle_since(now).await;
        }
    }

    pub async fn do_display(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /display <variable>[,<variable>...]\n").await;
            return;
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (var, rest) = getword(remaining, Some(','));
            remaining = rest;

            if var.starts_with('$') {
                if let Some(value) = self.user_vars.read().await.get(var) {
                    self.output(&format!("{var} = \"{value}\"\n")).await;
                } else {
                    self.output(&format!("Unknown user variable: \"{var}\"\n")).await;
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
                    self.output(&format!("Terminal height is currently set to {height}.\n")).await;
                }
            } else if let Some(_) = match_keyword(var, "idle", 4) {
                let now = Timestamp::new();
                self.output("Your idle time is").await;
                self.print_time_long(((now - *self.idle_since.read().await) / 60) as i32).await;
                self.output(".\n").await;
            } else if let Some(_) = match_keyword(var, "time_format", 11) {
                self.output("Your time format is ").await;
                if let Some(format) = self.get_sys_var("time_format").await {
                    match format.as_str() {
                        "verbose" => self.output("verbose.\n").await,
                        "both" => self.output("both verbose and terse.\n").await,
                        "terse" => self.output("terse.\n").await,
                        _ => self.output("unknown.\n").await,
                    }
                } else {
                    self.output("the default: ").await;
                    match DEFAULTS.read().await.get("time_format").map(|s| s.as_str()) {
                        Some("verbose") => self.output("verbose.\n").await,
                        Some("both") => self.output("both verbose and terse.\n").await,
                        Some("terse") => self.output("terse.\n").await,
                        _ => self.output("verbose.\n").await,
                    }
                }
            } else if let Some(_) = match_keyword(var, "uptime", 6) {
                let uptime = if let Some(system_up) = system_uptime() {
                    // TODO: Replace with actual server start uptime when available
                    system_up / 60
                } else {
                    let now = Timestamp::new();
                    // TODO: Replace with actual server start time when available
                    (now.unix() / 60) as i64
                };

                self.output("This server has been running for").await;
                self.print_time_long(uptime as i32).await;
                self.output(".\n").await;

                if let Some(system_up) = system_uptime() {
                    let system_minutes = system_up / 60;
                    self.output("(This machine has been running for").await;
                    self.print_time_long(system_minutes as i32).await;
                    self.output(".)\n").await;
                }
            } else if let Some(_) = match_keyword(var, "version", 7) {
                self.output(&format!("Phoenix server version: {VERSION}\n")).await;
            } else if let Some(_) = match_keyword(var, "width", 5) {
                if let Some(telnet) = &*self.telnet.read().await {
                    let width = telnet.set_width(0).await;
                    self.output(&format!("Terminal width is currently set to {width}.\n")).await;
                }
            } else {
                self.output(&format!("Unknown system variable: \"{var}\"\n")).await;
            }
        }
    }

    pub async fn do_also(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /also <sendlist>\n").await;
            return;
        }

        if let Some(last_msg) = &*self.last_message.read().await {
            let sendlist = Sendlist::new(&self, args, false, true, true).await;
            self.send_message(&sendlist, &last_msg.text).await;
        } else {
            self.output("You have no previous message to resend.\n").await;
        }
    }

    pub async fn do_oops(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /oops <sendlist> OR /oops text [<message>]\n").await;
            return;
        }

        if let Some(text_args) = match_keyword(args, "text", 4) {
            let text = text_args.trim();
            if !text.is_empty() {
                *self.oops_text.write().await = text.to_string();
                self.output(&format!("Your /oops text is now \"{text}\".\n")).await;
            } else {
                let oops_text = self.oops_text.read().await;
                self.output(&format!("Your /oops text is currently \"{oops_text}\".\n")).await;
            }
        } else {
            if let Some(last_msg) = &*self.last_message.read().await {
                let sendlist = Sendlist::new(&self, args, false, true, true).await;
                let text = last_msg.text.clone();
                let to = last_msg.to.clone();

                self.send_message(&to, &self.oops_text.read().await).await;
                self.send_message(&sendlist, &text).await;
                *self.last_sendlist.write().await = Some(sendlist.clone());
            } else {
                self.output("You have no previous message to resend.\n").await;
            }
        }
    }

    pub async fn do_help(&self, args: &str) {
        let args = args.trim();

        if match_keyword(args, "/who", 2).is_some()
            || match_keyword(args, "who", 3).is_some()
            || match_keyword(args, "/idle", 2).is_some()
            || match_keyword(args, "idle", 4).is_some()
        {
            self.output(
                "\
    The /who and /idle commands are used to list users on Phoenix.  Both /who\n\
    and /idle take identical arguments, but the output differs.  /who will give\n\
    more information, while /idle will give a more compact presentation.\n\n\
    Both /who and /idle will accept either categorical keywords or strings to\n\
    match against names and discussions; all matches found are listed.  If any\n\
    discussions are matched, all users in the discussions are listed.  The known\n\
    categorical keywords for /who and /idle are:\n\n\
       here   away   attached   active     idle     privileged   all\n\
       busy   gone   detached   inactive   unidle   guests       everyone\n\n\
    The categorical keywords match users in the given state.  The \"active\"\n\
    state is special, and defined as follows:\n\
       \"here\", attached, idle < 1 hour; or\n\
       \"here\", detached, idle < 10 minutes; or\n\
       \"away\", attached, idle < 10 minutes.\n\
    The keyword \"all\" is treated as \"active,attached\", while \"everyone\"\n\
    matches all users.  \"unidle\" matches users with idle < 10 minutes.  The\n\
    default if no arguments are given is to match \"everyone\" for now.  (When\n\
    more people are using the system, the default will change back to \"active\".)\n\
    Match strings and multiple categorical keywords can be piled together as\n\
    desired.  When only a single person is printed by /who, long blurbs are\n\
    printed in full.\n",
            )
            .await;
        } else if match_keyword(args, "/blurb", 3).is_some() || match_keyword(args, "blurb", 5).is_some() {
            self.output(
                "\
    The /blurb command allows you to set a descriptive \"blurb\".  It is usually\n\
    printed along with your name in most messages and notifications.  There is\n\
    no set limit to blurb length, but out of courtesy, try to keep it short.\n\
    Under 30 characters is a good size.  Long blurbs are normally truncated in\n\
    /who and /idle listings, so your entire blurb may not be seen at all times.\n\
    When only one person is printed by /who, long blurbs are printed in full.\n\n\
    Syntax: /blurb [blurb]\n\
            /blurb \"blurb\"\n\
            /blurb blurb\n\n\
    \"/blurb off\" turns off your blurb.  \"/blurb\" alone reports your blurb.\n\n\
    In many cases, it is preferable to use one of the away-state commands (/here,\n\
    /away, /busy, /gone) instead of /blurb.  All of the away-state commands will\n\
    take blurb arguments exactly like /blurb, but will set a meaningful status\n\
    at the same time, so their use is encouraged.  Also, every away-state command\n\
    may be abbreviated to a single letter, while /bl is the minimum abbreviation\n\
    for the /blurb command, since /busy abbreviates to /b.\n\n\
    See also: /here, /away, /busy, /gone.\n",
            )
            .await;
        } else if match_keyword(args, "/here", 2).is_some() || match_keyword(args, "here", 4).is_some() {
            self.output(
                "\
    The /here command accepts /blurb arguments to set the blurb, and then sets\n\
    your away status to \"here\".  Even if you are already \"here\", others will\n\
    still be notified that you are now \"here\".\n\n\
    Being \"here\" implies that you are willing to engage in new conversations,\n\
    and that you are reasonably likely to respond to messages quickly.\n\n\
    If you wish to actively talk to certain people but not anyone else in general,\n\
    then you should use /busy instead.\n\n\
    Since people sometimes forget to set a new away status when they leave, the\n\
    default /who target of \"active\" will only list \"here\" people if they are\n\
    under one hour idle if attached, or if they are under ten minutes idle if\n\
    detached.  (On the assumption they intend to return almost immediately.)\n\
    Overly-idle \"here\" people aren't normally listed, so their away state is\n\
    not changed due to idle time.\n\n\
    The /here command may be abbreviated to /h.\n\n\
    See also: /blurb, /away, /busy, /gone.\n",
            )
            .await;
        } else if match_keyword(args, "/away", 2).is_some() || match_keyword(args, "away", 4).is_some() {
            self.output(
                "\
    The /away command accepts /blurb arguments to set the blurb, and then sets\n\
    your away status to \"away\".  Even if you are already \"away\", others will\n\
    still be notified you are now \"away\".\n\n\
    Being \"away\" implies you are either gone for a brief period (maybe around\n\
    5-10 minutes), or you are around but likely to be inattentive.  It implies\n\
    you are not unwilling to engage in new conversations, but may well be slow\n\
    to respond.  \"away\" is a good state to use if you're reading Usenet news\n\
    in another window, watching TV across the room from the keyboard, or taking\n\
    a shower.  Your blurb should reflect your present activity, ideally.\n\n\
    If you wish to actively talk to certain people but not anyone else in general,\n\
    then you should use /busy instead.\n\n\
    Since people sometimes forget to set a new away status when they leave, the\n\
    default /who target of \"active\" will only list \"away\" people if they are\n\
    attached and under ten minutes idle.  Overly-idle \"away\" people aren't\n\
    normally listed, so their away state is not changed due to idle time.\n\n\
    The /away command may be abbreviated to /a.\n\n\
    See also: /blurb, /here, /busy, /gone.\n",
            )
            .await;
        } else if match_keyword(args, "/busy", 2).is_some() || match_keyword(args, "busy", 4).is_some() {
            self.output(
                "\
    The /busy command accepts /blurb arguments to set the blurb, and then sets\n\
    your away status to \"busy\".  Even if you are already \"busy\", others will\n\
    still be notified you are now \"busy\".\n\n\
    Being \"busy\" implies you are either engaged in conversation with others\n\
    on the system, or around but busy doing something else.  In either case,\n\
    \"busy\" implies you would not appreciate interruptions that aren't very\n\
    inportant, especially if they would require a reply.  Those whose messages\n\
    are welcome would already know so.  Don't bother a person who is \"busy\"\n\
    without having a reason to do so.  \"busy\" is a good state if you're in a\n\
    deep conversation with someone, or if you're washing dishes, for example.\n\
    Your blurb should reflect what you're busy with, ideally.\n\n\
    The default /who target of \"active\" will never list \"busy\" people on the\n\
    assumption that they do not wish to be unduly disturbed.  Idle time will not\n\
    cause the away state to change, but if you become unidle while \"busy\" and\n\
    at least ten minutes idle, you will get a warning message that you are still\n\
    listed as \"busy\", in case it no longer applies and you forgot about it.\n\n\
    The /busy command may be abbreviated to /b.\n\n\
    See also: /blurb, /here, /away, /gone.\n",
            )
            .await;
        } else if match_keyword(args, "/gone", 2).is_some() || match_keyword(args, "gone", 4).is_some() {
            self.output(
                "\
    The /gone command accepts /blurb arguments to set the blurb, and then sets\n\
    your away status to \"gone\".  Even if you are already \"gone\", others will\n\
    still be notified you are now \"gone\".\n\n\
    Being \"gone\" implies you are gone and should not be expected to respond to\n\
    messages at all until you return, regardless of whether you are attached or\n\
    detached.  \"gone\" implies you are not having any conversations at all, and\n\
    all messages received will be seen later, much like an answering machine.\n\
    \"gone\" is a good state to use if you're asleep, off to work or class, etc.\n\
    Your blurb should reflect where you went, ideally.  (e.g. \"/gone [-> work]\")\n\n\
    If you wish to actively talk to certain people but not anyone else in general,\n\
    then you should use /busy instead.\n\n\
    The default /who target of \"active\" will never list \"gone\" people on the\n\
    assumption that they are truly gone.  Idle time will not cause the away state\n\
    to change, but if you send a message while \"gone\", you will be warned,\n\
    for every message you send while \"gone\".\n\n\
    The /gone command may be abbreviated to /g.\n\n\
    See also: /blurb, /here, /away, /busy.\n",
            )
            .await;
        } else if match_keyword(args, "/help", 2).is_some() || match_keyword(args, "help", 4).is_some() {
            self.output(
                "\
    The /help command is used to request helpful information about commands or\n\
    concepts.  For example, for help on the /gone command, you can type either\n\
    \"/help gone\" or \"/help /gone\".  If the slash form for command help is\n\
    used, the command name may be abbreviated in the same way as the actual\n\
    command.  Since the minimum abbreviation for /gone is /g, \"/help /g\" is\n\
    sufficient, although \"/help g\" is not.\n",
            )
            .await;
        } else if match_keyword(args, "/send", 2).is_some() || match_keyword(args, "send", 4).is_some() {
            self.output(
                "\
    The /send command is used to redirect your \"default sendlist\".  Simply type\n\
    \"/send <sendlist>\" and <sendlist> becomes the new destination for any\n\
    message which does not contain an explicit sendlist, including recognized\n\
    smileys.  (See \"/help smileys\".)  \"/send off\" will turn off your default\n\
    sendlist completely.  \"/send\" alone will display your current default\n\
    sendlist without changing it.  /send may be abbreviated to /s.\n",
            )
            .await;
        } else if match_keyword(args, "/bye", 4).is_some() || match_keyword(args, "bye", 3).is_some() {
            self.output(
                "\
    The /bye command is used to leave Phoenix completely.  If you sign off, you\n\
    will be disconnected from the system and unable to receive messages at all.\n\
    You may wish to consider using the /detach command instead.\n",
            )
            .await;
        } else if match_keyword(args, "/what", 3).is_some() || match_keyword(args, "what", 4).is_some() {
            self.output(
                "\
    The /what command is used to list currently existing discussions.\n",
            )
            .await;
        } else if match_keyword(args, "/join", 2).is_some() || match_keyword(args, "join", 4).is_some() {
            self.output(
                "\
    The /join command is used to join one or more discussions.\n",
            )
            .await;
        } else if match_keyword(args, "/quit", 2).is_some() || match_keyword(args, "quit", 4).is_some() {
            self.output(
                "\
    The /quit command is used to quit one or more discussions.\n",
            )
            .await;
        } else if match_keyword(args, "/create", 3).is_some() || match_keyword(args, "create", 6).is_some() {
            self.output(
                "\
    The /create command is used to create a new discussion.\n",
            )
            .await;
        } else if match_keyword(args, "/destroy", 4).is_some() || match_keyword(args, "destroy", 7).is_some() {
            self.output(
                "\
    The /destroy command is used to destroy one or more discussions.\n",
            )
            .await;
        } else if match_keyword(args, "/permit", 4).is_some() || match_keyword(args, "permit", 6).is_some() {
            self.output(
                "\
    The /permit command is used to permit one or more members to a discussion.\n",
            )
            .await;
        } else if match_keyword(args, "/depermit", 4).is_some() || match_keyword(args, "depermit", 8).is_some() {
            self.output(
                "\
    The /depermit command is used to depermit one or more members from a\n\
    discussion.\n",
            )
            .await;
        } else if match_keyword(args, "/appoint", 4).is_some() || match_keyword(args, "appoint", 7).is_some() {
            self.output(
                "\
    The /appoint command is used to appoint one or more moderators to a\n\
    discussion.\n",
            )
            .await;
        } else if match_keyword(args, "/unappoint", 10).is_some() || match_keyword(args, "unappoint", 9).is_some() {
            self.output(
                "\
    The /unappoint command is used to unappoint one or more moderators from a\n\
    discussion.\n",
            )
            .await;
        } else if match_keyword(args, "/rename", 7).is_some() || match_keyword(args, "rename", 6).is_some() {
            self.output(
                "\
    The /rename command is used to change your name in the system.  There are\n\
    currently some bugs with this, so use of /rename is presently discouraged\n\
    until those bugs are fixed.\n",
            )
            .await;
        } else if match_keyword(args, "/clear", 3).is_some() || match_keyword(args, "clear", 5).is_some() {
            self.output(
                "\
    The /clear command simply clears the terminal screen.\n\n\
    Alternatively, type Escape then Control-L to clear the screen.\n",
            )
            .await;
        } else if match_keyword(args, "/unidle", 7).is_some() || match_keyword(args, "unidle", 6).is_some() {
            self.output(
                "\
    The /unidle command simply resets your idle time as if you sent a message.\n\n\
    Alternatively, send a line consisting of a single space only.  There is a\n\
    slight difference in that <space><return> is silent if idle under one minute,\n\
    while /unidle will report that the idle time was reset.  For both, if the\n\
    idle time was at least one minute, it is reported before being reset.\n\n\
    In general, when you become unidle, you will receive a report of the previous\n\
    idle time if it exceeded the normal threshold of ten minutes.\n",
            )
            .await;
        } else if match_keyword(args, "/detach", 4).is_some() || match_keyword(args, "detach", 6).is_some() {
            self.output(
                "\
    The /detach command is used to disconnect from Phoenix without signing off.\n\
    You can still receive messages while detached, to be reviewed later.  When\n\
    the /detach command is used, others are notified that you intentionally\n\
    detached.  If any other event causes you to become detached (e.g. network\n\
    failure), then others are notified that you accidentally detached.\n\n\
    To reattach to a detached session, simply sign back on with the same account\n\
    and name, and you will be automatically attached.  Currently, all pending\n\
    output will be output very quickly; local scrollback is highly recommended.\n\
    If you miss some of the detached output, do NOT press return, but disconnect\n\
    instead locally.  When you reattach, the same output will be reviewed again.\n\
    Output is only discarded when it has crossed the network (acknowledgements\n\
    are used) and the user has entered an input line.\n",
            )
            .await;
        } else if match_keyword(args, "/howmany", 3).is_some() || match_keyword(args, "howmany", 7).is_some() || match_keyword(args, "how", 3).is_some() {
            self.output(
                "\
    The /howmany command shows how many users are \"here\", \"away\", \"busy\"\n\
    and \"gone\", how many users are attached and detached, total number of\n\
    users signed on, and how many discussions are active.\n",
            )
            .await;
        } else if match_keyword(args, "/why", 4).is_some() || match_keyword(args, "why", 3).is_some() {
            self.output(
                "\
    The /why command is pretty self-explanatory. (try it!)\n",
            )
            .await;
        } else if match_keyword(args, "/date", 3).is_some() || match_keyword(args, "date", 4).is_some() {
            self.output(
                "\
    The /date command prints the current date and time like the date(1) command.\n",
            )
            .await;
        } else if match_keyword(args, "/signal", 3).is_some() || match_keyword(args, "signal", 6).is_some() {
            self.output(
                "\
    The /signal command is used to control whether or not to ring the terminal\n\
    bell when incoming messages arrive.  There are separate controls for public\n\
    and private messages.  The default is on for both.\n\n\
    Syntax: /signal [public|private] [on|off]\n",
            )
            .await;
        } else if match_keyword(args, "smileys", 6).is_some() {
            self.output(
                "\
    The following are recognized smileys:\n\n\
       :-)   :-(   :-P   ;-)   :_)   :_(   :)   :(   :P   ;)\n\n\
    When a message begins with one of these recognized smileys, either alone or\n\
    followed immediately by whitespace, the smiley as assumed to be part of the\n\
    message and sent to the default sendlist, instead of attempting to interpret\n\
    the smiley as an explicit sendlist.  This does not attempt to special-case\n\
    every type of smiley, but it does attempt to catch the common ones likely\n\
    to be typed reflexively.  Only smileys containing a semicolon or colon are\n\
    an issue here, since a smiley like \"8-)\" will already go to the default.\n\n\
    In general, any message can be forced to be interpreted as either explicit\n\
    or default sendlist sending by proper use of a space.  If a space leads the\n\
    input line, it guarantees sending to the default sendlist.  If a space is\n\
    immediately following a semicolon or colon in what would otherwise be one\n\
    of the recognized smileys, it guarantees the explicit sendlist interpretation.\n\
    In all cases, a single leading space in the message text will be removed\n\
    if it is present, to allow such control over sending without changing the\n\
    body of the message.\n\n\
    Since this technique makes a single space alone on a line effectively the\n\
    same as a blank line, this special case was used instead to reset idle time\n\
    without actually sending any message.  (See \"/help unidle\".)\n",
            )
            .await;
        } else if match_keyword(args, "/set", 4).is_some() || match_keyword(args, "set", 3).is_some() {
            let set_args = if args.starts_with('/') { args.strip_prefix("/set").unwrap_or("").trim() } else { args.strip_prefix("set").unwrap_or("").trim() };

            if match_keyword(set_args, "uptime", 6).is_some() {
                self.output(
                    "\
    Server uptime is a readonly system variable and cannot be set.\n",
                )
                .await;
            } else if match_keyword(set_args, "idle", 4).is_some() {
                self.output(
                    "\
    The \"/set idle\" command is used to set an arbitrary idle time.  Arguments\n\
    are a time specification in the format used by /who. (<d>d<hh>:<mm>)  You\n\
    may not make yourself idle longer than you've been signed on.  Use of this\n\
    command is actually discouraged.  In fact, it exists solely to discourage\n\
    people from using idle time as a reason not to be active on the system.\n\
    Idle time has no inherent value, and to hoard it is silly.  Yet this has\n\
    been done, if only because of the time needed to build up a high idle time.\n\
    This command is intended to take all the fun out of this game by eliminating\n\
    the challenge of accumulating a high idle time, to discourage such misuse.\n",
                )
                .await;
            } else if match_keyword(set_args, "time_format", 11).is_some() {
                self.output(
                    "\
    The \"/set time_format\" command will set the current format used to display\n\
    times in a verbose context.\n\n\
    Valid options are: terse, verbose, both, default.\n",
                )
                .await;
            } else if !set_args.is_empty() {
                self.output(&format!("No help available for \"/set {set_args}\".\n")).await;
            } else {
                self.output(
                    "\
    The /set command is used to set both system variables and user variables.\n\
    System variables are specified with predefined keywords, and user variables\n\
    must be prefixed with a dollar sign.  (e.g. \"idle\" is a system variable\n\
    with a predefined purpose, and \"$idle\" is a user variable with no such\n\
    predefined purpose.)\n\n\
    Known system variables:\n\n\
       uptime   idle     time_format\n",
                )
                .await;
            }
        } else if match_keyword(args, "/display", 2).is_some() || match_keyword(args, "display", 7).is_some() {
            let display_args =
                if args.starts_with('/') { args.strip_prefix("/display").unwrap_or("").trim() } else { args.strip_prefix("display").unwrap_or("").trim() };

            if match_keyword(display_args, "uptime", 6).is_some() {
                self.output(
                    "\
    The \"/display uptime\" command will display how long the server has been\n\
    running, and may also display how long the machine has been running.\n",
                )
                .await;
            } else if match_keyword(display_args, "idle", 4).is_some() {
                self.output(
                    "\
    The \"/display idle\" command will display your idle time.\n",
                )
                .await;
            } else if match_keyword(display_args, "time_format", 11).is_some() {
                self.output(
                    "\
    The \"/display time_format\" command will display the current format used to\n\
    display times in a verbose context.\n\n\
    Valid options are: terse, verbose, both, default.\n",
                )
                .await;
            } else if !display_args.is_empty() {
                self.output(&format!("No help available for \"/display {display_args}\".\n")).await;
            } else {
                self.output(
                    "\
    The /display command is used to display both system variables and user\n\
    variables.  System variables are specified with predefined keywords, and\n\
    user variables must be prefixed with a dollar sign.  (e.g. \"idle\" is a\n\
    system variable with a predefined purpose, and \"$idle\" is a user variable\n\
    with no such predefined purpose.)\n\n\
    Known system variables:\n\n\
       uptime   idle     time_format\n",
                )
                .await;
            }
        } else if match_keyword(args, "/also", 3).is_some() || match_keyword(args, "also", 4).is_some() {
            self.output(
                "\
    The /also command is used to send a copy of the last message to another\n\
    sendlist.\n",
            )
            .await;
        } else if match_keyword(args, "/oops", 3).is_some() || match_keyword(args, "oops", 4).is_some() {
            self.output(
                "\
    The /oops command is used to send an \"oops\" message to the (unintended)\n\
    recipient of the last message, and to resend the last message to another\n\
    sendlist.  The \"/oops text <message>\" form can be used to change the\n\
    text of the \"oops\" message.\n",
            )
            .await;
        } else if !args.is_empty() {
            self.output(&format!("No help available for \"{args}\".\n")).await;
        } else {
            self.output(
                "\
    Known commands:\n\n\
       /who     /blurb    /create    /permit     /clear     /howmany\n\
       /what    /here     /destroy   /depermit   /unidle    /detach\n\
       /why     /away     /join      /appoint    /date      /bye\n\
       /idle    /busy     /quit      /unappoint  /set\n\
       /help    /gone     /send      /rename     /signal\n\n\
    Type \"/help <command>\" for more information about a particular command.\n",
            )
            .await;
        }
    }

    pub async fn do_reset(&self) {
        self.reset_idle(1).await;
    }

    pub async fn do_message(&self, line: &str) {
        let (msg_start, sendlist_str, last_explicit, is_explicit) = message_start(line);
        let msg_start = msg_start.trim();

        if is_explicit {
            *self.last_explicit.write().await = last_explicit;
        }

        let sendlist = if sendlist_str.is_empty() {
            if let Some(last) = &*self.last_sendlist.read().await {
                last.clone()
            } else {
                self.output("\x07\x07You have no previous sendlist. (message not sent)\n").await;
                return;
            }
        } else if sendlist_str.eq_ignore_ascii_case("default") {
            if let Some(default) = &*self.default_sendlist.read().await {
                default.clone()
            } else {
                self.output("\x07\x07You have no default sendlist. (message not sent)\n").await;
                return;
            }
        } else {
            Sendlist::new(&self, &sendlist_str, false, true, true).await
        };

        *self.last_sendlist.write().await = Some(sendlist.clone());

        if msg_start.is_empty() {
            let sendlist_typed = &sendlist.typed;
            if sendlist_str == "default" {
                self.output("\x07\x07There is no message after \"default\". (message not sent)\n").await;
            } else if is_explicit {
                self.output(&format!("\x07\x07There is no message after \"{sendlist_typed}:\". (message not sent)\n")).await;
            } else {
                self.output(&format!("\x07\x07There is no message after \"{sendlist_typed};\". (message not sent)\n")).await;
            }
            return;
        }

        self.send_message(&sendlist, msg_start).await;
    }

    pub async fn send_message(&self, sendlist: &Sendlist, text: &str) {
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
        let count = sendlist.expand(&mut who, Some(self.clone())).await;

        let output_type = if count > 1 || !sendlist.discussions.is_empty() { OutputType::PublicMessage } else { OutputType::PrivateMessage };

        let msg = Arc::new(Message::new(output_type, self.name(), sendlist.clone(), text));
        *self.last_message.write().await = Some(msg.clone());

        for session in &who {
            session.enqueue(msg.clone()).await;
        }

        for disc in &sendlist.discussions {
            *disc.set_idle_since(Timestamp::new()).await;
        }

        self.output("(message sent to ").await;
        self.print_sendlist(sendlist).await;
        self.output(")");

        if idle >= 30 {
            self.output(&format!(" [idle {idle}]")).await;
        }
        self.output("\n").await;
    }

    pub async fn get_who_set(&self, args: &str) -> (OrderedSet<Session>, String, String) {
        let mut who = OrderedSet::new();
        let mut errors = String::new();
        let mut msg = String::new();

        if args.is_empty() {
            // Show all sessions
            for session in &SESSIONS {
                who.insert(session.value().clone());
            }

            let count = who.len();
            let s = if count == 1 { "" } else { "s" };
            msg = format!("\n{count} user{s} signed on.\n");
        } else {
            let sendlist = Sendlist::new(&self, args, true, true, true).await;

            let total = sendlist.expand(&mut who, None).await;

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

    pub async fn session_matches(&self, name: &str, matches: &OrderedSet<Session>) {
        if !matches.is_empty() {
            let count = matches.len();

            for (i, session) in matches.iter().enumerate() {
                match i {
                    0 if count == 1 => self.output(&format!("\"{name}\" matches one name: ")).await,
                    0 => self.output(&format!("\"{name}\" matches {count} names: ")).await,
                    _ if i == count - 1 => self.output(" and ").await,
                    _ => self.output(", ").await,
                };

                self.output(&session.name());
            }

            self.output(".\n").await;
        } else {
            self.output(&format!("No names matched \"{name}\".\n")).await;
        }
    }

    pub async fn discussion_matches(&self, name: &str, matches: &OrderedSet<Discussion>) {
        if !matches.is_empty() {
            let count = matches.len();

            for (i, disc) in matches.iter().enumerate() {
                match i {
                    0 if count == 1 => self.output(&format!("\"{name}\" matches one discussion: ")).await,
                    0 => self.output(&format!("\"{name}\" matches {count} discussions: ")).await,
                    _ if i == count - 1 => self.output(" and ").await,
                    _ => self.output(", ").await,
                };

                self.output(disc.name().await);
            }

            self.output(".\n").await;
        } else {
            self.output(&format!("No discussions matched \"{name}\".\n")).await;
        }
    }
}

impl ConnectionSession for Session {
    fn name_opt(&self) -> Option<&Name> {
        Some(self.name())
    }

    async fn acknowledge_output(&self) {
        self.pending.read().await.acknowledge().await;
    }

    async fn last_explicit(&self) -> Text {
        self.read().await.last_explicit.clone()
    }

    async fn reply_sendlist(&self) -> Text {
        self.read().await.reply_sendlist.clone()
    }

    async fn output_next(&self, telnet: &Telnet) -> bool {
        self.pending.read().await.send_next(telnet).await
    }

    async fn output(&mut self, text: impl AsRef<str>) {
        self.output(text.as_ref()).await;
    }

    async fn handle_input(&mut self, line: String) {
        self.handle_input(line).await;
    }

    async fn print_message(&self, telnet: &mut Telnet) {
        self.print_message(telnet).await;
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

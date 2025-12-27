// -*- Rust -*-
//
// Phoenix CMC library: session module
//
// Copyright 1992-2025 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::atomic::{
    AtomicAwayState, AtomicHashMap, AtomicLoginState, AtomicMessageOption, AtomicName, AtomicSendlistOption, AtomicSessionType, AtomicTelnetOption, AtomicText,
    AtomicTextOption, AtomicUserOption, SessionTypeBorrow,
};
use crate::constants::*;
use crate::discussion::Discussion;
use crate::name::Name;
use crate::output::*;
use crate::sendlist::{Sendlist, message_start};
use crate::server::Server;
use crate::telnet::{BELL_STR, TELNET_ENABLED, Telnet};
use crate::text::Text;
use crate::timestamp::{Timestamp, system_uptime};
use crate::user::{User, UserManager, verify_password};
use crate::{VERSION, getword, match_keyword, match_name};
use arc_swap::{ArcSwap, ArcSwapOption};
use async_backtrace::framed;
use im::OrdSet;
use log::info;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

pub const LOGIN_TIMEOUT: Duration  = Duration::from_secs(300);
pub const MAX_LOGIN_ATTEMPTS: i32  = 3;
pub const REPORT_IDLE_DEFAULT: i64 = 10;

pub static SESSIONS: LazyLock<AtomicHashMap<usize, Session>> = LazyLock::new(AtomicHashMap::default);
pub static DISCUSSIONS: LazyLock<AtomicHashMap<Text, Discussion>> = LazyLock::new(AtomicHashMap::default);
pub static SESSION_COUNTER: AtomicUsize = AtomicUsize::new(1);
pub static DEFAULTS: LazyLock<AtomicHashMap<Text, Text>> =
    LazyLock::new(|| AtomicHashMap::from(im::HashMap::from(&[(Text::from("time_format"), Text::from("verbose"))][..])));
pub static USER_MANAGER: LazyLock<UserManager> = LazyLock::new(UserManager::new);

/// Session handle.
#[derive(Debug, Clone)]
pub struct Session(pub Arc<SessionInner>);

#[derive(Debug)]
pub struct SessionInner {
    // Immutable fields
    pub id: usize,
    pub server: Server,

    // Telnet connection, if any.
    pub telnet: AtomicTelnetOption,

    // Input buffering.
    pub lines: Mutex<VecDeque<Text>>,

    // Output buffering.
    pub output_buffer: Mutex<String>,
    pub pending: Mutex<OutputStream>,

    // Login and idle timestamps.
    pub login_time: ArcSwap<Timestamp>,
    pub idle_since: ArcSwap<Timestamp>,

    // Session type enum with type-specific fields.
    pub session_type: AtomicSessionType,
}

#[derive(Debug)]
pub enum SessionType {
    PreLogin {
        user_entered: AtomicTextOption,
        name_entered: AtomicTextOption,
        login_state: AtomicLoginState,
        login_timeout: ArcSwapOption<AbortHandle>,
        attempts: AtomicI32,
    },
    LoggedIn {
        // User-visible session name.
        name: AtomicName,

        // User object, if any.
        user: AtomicUserOption,

        // User preferences and variables.
        user_vars: ArcSwap<HashMap<Text, Text>>,
        sys_vars: ArcSwap<HashMap<Text, Text>>,
        signal_public: AtomicBool,
        signal_private: AtomicBool,

        // Session state.
        away: AtomicAwayState,
        priv_level: AtomicI32,
        closing: AtomicBool,

        // Message handling.
        last_message: AtomicMessageOption,
        default_sendlist: AtomicSendlistOption,
        last_sendlist: AtomicSendlistOption,
        last_explicit: AtomicText,
        reply_sendlist: AtomicText,
        oops_text: AtomicText,
    },
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoginState {
    PreLogin = 0,
    AwaitingLogin = 1,
    AwaitingPassword = 2,
    AwaitingName = 3,
    AwaitingBlurb = 4,
    AwaitingTransferConfirmation = 5,
    LoggedIn = 6,
}

impl Default for LoginState {
    #[inline]
    fn default() -> Self {
        LoginState::PreLogin
    }
}

impl From<LoginState> for u8 {
    #[inline]
    fn from(state: LoginState) -> u8 {
        state as u8
    }
}

impl From<u8> for LoginState {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0 => LoginState::PreLogin,
            1 => LoginState::AwaitingLogin,
            2 => LoginState::AwaitingPassword,
            3 => LoginState::AwaitingName,
            4 => LoginState::AwaitingBlurb,
            5 => LoginState::AwaitingTransferConfirmation,
            6 => LoginState::LoggedIn,
            _ => LoginState::default(),
        }
    }
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
    #[inline]
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

impl Session {
    /// Create a new session in PreLogin state.
    pub fn new(server: Server, telnet: Option<Telnet>) -> Self {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        println!("=== DEBUG: Session::new() creating session with ID: {id} ===");
        let now = Timestamp::new();
        let inner = SessionInner {
            id,
            server,
            telnet: AtomicTelnetOption::new(telnet.clone()),
            lines: Mutex::new(VecDeque::new()),
            output_buffer: Mutex::new(String::new()),
            pending: Mutex::new(OutputStream::new()),
            login_time: ArcSwap::new(Arc::new(now.clone())),
            idle_since: ArcSwap::new(Arc::new(now)),
            session_type: AtomicSessionType::new(SessionType::PreLogin {
                user_entered: AtomicTextOption::new(None),
                name_entered: AtomicTextOption::new(None),
                login_state: AtomicLoginState::new(LoginState::PreLogin),
                login_timeout: ArcSwapOption::new(None),
                attempts: AtomicI32::new(0),
            }),
        };

        let session = Self(Arc::new(inner));
        println!("=== DEBUG: Session::new() created session wrapper ===");

        // Set telnet session.
        if let Some(telnet) = telnet {
            println!("=== DEBUG: Setting telnet session reference ===");
            telnet.set_session(session.clone());
        } else {
            println!("=== DEBUG: No telnet connection to link ===");
        }

        SESSIONS.insert(session.id(), session.clone());

        println!("=== DEBUG: Session::new() completed for ID: {id} ===");
        session
    }

    /// Convert session to LoggedIn state.
    pub fn logged_in(&self, name: Name, user: Option<User>) {
        let now = Timestamp::new();
        let priv_level = user.as_ref().and_then(|user| Some(user.priv_level())).unwrap_or(0);
        self.0.login_time.store(Arc::new(now.clone()));
        self.0.idle_since.store(Arc::new(now));

        // Cancel login timeout and clear close-on-EOF flag now that login sequence is finished
        self.cancel_login_timeout();
        if let Some(telnet) = self.telnet() {
            telnet.set_close_on_eof(false);
        }

        self.0.session_type.set(SessionType::LoggedIn {
            name: AtomicName::new(name),
            user: AtomicUserOption::new(user),
            user_vars: ArcSwap::new(Arc::new(HashMap::new())),
            sys_vars: ArcSwap::new(Arc::new(HashMap::new())),
            signal_public: AtomicBool::new(true),
            signal_private: AtomicBool::new(true),
            away: AtomicAwayState::default(),
            priv_level: AtomicI32::new(priv_level),
            closing: AtomicBool::new(false),
            last_message: AtomicMessageOption::new(None),
            default_sendlist: AtomicSendlistOption::new(None),
            last_sendlist: AtomicSendlistOption::new(None),
            last_explicit: AtomicText::new(Text::default()),
            reply_sendlist: AtomicText::new(Text::default()),
            oops_text: AtomicText::new(Text::from("Oops!  Sorry, that last message was intended for someone else...")),
        });
    }

    /// Get the session ID.
    pub fn id(&self) -> usize {
        self.0.id
    }

    /// Get the `Server` object.
    pub fn server(&self) -> Server {
        self.0.server.clone()
    }

    /// Get the `Telnet` object, if any.
    pub fn telnet(&self) -> Option<Telnet> {
        self.0.telnet.snapshot()
    }

    /// Set the `Telnet` object, if any.
    pub fn set_telnet(&self, value: Option<Telnet>) {
        self.0.telnet.set(value);
    }

    /// Return a single-character detached indicator.
    pub fn detached_indicator(&self) -> &str {
        if self.telnet().is_some() { " " } else { "~" }
    }

    /// Get the input lines queue.
    #[framed]
    pub async fn lines(&self) -> tokio::sync::MutexGuard<'_, VecDeque<Text>> {
        self.0.lines.lock().await
    }

    /// Get the output buffer.
    #[framed]
    pub async fn output_buffer(&self) -> tokio::sync::MutexGuard<'_, String> {
        self.0.output_buffer.lock().await
    }

    /// Add text to output buffer.
    #[framed]
    pub async fn output(&self, text: impl AsRef<str>) {
        self.output_buffer().await.push_str(text.as_ref());
    }

    /// Enqueue output buffer as a new `TextOutput`.
    #[framed]
    pub async fn enqueue_output(&self) -> tokio::io::Result<()> {
        let mut output_buffer = self.output_buffer().await;
        if !output_buffer.is_empty() {
            let text_output = TextOutput::new(output_buffer.clone());
            self.pending().await.enqueue(self.telnet().as_ref(), text_output).await?;
            output_buffer.clear();
        }

        Ok(())
    }

    /// Get the `OutputStream`.
    #[framed]
    pub async fn pending(&self) -> tokio::sync::MutexGuard<'_, OutputStream> {
        self.0.pending.lock().await
    }

    /// Send the next output in the output stream.
    #[framed]
    pub async fn output_next(&self, telnet: &Telnet) -> tokio::io::Result<bool> {
        self.pending().await.send_next(telnet).await
    }

    /// Acknowledge output.
    #[framed]
    pub async fn acknowledge_output(&self) {
        self.pending().await.acknowledge().await;
    }

    /// Get the login time.
    pub fn login_time(&self) -> Arc<Timestamp> {
        self.0.login_time.load_full()
    }

    /// Set the login time.
    pub fn set_login_time(&self, value: Timestamp) {
        self.0.login_time.store(Arc::new(value));
    }

    /// Get the idle-since timestamp.
    pub fn idle_since(&self) -> Arc<Timestamp> {
        self.0.idle_since.load_full()
    }

    /// Set the idle-since timestamp.
    pub fn set_idle_since(&self, value: Timestamp) {
        self.0.idle_since.store(Arc::new(value));
    }

    /// Get the session type.
    pub fn session_type(&self) -> SessionTypeBorrow {
        self.0.session_type.borrow()
    }

    /// Get the login entered.
    pub fn user_entered(&self) -> Option<Text> {
        println!("=== DEBUG: user_entered() called ===");
        let user_entered = match self.session_type().as_ref() {
            SessionType::PreLogin { user_entered, .. } => user_entered.snapshot(),
            SessionType::LoggedIn { .. } => None,
        };
        println!("=== DEBUG: user_entered={user_entered:?} ===");
        user_entered
    }

    /// Set the name entered.
    pub fn set_user_entered(&self, value: Option<Text>) {
        println!("=== DEBUG: set_user_entered({value:?}) called ===");
        match self.session_type().as_ref() {
            SessionType::PreLogin { user_entered, .. } => user_entered.set(value),
            SessionType::LoggedIn { .. } => (),
        };
    }

    /// Get the name entered.
    pub fn name_entered(&self) -> Option<Text> {
        println!("=== DEBUG: name_entered() called ===");
        let name_entered = match self.session_type().as_ref() {
            SessionType::PreLogin { name_entered, .. } => name_entered.snapshot(),
            SessionType::LoggedIn { .. } => None,
        };
        println!("=== DEBUG: name_entered={name_entered:?} ===");
        name_entered
    }

    /// Set the name entered.
    pub fn set_name_entered(&self, value: Option<Text>) {
        println!("=== DEBUG: set_name_entered({value:?}) called ===");
        match self.session_type().as_ref() {
            SessionType::PreLogin { name_entered, .. } => name_entered.set(value),
            SessionType::LoggedIn { .. } => (),
        };
    }

    /// Get the `LoginState`.
    pub fn login_state(&self) -> LoginState {
        match self.session_type().as_ref() {
            SessionType::PreLogin { login_state, .. } => login_state.get(),
            SessionType::LoggedIn { .. } => LoginState::LoggedIn,
        }
    }

    /// Set the `LoginState`.
    pub fn set_login_state(&self, value: LoginState) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { login_state, .. } => login_state.set(value),
            SessionType::LoggedIn { .. } => (),
        };
    }

    /// Switch login state and prompt.
    #[framed]
    pub async fn switch_login_state(&self, state: LoginState, prompt: Option<&str>) -> tokio::io::Result<()> {
        self.enqueue_output().await?;

        self.set_login_state(state);
        if let Some(prompt) = prompt {
            if let Some(telnet) = self.telnet() {
                telnet.output(prompt).await;
                telnet.flush_output().await.ok();
            }
        }

        // If there are pending input lines, process one to continue the flow
        let next_line = {
            let mut lines = self.lines().await;
            lines.pop_front()
        };
        if let Some(next_line) = next_line {
            Box::pin(self.handle_input(next_line)).await?;
        }

        Ok(())
    }

    /// Get the login timeout Tokio task `AbortHandle`, if any.
    pub fn login_timeout(&self) -> Option<AbortHandle> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { login_timeout, .. } => login_timeout.load_full().map(|arc| (*arc).clone()),
            SessionType::LoggedIn { .. } => None,
        }
    }

    /// Set the login timeout Tokio task `AbortHandle`, if any.
    pub fn set_login_timeout(&self, value: Option<AbortHandle>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { login_timeout, .. } => login_timeout.store(value.map(Arc::new)),
            SessionType::LoggedIn { .. } => (),
        };
    }

    /// Cancel the login timeout.
    pub fn cancel_login_timeout(&self) {
        if let Some(handle) = self.login_timeout() {
            handle.abort();
        }
        self.set_login_timeout(None);
    }

    /// Reset the login timeout to the full duration.
    pub fn reset_login_timeout(&self) {
        // Cancel existing timeout if any
        if let Some(handle) = self.login_timeout() {
            handle.abort();
        }

        // Start new timeout
        let session = self.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(LOGIN_TIMEOUT).await;
            if let Some(telnet) = session.telnet() {
                let _ = telnet.output("Login timeout.\n").await;
                let _ = session.close(true).await;
            }
        })
        .abort_handle();

        self.set_login_timeout(Some(handle));
    }

    /// Get the login attempts count.
    pub fn attempts(&self) -> i32 {
        match self.session_type().as_ref() {
            SessionType::PreLogin { attempts, .. } => attempts.load(Ordering::Relaxed),
            SessionType::LoggedIn { .. } => -1,
        }
    }

    /// Set the login attempts count.
    pub fn set_attempts(&self, value: i32) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { attempts, .. } => attempts.store(value, Ordering::Relaxed),
            SessionType::LoggedIn { .. } => (),
        };
    }

    /// Increment the login attempts count.
    pub fn increment_attempts(&self) -> i32 {
        match self.session_type().as_ref() {
            SessionType::PreLogin { attempts, .. } => attempts.fetch_add(1, Ordering::Relaxed) + 1,
            SessionType::LoggedIn { .. } => 0,
        }
    }

    /// Get the `Name` object.
    pub fn name(&self) -> Name {
        println!("=== DEBUG: name() called ===");
        let name = match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => Name::new("", None),
            SessionType::LoggedIn { name, .. } => name.snapshot(),
        };
        println!("=== DEBUG: name={name:?} ===");
        name
    }

    /// Get only the name from the `Name` object.
    pub fn name_only(&self) -> Text {
        println!("=== DEBUG: name_only() called ===");
        let name = match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => Text::new("<LoginSession>"),
            SessionType::LoggedIn { name, .. } => name.borrow().name().clone(),
        };
        println!("=== DEBUG: name={name:?} ===");
        name
    }

    /// Return formatted name and username.
    pub fn name_user(&self) -> Text {
        println!("=== DEBUG: name_user() called ===");
        // TODO: This should be cached instead, like `name_blurb` in `Name`.
        let name = self.name_only();
        let user = self.user();
        let name_user = if let Some(user) = user {
            let username = user.username();
            Text::from(format!("{name} ({username})"))
        } else {
            name.clone()
        };
        println!("=== DEBUG: name_user={name_user:?} ===");
        name_user
    }

    /// Set the name.
    pub fn set_name(&self, value: Text) {
        println!("=== DEBUG: set_name({value:?}) called ===");
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { name, .. } => {
                let blurb = self.blurb();
                // name.set(Name::new(value, blurb));
                let new_name = Name::new(value, blurb);
                println!("=== DEBUG: new_name={new_name:?} ===");
                name.set(new_name);
            }
        };
    }

    /// Check if a blurb is set.
    pub fn has_blurb(&self) -> bool {
        println!("=== DEBUG: has_blurb() called ===");
        let ret = match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => false,
            SessionType::LoggedIn { name, .. } => name.borrow().has_blurb(),
        };
        println!("=== DEBUG: returning {ret:?} ===");
        ret
    }

    /// Get the blurb, if any.
    pub fn blurb(&self) -> Option<Text> {
        println!("=== DEBUG: blurb() called ===");
        let blurb = match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { name, .. } => name.borrow().blurb().cloned(),
        };
        println!("=== DEBUG: blurb={blurb:?} ===");
        blurb
    }

    /// Set the blurb.
    pub fn set_blurb(&self, value: Option<Text>) {
        println!("=== DEBUG: set_blurb({value:?}) called ===");
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            // SessionType::LoggedIn { name, .. } => name.set(Name::new(self.name_only(), value)),
            SessionType::LoggedIn { name, .. } => {
                let new_name = self.name_only();
                println!("=== DEBUG: new_name={new_name:?} ===");
                let new_name = Name::new(new_name, value);
                println!("=== DEBUG: new_name={new_name:?} ===");
                name.set(new_name);
            }
        }
    }

    /// Remove the blurb.
    pub fn remove_blurb(&self) {
        println!("=== DEBUG: remove_blurb() called ===");
        if self.has_blurb() {
            self.set_blurb(None);
        }
    }

    /// Set both name and blurb atomically.
    pub fn set_name_and_blurb(&self, new_name: Text, blurb: Option<Text>) {
        println!("=== DEBUG: set_name_and_blurb({new_name:?}, {blurb:?}) called ===");
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            // SessionType::LoggedIn { name, .. } => name.set(Name::new(new_name, blurb)),
            SessionType::LoggedIn { name, .. } => {
                let new_name = Name::new(new_name, blurb);
                println!("=== DEBUG: new_name={new_name:?} ===");
                name.set(new_name);
            }
        }
    }

    /// Get the `User` object, if any.
    pub fn user(&self) -> Option<User> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { user, .. } => user.snapshot(),
        }
    }

    /// Set the `User` object, if any.
    pub fn set_user(&self, value: Option<User>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { user, .. } => user.set(value),
        }
    }

    /// Get a user variable.
    pub fn get_user_var(&self, key: impl AsRef<str>) -> Option<Text> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { user_vars, .. } => {
                let vars = user_vars.load();
                vars.get(key.as_ref()).cloned()
            }
        }
    }

    /// Set a user variable.
    pub fn set_user_var(&self, key: impl Into<Text>, value: impl Into<Text>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { user_vars, .. } => {
                let vars = user_vars.load();
                let mut new_vars = (**vars).clone();
                new_vars.insert(key.into(), value.into());
                user_vars.store(Arc::new(new_vars));
            }
        };
    }

    /// Remove a user variable.
    pub fn remove_user_var(&self, key: impl AsRef<str>) -> Option<Text> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { user_vars, .. } => {
                let vars = user_vars.load();
                let mut new_vars = (**vars).clone();
                let result = new_vars.remove(key.as_ref());
                user_vars.store(Arc::new(new_vars));

                result
            }
        }
    }

    /// Clear all user variables.
    pub fn clear_user_vars(&self) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { user_vars, .. } => user_vars.store(Arc::new(HashMap::new())),
        };
    }

    /// Get a system variable.
    pub fn get_sys_var(&self, key: impl AsRef<str>) -> Option<Text> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { sys_vars, .. } => {
                let vars = sys_vars.load();
                vars.get(key.as_ref()).cloned()
            }
        }
    }

    /// Set a system variable.
    pub fn set_sys_var(&self, key: impl Into<Text>, value: impl Into<Text>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { sys_vars, .. } => {
                let vars = sys_vars.load();
                let mut new_vars = (**vars).clone();
                new_vars.insert(key.into(), value.into());
                sys_vars.store(Arc::new(new_vars));
            }
        };
    }

    /// Remove a system variable.
    pub fn remove_sys_var(&self, key: impl AsRef<str>) -> Option<Text> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { sys_vars, .. } => {
                let vars = sys_vars.load();
                let mut new_vars = (**vars).clone();
                let result = new_vars.remove(key.as_ref());
                sys_vars.store(Arc::new(new_vars));

                result
            }
        }
    }

    /// Clear all system variables.
    pub fn clear_sys_vars(&self) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { sys_vars, .. } => sys_vars.store(Arc::new(HashMap::new())),
        };
    }

    /// Get the public signal flag.
    pub fn signal_public(&self) -> bool {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => true,
            SessionType::LoggedIn { signal_public, .. } => signal_public.load(Ordering::Relaxed),
        }
    }

    /// Set the public signal flag.
    pub fn set_signal_public(&self, value: bool) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { signal_public, .. } => signal_public.store(value, Ordering::Relaxed),
        };
    }

    /// Get the private signal flag.
    pub fn signal_private(&self) -> bool {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => true,
            SessionType::LoggedIn { signal_private, .. } => signal_private.load(Ordering::Relaxed),
        }
    }

    /// Set the private signal flag.
    pub fn set_signal_private(&self, value: bool) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { signal_private, .. } => signal_private.store(value, Ordering::Relaxed),
        };
    }

    /// Get the signed-on flag.
    pub fn signed_on(&self) -> bool {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => false,
            SessionType::LoggedIn { .. } => true,
        }
    }

    /// Get the away state.
    pub fn away(&self) -> AwayState {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => AwayState::default(),
            SessionType::LoggedIn { away, .. } => away.get(),
        }
    }

    /// Set the away state.
    pub fn set_away(&self, value: AwayState) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { away, .. } => away.set(value),
        }
    }

    /// Get the privilege level.
    pub fn priv_level(&self) -> i32 {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => -1,
            SessionType::LoggedIn { priv_level, .. } => priv_level.load(Ordering::Relaxed),
        }
    }

    /// Set the privilege level.
    pub fn set_priv_level(&self, value: i32) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { priv_level, .. } => priv_level.store(value, Ordering::Relaxed),
        };
    }

    /// Check if the session has privileged access.
    pub fn privileged(&self) -> bool {
        self.priv_level() >= 50
    }

    /// Get the closing flag.
    pub fn closing(&self) -> bool {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => false,
            SessionType::LoggedIn { closing, .. } => closing.load(Ordering::Relaxed),
        }
    }

    /// Set the closing flag.
    pub fn set_closing(&self, value: bool) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { closing, .. } => closing.store(value, Ordering::Relaxed),
        };
    }

    /// Get the last message.
    pub fn last_message(&self) -> Option<Message> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { last_message, .. } => last_message.snapshot(),
        }
    }

    /// Set the last message.
    pub fn set_last_message(&self, value: Option<Message>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { last_message, .. } => last_message.set(value),
        }
    }

    /// Get the default sendlist.
    pub fn default_sendlist(&self) -> Option<Sendlist> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { default_sendlist, .. } => default_sendlist.snapshot(),
        }
    }

    /// Set the default sendlist.
    pub fn set_default_sendlist(&self, value: Option<Sendlist>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { default_sendlist, .. } => default_sendlist.set(value),
        }
    }

    /// Get the last sendlist.
    pub fn last_sendlist(&self) -> Option<Sendlist> {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => None,
            SessionType::LoggedIn { last_sendlist, .. } => last_sendlist.snapshot(),
        }
    }

    /// Set the last sendlist.
    pub fn set_last_sendlist(&self, value: Option<Sendlist>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { last_sendlist, .. } => last_sendlist.set(value),
        }
    }

    /// Get the last explicit sendlist.
    pub fn last_explicit(&self) -> Text {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => Text::default(),
            SessionType::LoggedIn { last_explicit, .. } => last_explicit.snapshot(),
        }
    }

    /// Set the last explicit sendlist.
    pub fn set_last_explicit(&self, value: impl Into<Text>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { last_explicit, .. } => last_explicit.set(value.into()),
        }
    }

    /// Get the reply sendlist.
    pub fn reply_sendlist(&self) -> Text {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => Text::default(),
            SessionType::LoggedIn { reply_sendlist, .. } => reply_sendlist.snapshot(),
        }
    }

    /// Set the reply sendlist.
    pub fn set_reply_sendlist(&self, sendlist: impl Into<Text>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { reply_sendlist, .. } => {
                let sendlist: Text = sendlist.into();

                // Quote if necessary
                let sendlist = if sendlist.chars().any(|c| c == ' ' || c == ',' || c == ':' || c == ';' || c == '_') {
                    Text::from(format!("\"{sendlist}\""))
                } else {
                    sendlist
                };

                reply_sendlist.set(sendlist);
            }
        };
    }

    /// Get the oops text.
    pub fn oops_text(&self) -> Text {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => Text::default(),
            SessionType::LoggedIn { oops_text, .. } => oops_text.snapshot(),
        }
    }

    /// Set the oops text.
    pub fn set_oops_text(&self, value: impl Into<Text>) {
        match self.session_type().as_ref() {
            SessionType::PreLogin { .. } => (),
            SessionType::LoggedIn { oops_text, .. } => oops_text.set(value.into()),
        }
    }

    /// Handle a line of input.
    #[framed]
    pub async fn handle_input(&self, line: Text) -> tokio::io::Result<()> {
        println!("=== DEBUG: handle_input() starting ===");
        self.pending().await.dequeue().await;

        // Reset login timeout if still in pre-login state
        if !self.signed_on() {
            self.reset_login_timeout();
        }

        match self.login_state() {
            LoginState::PreLogin => println!("=== DEBUG: calling save_input_line({line:?}) ==="),
            LoginState::AwaitingLogin => println!("=== DEBUG: calling handle_login_input({line:?}) ==="),
            LoginState::AwaitingPassword => println!("=== DEBUG: calling handle_password_input({line:?}) ==="),
            LoginState::AwaitingName => println!("=== DEBUG: calling handle_name_input({line:?}) ==="),
            LoginState::AwaitingBlurb => println!("=== DEBUG: calling handle_blurb_input({line:?}) ==="),
            LoginState::AwaitingTransferConfirmation => println!("=== DEBUG: calling handle_transfer_input({line:?}) ==="),
            LoginState::LoggedIn => println!("=== DEBUG: calling process_input({line:?}) ==="),
        }

        match self.login_state() {
            LoginState::PreLogin => self.save_input_line(line).await?,
            LoginState::AwaitingLogin => self.handle_login_input(line).await?,
            LoginState::AwaitingPassword => self.handle_password_input(line).await?,
            LoginState::AwaitingName => self.handle_name_input(line).await?,
            LoginState::AwaitingBlurb => self.handle_blurb_input(line).await?,
            LoginState::AwaitingTransferConfirmation => self.handle_transfer_input(line).await?,
            LoginState::LoggedIn => self.process_input(line).await?,
        }

        println!("=== DEBUG: handle_input() calling enqueue_output() ===");
        self.enqueue_output().await?;

        println!("=== DEBUG: handle_input() finished ===");
        Ok(())
    }

    #[framed]
    pub async fn handle_login_input(&self, line: Text) -> tokio::io::Result<()> {
        let line = line.trim();
        if let Some(args) = match_keyword(&line, "/bye", 4) {
            return self.do_bye(args).await;
        }
        if line.is_empty() {
            if let Some(telnet) = self.telnet() {
                telnet.output("login: ").await;
            }
            return Ok(());
        }
        let user = (*USER_MANAGER).get_user(&line).await;
        self.set_user_entered(Some(line));
        if let Some(telnet) = self.telnet() {
            if let Some(user) = user {
                // Warn if echo can't be turned off.
                match telnet.echo() {
                    TELNET_ENABLED => (),
                    0 => {
                        telnet.output(BELL_STR).await;
                        telnet.output("\nSorry, password probably WILL echo.\n\n").await;
                    }
                    _ => telnet.output("\nWarning: password may echo.\n\n").await,
                };

                match user.password() {
                    Some(_) => {
                        telnet.set_do_echo(false); // Disable echoing.
                        self.switch_login_state(LoginState::AwaitingPassword, Some("Password: ")).await?;
                    }
                    None => {
                        // No password required. (guest account)
                        self.print_reserved_names().await;
                        self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await?;
                    }
                }
            } else {
                telnet.output("Invalid login.\n").await;
                let attempts = self.increment_attempts();
                if attempts >= MAX_LOGIN_ATTEMPTS {
                    return self.close(true).await;
                }
                telnet.output("login: ").await;
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn handle_password_input(&self, line: Text) -> tokio::io::Result<()> {
        println!("=== DEBUG: handle_password_input(): line={line:?} ===");
        println!("=== DEBUG: handle_password_input(): user={user:?} ===", user = self.user());
        if let Some(telnet) = self.telnet() {
            telnet.output("\n").await;
            telnet.set_do_echo(true);
        }
        println!("=== DEBUG: handle_password_input(): calling update_all() ===");
        (*USER_MANAGER).update_all().await.ok();

        let user = match self.user_entered().as_ref() {
            Some(user) => (*USER_MANAGER).get_user(user).await,
            None => None,
        };

        let valid = if let Some(user) = user {
            println!("=== DEBUG: handle_password_input(): user={user:?} ===");
            if let Some(password) = user.password() {
                println!("=== DEBUG: handle_password_input(): password={password:?} ===");
                verify_password(&line, &password)
            } else {
                println!("=== DEBUG: handle_password_input(): password not set ===");
                false
            }
        } else {
            println!("=== DEBUG: handle_password_input(): user not set ===");
            false
        };

        println!("=== DEBUG: handle_password_input(): valid={valid:?} ===");
        if !valid {
            self.output("Login incorrect.\n").await;
            let attempts = self.increment_attempts();
            if attempts >= MAX_LOGIN_ATTEMPTS {
                return self.close(true).await;
            }
            self.set_user(None);
            return self.switch_login_state(LoginState::AwaitingLogin, Some("login: ")).await;
        }

        self.print_reserved_names().await;
        self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await?;

        Ok(())
    }

    #[framed]
    pub async fn handle_name_input(&self, line: Text) -> tokio::io::Result<()> {
        let line = line.trim();

        (*USER_MANAGER).update_all().await.ok();

        let user = match self.user_entered().as_ref() {
            Some(user) => (*USER_MANAGER).get_user(user).await,
            None => None,
        };

        let name = if !line.is_empty() { Some(line.clone()) } else { user.and_then(|user| user.reserved().front().cloned()) };

        match name {
            Some(name) if self.check_name_availability(&name, false, false).await? => {
                self.set_name_entered(Some(name.into()));
                self.switch_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await?;
            }
            _ => {
                self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await?;
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn handle_blurb_input(&self, line: Text) -> tokio::io::Result<()> {
        (*USER_MANAGER).update_all().await.ok();

        let user = match self.user_entered().as_ref() {
            Some(user) => (*USER_MANAGER).get_user(user).await,
            None => None,
        };

        let blurb = (!line.is_empty()).then_some(line).or_else(|| user.as_ref().and_then(|u| u.blurb()));

        match self.name_entered() {
            Some(name) if !self.check_name_availability(&name, true, false).await? => {
                return self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
            }
            _ => {}
        }

        let name = Name::new(self.name_entered().unwrap_or_default(), blurb);
        println!("=== DEBUG: handle_blurb_input(): name={name:?} ===");
        self.logged_in(name, user);
        println!("=== DEBUG: handle_blurb_input(): self.name()={name:?} ===", name = self.name());

        // Send entry notification
        self.notify_entry().await?;

        // Welcome message and automatic commands
        self.output("\n\nWelcome to Phoenix.  Type \"/help\" for a list of commands.\n\n").await;

        // Make sure discussion A exists
        match self.find_sendable("A", false, true, true, true).await {
            (_, _, None, _) => {
                let disc = Discussion::new(None, "A", "General Discussion", true).await;
                DISCUSSIONS.insert(Text::from("A"), disc);
            }
            _ => {}
        }

        // Automatic commands
        self.do_join("A").await?;
        self.do_send("A").await?;
        self.do_who("").await?;
        self.do_howmany("").await?;

        if let Some(telnet) = self.telnet() {
            telnet.reset_history().await;
        }

        Ok(())
    }

    #[framed]
    pub async fn handle_transfer_input(&self, line: Text) -> tokio::io::Result<()> {
        let line = line.trim();
        if match_keyword(&line, "yes", 1).is_none() {
            self.output("Session not transferred.\n").await;
            return self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
        }

        match self.name_entered() {
            Some(name) if self.check_name_availability(&name, true, true).await? => {
                self.output("(That session is now gone.)\n").await;
                self.switch_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await?;
            }
            _ => {
                self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await?;
            }
        }

        Ok(())
    }

    /// Print user's reserved names.
    #[framed]
    pub async fn print_reserved_names(&self) {
        if let Some(telnet) = self.telnet() {
            if let Some(user) = self.user() {
                let reserved = user.reserved();

                if let Some(default_name) = reserved.front() {
                    telnet.output(&format!("\nYour default (reserved) name is \"{default_name}\".\n")).await;

                    let remaining: Vec<_> = reserved.iter().skip(1).collect();
                    let left = remaining.len();

                    if left > 0 {
                        let mut other = String::new();
                        other.push_str("\nYou also have \"");
                        other.push_str(remaining[0].as_str());

                        for name in remaining.iter().skip(1).take(left.saturating_sub(1)) {
                            other.push_str("\", \"");
                            other.push_str(name.as_str());
                        }

                        if left > 1 {
                            other.push_str("\" and \"");
                            other.push_str(remaining[left - 1].as_str());
                        }

                        other.push_str("\" reserved.\n");
                        telnet.output(&other).await;
                    }
                }
            }

            telnet.output("\n").await;
        }
    }

    #[framed]
    pub async fn save_input_line(&self, line: Text) -> tokio::io::Result<()> {
        self.lines().await.push_back(line);

        Ok(())
    }

    #[framed]
    pub async fn init_login_sequence(&self) -> tokio::io::Result<()> {
        println!("=== DEBUG: Session::init_login_sequence() starting ===");

        // Start login timeout
        self.reset_login_timeout();

        println!("=== DEBUG: Switching to AwaitingLogin state ===");
        self.switch_login_state(LoginState::AwaitingLogin, Some("login: ")).await?;
        println!("=== DEBUG: Session::init_login_sequence() completed ===");
        Ok(())
    }

    /// Check name availability.
    #[framed]
    pub async fn check_name_availability(&self, name: &str, double_check: bool, transferring: bool) -> tokio::io::Result<bool> {
        let telnet = match self.telnet() {
            Some(telnet) => telnet,
            None => return Ok(false),
        };

        if name.eq_ignore_ascii_case("me") {
            telnet.output("The keyword \"me\" is reserved.  Choose another name.\n").await;
            return Ok(false);
        }

        // Special-case test for discussion A, in case it hasn't actually been created yet.
        if name.eq_ignore_ascii_case("A") {
            telnet.output("There is already a discussion named \"A\".  Choose another name.\n").await;
            return Ok(false);
        }

        (*USER_MANAGER).update_all().await.ok();

        let user = match self.user_entered().as_ref() {
            Some(user) => (*USER_MANAGER).get_user(user).await,
            None => None,
        };

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(name).await {
            if Some(found_user) != user {
                let now = if double_check { " now" } else { "" };
                telnet.output(&format!("\"{reserved}\" is{now} a reserved name.  Choose another.\n")).await;
                return Ok(false);
            }
        }

        match self.find_sendable(name, false, true, true, true).await {
            (Some(session), _, _, _) if user.is_some() && user == session.user() && user.unwrap().priv_level() > 0 => {
                if session.telnet().is_some() {
                    if transferring {
                        telnet.output("Transferring active session...\n").await;
                        session.transfer(telnet).await?;
                        self.set_telnet(None);
                        self.close(false).await?;
                    } else {
                        let now = if double_check { " now" } else { "" };
                        telnet.output(&format!("You are{now} attached elsewhere under that name.\n")).await;
                        self.switch_login_state(LoginState::AwaitingTransferConfirmation, Some("Transfer active session? [no] ")).await?;
                    }
                } else {
                    telnet.output("Attaching to detached session...\n").await;
                    session.attach(telnet).await?;
                    self.set_telnet(None);
                    self.close(false).await?;
                }
            }
            (Some(session), _, _, _) => {
                let session_name = session.name();
                let already = if double_check { "now" } else { "already" };
                telnet.output(&format!("The name \"{session_name}\" is {already} in use.  Choose another.\n")).await;
            }
            (_, _, Some(discussion), _) => {
                let already = if double_check { "now" } else { "already" };
                let disc_name = discussion.name();
                telnet.output(&format!("There is {already} a discussion named \"{disc_name}\".  Choose another name.\n")).await;
            }
            _ => return Ok(true),
        }

        return Ok(false);
    }

    #[framed]
    pub async fn close(&self, drain: bool) -> tokio::io::Result<()> {
        let id = self.id();
        SESSIONS.remove(&id);

        if self.signed_on() {
            self.notify_exit().await?;
        }

        // Quit all discussions silently
        let disc_keys: Vec<_> = DISCUSSIONS.iter().map(|(key, _)| key.clone()).collect();
        for key in &disc_keys {
            if let Some(disc) = DISCUSSIONS.get(key) {
                disc.quit(&self).await?;
            }
        }

        // Close telnet connection if attached
        if let Some(telnet) = self.telnet() {
            telnet.close(drain).await?;
        }
        self.set_telnet(None);

        // Disassociate from user
        if let Some(user) = self.user() {
            user.remove_session(self);
            self.set_user(None);
        }

        // Check if server should shutdown after this session closes
        self.server().check_shutdown().await;

        Ok(())
    }

    #[framed]
    pub async fn transfer(&self, new_telnet: Telnet) -> tokio::io::Result<()> {
        let old_telnet = self.telnet();
        self.set_telnet(Some(new_telnet.clone()));
        new_telnet.set_session(self.clone());

        // Clear close-on-EOF flag and cancel login timeout (equivalent to LoginSequenceFinished)
        new_telnet.set_close_on_eof(false);
        self.cancel_login_timeout();

        if let Some(old) = old_telnet {
            let who = self.name_user();
            info!("Transfer: {who} from fd to new connection");
            old.output("*** This session has been transferred to a new connection. ***\n").await;
            old.close(true).await?;
        }

        self.enqueue_others(TransferNotify::new(self.name())).await?;
        self.pending().await.attach(&new_telnet).await?;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await?;

        Ok(())
    }

    #[framed]
    pub async fn attach(&self, telnet: Telnet) -> tokio::io::Result<()> {
        self.set_telnet(Some(telnet.clone()));
        telnet.set_session(self.clone());

        // Clear close-on-EOF flag and cancel login timeout for the attached connection
        telnet.set_close_on_eof(false);
        self.cancel_login_timeout();

        let who = self.name_user();
        info!("Attach: {who} on new connection");

        self.enqueue_others(AttachNotify::new(self.name())).await?;
        self.pending().await.attach(&telnet).await?;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await?;

        Ok(())
    }

    #[framed]
    pub async fn detach(&self, telnet: &Telnet, intentional: bool) -> tokio::io::Result<()> {
        if self.signed_on() && self.priv_level() > 0 {
            if let Some(t) = self.telnet() {
                if Arc::ptr_eq(&t.0, &telnet.0) {
                    let who = self.name_user();
                    if intentional {
                        info!("Detach: {who} (intentional)");
                    } else {
                        info!("Detach: {who} (accidental)");
                    };

                    self.enqueue_others(DetachNotify::new(self.name(), intentional)).await?;
                    self.set_telnet(None);
                }
            }
        } else {
            self.close(true).await?;
        }

        Ok(())
    }

    #[framed]
    pub async fn announce(message: &str) -> tokio::io::Result<()> {
        let mut result = Ok(());

        // Send announcement to ALL sessions, including pre-login sessions.
        for session in SESSIONS.snapshot().values() {
            session.output(message).await;
            if let Err(e) = session.enqueue_output().await {
                println!("=== DEBUG: Error in enqueue_output() during announce(): {e} ===");
                if result.is_ok() {
                    result = Err(e);
                }
            }
        }

        result
    }

    #[framed]
    pub async fn remove_discussion(name: Text) {
        DISCUSSIONS.remove(&name);
    }

    #[framed]
    pub async fn enqueue(&self, out: Output) -> tokio::io::Result<()> {
        self.enqueue_output().await?;
        if let Some(telnet) = self.telnet() {
            self.pending().await.enqueue(Some(&telnet), out).await?;
        } else {
            self.pending().await.enqueue(None, out).await?;
        }

        Ok(())
    }

    #[framed]
    pub async fn enqueue_others(&self, out: Output) -> tokio::io::Result<()> {
        let mut result = Ok(());

        for session in SESSIONS.snapshot().values().filter(|s| s.signed_on()) {
            if session != self {
                if let Err(e) = session.enqueue(out.clone()).await {
                    println!("=== DEBUG: Error in enqueue() during enqueue_others(): {e} ===");
                    if result.is_ok() {
                        result = Err(e);
                    }
                }
            }
        }

        result
    }

    #[framed]
    pub async fn find_sendable(
        &self,
        sendlist: &str,
        member: bool,
        exact: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) -> (Option<Session>, OrdSet<Session>, Option<Discussion>, OrdSet<Discussion>) {
        let mut session = None;
        let mut session_matches = OrdSet::new();
        let mut discussion = None;
        let mut discussion_matches = OrdSet::new();

        let mut count = 0;
        let mut session_lead = None;
        let mut discussion_lead = None;

        if do_sessions {
            if sendlist.eq_ignore_ascii_case("me") {
                session = Some(self.clone());
                session_matches.insert(self.clone());
                return (session, session_matches, discussion, discussion_matches);
            }

            for s in SESSIONS.snapshot().values().filter(|s| s.signed_on()) {
                let s_name = s.name();
                if s_name.eq_ignore_ascii_case(sendlist) {
                    session = Some(s.clone());
                    session_matches.insert(s.clone());
                } else if !exact {
                    if let Some(pos) = match_name(&s_name, sendlist) {
                        if pos == 1 {
                            count += 1;
                            session_lead = Some(s.clone());
                        }
                        session_matches.insert(s.clone());
                    }
                }
            }
        }

        if do_discussions {
            for (_, d) in DISCUSSIONS.iter() {
                if member {
                    if !d.members().contains(self) {
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
                            count += 1;
                            discussion_lead = Some(d.clone());
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

        // If we found exactly one lead match, use it
        if count == 1 {
            session = session_lead;
            discussion = discussion_lead;
            return (session, session_matches, discussion, discussion_matches);
        }

        // If we have exactly one match total, use it
        if session_matches.len() + discussion_matches.len() == 1 {
            if session_matches.len() == 1 {
                session = session_matches.iter().next().cloned();
            }
            if discussion_matches.len() == 1 {
                discussion = discussion_matches.iter().next().cloned();
            }
        }

        (session, session_matches, discussion, discussion_matches)
    }

    #[framed]
    pub async fn find_session(&self, sendlist: &str) -> (Option<Session>, OrdSet<Session>) {
        let (session, matches, _, _) = self.find_sendable(sendlist, false, false, true, false).await;
        (session, matches)
    }

    #[framed]
    pub async fn find_discussion(&self, sendlist: &str, member: bool) -> (Option<Discussion>, OrdSet<Discussion>) {
        let (_, _, discussion, matches) = self.find_sendable(sendlist, member, false, false, true).await;
        (discussion, matches)
    }

    #[framed]
    pub async fn notify_entry(&self) -> tokio::io::Result<()> {
        let who = self.name_user();
        if let Some(_telnet) = self.telnet() {
            info!("Enter: {who} on connection");
        } else {
            info!("Enter: {who}, detached");
        }

        let now = Timestamp::new();
        self.set_idle_since(now.clone());
        self.set_login_time(now);

        self.enqueue_others(EntryNotify::new(self.name())).await?;

        Ok(())
    }

    #[framed]
    pub async fn notify_exit(&self) -> tokio::io::Result<()> {
        let who = self.name_user();
        if let Some(_telnet) = self.telnet() {
            info!("Exit: {who} on connection");
        } else {
            info!("Exit: {who}, detached");
        }

        self.enqueue_others(ExitNotify::new(self.name())).await?;

        Ok(())
    }

    #[framed]
    pub async fn process_input(&self, line: Text) -> tokio::io::Result<()> {
        if line.starts_with("!") {
            let line = &line.trim();
            // XXX Make ! normal for average users? normal if not a valid command?
            if self.priv_level() < 50 {
                self.output("Sorry, all !commands are privileged.\n").await;
                return Ok(());
            }

            // XXX add !priv command?
            // XXX do individual privilege levels for each !command?

            if let Some(args) = match_keyword(line, "!restart", 8) {
                self.do_restart(args).await?;
            } else if let Some(args) = match_keyword(line, "!down", 5) {
                self.do_down(args).await?;
            } else if let Some(args) = match_keyword(line, "!nuke", 5) {
                self.do_nuke(args).await?;
            } else {
                self.output("Unknown !command.\n").await;
            }
        } else if line.starts_with("/") {
            let line = &line.trim();
            if let Some(args) = match_keyword(line, "/who", 2) {
                self.do_who(args).await?;
            } else if let Some(args) = match_keyword(line, "/idle", 2) {
                self.do_idle(args).await?;
            } else if let Some(args) = match_keyword(line, "/blurb", 3) {
                self.do_blurb(args).await?;
            } else if let Some(args) = match_keyword(line, "/here", 2) {
                self.do_here(args).await?;
            } else if let Some(args) = match_keyword(line, "/away", 2) {
                self.do_away(args).await?;
            } else if let Some(args) = match_keyword(line, "/busy", 2) {
                self.do_busy(args).await?;
            } else if let Some(args) = match_keyword(line, "/gone", 2) {
                self.do_gone(args).await?;
            } else if let Some(args) = match_keyword(line, "/help", 2) {
                self.do_help(args).await?;
            } else if let Some(args) = match_keyword(line, "/send", 2) {
                self.do_send(args).await?;
            } else if let Some(args) = match_keyword(line, "/bye", 4) {
                self.do_bye(args).await?;
            } else if let Some(args) = match_keyword(line, "/what", 3) {
                self.do_what(args).await?;
            } else if let Some(args) = match_keyword(line, "/join", 2) {
                self.do_join(args).await?;
            } else if let Some(args) = match_keyword(line, "/quit", 2) {
                self.do_quit(args).await?;
            } else if let Some(args) = match_keyword(line, "/create", 3) {
                self.do_create(args).await?;
            } else if let Some(args) = match_keyword(line, "/destroy", 4) {
                self.do_destroy(args).await?;
            } else if let Some(args) = match_keyword(line, "/permit", 4) {
                self.do_permit(args).await?;
            } else if let Some(args) = match_keyword(line, "/depermit", 4) {
                self.do_depermit(args).await?;
            } else if let Some(args) = match_keyword(line, "/appoint", 4) {
                self.do_appoint(args).await?;
            } else if let Some(args) = match_keyword(line, "/unappoint", 10) {
                self.do_unappoint(args).await?;
            } else if let Some(args) = match_keyword(line, "/rename", 7) {
                self.do_rename(args).await?;
            } else if let Some(args) = match_keyword(line, "/clear", 3) {
                self.do_clear(args).await?;
            } else if let Some(args) = match_keyword(line, "/unidle", 7) {
                self.do_unidle(args).await?;
            } else if let Some(args) = match_keyword(line, "/detach", 4) {
                self.do_detach(args).await?;
            } else if let Some(args) = match_keyword(line, "/howmany", 3) {
                self.do_howmany(args).await?;
            } else if let Some(args) = match_keyword(line, "/why", 4) {
                self.do_why(args).await?;
            } else if let Some(args) = match_keyword(line, "/date", 3) {
                self.do_date(args).await?;
            } else if let Some(args) = match_keyword(line, "/signal", 3) {
                self.do_signal(args).await?;
            } else if let Some(args) = match_keyword(line, "/set", 4) {
                self.do_set(args).await?;
            } else if let Some(args) = match_keyword(line, "/display", 2) {
                self.do_display(args).await?;
            } else if let Some(args) = match_keyword(line, "/also", 3) {
                self.do_also(args).await?;
            } else if let Some(args) = match_keyword(line, "/oops", 3) {
                self.do_oops(args).await?;
            } else {
                self.output("Unknown /command.  Type /help for help.\n").await;
            }
        } else if line == " " {
            self.do_reset().await?;
        } else if !line.is_empty() {
            self.do_message(&line).await?;
        }

        Ok(())
    }

    /// Reset idle time to now.
    #[framed]
    pub async fn reset_idle(&self, min: i64) -> i64 {
        let now = Timestamp::new();
        let idle = (now.unix() - self.idle_since().unix()) / 60;

        if min > 0 && idle >= min {
            self.output("[You were idle for").await;
            self.print_time_long(idle).await;
            self.output(".]\n").await;
        }

        self.set_idle_since(now.clone());
        idle
    }

    #[framed]
    pub async fn print_time_long(&self, minutes: i64) {
        // Determine time format (0 = verbose, 1 = both, 2 = terse)
        let format = if let Some(fmt) = self.get_sys_var("time_format") {
            match fmt.as_str() {
                "verbose" => 0,
                "both" => 1,
                "terse" => 2,
                _ => 0,
            }
        } else {
            match DEFAULTS.get(&Text::from("time_format")) {
                Some(s) if s.as_str() == "verbose" => 0,
                Some(s) if s.as_str() == "both" => 1,
                Some(s) if s.as_str() == "terse" => 2,
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

    #[framed]
    pub async fn do_restart(&self, args: &str) -> tokio::io::Result<()> {
        let who = self.name_user();
        let name = self.name();

        if args == "!" {
            // Immediate restart
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await?;
            self.server().schedule_restart(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel restart
            match self.server().cancel_shutdown().await {
                Some(true) => {
                    info!("Restart cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server restart. ***\n")).await?;
                }
                Some(false) => {
                    info!("Shutdown cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server shutdown. ***\n")).await?;
                }
                None => self.output("The server was not about to shut down or restart.\n").await,
            }
        } else {
            // Delayed restart
            let seconds = args.parse::<u64>().unwrap_or(30);
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await?;
            self.server().schedule_restart(who.clone(), seconds).await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_down(&self, args: &str) -> tokio::io::Result<()> {
        let who = self.name_user();
        let name = self.name();

        if args == "!" {
            // Immediate shutdown
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await?;
            self.server().schedule_shutdown(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel shutdown
            match self.server().cancel_shutdown().await {
                Some(true) => {
                    info!("Restart cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server restart. ***\n")).await?;
                }
                Some(false) => {
                    info!("Shutdown cancelled by {who}.");
                    Self::announce(&format!("*** {name} has cancelled the server shutdown. ***\n")).await?;
                }
                None => self.output("The server was not about to shut down or restart.\n").await,
            }
        } else {
            // Delayed shutdown
            let seconds = args.parse::<u64>().unwrap_or(30);
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await?;
            self.server().schedule_shutdown(who.clone(), seconds).await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_nuke(&self, args: &str) -> tokio::io::Result<()> {
        // Nuke target session. // XXX Should require confirmation!
        let drain = !args.starts_with('!');
        let args = if drain { args } else { &args[1..] };

        match self.find_session(args).await {
            (Some(target), _) => {
                let who = target.name_user();
                let name = target.name();
                let by_who = self.name_user();
                let by_name = self.name();

                if drain {
                    self.output(&format!("\"{name}\" has been nuked.\n")).await;
                } else {
                    self.output(&format!("\"{name}\" has been nuked immediately.\n")).await;
                }

                if let Some(telnet) = target.telnet() {
                    target.set_telnet(None);
                    info!("{who} has been nuked by {by_who}");
                    telnet.undraw_input().await;
                    telnet.output(&format!("\x07\x07\x07*** You have been nuked by {by_name}. ***\n")).await;
                    telnet.redraw_input().await;
                    telnet.close(drain).await?;
                } else {
                    info!("{who}, detached, has been nuked by {by_who}");
                    target.close(true).await?;
                }
            }
            (_, matches) => {
                self.output("\x07\x07").await;
                self.session_matches(args, &matches).await;
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn do_bye(&self, _args: &str) -> tokio::io::Result<()> {
        self.close(true).await?;

        Ok(())
    }

    #[framed]
    pub async fn do_who(&self, args: &str) -> tokio::io::Result<()> {
        // Get set of users to display.
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return Ok(());
        }

        let now = Timestamp::new();
        let mut extend = 0;

        // Find longest idle time for formatting.
        for session in &who {
            let days = (now.unix() - session.idle_since().unix()) / 86400;
            if days == 0 {
                continue;
            }

            let mut width = days.to_string().len();
            if session.telnet().is_none() || (now.unix() - session.login_time().unix()) >= 31536000 {
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
            let detached = session.detached_indicator();
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Login time or "detached".
            if session.telnet().is_some() {
                let login_time = session.login_time();
                if (now.unix() - login_time.unix()) < 86400 {
                    self.output(&login_time.date(11, 8)).await;
                } else if (now.unix() - login_time.unix()) < 31536000 {
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
            let idle = (now.unix() - session.idle_since().unix()) / 60;
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
            match session.away() {
                AwayState::Here => self.output("Here\n").await,
                AwayState::Away => self.output("Away\n").await,
                AwayState::Busy => self.output("Busy\n").await,
                AwayState::Gone => self.output("Gone\n").await,
            }

            // Show continuation of long name if only one user.
            if name.len() > 33 && who.len() == 1 {
                self.output(&format!(">{name}\n", name = &name.as_str()[32..])).await;
            }
        }

        // Output message and errors from get_who_set().
        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_idle(&self, args: &str) -> tokio::io::Result<()> {
        // Get set of users to display.
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return Ok(());
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
            let detached = session.detached_indicator();
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Idle time.
            let idle = (now.unix() - session.idle_since().unix()) / 60;
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

        Ok(())
    }

    #[framed]
    pub async fn do_why(&self, args: &str) -> tokio::io::Result<()> {
        // This is a privileged command.
        if self.priv_level() < 50 {
            self.output("Why not?\n").await;
            return Ok(());
        }

        // Get set of users to display.
        let (who, errors, msg) = self.get_who_set(args).await;
        if who.is_empty() {
            if !errors.is_empty() {
                self.output("\x07\x07").await;
                self.output(&errors).await;
            }
            return Ok(());
        }

        let now = Timestamp::new();
        let mut extend = 0;

        // Find longest idle time for formatting.
        for session in &who {
            let days = (now.unix() - session.idle_since().unix()) / 86400;
            if days == 0 {
                continue;
            }

            let mut width = days.to_string().len();
            if (now.unix() - session.login_time().unix()) >= 31536000 {
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
            let detached = session.detached_indicator();
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Login time.
            let login_time = session.login_time();
            if (now.unix() - login_time.unix()) < 86400 {
                self.output(&login_time.date(11, 8)).await;
            } else if (now.unix() - login_time.unix()) < 31536000 {
                self.output(" ").await;
                self.output(&login_time.date(4, 6)).await;
                self.output(" ").await;
            } else {
                self.output(&login_time.date(4, 4)).await;
                self.output(&login_time.date(20, 4)).await;
            }

            // Idle time.
            let idle = (now.unix() - session.idle_since().unix()) / 60;
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
            match session.away() {
                AwayState::Here => self.output("Here  ").await,
                AwayState::Away => self.output("Away  ").await,
                AwayState::Busy => self.output("Busy  ").await,
                AwayState::Gone => self.output("Gone  ").await,
            }

            // Username.
            if let Some(user) = session.user() {
                let username = user.username();
                self.output(&format!("{username:<8}  ")).await;
            } else {
                self.output("guest     ").await;
            }

            // File descriptor.
            if session.telnet().is_some() {
                self.output("?? ").await;
            } else {
                self.output("-- ").await;
            }

            // Privilege level.
            let session_priv = session.priv_level();
            let user_priv = if let Some(user) = session.user() { user.priv_level() } else { 0 };
            let indicator = if session_priv == user_priv { " " } else { "*" };
            self.output(&format!("{indicator}{session_priv:4}\n")).await;
        }

        // Output message and errors from get_who_set().
        self.output(&msg).await;
        if !errors.is_empty() {
            self.output("\x07\x07").await;
            self.output(&errors).await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_blurb(&self, args: &str) -> tokio::io::Result<()> {
        let args = args.trim();

        if !args.is_empty() {
            let mut start = 0;
            let mut end = args.len();

            if args.len() == 3 && args.eq_ignore_ascii_case("off") {
                if self.blurb().map_or(false, |b| !b.is_empty()) {
                    self.reset_idle(REPORT_IDLE_DEFAULT).await;
                    self.remove_blurb();
                    self.output("Your blurb has been turned off.\n").await;
                } else {
                    self.output("Your blurb was already turned off.\n").await;
                }
            } else {
                if (args.starts_with('"') && args.ends_with('"') && args.len() > 2) || (args.starts_with('[') && args.ends_with(']')) {
                    start = 1;
                    end = args.len() - 1;
                }

                self.reset_idle(REPORT_IDLE_DEFAULT).await;

                let blurb = &args[start..end];
                self.set_blurb(Some(blurb.into()));
                self.output(&format!("Your blurb has been set to [{blurb}].\n")).await;
            }
        } else if self.has_blurb() {
            let blurb = self.blurb().unwrap();
            self.output(&format!("Your blurb is currently set to [{blurb}].\n")).await;
        } else {
            self.output("You do not currently have a blurb set.\n").await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_here(&self, args: &str) -> tokio::io::Result<()> {
        self.reset_idle(REPORT_IDLE_DEFAULT).await;
        if !args.trim().is_empty() {
            self.do_blurb(args).await?;
        }
        self.output("You are now \"here\".\n").await;
        self.set_away(AwayState::Here);
        self.enqueue_others(HereNotify::new(self.name())).await?;

        Ok(())
    }

    #[framed]
    pub async fn do_away(&self, args: &str) -> tokio::io::Result<()> {
        self.reset_idle(REPORT_IDLE_DEFAULT).await;
        if !args.trim().is_empty() {
            self.do_blurb(args).await?;
        }
        self.output("You are now \"away\".\n").await;
        self.set_away(AwayState::Away);
        self.enqueue_others(AwayNotify::new(self.name())).await?;

        Ok(())
    }

    #[framed]
    pub async fn do_busy(&self, args: &str) -> tokio::io::Result<()> {
        self.reset_idle(REPORT_IDLE_DEFAULT).await;
        if !args.trim().is_empty() {
            self.do_blurb(args).await?;
        }
        self.output("You are now \"busy\".\n").await;
        self.set_away(AwayState::Busy);
        self.enqueue_others(BusyNotify::new(self.name())).await?;

        Ok(())
    }

    #[framed]
    pub async fn do_gone(&self, args: &str) -> tokio::io::Result<()> {
        self.reset_idle(REPORT_IDLE_DEFAULT).await;
        if !args.trim().is_empty() {
            self.do_blurb(args).await?;
        }
        self.output("You are now \"gone\".\n").await;
        self.set_away(AwayState::Gone);
        self.enqueue_others(GoneNotify::new(self.name())).await?;

        Ok(())
    }

    #[framed]
    pub async fn do_clear(&self, _args: &str) -> tokio::io::Result<()> {
        self.output("\x1b[H\x1b[J").await; // XXX ANSI!

        Ok(())
    }

    #[framed]
    pub async fn do_unidle(&self, _args: &str) -> tokio::io::Result<()> {
        let idle = self.reset_idle(1).await;
        if idle == 0 {
            self.output("Your idle time has been reset.\n").await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_detach(&self, _args: &str) -> tokio::io::Result<()> {
        if self.priv_level() > 0 {
            self.reset_idle(REPORT_IDLE_DEFAULT).await;
            self.output("You have been detached.\n").await;
            self.enqueue_output().await?;
            if let Some(telnet) = self.telnet() {
                telnet.close(true).await?;
            }
        } else {
            self.output("Guest users are not allowed to detach from the system.  Use /bye to sign off.\n").await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_howmany(&self, _args: &str) -> tokio::io::Result<()> {
        let mut here = 0;
        let mut away = 0;
        let mut busy = 0;
        let mut gone = 0;
        let mut attached = 0;
        let mut detached = 0;
        let mut total = 0;

        for session in SESSIONS.snapshot().values().filter(|s| s.signed_on()) {
            match session.away() {
                AwayState::Here => here += 1,
                AwayState::Away => away += 1,
                AwayState::Busy => busy += 1,
                AwayState::Gone => gone += 1,
            }
            if session.telnet().is_some() {
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

        Ok(())
    }

    #[framed]
    pub async fn do_what(&self, args: &str) -> tokio::io::Result<()> {
        if DISCUSSIONS.is_empty() {
            self.output("No discussions currently exist.\n").await;
            return Ok(());
        }

        let sendlist = Sendlist::new(&self, args, true, false, true).await;

        if !args.is_empty() && sendlist.discussions().is_empty() {
            self.output(&sendlist.errors().to_string()).await;
            return Ok(());
        }

        let discussions = if args.is_empty() { DISCUSSIONS.iter().map(|r| r.1.clone()).collect() } else { sendlist.discussions() };

        self.output("\n Name            Users  Idle  Title\n").await;
        self.output(" ----            -----  ----  -----\n").await;

        let now = Timestamp::new();

        for disc in &discussions {
            let disc_name = disc.name();
            self.output(" ").await;
            let name = if disc_name.len() > 15 { format!("{disc_name:<14.14}+") } else { format!("{disc_name:<15}") };
            self.output(&name).await;

            let members = disc.members();
            let member_count = members.len();
            let is_member = if members.contains(self) { '*' } else { ' ' };
            self.output(&format!("{member_count:>3}{is_member} ")).await;

            let idle = (now.unix() - disc.idle_since().unix()) / 60;
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

            if disc.is_permitted(&self.name()) {
                let title = &disc.title();
                if title.len() > 49 {
                    self.output(&format!("{title:<48.48}+\n")).await;
                } else {
                    self.output(&format!("{title}\n")).await;
                }
            } else {
                self.output("<Private>\n").await;
            }
        }

        if !sendlist.errors().is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors().to_string()).await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_date(&self, _args: &str) -> tokio::io::Result<()> {
        let t = Timestamp::new();
        let date = t.date(0, 0);
        self.output(&format!("{date}\n")).await;

        Ok(())
    }

    #[framed]
    pub async fn do_signal(&self, args: &str) -> tokio::io::Result<()> {
        let mut args = args;

        if let Some(_rest) = match_keyword(args, "on", 2) {
            self.set_signal_public(true);
            self.set_signal_private(true);
            self.output("All signals are now on.\n").await;
        } else if let Some(_rest) = match_keyword(args, "off", 2) {
            self.set_signal_public(false);
            self.set_signal_private(false);
            self.output("All signals are now off.\n").await;
        } else if let Some(rest) = match_keyword(args, "public", 2) {
            args = rest;
            if let Some(_) = match_keyword(args, "on", 2) {
                self.set_signal_public(true);
                self.output("Signals for public messages are now on.\n").await;
            } else if let Some(_) = match_keyword(args, "off", 2) {
                self.set_signal_public(false);
                self.output("Signals for public messages are now off.\n").await;
            } else if args.is_empty() {
                let on = if self.signal_public() { "on" } else { "off" };
                self.output(&format!("Signals are {on} for public messages.\n")).await;
            } else {
                self.output("Usage: /signal public [on|off]\n").await;
            }
        } else if let Some(rest) = match_keyword(args, "private", 2) {
            args = rest;
            if let Some(_) = match_keyword(args, "on", 2) {
                self.set_signal_private(true);
                self.output("Signals for private messages are now on.\n").await;
            } else if let Some(_) = match_keyword(args, "off", 2) {
                self.set_signal_private(false);
                self.output("Signals for private messages are now off.\n").await;
            } else if args.is_empty() {
                let on = if self.signal_private() { "on" } else { "off" };
                self.output(&format!("Signals are {on} for private messages.\n")).await;
            } else {
                self.output("Usage: /signal private [on|off]\n").await;
            }
        } else if args.is_empty() {
            let pub_sig = self.signal_public();
            let priv_sig = self.signal_private();

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

        Ok(())
    }

    #[framed]
    pub async fn do_send(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            if let Some(sendlist) = self.default_sendlist() {
                self.output("You are sending to ").await;
                self.print_sendlist(&sendlist).await;
                self.output(".\n").await;
            } else {
                self.output("Your default sendlist is turned off.\n").await;
            }
            return Ok(());
        }

        if args.eq_ignore_ascii_case("off") {
            self.set_default_sendlist(None);
            self.output("Your default sendlist has been turned off.\n").await;
            return Ok(());
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

        if !sendlist.errors().is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors().to_string()).await;
        }

        if sendlist.sessions().is_empty() && sendlist.discussions().is_empty() {
            self.output("Your default sendlist is unchanged.\n").await;
            return Ok(());
        }

        self.set_default_sendlist(Some(sendlist.clone()));
        self.output("You are now sending to ").await;
        self.print_sendlist(&sendlist).await;
        self.output(".\n").await;

        Ok(())
    }

    #[framed]
    pub async fn print_sendlist(&self, sendlist: &Sendlist) {
        if !sendlist.sessions().is_empty() {
            let mut first = true;
            for session in &sendlist.sessions() {
                if first {
                    first = false;
                } else {
                    self.output(", ").await;
                }
                self.output(&session.name()).await;
            }

            if !sendlist.discussions().is_empty() {
                let s = if sendlist.discussions().len() == 1 { "" } else { "s" };
                self.output(&format!(" and discussion{s} ")).await;

                first = true;
                for discussion in &sendlist.discussions() {
                    if first {
                        first = false;
                    } else {
                        self.output(", ").await;
                    }
                    self.output(discussion.name()).await;
                }
            }
        } else {
            let mut first = true;
            for discussion in &sendlist.discussions() {
                if first {
                    first = false;
                } else {
                    self.output(", ").await;
                }
                self.output(discussion.name()).await;
            }
        }
    }

    #[framed]
    pub async fn do_join(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /join <disc>[,<disc>...]\n").await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (name, rest) = getword(remaining, Some(','));
            remaining = rest;

            match self.find_discussion(name, false).await {
                (Some(discussion), _) => discussion.join(&self).await?,
                (_, matches) => self.discussion_matches(name, &matches).await,
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn do_quit(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /quit <disc>[,<disc>...]\n").await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (name, rest) = getword(remaining, Some(','));
            remaining = rest;

            match self.find_discussion(name, false).await {
                (Some(discussion), _) => discussion.quit(&self).await?,
                _ => match self.find_discussion(name, true).await {
                    (Some(discussion), _) => discussion.quit(&self).await?,
                    (_, matches) => self.discussion_matches(name, &matches).await,
                },
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn do_create(&self, args: &str) -> tokio::io::Result<()> {
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
            return Ok(());
        }

        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (not created)\n").await;
            return Ok(());
        }

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(name).await {
            let my_user = self.user();
            let found_user_ref = Some(found_user);
            let is_same_user = match (my_user, found_user_ref) {
                (Some(my_user), Some(found_user)) if my_user.username() == found_user.username() => true,
                _ => false,
            };

            let a = if is_same_user { "your" } else { "a" };
            self.output(&format!("\"{reserved}\" is {a} reserved name. (not created)\n")).await;
            return Ok(());
        }

        match self.find_sendable(name, false, true, true, true).await {
            (Some(session), _, _, _) => {
                let name = session.name();
                self.output(&format!("There is already someone named \"{name}\". (not created)\n")).await;
                return Ok(());
            }
            (_, _, Some(discussion), _) => {
                let name = discussion.name();
                self.output(&format!("There is already a discussion named \"{name}\". (not created)\n")).await;
                return Ok(());
            }
            _ => {}
        }

        let disc = Discussion::new(Some(self.clone()), name, title, is_public).await;
        DISCUSSIONS.insert(name.to_string().into(), disc.clone());

        self.enqueue_others(CreateNotify::new(disc.name(), disc.title(), disc.is_public(), self.name())).await?;

        let name = disc.name();
        let title = disc.title();
        self.output(&format!("You have created discussion {name}, \"{title}\".\n")).await;

        Ok(())
    }

    #[framed]
    pub async fn do_destroy(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /destroy <disc>[,<disc>...]\n").await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (name, rest) = getword(remaining, Some(','));
            remaining = rest;

            match self.find_discussion(name, false).await {
                (Some(discussion), _) => discussion.destroy(&self).await?,
                _ => match self.find_discussion(name, true).await {
                    (Some(discussion), _) => discussion.destroy(&self).await?,
                    (_, matches) => self.discussion_matches(name, &matches).await,
                },
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn do_permit(&self, args: &str) -> tokio::io::Result<()> {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /permit <disc> <person>[,<person>...]\n").await;
            return Ok(());
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.permit(&self, rest).await?,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.permit(&self, rest).await?,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }

        Ok(())
    }

    #[framed]
    pub async fn do_depermit(&self, args: &str) -> tokio::io::Result<()> {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /depermit <disc> <person>[,<person>...]\n").await;
            return Ok(());
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.depermit(&self, rest).await?,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.depermit(&self, rest).await?,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }

        Ok(())
    }

    #[framed]
    pub async fn do_appoint(&self, args: &str) -> tokio::io::Result<()> {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /appoint <disc> <person>[,<person>...]\n").await;
            return Ok(());
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.appoint(&self, rest).await?,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.appoint(&self, rest).await?,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }

        Ok(())
    }

    #[framed]
    pub async fn do_unappoint(&self, args: &str) -> tokio::io::Result<()> {
        let (name, rest) = getword(args, None);
        if name.is_empty() || rest.is_empty() {
            self.output("Usage: /unappoint <disc> <person>[,<person>...]\n").await;
            return Ok(());
        }

        match self.find_discussion(name, false).await {
            (Some(discussion), _) => discussion.unappoint(&self, rest).await?,
            _ => match self.find_discussion(name, true).await {
                (Some(discussion), _) => discussion.unappoint(&self, rest).await?,
                (_, matches) => self.discussion_matches(name, &matches).await,
            },
        }

        Ok(())
    }

    #[framed]
    pub async fn do_rename(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /rename <name>\n").await;
            return Ok(());
        }

        if args.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  (name unchanged)\n").await;
            return Ok(());
        }

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(args).await {
            let my_user = self.user();
            let found_user_ref = Some(found_user);
            match (my_user, found_user_ref) {
                (Some(my_user), Some(found_user)) if my_user.username() == found_user.username() => {
                    self.output(&format!("\"{reserved}\" is a reserved name.  (name unchanged)\n")).await;
                    return Ok(());
                }
                _ => {}
            }
        }

        match self.find_sendable(args, false, true, true, true).await {
            (Some(found_session), _, _, _) if &found_session != self => {
                let found_name = found_session.name();
                self.output(&format!("The name \"{found_name}\" is already in use.  (name unchanged)\n")).await;
            }
            (_, _, Some(found_discussion), _) => {
                let found_name = found_discussion.name();
                self.output(&format!("There is already a discussion named \"{found_name}\".  (name unchanged)\n")).await;
            }
            _ => {}
        }

        self.enqueue_others(RenameNotify::new(self.name().name().clone(), args)).await?;

        self.output(&format!("You have changed your name to \"{args}\".\n")).await;
        self.set_name_and_blurb(args.into(), self.blurb());

        Ok(())
    }

    #[framed]
    pub async fn do_set(&self, args: &str) -> tokio::io::Result<()> {
        let (var, value) = getword(args, Some('='));
        if var.is_empty() || value.is_empty() {
            self.output("Usage: /set <variable>=<value>\n").await;
            return Ok(());
        }

        if var.starts_with('$') {
            self.set_user_var(var, value);
        } else if let Some(_) = match_keyword(var, "echo", 4) {
            if let Some(telnet) = self.telnet() {
                let (val, _) = getword(value, None);
                if let Some(_) = match_keyword(val, "on", 2) {
                    telnet.set_echo(TELNET_ENABLED);
                    self.output("Remote echoing is now enabled.\n").await;
                } else if let Some(_) = match_keyword(val, "off", 3) {
                    telnet.set_echo(0);
                    self.output("Remote echoing is now disabled.\n").await;
                } else {
                    self.output("Usage: /set echo=[on|off]\n").await;
                }
            }
        } else if let Some(_) = match_keyword(var, "height", 6) {
            if let Ok(height) = value.parse::<usize>() {
                if height > 0 {
                    if let Some(telnet) = self.telnet() {
                        telnet.set_height(height);
                        self.output(&format!("Terminal height is now set to {height}.\n")).await;
                    }
                } else {
                    self.output("Usage: /set height=<number of rows>\n").await;
                }
            } else {
                self.output("Usage: /set height=<number of rows>\n").await;
            }
        } else if let Some(_) = match_keyword(var, "idle", 4) {
            self.set_idle(value).await?;
        } else if let Some(_) = match_keyword(var, "time_format", 11) {
            if let Some(_) = match_keyword(value, "verbose", 7) {
                self.set_sys_var("time_format", "verbose");
            } else if let Some(_) = match_keyword(value, "both", 4) {
                self.set_sys_var("time_format", "both");
            } else if let Some(_) = match_keyword(value, "terse", 5) {
                self.set_sys_var("time_format", "terse");
            } else if let Some(_) = match_keyword(value, "default", 7) {
                self.remove_sys_var("time_format");
            } else {
                self.output("Usage: /set time_format [terse|verbose|both|default]\n").await;
            }
        } else if let Some(_) = match_keyword(var, "uptime", 6) {
            self.output("Server uptime is a readonly variable.\n").await;
        } else if let Some(_) = match_keyword(var, "width", 5) {
            if let Ok(width) = value.parse::<usize>() {
                if width > 0 {
                    if let Some(telnet) = self.telnet() {
                        telnet.set_width(width);
                        self.output(&format!("Terminal width is now set to {width}.\n")).await;
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

        Ok(())
    }

    #[framed]
    pub async fn set_idle(&self, args: &str) -> tokio::io::Result<()> {
        let now = Timestamp::new();
        let current_idle = (now.unix() - self.idle_since().unix()) / 60;

        // Parse time specification: <d>d<hh>:<mm>
        let mut chars = args.trim().chars().peekable();
        let mut days = 0i64;
        let mut hours = 0i64;

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

        let minutes = num;

        // Skip trailing whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }

        // If there are remaining characters, it's a syntax error
        if chars.peek().is_some() {
            self.output("Syntax error in time specification.  Format: <d>d<hh>:<mm>\n").await;
            return Ok(());
        }

        // Calculate new idle_since timestamp
        let total_minutes = days * 24 * 60 + hours * 60 + minutes;
        let new_idle_since = Timestamp::from_unix(now.unix() - total_minutes * 60);

        // Check permissions
        if new_idle_since < *self.login_time() && self.priv_level() < 50 {
            self.output("Sorry, you can't be idle longer than you've been signed on.\n").await;
            return Ok(());
        }

        // Set the new idle time
        self.set_idle_since(new_idle_since);
        if self.idle_since().unix() < self.login_time().unix() {
            self.set_login_time(self.idle_since().as_ref().clone());
        }

        // Output results
        let new_idle = (now.unix() - self.idle_since().unix()) / 60;

        if current_idle > 0 && current_idle != new_idle {
            self.output("[You were idle for").await;
            self.print_time_long(current_idle).await;
            self.output(".]\n").await;
        }

        if current_idle == new_idle {
            self.output("Your idle time is still").await;
            self.print_time_long(current_idle).await;
            self.output(".\n").await;
        } else if new_idle > 0 {
            self.output("Your idle time has been set to").await;
            self.print_time_long(new_idle).await;
            self.output(".\n").await;
        } else {
            self.output("Your idle time has been reset.\n").await;
            self.set_idle_since(now);
        }

        Ok(())
    }

    #[framed]
    pub async fn do_display(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /display <variable>[,<variable>...]\n").await;
            return Ok(());
        }

        let mut remaining = args;
        while !remaining.is_empty() {
            let (var, rest) = getword(remaining, Some(','));
            remaining = rest;

            if var.starts_with('$') {
                if let Some(value) = self.get_user_var(var) {
                    self.output(&format!("{var} = \"{value}\"\n")).await;
                } else {
                    self.output(&format!("Unknown user variable: \"{var}\"\n")).await;
                }
            } else if let Some(_) = match_keyword(var, "echo", 4) {
                if let Some(telnet) = self.telnet() {
                    if telnet.echo() == TELNET_ENABLED {
                        self.output("Remote echoing is currently enabled.\n").await;
                    } else {
                        self.output("Remote echoing is currently disabled.\n").await;
                    }
                }
            } else if let Some(_) = match_keyword(var, "height", 6) {
                if let Some(telnet) = self.telnet() {
                    let height = telnet.height();
                    self.output(&format!("Terminal height is currently set to {height}.\n")).await;
                }
            } else if let Some(_) = match_keyword(var, "idle", 4) {
                let now = Timestamp::new();
                self.output("Your idle time is").await;
                self.print_time_long((now.unix() - self.idle_since().unix()) / 60).await;
                self.output(".\n").await;
            } else if let Some(_) = match_keyword(var, "time_format", 11) {
                self.output("Your time format is ").await;
                if let Some(format) = self.get_sys_var("time_format") {
                    match format.as_str() {
                        "verbose" => self.output("verbose.\n").await,
                        "both" => self.output("both verbose and terse.\n").await,
                        "terse" => self.output("terse.\n").await,
                        _ => self.output("unknown.\n").await,
                    }
                } else {
                    self.output("the default: ").await;
                    match DEFAULTS.get(&Text::from("time_format")) {
                        Some(s) if s.as_str() == "verbose" => self.output("verbose.\n").await,
                        Some(s) if s.as_str() == "both" => self.output("both verbose and terse.\n").await,
                        Some(s) if s.as_str() == "terse" => self.output("terse.\n").await,
                        _ => self.output("verbose.\n").await,
                    }
                }
            } else if let Some(_) = match_keyword(var, "uptime", 6) {
                let uptime = if let Some(system_up) = system_uptime().await {
                    // TODO: Replace with actual server start uptime when available
                    system_up / 60
                } else {
                    let now = Timestamp::new();
                    // TODO: Replace with actual server start time when available
                    (now.unix() / 60)
                };

                self.output("This server has been running for").await;
                self.print_time_long(uptime).await;
                self.output(".\n").await;

                if let Some(system_up) = system_uptime().await {
                    let system_minutes = system_up / 60;
                    self.output("(This machine has been running for").await;
                    self.print_time_long(system_minutes).await;
                    self.output(".)\n").await;
                }
            } else if let Some(_) = match_keyword(var, "version", 7) {
                self.output(&format!("Phoenix server version: {VERSION}\n")).await;
            } else if let Some(_) = match_keyword(var, "width", 5) {
                if let Some(telnet) = self.telnet() {
                    let width = telnet.width();
                    self.output(&format!("Terminal width is currently set to {width}.\n")).await;
                }
            } else {
                self.output(&format!("Unknown system variable: \"{var}\"\n")).await;
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn do_also(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /also <sendlist>\n").await;
            return Ok(());
        }

        if let Some(last_msg) = self.last_message() {
            let sendlist = Sendlist::new(&self, args, false, true, true).await;
            self.send_message(&sendlist, last_msg.text()).await?;
        } else {
            self.output("You have no previous message to resend.\n").await;
        }

        Ok(())
    }

    #[framed]
    pub async fn do_oops(&self, args: &str) -> tokio::io::Result<()> {
        if args.is_empty() {
            self.output("Usage: /oops <sendlist> OR /oops text [<message>]\n").await;
            return Ok(());
        }

        if let Some(text_args) = match_keyword(args, "text", 4) {
            let text = text_args.trim();
            if !text.is_empty() {
                self.set_oops_text(text);
                self.output(&format!("Your /oops text is now \"{text}\".\n")).await;
            } else {
                let oops_text = self.oops_text();
                self.output(&format!("Your /oops text is currently \"{oops_text}\".\n")).await;
            }
        } else {
            if let Some(last_msg) = self.last_message() {
                let sendlist = Sendlist::new(&self, args, false, true, true).await;
                let text = last_msg.text().clone();
                let to = last_msg.to().clone();

                self.send_message(&to, &*self.oops_text()).await?;
                self.send_message(&sendlist, &text).await?;
                self.set_last_sendlist(Some(sendlist.clone()));
            } else {
                self.output("You have no previous message to resend.\n").await;
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn do_help(&self, args: &str) -> tokio::io::Result<()> {
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

        Ok(())
    }

    #[framed]
    pub async fn do_reset(&self) -> tokio::io::Result<()> {
        self.reset_idle(1).await;

        Ok(())
    }

    #[framed]
    pub async fn do_message(&self, line: &str) -> tokio::io::Result<()> {
        let (msg_start, sendlist_str, last_explicit, is_explicit) = message_start(line);
        let msg_start = msg_start.trim();

        if is_explicit {
            self.set_last_explicit(last_explicit);
        }

        let sendlist = if sendlist_str.is_empty() {
            if let Some(last) = self.last_sendlist() {
                last.clone()
            } else {
                self.output("\x07\x07You have no previous sendlist. (message not sent)\n").await;
                return Ok(());
            }
        } else if sendlist_str.eq_ignore_ascii_case("default") {
            if let Some(default) = self.default_sendlist() {
                default.clone()
            } else {
                self.output("\x07\x07You have no default sendlist. (message not sent)\n").await;
                return Ok(());
            }
        } else {
            Sendlist::new(&self, &sendlist_str, false, true, true).await
        };

        self.set_last_sendlist(Some(sendlist.clone()));

        if msg_start.is_empty() {
            let sendlist_typed = sendlist.typed();
            if sendlist_str == "default" {
                self.output("\x07\x07There is no message after \"default\". (message not sent)\n").await;
            } else if is_explicit {
                self.output(&format!("\x07\x07There is no message after \"{sendlist_typed}:\". (message not sent)\n")).await;
            } else {
                self.output(&format!("\x07\x07There is no message after \"{sendlist_typed};\". (message not sent)\n")).await;
            }
            return Ok(());
        }

        self.send_message(&sendlist, msg_start).await?;

        Ok(())
    }

    #[framed]
    pub async fn send_message(&self, sendlist: &Sendlist, text: &str) -> tokio::io::Result<()> {
        let mut who = OrdSet::new();
        let now = Timestamp::new();
        let count = sendlist.expand(&mut who, Some(self.clone())).await;

        // If no recipients, handle errors and return early
        if count == 0 {
            if !sendlist.errors().is_empty() {
                self.output("\x07\x07").await;
                self.output(&sendlist.errors().to_string()).await;
            }
            self.output("(message not sent)\n").await;
            return Ok(());
        }

        // Check sender status and warn if necessary
        match self.away() {
            AwayState::Gone => {
                self.output("[Warning: you are listed as \"gone\".]\n").await;
            }
            AwayState::Busy => {
                if (now.unix() - self.idle_since().unix()) >= 600 {
                    self.output("\x07").await;
                    self.output("[Warning: you are still listed as \"busy\".]\n").await;
                }
            }
            _ => {}
        }

        self.reset_idle(REPORT_IDLE_DEFAULT).await;

        // Create and send message
        let output_type = if count > 1 || !sendlist.discussions().is_empty() { OutputType::PublicMessage } else { OutputType::PrivateMessage };

        let msg = Message::new(output_type, self.name(), Arc::new(sendlist.clone()), text);
        if let Output::Message(message) = &msg {
            self.set_last_message(Some(message.clone()));
        }

        for session in &who {
            session.enqueue(msg.clone()).await.ok();
        }

        // Output confirmation with recipient status details
        self.output("(message sent to ").await;
        let mut first = true;
        for session in &who {
            if first {
                first = false;
            } else {
                self.output(", ").await;
            }

            let mut flag = false;
            self.output(&session.name().to_string()).await;
            self.output(&session.name().column_display()).await;

            // Check if detached
            if session.telnet().is_none() {
                self.output(if flag { ", " } else { " (" }).await;
                flag = true;
                self.output("detached").await;
            }

            // Check away status
            if session.away() != AwayState::Here {
                self.output(if flag { ", " } else { " (" }).await;
                flag = true;
                match session.away() {
                    AwayState::Here => {}
                    AwayState::Away => self.output("\"away\"").await,
                    AwayState::Busy => self.output("\"busy\"").await,
                    AwayState::Gone => self.output("\"gone\"").await,
                }
            }

            // Check idle time
            let idle_minutes = (now.unix() - session.idle_since().unix()) / 60;
            if idle_minutes > 0 {
                self.output(if flag { ", " } else { " (" }).await;
                flag = true;
                self.output("idle: ").await;

                let hours = idle_minutes / 60;
                let minutes = idle_minutes % 60;
                let days = hours / 24;
                let hours = hours % 24;

                if days > 0 {
                    self.output(&format!("{days}d{hours:02}:{minutes:02}")).await;
                } else if hours > 0 {
                    self.output(&format!("{hours}:{minutes:02}")).await;
                } else {
                    let s = if minutes == 1 { "" } else { "s" };
                    self.output(&format!("{minutes} minute{s}")).await;
                }
            }

            if flag {
                self.output(")").await;
            }
        }

        // Handle discussions
        if !sendlist.discussions().is_empty() {
            if !first {
                self.output("; ").await;
            }
            let disc_count = sendlist.discussions().len();
            let s = if disc_count == 1 { "" } else { "s" };
            self.output(&format!("discussion{s} ")).await;
            self.print_discussions(&sendlist.discussions()).await;

            // Set discussion idle times
            for disc in &sendlist.discussions() {
                disc.set_idle_since(now.clone());
            }
        }

        // Final output with count
        if count > 1 {
            self.output(&format!(".) [{count} people]\n")).await;
        } else if count == 1 && !sendlist.discussions().is_empty() {
            self.output(".) [1 person]\n").await;
        } else {
            self.output(".)\n").await;
        }

        // Show any errors at the end
        if !sendlist.errors().is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors().to_string()).await;
        }

        Ok(())
    }

    #[framed]
    pub async fn get_who_set(&self, args: &str) -> (OrdSet<Session>, String, String) {
        let mut who = OrdSet::new();
        let mut errors = String::new();
        let mut msg = String::new();

        // Check if anyone is signed on at all.
        let total_sessions = SESSIONS.snapshot().values().filter(|s| s.signed_on()).count();
        if total_sessions == 0 {
            self.output("Nobody is signed on.\n").await;
            return (who, errors, msg);
        }

        // Parse comma-separated arguments for filter keywords
        let mut everyone = args.is_empty();
        let mut here = false;
        let mut away = false;
        let mut busy = false;
        let mut gone = false;
        let mut attached = false;
        let mut detached = false;
        let mut active = false;
        let mut inactive = false;
        let mut idle = false;
        let mut unidle = false;
        let mut privileged = false;
        let mut guests = false;
        let mut sendlist_args = Vec::new();

        if !args.is_empty() {
            for arg in args.split(',') {
                let arg = arg.trim();
                if arg.is_empty() {
                    continue;
                }

                match arg.to_lowercase().as_str() {
                    "here" => here = true,
                    "away" => away = true,
                    "busy" => busy = true,
                    "gone" => gone = true,
                    "attached" => attached = true,
                    "detached" => detached = true,
                    "active" => active = true,
                    "inactive" => inactive = true,
                    "idle" => idle = true,
                    "unidle" => unidle = true,
                    "privileged" => privileged = true,
                    "guests" => guests = true,
                    "everyone" => everyone = true,
                    "all" => {
                        active = true;
                        attached = true;
                    }
                    _ => {
                        // Not a recognized filter keyword, save for sendlist expansion
                        sendlist_args.push(arg);
                    }
                }
            }
        }

        let has_filters = here || away || busy || gone || attached || detached || active || inactive || idle || unidle || privileged || guests || everyone;

        // Handle sendlist expansion first (matches go first in the set)
        if !sendlist_args.is_empty() {
            let sendlist_arg_str = sendlist_args.join(",");
            let sendlist = Sendlist::new(&self, &sendlist_arg_str, true, true, true).await;
            let _total = sendlist.expand(&mut who, None).await;

            if !sendlist.errors().is_empty() {
                errors = sendlist.errors().to_string();
            }
        }

        // Add filter matches to the set
        if has_filters {
            let now = Timestamp::new();
            for session in SESSIONS.snapshot().values().filter(|s| s.signed_on()) {
                let idle_time = (now.unix() - session.idle_since().unix()) / 60;
                let is_active = match session.away() {
                    AwayState::Here if session.telnet().is_some() && idle_time < 60 => true,
                    AwayState::Here if idle_time < 10 => true,
                    AwayState::Away if session.telnet().is_some() && idle_time < 10 => true,
                    _ => false,
                };
                let include = match session.away() {
                    AwayState::Here if here => true,
                    AwayState::Away if away => true,
                    AwayState::Busy if busy => true,
                    AwayState::Gone if gone => true,
                    _ if attached && session.telnet().is_some() => true,
                    _ if detached && session.telnet().is_none() => true,
                    _ if active && is_active => true,
                    _ if inactive && !is_active => true,
                    _ if idle && idle_time >= 10 => true,
                    _ if unidle && idle_time < 10 => true,
                    _ if privileged && session.privileged() => true,
                    _ if guests && session.priv_level() == 0 => true,
                    _ if everyone => true,
                    _ => false,
                };

                if include {
                    who.insert(session.clone());
                }
            }
        }

        // Handle no matches case
        if who.is_empty() {
            if has_filters {
                let mut filter_list = Vec::new();
                if here {
                    filter_list.push("\"here\"");
                }
                if away {
                    filter_list.push("\"away\"");
                }
                if busy {
                    filter_list.push("\"busy\"");
                }
                if gone {
                    filter_list.push("\"gone\"");
                }
                if attached {
                    filter_list.push("attached");
                }
                if detached {
                    filter_list.push("detached");
                }
                if active {
                    filter_list.push("active");
                }
                if inactive {
                    filter_list.push("inactive");
                }
                if idle {
                    filter_list.push("idle");
                }
                if unidle {
                    filter_list.push("unidle");
                }
                if privileged {
                    filter_list.push("privileged");
                }
                if guests {
                    filter_list.push("a guest");
                }

                if !filter_list.is_empty() {
                    let mut output = "Nobody is ".to_string();
                    if filter_list.len() == 1 {
                        output.push_str(filter_list[0]);
                    } else {
                        for (i, item) in filter_list.iter().enumerate() {
                            if i == filter_list.len() - 1 {
                                output.push_str(" or ");
                                output.push_str(item);
                            } else if i > 0 {
                                output.push_str(", ");
                                output.push_str(item);
                            } else {
                                output.push_str(item);
                            }
                        }
                    }
                    output.push_str(".\n");
                    self.output(&output).await;
                }
            }
            if !errors.is_empty() {
                self.output("\x07\x07").await; // Bell characters
                self.output(&errors).await;
            }
            return (who, String::new(), String::new());
        }

        // Generate message for successful matches with filters
        if has_filters {
            let others = total_sessions - who.len();
            if others == 1 {
                msg = "(There is 1 other person signed on.)\n".to_string();
            } else if others > 0 {
                msg = format!("(There are {others} other people signed on.)\n");
            }
        }

        (who, errors, msg)
    }

    #[framed]
    pub async fn session_matches(&self, name: &str, matches: &OrdSet<Session>) {
        // Convert UnquotedUnderscore characters to regular underscores for display
        let display_name = name.chars().map(|c| {
            if c as u8 == UNQUOTED_UNDERSCORE {
                '_'
            } else {
                c
            }
        }).collect::<String>();

        if !matches.is_empty() {
            let count = matches.len();

            for (i, session) in matches.iter().enumerate() {
                match i {
                    0 if count == 1 => self.output(&format!("\"{display_name}\" matches one name: ")).await,
                    0 => self.output(&format!("\"{display_name}\" matches {count} names: ")).await,
                    _ if i == count - 1 => self.output(" and ").await,
                    _ => self.output(", ").await,
                };

                self.output(&session.name()).await;
            }

            self.output(".\n").await;
        } else {
            self.output(&format!("No names matched \"{display_name}\".\n")).await;
        }
    }

    #[framed]
    pub async fn discussion_matches(&self, name: &str, matches: &OrdSet<Discussion>) {
        // Convert UnquotedUnderscore characters to regular underscores for display
        let display_name = name.chars().map(|c| {
            if c as u8 == UNQUOTED_UNDERSCORE {
                '_'
            } else {
                c
            }
        }).collect::<String>();

        if !matches.is_empty() {
            let count = matches.len();

            for (i, disc) in matches.iter().enumerate() {
                match i {
                    0 if count == 1 => self.output(&format!("\"{display_name}\" matches one discussion: ")).await,
                    0 => self.output(&format!("\"{display_name}\" matches {count} discussions: ")).await,
                    _ if i == count - 1 => self.output(" and ").await,
                    _ => self.output(", ").await,
                };

                self.output(disc.name()).await;
            }

            self.output(".\n").await;
        } else {
            self.output(&format!("No discussions matched \"{display_name}\".\n")).await;
        }
    }

    /// Print a set of sessions (comma-separated).
    pub async fn print_sessions(&self, sessions: &OrdSet<Session>) {
        if let Some((first, rest)) = sessions.iter().next().map(|first| (first, sessions.iter().skip(1))) {
            self.output(&first.name().to_string()).await;
            for session in rest {
                self.output(", ").await;
                self.output(&session.name().to_string()).await;
            }
        }
    }

    /// Print a set of discussions (comma-separated).
    pub async fn print_discussions(&self, discussions: &OrdSet<Discussion>) {
        if let Some((first, rest)) = discussions.iter().next().map(|first| (first, discussions.iter().skip(1))) {
            self.output(first.name()).await;
            for discussion in rest {
                self.output(", ").await;
                self.output(discussion.name()).await;
            }
        }
    }

    /// Output an item from a list (helper for comma-separated lists).
    pub async fn list_item(&self, flag: &mut bool, last: &mut String, str_val: &str) {
        if *flag {
            if !last.is_empty() {
                self.output(", ").await;
                self.output(&*last).await;
            }
            *last = str_val.to_string();
        } else {
            self.output(str_val).await;
            *flag = true;
        }
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Session {}

impl PartialOrd for Session {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Session {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(&other.id())
    }
}

impl std::hash::Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

//#[cfg(test)]
const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<AwayState>();
    assert_send_sync_static::<LoginState>();
    assert_send_sync_static::<Session>();
    assert_send_sync_static::<SessionInner>();
    assert_send_sync_static::<SessionType>();
};

use crate::atomic::{AtomicAwayState, AtomicHashMap, AtomicMessageOption, AtomicName, AtomicSendlistOption, AtomicTelnetOption, AtomicText, AtomicUserOption};
use crate::constants::*;
use crate::discussion::Discussion;
use crate::name::Name;
use crate::output::*;
use crate::sendlist::{message_start, Sendlist};
use crate::server::Server;
use crate::telnet::{Telnet, TELNET_ENABLED};
use crate::text::Text;
use crate::timestamp::{system_uptime, Timestamp};
use crate::user::{verify_password, User, UserManager};
use crate::{getword, match_keyword, match_name, VERSION};
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

const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

static INITS: LazyLock<AtomicHashMap<usize, Session>> = LazyLock::new(AtomicHashMap::default);
static SESSIONS: LazyLock<AtomicHashMap<usize, Session>> = LazyLock::new(AtomicHashMap::default);
static DISCUSSIONS: LazyLock<AtomicHashMap<Text, Discussion>> = LazyLock::new(AtomicHashMap::default);
static SESSION_COUNTER: AtomicUsize = AtomicUsize::new(1);
static DEFAULTS: LazyLock<AtomicHashMap<Text, Text>> =
    LazyLock::new(|| AtomicHashMap::from(im::HashMap::from(&[(Text::from("time_format"), Text::from("verbose"))][..])));
static USER_MANAGER: LazyLock<UserManager> = LazyLock::new(UserManager::new);

/// Session handle.
#[derive(Debug, Clone)]
pub struct Session(pub Arc<SessionInner>);

#[derive(Debug)]
pub struct SessionInner
where
    Self: Send + Sync + 'static,
{
    // Immutable fields
    pub id: usize,
    pub server: Server,

    // User and connection state
    pub user: AtomicUserOption,
    pub telnet: AtomicTelnetOption,

    // I/O handling
    pub output_buffer: Mutex<String>,
    pub pending: Mutex<OutputStream>,

    // User preferences and variables
    pub user_vars: ArcSwap<HashMap<Text, Text>>,
    pub sys_vars: ArcSwap<HashMap<Text, Text>>,
    pub signal_public: AtomicBool,
    pub signal_private: AtomicBool,

    // Session state
    pub login_time: ArcSwap<Timestamp>,
    pub idle_since: ArcSwap<Timestamp>,
    pub away: AtomicAwayState,
    pub priv_level: AtomicI32,
    pub name: AtomicName,
    pub signed_on: AtomicBool,
    pub closing: AtomicBool,

    // Message handling
    pub last_message: AtomicMessageOption,
    pub default_sendlist: AtomicSendlistOption,
    pub last_sendlist: AtomicSendlistOption,
    pub last_explicit: AtomicText,
    pub reply_sendlist: AtomicText,
    pub oops_text: AtomicText,
}

/// Pre-login session for managing connections before authentication.
#[derive(Debug)]
pub struct LoginSession
where
    Self: Send + Sync + 'static,
{
    pub server: Server,
    pub user: AtomicUserOption,
    pub telnet: AtomicTelnetOption,
    pub login_state: ArcSwap<LoginState>,
    pub login_timeout: ArcSwapOption<AbortHandle>,
    pub attempts: AtomicI32,
    pub lines: Mutex<VecDeque<Text>>,
    pub output_buffer: Mutex<String>,
    pub pending: Mutex<OutputStream>,
}

/// Enum for session objects that can be associated with a Telnet connection.
/// Handles both pre-login and post-login states.
#[derive(Debug)]
pub enum SessionConnection
where
    Self: Send + Sync + 'static,
{
    PreLogin(LoginSession),
    LoggedIn(Session),
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

impl LoginSession {
    pub fn new(server: Server) -> Self {
        Self {
            server,
            user: AtomicUserOption::new(None),
            telnet: AtomicTelnetOption::new(None),
            login_state: ArcSwap::new(Arc::new(LoginState::PreLogin)),
            login_timeout: ArcSwapOption::new(None),
            attempts: AtomicI32::new(0),
            lines: Mutex::new(VecDeque::new()),
            output_buffer: Mutex::new(String::new()),
            pending: Mutex::new(OutputStream::new()),
        }
    }

    pub async fn handle_input(&self, line: Text) {
        self.pending().await.dequeue().await;

        match **self.login_state.load() {
            LoginState::PreLogin => self.save_input_line(line).await,
            LoginState::AwaitingLogin => self.handle_login_input(line).await,
            LoginState::AwaitingPassword => self.handle_password_input(line).await,
            LoginState::AwaitingName => self.handle_name_input(line).await,
            LoginState::AwaitingBlurb => self.handle_blurb_input(line).await,
            LoginState::AwaitingTransferConfirmation => self.handle_transfer_input(line).await,
        }

        self.enqueue_output().await;
    }

    pub async fn handle_login_input(&self, line: Text) {
        let line = line.trim();
        if let Some(_args) = match_keyword(&line, "/bye", 4) {
            self.do_bye().await;
            return;
        }
        if line.is_empty() {
            if let Some(telnet) = self.telnet() {
                telnet.output("login: ").await;
            }
            return;
        }
        let user = (*USER_MANAGER).get_user(&line).await;
        self.user.set(user.clone());
        if let Some(_user) = &user {
            if let Some(telnet) = self.telnet() {
                telnet.set_do_echo(false);
            }
            self.switch_login_state(LoginState::AwaitingPassword, Some("password: ")).await;
        } else {
            self.output("Invalid login.\n").await;
            let attempts = self.attempts().fetch_add(1, Ordering::Relaxed) + 1;
            if attempts >= Session::MAX_LOGIN_ATTEMPTS {
                self.close(true).await;
                return;
            }
            if let Some(telnet) = self.telnet() {
                telnet.output("login: ").await;
            }
        }
    }

    pub async fn handle_password_input(&self, line: Text) {
        if let Some(telnet) = self.telnet() {
            telnet.output("\n").await;
            telnet.set_do_echo(true);
        }
        (*USER_MANAGER).update_all().await.ok();

        let valid = if let Some(user) = self.user() {
            if let Some(password) = user.password() {
                verify_password(&line, &password)
            } else {
                false
            }
        } else {
            false
        };

        if !valid {
            self.output("Login incorrect.\n").await;
            let attempts = self.attempts().fetch_add(1, Ordering::Relaxed) + 1;
            if attempts >= Session::MAX_LOGIN_ATTEMPTS {
                self.close(true).await;
                return;
            }
            self.switch_login_state(LoginState::AwaitingLogin, Some("login: ")).await;
            self.user.set(None);
            return;
        }

        if let Some(user) = self.user() {
            if user.reserved().is_empty() {
                self.output("You don't have any reserved names.\n").await;
                self.close(true).await;
                return;
            }

            let reserved = user.reserved();
            if reserved.len() == 1 {
                let name = reserved[0].clone();
                if self.check_name_availability(&name, false, false).await {
                    self.switch_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await;
                }
                return;
            }
        }

        self.print_reserved_names().await;
        self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
    }

    pub async fn handle_name_input(&self, line: Text) {
        let line = line.trim();
        let name = if line.is_empty() {
            if let Some(user) = self.user() {
                if let Some(reserved) = user.reserved().front() {
                    reserved.clone()
                } else {
                    if let Some(telnet) = self.telnet() {
                        telnet.output("Enter name: ").await;
                    }
                    return;
                }
            } else {
                return;
            }
        } else {
            Text::new(line)
        };

        if self.check_name_availability(&name, false, false).await {
            self.switch_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await;
        }
    }

    pub async fn handle_blurb_input(&self, line: Text) {
        let name = if let Some(user) = self.user() {
            if let Some(reserved) = user.reserved().front() {
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

        let blurb_text = if line.is_empty() {
            if let Some(user) = self.user() {
                user.blurb().as_ref().map(|b| b.to_string()).unwrap_or_else(|| String::new())
            } else {
                String::new()
            }
        } else {
            line.to_string()
        };

        // Create a full Session to complete the login process
        if let Some(telnet) = self.telnet() {
            let session = Session::new(self.server().clone(), telnet).await;

            // Transfer the login state to the session
            if let Some(user) = self.user() {
                session.set_user(Some(user.clone()));
                session.set_priv_level(user.priv_level());
                user.add_session(session.clone());
            }

            // Set the name and blurb
            session.set_name(Name::new(&name, Some(blurb_text)));
            session.set_signed_on(true);

            // Add to global sessions and remove from inits
            SESSIONS.insert(session.id(), session.clone());
            // Note: INITS removal would need the session ID, which we don't have from LoginSession

            // Send entry notification
            session.notify_entry().await;

            // Welcome message and automatic commands
            session.output("\n\nWelcome to Phoenix.  Type \"/help\" for a list of commands.\n\n").await;

            // Make sure discussion A exists
            match session.find_sendable("A", false, true, true, true).await {
                (_, _, None, _) => {
                    let disc = Discussion::new(None, "A", "General Discussion", true).await;
                    DISCUSSIONS.insert(Text::from("A"), disc);
                }
                _ => {}
            }

            // Automatic commands
            session.do_join("A").await;
            session.do_send("A").await;
            session.do_who("").await;
            session.do_howmany("").await;

            if let Some(telnet) = session.telnet() {
                telnet.reset_history().await;
            }
        }
    }

    pub async fn handle_transfer_input(&self, line: Text) {
        let line = line.trim();
        if match_keyword(&line, "yes", 1).is_none() {
            self.output("Session not transferred.\n").await;
            self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
            return;
        }

        let name = if let Some(user) = self.user() {
            if let Some(reserved) = user.reserved().front() {
                reserved.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        if self.check_name_availability(&name, true, true).await {
            self.output("(That session is now gone.)\n").await;
            self.switch_login_state(LoginState::AwaitingBlurb, Some("Enter blurb: ")).await;
        }
    }

    pub async fn print_reserved_names(&self) {
        self.output("\nYour reserved names are:\n\n").await;
        if let Some(user) = self.user() {
            for (i, name) in user.reserved().iter().enumerate() {
                self.output(&format!("    {}: {}\n", i + 1, name)).await;
            }
        }
        self.output("\n").await;
    }

    pub async fn switch_login_state(&self, state: LoginState, prompt: Option<&str>) {
        self.set_login_state(state);
        if let Some(prompt) = prompt {
            if let Some(telnet) = self.telnet() {
                telnet.output(prompt).await;
            }
        }
    }

    pub async fn acknowledge_output(&self) {
        self.pending().await.acknowledge().await;
    }

    pub async fn enqueue_output(&self) {
        let mut output_buffer = self.output_buffer().await;
        if !output_buffer.is_empty() {
            let text_output = TextOutput::new(output_buffer.clone());
            self.pending().await.enqueue(self.telnet().as_ref(), text_output).await;
            output_buffer.clear();
        }
    }

    /// Get the output buffer.
    pub async fn output_buffer(&self) -> tokio::sync::MutexGuard<'_, String> {
        self.output_buffer.lock().await
    }

    pub async fn lines(&self) -> tokio::sync::MutexGuard<'_, VecDeque<Text>> {
        self.lines.lock().await
    }

    pub fn attempts(&self) -> &AtomicI32 {
        &self.attempts
    }

    pub fn user(&self) -> Option<User> {
        self.user.snapshot()
    }

    pub fn telnet(&self) -> Option<Telnet> {
        self.telnet.snapshot()
    }

    pub async fn pending(&self) -> tokio::sync::MutexGuard<'_, OutputStream> {
        self.pending.lock().await
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    pub fn login_state(&self) -> LoginState {
        (**self.login_state.load()).clone()
    }

    pub fn set_login_state(&self, state: LoginState) {
        self.login_state.store(Arc::new(state));
    }

    pub fn login_timeout(&self) -> Option<AbortHandle> {
        self.login_timeout.load_full().map(|arc| (*arc).clone())
    }

    pub fn set_login_timeout(&self, timeout: Option<AbortHandle>) {
        self.login_timeout.store(timeout.map(Arc::new));
    }

    pub fn set_user(&self, user: Option<User>) {
        self.user.set(user);
    }

    pub fn set_telnet(&self, telnet: Option<Telnet>) {
        self.telnet.set(telnet);
    }

    pub async fn output(&self, text: impl AsRef<str>) {
        self.output_buffer().await.push_str(text.as_ref());
    }

    pub async fn output_next(&self, telnet: &Telnet) -> bool {
        self.pending().await.send_next(telnet).await
    }

    pub async fn do_bye(&self) {
        self.close(true).await;
    }

    pub async fn close(&self, drain: bool) {
        if let Some(telnet) = self.telnet() {
            telnet.close(drain).await;
        }
    }

    pub async fn save_input_line(&self, line: Text) {
        self.lines().await.push_back(line);
    }

    pub async fn init_login_sequence(&self) {
        self.switch_login_state(LoginState::AwaitingLogin, Some("login: ")).await;
    }

    pub async fn check_name_availability(&self, name: &str, double_check: bool, _transferring: bool) -> bool {
        if name.eq_ignore_ascii_case("me") {
            self.output("The keyword \"me\" is reserved.  Choose another name.\n").await;
            self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
            return false;
        }

        if let Some((reserved, found_user)) = (*USER_MANAGER).find_reserved(name).await {
            let my_user = self.user();
            let found_user_ref = Some(found_user);
            match (my_user, found_user_ref) {
                (Some(my_user), Some(found_user)) if my_user.username() == found_user.username() => {
                    let now = if double_check { " now" } else { "" };
                    self.output(&format!("\"{reserved}\" is{now} a reserved name.  Choose another.\n")).await;
                    self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
                    return false;
                }
                _ => {}
            }
        }

        // Note: LoginSession doesn't have find_sendable, so we need to check SESSIONS and DISCUSSIONS directly
        // Check for existing sessions with this name
        for (_, existing_session) in SESSIONS.iter() {
            if existing_session.name().name().eq_ignore_ascii_case(name) {
                let my_user = self.user();
                let their_user = existing_session.user();
                match (my_user, their_user) {
                    (Some(my_user), Some(their_user)) if my_user.username() == their_user.username() && existing_session.priv_level() > 0 => {
                        if let Some(_their_telnet) = existing_session.telnet() {
                            if _transferring {
                                self.output("Transferring active session...\n").await;
                                // TODO: Implement session transfer for LoginSession
                                return false;
                            } else {
                                let now = if double_check { " now" } else { "" };
                                self.output(&format!("You are{now} attached elsewhere under that name.\n")).await;
                                self.switch_login_state(LoginState::AwaitingTransferConfirmation, Some("Transfer active session? [no] ")).await;
                            }
                        } else {
                            self.output("Attaching to detached session...\n").await;
                            // TODO: Implement session attachment for LoginSession
                            return false;
                        }
                    }
                    _ => {
                        let found_name = existing_session.name();
                        let already = if double_check { "now" } else { "already" };
                        self.output(&format!("The name \"{found_name}\" is {already} in use.  Choose another.\n")).await;
                        self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
                    }
                }
                return false;
            }
        }

        // Check for discussion name conflicts
        for (disc_name, _) in DISCUSSIONS.iter() {
            if disc_name.eq_ignore_ascii_case(name) {
                let already = if double_check { "now" } else { "already" };
                self.output(&format!("There is {already} a discussion named \"{disc_name}\".  Choose another name.\n")).await;
                self.switch_login_state(LoginState::AwaitingName, Some("Enter name: ")).await;
                return false;
            }
        }

        true
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
            user: AtomicUserOption::new(None),
            telnet: AtomicTelnetOption::new(Some(telnet.clone())),
            output_buffer: Mutex::new(String::new()),
            pending: Mutex::new(OutputStream::new()),
            user_vars: ArcSwap::new(Arc::new(HashMap::new())),
            sys_vars: ArcSwap::new(Arc::new(HashMap::new())),
            signal_public: AtomicBool::new(true),
            signal_private: AtomicBool::new(true),
            login_time: ArcSwap::new(Arc::new(now)),
            idle_since: ArcSwap::new(Arc::new(now)),
            away: AtomicAwayState::default(),
            priv_level: AtomicI32::new(0),
            name: AtomicName::new(Name::new("", None)),
            signed_on: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            last_message: AtomicMessageOption::new(None),
            default_sendlist: AtomicSendlistOption::new(None),
            last_sendlist: AtomicSendlistOption::new(None),
            last_explicit: AtomicText::new(Text::default()),
            reply_sendlist: AtomicText::new(Text::default()),
            oops_text: AtomicText::new(Text::from("Oops!  Sorry, that last message was intended for someone else...")),
        };

        let session = Session(Arc::new(inner));

        // Add to initializing sessions
        INITS.insert(id, session.clone());

        // Set telnet session
        telnet.set_session(session.clone());

        session
    }

    /// Get the session ID.
    pub fn id(&self) -> usize {
        self.0.id
    }

    /// Get the `Server` object.
    pub fn server(&self) -> Server {
        self.0.server.clone()
    }

    /// Get the `User` object, if any.
    pub fn user(&self) -> Option<User> {
        self.0.user.snapshot()
    }

    /// Set the `User` object, if any.
    pub fn set_user(&self, value: Option<User>) {
        self.0.user.set(value);
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
        if self.0.telnet.snapshot().is_some() {
            " "
        } else {
            "~"
        }
    }

    /// Get the login timeout Tokio task `AbortHandle`, if any.
    /// Note: This method is for compatibility - login_timeout is not part of SessionInner
    pub fn login_timeout(&self) -> Option<AbortHandle> {
        None // SessionInner doesn't have login_timeout field
    }

    /// Set the login timeout Tokio task `AbortHandle`, if any.
    /// Note: This method is for compatibility - login_timeout is not part of SessionInner
    pub fn set_login_timeout(&self, _value: Option<AbortHandle>) {
        // SessionInner doesn't have login_timeout field - this is a no-op
    }

    /// Get the `LoginState`.
    /// Note: This method is for compatibility - login_state is not part of SessionInner
    pub fn login_state(&self) -> LoginState {
        LoginState::PreLogin // SessionInner doesn't have login_state field
    }

    /// Set the `LoginState`.
    /// Note: This method is for compatibility - login_state is not part of SessionInner
    pub fn set_login_state(&self, _value: LoginState) {
        // SessionInner doesn't have login_state field - this is a no-op
    }

    /// Add a line to the pending input line queue.
    /// Note: This method is for compatibility - lines are not part of SessionInner
    pub async fn add_pending_line(&self, _line: String) {
        // SessionInner doesn't have lines field - this is a no-op
    }

    /// Take the next pending input line (FIFO).
    /// Note: This method is for compatibility - lines are not part of SessionInner
    pub async fn take_pending_line(&self) -> Option<String> {
        None // SessionInner doesn't have lines field
    }

    /// Check if there are pending input lines.
    /// Note: This method is for compatibility - lines are not part of SessionInner
    pub async fn has_pending_lines(&self) -> bool {
        false // SessionInner doesn't have lines field
    }

    /// Get count of pending input lines.
    /// Note: This method is for compatibility - lines are not part of SessionInner
    pub async fn pending_line_count(&self) -> usize {
        0 // SessionInner doesn't have lines field
    }

    /// Clear all pending input lines.
    /// Note: This method is for compatibility - lines are not part of SessionInner
    pub async fn clear_pending_lines(&self) {
        // SessionInner doesn't have lines field - this is a no-op
    }

    /// Take all pending input lines at once.
    /// Note: This method is for compatibility - lines are not part of SessionInner
    pub async fn take_all_pending_lines(&self) -> VecDeque<String> {
        VecDeque::new() // SessionInner doesn't have lines field
    }

    /// Get the output buffer.
    pub async fn output_buffer(&self) -> tokio::sync::MutexGuard<'_, String> {
        self.0.output_buffer.lock().await
    }

    /// Append text to output buffer.
    pub async fn output(&self, text: impl AsRef<str>) {
        let mut buffer = self.output_buffer().await;
        buffer.push_str(text.as_ref());
    }

    /// Get the `OutputStream`.
    pub async fn pending(&self) -> tokio::sync::MutexGuard<'_, OutputStream> {
        self.0.pending.lock().await
    }

    /// Set the `OutputStream`.
    pub async fn set_pending(&self, value: OutputStream) {
        *self.0.pending.lock().await = value;
    }

    /// Get a user variable.
    pub fn get_user_var(&self, key: impl AsRef<str>) -> Option<Text> {
        let vars = self.0.user_vars.load_full();
        vars.get(key.as_ref()).cloned()
    }

    /// Set a user variable.
    pub fn set_user_var(&self, key: impl Into<Text>, value: impl Into<Text>) {
        let vars = self.0.user_vars.load();
        let mut new_vars = (**vars).clone();
        new_vars.insert(key.into(), value.into());
        self.0.user_vars.store(Arc::new(new_vars));
    }

    /// Remove a user variable.
    pub fn remove_user_var(&self, key: impl AsRef<str>) -> Option<Text> {
        let vars = self.0.user_vars.load();
        let mut new_vars = (**vars).clone();
        let result = new_vars.remove(key.as_ref());
        self.0.user_vars.store(Arc::new(new_vars));
        result
    }

    /// Clear all user variables.
    pub fn clear_user_vars(&self) {
        self.0.user_vars.store(Arc::new(HashMap::new()));
    }

    /// Get a system variable.
    pub fn get_sys_var(&self, key: impl AsRef<str>) -> Option<Text> {
        let vars = self.0.sys_vars.load_full();
        vars.get(key.as_ref()).cloned()
    }

    /// Set a system variable.
    pub fn set_sys_var(&self, key: impl Into<Text>, value: impl Into<Text>) {
        let vars = self.0.sys_vars.load();
        let mut new_vars = (**vars).clone();
        new_vars.insert(key.into(), value.into());
        self.0.sys_vars.store(Arc::new(new_vars));
    }

    /// Remove a system variable.
    pub fn remove_sys_var(&self, key: impl AsRef<str>) -> Option<Text> {
        let vars = self.0.sys_vars.load();
        let mut new_vars = (**vars).clone();
        let result = new_vars.remove(key.as_ref());
        self.0.sys_vars.store(Arc::new(new_vars));
        result
    }

    /// Clear all system variables.
    pub fn clear_sys_vars(&self) {
        self.0.sys_vars.store(Arc::new(HashMap::new()));
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

    /// Get the away state.
    pub fn away(&self) -> AwayState {
        self.0.away.get()
    }

    /// Set the away state.
    pub fn set_away(&self, value: AwayState) {
        self.0.away.set(value);
    }

    /// Get the public signal flag.
    pub fn signal_public(&self) -> bool {
        self.0.signal_public.load(Ordering::Relaxed)
    }

    /// Set the public signal flag.
    pub fn set_signal_public(&self, value: bool) {
        self.0.signal_public.store(value, Ordering::Relaxed);
    }

    /// Get the private signal flag.
    pub fn signal_private(&self) -> bool {
        self.0.signal_private.load(Ordering::Relaxed)
    }

    /// Set the private signal flag.
    pub fn set_signal_private(&self, value: bool) {
        self.0.signal_private.store(value, Ordering::Relaxed);
    }

    /// Get the signed-on flag.
    pub fn signed_on(&self) -> bool {
        self.0.signed_on.load(Ordering::Relaxed)
    }

    /// Set the signed-on flag.
    pub fn set_signed_on(&self, value: bool) {
        self.0.signed_on.store(value, Ordering::Relaxed);
    }

    /// Get the closing flag.
    pub fn closing(&self) -> bool {
        self.0.closing.load(Ordering::Relaxed)
    }

    /// Set the closing flag.
    pub fn set_closing(&self, value: bool) {
        self.0.closing.store(value, Ordering::Relaxed);
    }

    /// Get the login attempts count.
    /// Note: This method is for compatibility - attempts is not part of SessionInner
    pub fn attempts(&self) -> i32 {
        0 // SessionInner doesn't have attempts field
    }

    /// Set the login attempts count.
    /// Note: This method is for compatibility - attempts is not part of SessionInner
    pub fn set_attempts(&self, _value: i32) {
        // SessionInner doesn't have attempts field - this is a no-op
    }

    /// Increment the login attempts count.
    /// Note: This method is for compatibility - attempts is not part of SessionInner
    pub fn increment_attempts(&self) -> i32 {
        0 // SessionInner doesn't have attempts field
    }

    /// Get the privilege level.
    pub fn priv_level(&self) -> i32 {
        self.0.priv_level.load(Ordering::Relaxed)
    }

    /// Set the privilege level.
    pub fn set_priv_level(&self, value: i32) {
        self.0.priv_level.store(value, Ordering::Relaxed);
    }

    /// Get the `Name` object.
    pub fn name(&self) -> Name {
        self.0.name.snapshot()
    }

    /// Get only the name from the `Name` object.
    pub fn name_only(&self) -> Text {
        self.0.name.borrow().name().clone()
    }

    /// Set the name.
    pub fn set_name(&self, value: impl AsRef<str>) {
        let blurb = self.blurb().map(|t| t.to_string());
        self.0.name.set(Name::new(value.as_ref(), blurb));
    }

    /// Check if a blurb is set.
    pub fn has_blurb(&self) -> bool {
        self.0.name.borrow().has_blurb()
    }

    /// Get the blurb, if any.
    pub fn blurb(&self) -> Option<Text> {
        self.0.name.borrow().blurb().cloned()
    }

    /// Set the blurb.
    pub fn set_blurb(&self, value: Option<impl AsRef<str>>) {
        let blurb = value.map(|s| s.as_ref().to_string());
        self.0.name.set(Name::new(self.name_only(), blurb));
    }

    /// Remove the blurb.
    pub fn remove_blurb(&self) {
        if self.has_blurb() {
            self.0.name.set(Name::new(self.name_only(), None));
        }
    }

    /// Set both name and blurb atomically.
    pub fn set_name_and_blurb(&self, name: impl AsRef<str>, blurb: Option<impl AsRef<str>>) {
        let blurb = blurb.map(|s| s.as_ref().to_string());
        self.0.name.set(Name::new(name.as_ref(), blurb));
    }

    /// Get the last message.
    pub fn last_message(&self) -> Option<Message> {
        self.0.last_message.snapshot()
    }

    /// Set the last message.
    pub fn set_last_message(&self, value: Option<Message>) {
        self.0.last_message.set(value);
    }

    /// Get the default sendlist.
    pub fn default_sendlist(&self) -> Option<Sendlist> {
        self.0.default_sendlist.snapshot()
    }

    /// Set the default sendlist.
    pub fn set_default_sendlist(&self, value: Option<Sendlist>) {
        self.0.default_sendlist.set(value);
    }

    /// Get the last sendlist.
    pub fn last_sendlist(&self) -> Option<Sendlist> {
        self.0.last_sendlist.snapshot()
    }

    /// Set the last sendlist.
    pub fn set_last_sendlist(&self, value: Option<Sendlist>) {
        self.0.last_sendlist.set(value);
    }

    /// Get the last explicit sendlist.
    pub fn last_explicit(&self) -> Text {
        self.0.last_explicit.snapshot()
    }

    /// Set the last explicit sendlist.
    pub fn set_last_explicit(&self, value: impl Into<Text>) {
        self.0.last_explicit.set(value.into());
    }

    /// Get the reply sendlist.
    pub fn reply_sendlist(&self) -> Text {
        self.0.reply_sendlist.snapshot()
    }

    /// Set the reply sendlist.
    pub fn set_reply_sendlist(&self, sendlist: impl Into<Text>) {
        let sendlist: Text = sendlist.into();

        // Quote if necessary
        let text =
            if sendlist.chars().any(|c| c == ' ' || c == ',' || c == ':' || c == ';' || c == '_') { Text::from(format!("\"{sendlist}\"")) } else { sendlist };

        self.0.reply_sendlist.set(text);
    }

    /// Get the oops text.
    pub fn oops_text(&self) -> Text {
        self.0.oops_text.snapshot()
    }

    /// Set the oops text.
    pub fn set_oops_text(&self, value: impl Into<Text>) {
        self.0.oops_text.set(value.into());
    }

    pub fn name_user(&self) -> Text {
        let name = self.name();
        let user = self.user();
        if let Some(user) = user {
            let username = user.username();
            let name_user = format!("{name} ({username})");
            Text::from(name_user)
        } else {
            name.name().clone()
        }
    }

    pub async fn close(&self, drain: bool) {
        let id = self.id();
        INITS.remove(&id);
        SESSIONS.remove(&id);

        if self.signed_on() {
            self.notify_exit().await;
        }
        self.set_signed_on(false);

        // Quit all discussions silently
        let disc_keys: Vec<_> = DISCUSSIONS.iter().map(|(key, _)| key.clone()).collect();
        for key in &disc_keys {
            if let Some(disc) = DISCUSSIONS.get(key) {
                disc.quit(&self).await;
            }
        }

        // Close telnet connection if attached
        if let Some(telnet) = self.telnet() {
            telnet.close(drain).await;
        }
        self.set_telnet(None);

        // Disassociate from user
        if let Some(user) = self.user() {
            user.remove_session(self);
            self.set_user(None);
        }
    }

    pub async fn transfer(&self, new_telnet: Telnet) {
        let old_telnet = self.telnet();
        self.set_telnet(Some(new_telnet.clone()));
        new_telnet.set_session(self.clone());

        if let Some(old) = old_telnet {
            let who = self.name_user();
            info!("Transfer: {who} from fd to new connection");
            old.output("*** This session has been transferred to a new connection. ***\n").await;
            old.close(true).await;
        }

        self.enqueue_others(TransferNotify::new(self.name())).await;
        self.pending().await.attach(&new_telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn attach(&self, telnet: Telnet) {
        self.set_telnet(Some(telnet.clone()));
        telnet.set_session(self.clone());

        let who = self.name_user();
        info!("Attach: {who} on new connection");

        self.enqueue_others(AttachNotify::new(self.name())).await;
        self.pending().await.attach(&telnet).await;
        self.output("*** End of reviewed output. ***\n").await;
        self.enqueue_output().await;
    }

    pub async fn detach(&self, telnet: &Telnet, intentional: bool) {
        if self.signed_on() && self.priv_level() > 0 {
            if let Some(t) = self.telnet() {
                if Arc::ptr_eq(&t.0, &telnet.0) {
                    let who = self.name_user();
                    if intentional {
                        info!("Detach: {who} (intentional)");
                    } else {
                        info!("Detach: {who} (accidental)");
                    };

                    self.enqueue_others(DetachNotify::new(self.name(), intentional)).await;
                    self.set_telnet(None);
                }
            }
        } else {
            self.close(true).await;
        }
    }

    pub async fn announce(message: &str) {
        for (_, session) in SESSIONS.iter() {
            session.output(message).await;
            session.enqueue_output().await;
        }

        for (_, session) in INITS.iter() {
            session.output(message).await;
            session.enqueue_output().await;
        }
    }

    pub async fn remove_discussion(name: Text) {
        DISCUSSIONS.remove(&name);
    }

    pub async fn acknowledge_output(&self) {
        self.pending().await.acknowledge().await;
    }

    pub async fn enqueue(&self, out: Output) {
        self.enqueue_output().await;
        if let Some(telnet) = self.telnet() {
            self.pending().await.enqueue(Some(&telnet), out).await;
        } else {
            self.pending().await.enqueue(None, out).await;
        }
    }

    pub async fn enqueue_output(&self) {
        let text = {
            let mut buf = self.output_buffer().await;
            if buf.is_empty() {
                return;
            }
            std::mem::take(&mut *buf)
        };

        if let Some(telnet) = self.telnet() {
            self.pending().await.enqueue(Some(&telnet), TextOutput::new(text)).await;
        } else {
            self.pending().await.enqueue(None, TextOutput::new(text)).await;
        }
    }

    pub async fn enqueue_others(&self, out: Output) {
        for (_, session) in SESSIONS.iter() {
            if &session != self {
                session.enqueue(out.clone()).await;
            }
        }
    }

    pub async fn output_next(&self, telnet: &Telnet) -> bool {
        self.pending().await.send_next(telnet).await
    }

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

        if do_sessions {
            if sendlist.eq_ignore_ascii_case("me") {
                session = Some(self.clone());
                session_matches.insert(self.clone());
                return (session, session_matches, discussion, discussion_matches);
            }

            for (_, s) in SESSIONS.iter() {
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
                session = session_matches.iter().next().cloned();
            }
            if discussion_matches.len() == 1 {
                discussion = discussion_matches.iter().next().cloned();
            }
        }

        (session, session_matches, discussion, discussion_matches)
    }

    pub async fn find_session(&self, sendlist: &str) -> (Option<Session>, OrdSet<Session>) {
        let (session, matches, _, _) = self.find_sendable(sendlist, false, false, true, false).await;
        (session, matches)
    }

    pub async fn find_discussion(&self, sendlist: &str, member: bool) -> (Option<Discussion>, OrdSet<Discussion>) {
        let (_, _, discussion, matches) = self.find_sendable(sendlist, member, false, false, true).await;
        (discussion, matches)
    }

    pub async fn notify_entry(&self) {
        let who = self.name_user();
        if let Some(_telnet) = self.telnet() {
            info!("Enter: {who} on connection");
        } else {
            info!("Enter: {who}, detached");
        }

        let now = Timestamp::new();
        self.set_idle_since(now);
        self.set_login_time(now);

        self.enqueue_others(EntryNotify::new(self.name())).await;
    }

    pub async fn notify_exit(&self) {
        let who = self.name_user();
        if let Some(_telnet) = self.telnet() {
            info!("Exit: {who} on connection");
        } else {
            info!("Exit: {who}, detached");
        }

        self.enqueue_others(ExitNotify::new(self.name())).await;
    }
}

impl SessionInner {
    // Methods for SessionInner would go here if needed
}

impl Session {
    pub async fn handle_input(&self, line: Text) {
        self.pending().await.dequeue().await;
        self.process_input(line).await;
        self.enqueue_output().await;
    }

    pub async fn process_input(&self, line: Text) {
        if line.starts_with("!") {
            let line = line[1..].trim();
            if self.priv_level() < 50 {
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
        } else if line.starts_with("/") {
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
            self.do_message(&line).await;
        }
    }

    /// Reset idle time to now.
    pub async fn reset_idle(&self, min: usize) -> i64 {
        let now = Timestamp::new();
        let idle = (now - *self.idle_since()) / 60;

        if min > 0 && idle >= min as i64 {
            self.output("[You were idle for").await;
            self.print_time_long(idle as i32).await;
            self.output(".]\n").await;
        }

        self.set_idle_since(now);
        idle
    }

    pub async fn print_time_long(&self, minutes: i32) {
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

    pub async fn do_restart(&self, args: &str) {
        let who = self.name_user();
        let name = self.name();

        if args == "!" {
            // Immediate restart
            Self::announce(&format!("*** {name} has restarted Phoenix! ***\n")).await;
            self.server().schedule_restart(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel restart
            match self.server().cancel_shutdown().await {
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
            self.server().schedule_restart(who.clone(), seconds).await;
        }
    }

    pub async fn do_down(&self, args: &str) {
        let who = self.name_user();
        let name = self.name();

        if args == "!" {
            // Immediate shutdown
            Self::announce(&format!("*** {name} has shut down Phoenix! ***\n")).await;
            self.server().schedule_shutdown(who, 0).await;
        } else if match_keyword(args, "cancel", 6).is_some() {
            // Cancel shutdown
            match self.server().cancel_shutdown().await {
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
            self.server().schedule_shutdown(who.clone(), seconds).await;
        }
    }

    pub async fn do_nuke(&self, args: &str) {
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
            let days = (now - *session.idle_since()) / 86400;
            if days == 0 {
                continue;
            }

            let mut width = days.to_string().len();
            if session.telnet().is_none() || (now - *session.login_time()) >= 31536000 {
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
                let login_time = *session.login_time();
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
            let idle = (now - *session.idle_since()) / 60;
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
            let detached = session.detached_indicator();
            self.output(detached).await;

            // Name and blurb.
            let name = session.name();
            self.output(name.column_display()).await;

            // Idle time.
            let idle = (now - *session.idle_since()) / 60;
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
        if self.priv_level() < 50 {
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
            let days = (now - *session.idle_since()) / 86400;
            if days == 0 {
                continue;
            }

            let mut width = days.to_string().len();
            if (now - *session.login_time()) >= 31536000 {
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
            let login_time = *session.login_time();
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
            let idle = (now - *session.idle_since()) / 60;
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
    }

    pub async fn do_blurb(&self, args: &str, entry: bool) {
        let args = args.trim();

        if !args.is_empty() {
            let mut start = 0;
            let mut end = args.len();

            if args.len() == 3 && args.eq_ignore_ascii_case("off") {
                if entry || self.blurb().map_or(false, |b| !b.is_empty()) {
                    self.reset_idle(10).await;
                    self.remove_blurb();
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
                self.set_blurb(Some(blurb));
                if !entry {
                    self.output(&format!("Your blurb has been set to [{blurb}].\n")).await;
                }
            }
        } else if entry {
            self.remove_blurb();
        } else if self.has_blurb() {
            let blurb = self.blurb().unwrap();
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
        self.enqueue_others(HereNotify::new(self.name())).await;
    }

    pub async fn do_away(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"away\".\n").await;
        self.set_away(AwayState::Away);
        self.enqueue_others(AwayNotify::new(self.name())).await;
    }

    pub async fn do_busy(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"busy\".\n").await;
        self.set_away(AwayState::Busy);
        self.enqueue_others(BusyNotify::new(self.name())).await;
    }

    pub async fn do_gone(&self, args: &str) {
        self.reset_idle(10).await;
        if !args.trim().is_empty() {
            self.do_blurb(args, false).await;
        }
        self.output("You are now \"gone\".\n").await;
        self.set_away(AwayState::Gone);
        self.enqueue_others(GoneNotify::new(self.name())).await;
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
        if self.priv_level() > 0 {
            self.reset_idle(10).await;
            self.output("You have been detached.\n").await;
            self.enqueue_output().await;
            if let Some(telnet) = self.telnet() {
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

        for (_, session) in SESSIONS.iter() {
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
    }

    pub async fn do_what(&self, args: &str) {
        if DISCUSSIONS.is_empty() {
            self.output("No discussions currently exist.\n").await;
            return;
        }

        let sendlist = Sendlist::new(&self, args, true, false, true).await;

        if !args.is_empty() && sendlist.discussions().is_empty() {
            self.output(&sendlist.errors().to_string()).await;
            return;
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

            let idle = (now - disc.idle_since()) / 60;
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
    }

    pub async fn do_date(&self, _args: &str) {
        let t = Timestamp::new();
        let date = t.date(0, 0);
        self.output(&format!("{date}\n")).await;
    }

    pub async fn do_signal(&self, args: &str) {
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
    }

    pub async fn do_send(&self, args: &str) {
        if args.is_empty() {
            if let Some(sendlist) = self.default_sendlist() {
                self.output("You are sending to ").await;
                self.print_sendlist(&sendlist).await;
                self.output(".\n").await;
            } else {
                self.output("Your default sendlist is turned off.\n").await;
            }
            return;
        }

        if args.eq_ignore_ascii_case("off") {
            self.set_default_sendlist(None);
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

        if !sendlist.errors().is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors().to_string()).await;
        }

        if sendlist.sessions().is_empty() && sendlist.discussions().is_empty() {
            self.output("Your default sendlist is unchanged.\n").await;
            return;
        }

        self.set_default_sendlist(Some(sendlist.clone()));
        self.output("You are now sending to ").await;
        self.print_sendlist(&sendlist).await;
        self.output(".\n").await;
    }

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
            let my_user = self.user();
            let found_user_ref = Some(found_user);
            let is_same_user = match (my_user, found_user_ref) {
                (Some(my_user), Some(found_user)) if my_user.username() == found_user.username() => true,
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
                let name = discussion.name();
                self.output(&format!("There is already a discussion named \"{name}\". (not created)\n")).await;
                return;
            }
            _ => {}
        }

        let disc = Discussion::new(Some(self.clone()), name, title, is_public).await;
        DISCUSSIONS.insert(name.to_string().into(), disc.clone());

        self.enqueue_others(CreateNotify::new(disc.name(), disc.title(), disc.is_public(), self.name())).await;

        let name = disc.name();
        let title = disc.title();
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
            let my_user = self.user();
            let found_user_ref = Some(found_user);
            match (my_user, found_user_ref) {
                (Some(my_user), Some(found_user)) if my_user.username() == found_user.username() => {
                    self.output(&format!("\"{reserved}\" is a reserved name.  (name unchanged)\n")).await;
                    return;
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

        self.enqueue_others(RenameNotify::new(self.name().name().clone(), args)).await;

        self.output(&format!("You have changed your name to \"{args}\".\n")).await;
        self.set_name(Name::new(args, self.blurb().map(|b| b.to_string())));
    }

    pub async fn do_set(&self, args: &str) {
        let (var, value) = getword(args, Some('='));
        if var.is_empty() || value.is_empty() {
            self.output("Usage: /set <variable>=<value>\n").await;
            return;
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
            self.set_idle(value).await;
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
    }

    pub async fn set_idle(&self, args: &str) {
        let now = Timestamp::new();
        let current_idle = (now - *self.idle_since()) / 60;

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
            return;
        }

        // Calculate new idle_since timestamp
        let total_minutes = days * 24 * 60 + hours * 60 + minutes;
        let new_idle_since = Timestamp::from_unix(now.unix() - total_minutes * 60);

        // Check permissions
        if new_idle_since < *self.login_time() && self.priv_level() < 50 {
            self.output("Sorry, you can't be idle longer than you've been signed on.\n").await;
            return;
        }

        // Set the new idle time
        self.set_idle_since(new_idle_since);
        if *self.idle_since() < *self.login_time() {
            self.set_login_time(*self.idle_since());
        }

        // Output results
        let new_idle = (now - *self.idle_since()) / 60;

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
            self.set_idle_since(now);
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
                self.print_time_long(((now - *self.idle_since()) / 60) as i32).await;
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
                    (now.unix() / 60) as i64
                };

                self.output("This server has been running for").await;
                self.print_time_long(uptime as i32).await;
                self.output(".\n").await;

                if let Some(system_up) = system_uptime().await {
                    let system_minutes = system_up / 60;
                    self.output("(This machine has been running for").await;
                    self.print_time_long(system_minutes as i32).await;
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
    }

    pub async fn do_also(&self, args: &str) {
        if args.is_empty() {
            self.output("Usage: /also <sendlist>\n").await;
            return;
        }

        if let Some(last_msg) = self.last_message() {
            let sendlist = Sendlist::new(&self, args, false, true, true).await;
            self.send_message(&sendlist, last_msg.text()).await;
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

                self.send_message(&to, &*self.oops_text()).await;
                self.send_message(&sendlist, &text).await;
                self.set_last_sendlist(Some(sendlist.clone()));
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
            self.set_last_explicit(last_explicit);
        }

        let sendlist = if sendlist_str.is_empty() {
            if let Some(last) = self.last_sendlist() {
                last.clone()
            } else {
                self.output("\x07\x07You have no previous sendlist. (message not sent)\n").await;
                return;
            }
        } else if sendlist_str.eq_ignore_ascii_case("default") {
            if let Some(default) = self.default_sendlist() {
                default.clone()
            } else {
                self.output("\x07\x07You have no default sendlist. (message not sent)\n").await;
                return;
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
            return;
        }

        self.send_message(&sendlist, msg_start).await;
    }

    pub async fn send_message(&self, sendlist: &Sendlist, text: &str) {
        if !sendlist.errors().is_empty() {
            self.output("\x07\x07").await;
            self.output(&sendlist.errors().to_string()).await;
        }

        if sendlist.sessions().is_empty() && sendlist.discussions().is_empty() {
            self.output("Your message is unchanged.\n").await;
            return;
        }

        let idle = self.reset_idle(30).await;

        let mut who = OrdSet::new();
        let count = sendlist.expand(&mut who, Some(self.clone())).await;

        let output_type = if count > 1 || !sendlist.discussions().is_empty() { OutputType::PublicMessage } else { OutputType::PrivateMessage };

        let msg = Message::new(output_type, self.name(), Arc::new(sendlist.clone()), text);
        if let Output::Message(message) = &msg {
            self.set_last_message(Some(message.clone()));
        }

        for session in &who {
            session.enqueue(msg.clone()).await;
        }

        for disc in &sendlist.discussions() {
            disc.set_idle_since(Timestamp::new());
        }

        self.output("(message sent to ").await;
        self.print_sendlist(sendlist).await;
        self.output(")").await;

        if idle >= 30 {
            self.output(&format!(" [idle {idle}]")).await;
        }
        self.output("\n").await;
    }

    pub async fn get_who_set(&self, args: &str) -> (OrdSet<Session>, String, String) {
        let mut who = OrdSet::new();
        let mut errors = String::new();
        let mut msg = String::new();

        if args.is_empty() {
            // Show all sessions
            for (_, session) in SESSIONS.iter() {
                who.insert(session.clone());
            }

            let count = who.len();
            let s = if count == 1 { "" } else { "s" };
            msg = format!("\n{count} user{s} signed on.\n");
        } else {
            let sendlist = Sendlist::new(&self, args, true, true, true).await;

            let _total = sendlist.expand(&mut who, None).await;

            if !sendlist.errors().is_empty() {
                errors = sendlist.errors().to_string();
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

    pub async fn session_matches(&self, name: &str, matches: &OrdSet<Session>) {
        if !matches.is_empty() {
            let count = matches.len();

            for (i, session) in matches.iter().enumerate() {
                match i {
                    0 if count == 1 => self.output(&format!("\"{name}\" matches one name: ")).await,
                    0 => self.output(&format!("\"{name}\" matches {count} names: ")).await,
                    _ if i == count - 1 => self.output(" and ").await,
                    _ => self.output(", ").await,
                };

                self.output(&session.name()).await;
            }

            self.output(".\n").await;
        } else {
            self.output(&format!("No names matched \"{name}\".\n")).await;
        }
    }

    pub async fn discussion_matches(&self, name: &str, matches: &OrdSet<Discussion>) {
        if !matches.is_empty() {
            let count = matches.len();

            for (i, disc) in matches.iter().enumerate() {
                match i {
                    0 if count == 1 => self.output(&format!("\"{name}\" matches one discussion: ")).await,
                    0 => self.output(&format!("\"{name}\" matches {count} discussions: ")).await,
                    _ if i == count - 1 => self.output(" and ").await,
                    _ => self.output(", ").await,
                };

                self.output(disc.name()).await;
            }

            self.output(".\n").await;
        } else {
            self.output(&format!("No discussions matched \"{name}\".\n")).await;
        }
    }
}

impl SessionConnection {
    pub fn login_session(&self) -> Option<&LoginSession> {
        match self {
            SessionConnection::PreLogin(session) => Some(&session),
            SessionConnection::LoggedIn(_session) => None,
        }
    }

    pub fn session(&self) -> Option<&Session> {
        match self {
            SessionConnection::PreLogin(_session) => None,
            SessionConnection::LoggedIn(session) => Some(&session),
        }
    }

    pub fn name_opt(&self) -> Option<Name> {
        match self {
            SessionConnection::PreLogin(_session) => None,
            SessionConnection::LoggedIn(session) => Some(session.name()),
        }
    }

    pub fn signal_private(&self) -> bool {
        match self {
            SessionConnection::PreLogin(_session) => false,
            SessionConnection::LoggedIn(session) => session.signal_private(),
        }
    }

    pub fn signal_public(&self) -> bool {
        match self {
            SessionConnection::PreLogin(_session) => false,
            SessionConnection::LoggedIn(session) => session.signal_public(),
        }
    }

    pub async fn init_login_sequence(&self) {
        match self {
            SessionConnection::PreLogin(session) => session.init_login_sequence().await,
            SessionConnection::LoggedIn(_session) => (),
        }
    }

    pub async fn set_reply_sendlist(&self, sendlist: impl Into<Text>) {
        match self {
            SessionConnection::PreLogin(_session) => (),
            SessionConnection::LoggedIn(session) => session.set_reply_sendlist(sendlist),
        }
    }

    pub async fn acknowledge_output(&self) {
        match self {
            SessionConnection::PreLogin(session) => session.acknowledge_output().await,
            SessionConnection::LoggedIn(session) => session.acknowledge_output().await,
        }
    }

    pub fn last_explicit(&self) -> Text {
        match self {
            SessionConnection::PreLogin(_session) => Text::default(),
            SessionConnection::LoggedIn(session) => session.last_explicit().clone(),
        }
    }

    pub fn reply_sendlist(&self) -> Text {
        match self {
            SessionConnection::PreLogin(_session) => Text::default(),
            SessionConnection::LoggedIn(session) => session.reply_sendlist().clone(),
        }
    }

    pub async fn output_next(&self, telnet: &Telnet) -> bool {
        match self {
            SessionConnection::PreLogin(session) => session.output_next(telnet).await,
            SessionConnection::LoggedIn(session) => session.output_next(telnet).await,
        }
    }

    pub async fn output(&self, text: impl AsRef<str>) {
        match self {
            SessionConnection::PreLogin(session) => session.output(text.as_ref()).await,
            SessionConnection::LoggedIn(session) => session.output(text.as_ref()).await,
        }
    }

    pub async fn handle_input(&self, line: Text) {
        match self {
            SessionConnection::PreLogin(session) => session.handle_input(line).await,
            SessionConnection::LoggedIn(session) => session.handle_input(line).await,
        }
    }

    pub fn login_timeout(&self) -> Option<AbortHandle> {
        match self {
            SessionConnection::PreLogin(session) => session.login_timeout.load_full().map(|arc| (*arc).clone()),
            SessionConnection::LoggedIn(_session) => None, // No login timeout for logged-in sessions
        }
    }

    pub fn set_login_timeout(&self, value: Option<AbortHandle>) {
        match self {
            SessionConnection::PreLogin(session) => session.login_timeout.store(value.map(Arc::new)),
            SessionConnection::LoggedIn(_session) => (), // No-op for logged-in sessions
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

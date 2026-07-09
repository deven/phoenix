// -*- Rust -*-
//
// Phoenix CMC library: user module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::atomic::{AtomicHashMap, AtomicOrdSet, AtomicText, AtomicTextOption, AtomicVector};
use crate::session::{Session, SessionMsg};
use crate::text::Text;
use anyhow::Result;
use async_backtrace::framed;
use im::Vector;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;
use tokio::sync::mpsc;

pub static USERS: LazyLock<AtomicHashMap<Text, User>> = LazyLock::new(AtomicHashMap::default);
static USER_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// User handle.
#[derive(Debug, Clone)]
pub struct User(pub Arc<UserInner>);

#[derive(Debug)]
pub struct UserInner {
    pub id: usize,
    pub sessions: AtomicOrdSet<Session>,
    pub username: AtomicText,
    pub password: AtomicTextOption,
    pub reserved: AtomicVector<Text>,
    pub blurb: AtomicTextOption,
    pub priv_level: AtomicI32,
}

impl User {
    #[framed]
    pub async fn new(login: impl Into<Text>, pass: Option<String>, names: Option<&str>, bl: Option<impl Into<Text>>, p: i32) -> Self {
        let id = USER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let user_text = login.into();
        let blurb = bl.map(|b| b.into());

        let inner = UserInner {
            id,
            sessions: AtomicOrdSet::empty(),
            username: AtomicText::new(user_text),
            password: AtomicTextOption::new(pass.map(Text::new)),
            reserved: AtomicVector::empty(),
            blurb: AtomicTextOption::new(blurb),
            priv_level: AtomicI32::new(p),
        };

        let user = User(Arc::new(inner));

        user.set_reserved(names);

        user
    }

    /// Get the user ID.
    pub fn id(&self) -> usize {
        self.0.id
    }

    pub fn set_reserved(&self, names: Option<&str>) {
        let mut reserved = Vector::new();
        if let Some(names) = names {
            for name in names.split(',') {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    reserved.push_back(trimmed.into());
                }
            }
        }
        // Wholesale replacement computed from external input, not read-modify-write; set() is correct here.
        self.0.reserved.set(reserved);
    }

    pub fn add_session(&self, session: Session) {
        self.0.sessions.insert(session);
    }

    pub fn remove_session(&self, session: &Session) {
        self.0.sessions.remove(session);
    }

    pub fn password(&self) -> Option<Text> {
        self.0.password.snapshot()
    }

    pub fn sessions(&self) -> im::OrdSet<Session> {
        self.0.sessions.snapshot()
    }

    pub fn username(&self) -> Text {
        self.0.username.snapshot()
    }

    pub fn reserved(&self) -> im::Vector<Text> {
        self.0.reserved.snapshot()
    }

    pub fn blurb(&self) -> Option<Text> {
        self.0.blurb.snapshot()
    }

    pub fn priv_level(&self) -> i32 {
        self.0.priv_level.load(Ordering::Relaxed)
    }

    pub fn find_reserved(&self, name: &str) -> Option<Text> {
        let reserved = self.0.reserved.snapshot();
        reserved.iter().find(|reserved| reserved.eq_ignore_ascii_case(name)).cloned()
    }
}

/// Messages to the user manager actor.
#[derive(Debug)]
pub enum UserMsg {
    /// Look up an account: reload the passwd file if stale, then find the user; the outcome returns to the requester as
    /// SessionMsg::UserLookup.
    Lookup { login: Text, requester: Session },
    /// Reload the passwd file if stale.
    Reload,
}

/// UserManager handle.
#[derive(Debug, Clone)]
pub struct UserManager(pub Arc<UserManagerInner>);

#[derive(Debug)]
pub struct UserManagerInner {
    pub tx: mpsc::UnboundedSender<UserMsg>,
}

/// Private user-manager state, owned by the user manager actor task: the mailbox is the serializer for passwd reloads
/// (the reload mutex retires), and the future in-server account administration lands here as messages.
#[derive(Debug)]
pub struct UserManagerObj {
    rx: mpsc::UnboundedReceiver<UserMsg>,
    last_update: Option<SystemTime>,
}

impl UserManagerObj {
    pub async fn update(&mut self, login: impl Into<Text>, pass: Option<String>, names: Option<&str>, defblurb: Option<impl Into<Text>>, p: i32) -> Result<()> {
        let login_str: Text = login.into();

        if let Some(existing_user) = USERS.get(&login_str) {
            // Update existing user's fields atomically
            existing_user.0.password.set(pass.map(Text::new));
            existing_user.set_reserved(names);
            existing_user.0.blurb.set(defblurb.map(|b| b.into()));
            existing_user.0.priv_level.store(p, Ordering::Relaxed);
        } else {
            let user = User::new(login_str.clone(), pass, names, defblurb, p).await;
            USERS.insert(login_str, user);
        }

        Ok(())
    }

    pub async fn update_all(&mut self) -> Result<()> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::Path;

        // Reloads are serialized by the user manager actor's mailbox.
        let passwd_path = Path::new("passwd");
        if !passwd_path.exists() {
            #[cfg(feature = "guest-access")]
            {
                self.update("guest", None, None, None::<&str>, 0).await?;
            }
            return Ok(());
        }

        let metadata = std::fs::metadata(passwd_path)?;
        let modified = metadata.modified()?;

        if let Some(last_time) = self.last_update {
            if last_time == modified {
                return Ok(());
            }
        }

        let file = File::open(passwd_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("#") || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let username = parts[0];
                let password = if parts[1].is_empty() { None } else { Some(parts[1].to_string()) };
                let names = if parts[2].is_empty() { None } else { Some(parts[2]) };
                let priv_level = parts[3].parse::<i32>().unwrap_or(0);

                self.update(username, password, names, None::<&str>, priv_level).await?;
            }
        }

        #[cfg(feature = "guest-access")]
        {
            self.update("guest", None, None, None::<&str>, 0).await?;
        }

        self.last_update = Some(modified);
        Ok(())
    }

    /// The user manager actor.
    #[framed]
    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                UserMsg::Lookup { login, requester } => {
                    if let Err(e) = self.update_all().await {
                        log::error!("passwd reload: {e}");
                    }

                    // Text hashes and compares case-insensitively (UniCase), so this lookup folds case.
                    let user = USERS.get(&Text::from(login.as_ref()));
                    let _ = requester.0.tx.send(SessionMsg::UserLookup(user));
                }
                UserMsg::Reload => {
                    if let Err(e) = self.update_all().await {
                        log::error!("passwd reload: {e}");
                    }
                }
            }
        }
    }
}

impl UserManager {
    #[framed]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(UserManagerObj { rx, last_update: None }.run());
        Self(Arc::new(UserManagerInner { tx }))
    }

    /// Request an account lookup; the outcome arrives at the requester's session actor as SessionMsg::UserLookup.
    pub fn lookup(&self, login: Text, requester: Session) {
        let _ = self.0.tx.send(UserMsg::Lookup { login, requester });
    }

    /// Request a passwd reload if the file has changed.
    pub fn reload(&self) {
        let _ = self.0.tx.send(UserMsg::Reload);
    }

    pub async fn find_reserved(&self, name: &str) -> Option<(Text, User)> {
        // Read-model scan; freshness comes from the reload each Lookup performs.

        let users = USERS.snapshot();
        for (_login, user) in users.iter() {
            if let Some(reserved) = user.find_reserved(name) {
                return Some((reserved, user.clone()));
            }
        }
        None
    }
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new()
    }
}

// Password verification
pub fn verify_password(input: &str, encrypted: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    println!("=== DEBUG: verify_password(input={input:?}, encrypted={encrypted:?}) ===");
    // First try modern Argon2 verification
    if let Ok(parsed_hash) = PasswordHash::new(encrypted) {
        let argon2 = Argon2::default();
        let result = argon2.verify_password(input.as_bytes(), &parsed_hash).is_ok();
        println!("=== DEBUG: result={result:?} ===");
        return result;
    }

    // Fallback to legacy crypt() verification for backward compatibility
    verify_crypt_password(input, encrypted)
}

// Legacy crypt() password verification for backward compatibility
fn verify_crypt_password(input: &str, encrypted: &str) -> bool {
    use pwhash::unix;

    // Use pwhash's unix crypt verification.  This supports various crypt formats including DES, MD5, SHA-256, SHA-512,
    // etc.
    println!("=== DEBUG: verify_crypt_password(input={input:?}, encrypted={encrypted:?}) ===");
    let result = unix::verify(input, encrypted);
    println!("=== DEBUG: result={result:?} ===");
    result
}

// Password hashing for new passwords
pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).map_err(|e| anyhow::anyhow!("Password hashing failed: {e}"))?;
    Ok(password_hash.to_string())
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for User {}

impl std::hash::Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state);
    }
}

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<User>();
    assert_send_sync_static::<UserInner>();
    assert_send_sync_static::<UserManager>();
    assert_send_sync_static::<UserManagerInner>();
};

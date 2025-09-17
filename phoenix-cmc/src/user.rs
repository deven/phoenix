use crate::atomic::{AtomicHashMap, AtomicOrdSet, AtomicText, AtomicTextOption, AtomicVector};
use crate::session::Session;
use crate::text::Text;
use anyhow::Result;
use arc_swap::ArcSwapOption;
use async_backtrace::framed;
use im::Vector;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

static USER_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// User handle.
#[derive(Debug, Clone)]
pub struct User(pub Arc<UserInner>);

#[derive(Debug)]
pub struct UserInner
{
    pub id: usize,
    pub sessions: AtomicOrdSet<Session>,
    pub username: AtomicText,
    pub password: AtomicTextOption,
    pub reserved: AtomicVector<Text>,
    pub blurb: AtomicTextOption,
    pub priv_level: AtomicI32,
}

impl User {
    const BUF_SIZE: usize = 1024;

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
        self.0.reserved.set(reserved);
    }

    pub fn add_session(&self, session: Session) {
        let mut sessions = self.0.sessions.snapshot();
        sessions.insert(session);
        self.0.sessions.set(sessions);
    }

    pub fn remove_session(&self, session: &Session) {
        let mut sessions = self.0.sessions.snapshot();
        sessions.remove(session);
        self.0.sessions.set(sessions);
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

/// UserManager handle.
#[derive(Debug, Clone)]
pub struct UserManager(pub Arc<UserManagerInner>);

#[derive(Debug)]
pub struct UserManagerInner
{
    pub users: AtomicHashMap<Text, User>,
    pub last_update: ArcSwapOption<SystemTime>,
}

impl UserManager {
    #[framed]
    pub fn new() -> Self {
        let inner = UserManagerInner { users: AtomicHashMap::empty(), last_update: ArcSwapOption::new(None) };
        Self(Arc::new(inner))
    }

    pub async fn get_user(&self, login: &str) -> Option<User> {
        self.update_all().await.ok()?;
        let users = self.0.users.snapshot();
        users.iter().find(|(k, _)| k.eq_ignore_ascii_case(login)).map(|(_, v)| v.clone())
    }

    pub async fn update(&self, login: impl Into<Text>, pass: Option<String>, names: Option<&str>, defblurb: Option<impl Into<Text>>, p: i32) -> Result<()> {
        let login_str: Text = login.into();
        let mut users = self.0.users.snapshot();

        if let Some(existing_user) = users.get(&login_str) {
            // Update existing user's fields atomically
            existing_user.0.password.set(pass.map(Text::new));
            existing_user.set_reserved(names);
            existing_user.0.blurb.set(defblurb.map(|b| b.into()));
            existing_user.0.priv_level.store(p, Ordering::Relaxed);
        } else {
            let user = User::new(login_str.clone(), pass, names, defblurb, p).await;
            users.insert(login_str, user);
            self.0.users.set(users);
        }

        Ok(())
    }

    pub async fn update_all(&self) -> Result<()> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::Path;

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

        {
            let last = self.0.last_update.load();
            if let Some(last_time) = last.as_ref() {
                if **last_time == modified {
                    return Ok(());
                }
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

        self.0.last_update.store(Some(Arc::new(modified)));
        Ok(())
    }

    pub async fn find_reserved(&self, name: &str) -> Option<(Text, User)> {
        self.update_all().await.ok()?;

        let users = self.0.users.snapshot();
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

    // First try modern Argon2 verification
    if let Ok(parsed_hash) = PasswordHash::new(encrypted) {
        let argon2 = Argon2::default();
        return argon2.verify_password(input.as_bytes(), &parsed_hash).is_ok();
    }

    false
}

// Password hashing for new passwords
pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?;
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

//#[cfg(test)]
fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<User>();
    assert_send_sync_static::<UserInner>();
    assert_send_sync_static::<UserManager>();
    assert_send_sync_static::<UserManagerInner>();
};

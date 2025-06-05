use crate::session::Session;
use crate::types::{ArcStr, OrderedSet};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct User {
    pub sessions: RwLock<OrderedSet<Arc<Session>>>,
    pub user: ArcStr,
    pub password: Option<String>,
    pub reserved: RwLock<Vec<ArcStr>>,
    pub blurb: Option<ArcStr>,
    pub priv_level: i32,
}

impl User {
    const BUF_SIZE: usize = 1024;

    pub async fn new(
        login: impl Into<ArcStr>,
        pass: Option<String>,
        names: Option<&str>,
        bl: Option<impl Into<ArcStr>>,
        p: i32,
    ) -> Arc<Self> {
        let mut user = Self {
            sessions: RwLock::new(OrderedSet::new()),
            user: login.into(),
            password: pass,
            reserved: RwLock::new(Vec::new()),
            blurb: bl.map(|b| b.into()),
            priv_level: p,
        };
        user.set_reserved(names).await;
        Arc::new(user)
    }

    pub async fn set_reserved(self: &Arc<Self>, names: Option<&str>) {
        let reserved = *self.reserved.write().await;
        reserved.clear();
        if let Some(names) = names {
            for name in names.split(',') {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    reserved.push(trimmed.into());
                }
            }
        }
    }

    pub async fn add_session(self: &Arc<Self>, session: Arc<Session>) {
        self.sessions.write().await.insert(session);
    }

    pub async fn remove_session(self: &Arc<Self>, session: &Arc<Session>) {
        self.sessions.write().await.shift_remove(session);
    }

    pub async fn find_reserved(self: &Arc<Self>, name: &str) -> Option<&ArcStr> {
        self.reserved.read().await
            .iter()
            .find(|&reserved| reserved.eq_ignore_ascii_case(name))
    }
}

#[derive(Clone)]
pub struct UserManager {
    pub users: Arc<RwLock<HashMap<ArcStr, Arc<User>>>>,
    pub last_update: Arc<RwLock<Option<std::time::SystemTime>>>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_user(&self, login: &str) -> Option<Arc<User>> {
        self.update_all().await.ok()?;
        let users = self.users.read().await;
        users
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(login))
            .map(|(_, v)| Arc::clone(v))
    }

    pub async fn update(
        &self,
        login: impl Into<ArcStr>,
        pass: Option<String>,
        names: Option<&str>,
        defblurb: Option<impl Into<ArcStr>>,
        p: i32,
    ) -> Result<()> {
        let login_str: ArcStr = login.into();
        let mut users = self.users.write().await;

        if let Some(user_lock) = users.get(&login_str) {
            let mut user = user_lock.write().await;
            user.password = pass;
            user.set_reserved(names).await;
            user.blurb = defblurb.map(|b| b.into());
            user.priv_level = p;
        } else {
            let user = User::new(login_str.clone(), pass, names, defblurb, p);
            users.insert(login_str, user);
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
            let last = self.last_update.read().await;
            if let Some(last_time) = *last {
                if last_time == modified {
                    return Ok(());
                }
            }
        }

        let file = File::open(passwd_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let username = parts[0];
                let password = if parts[1].is_empty() {
                    None
                } else {
                    Some(parts[1].to_string())
                };
                let names = if parts[2].is_empty() {
                    None
                } else {
                    Some(parts[2])
                };
                let priv_level = parts[3].parse::<i32>().unwrap_or(0);

                self.update(username, password, names, None::<&str>, priv_level)
                    .await?;
            }
        }

        #[cfg(feature = "guest-access")]
        {
            self.update("guest", None, None, None::<&str>, 0).await?;
        }

        *self.last_update.write().await = Some(modified);
        Ok(())
    }

    pub async fn find_reserved(&self, name: &str) -> Option<(ArcStr, Arc<User>)> {
        self.update_all().await.ok()?;

        let users = self.users.read().await;
        for (_login, user_lock) in users.iter() {
            let user = user_lock.read().await;
            if let Some(reserved) = user.find_reserved(name).await {
                return Some((reserved.clone(), Arc::clone(user_lock)));
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
        return argon2
            .verify_password(input.as_bytes(), &parsed_hash)
            .is_ok();
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
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?;
    Ok(password_hash.to_string())
}

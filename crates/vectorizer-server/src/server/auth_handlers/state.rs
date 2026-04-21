//! `AuthHandlerState` — the shared state owned by every `/auth/*`
//! handler — plus the user cache, token blacklist, and rate-limit
//! bookkeeping that live on it.
//!
//! The first-run bootstrap path (auto-creating a root admin, writing
//! its credentials to a 0o600 file on disk instead of stdout) lives
//! here too because it is tightly coupled to both `AuthPersistence`
//! and the in-memory `users` map.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use vectorizer::auth::AuthManager;
use vectorizer::auth::persistence::{AuthPersistence, PersistedUser};
use vectorizer::auth::roles::Role;

/// Persist first-run root credentials to a file with the most restrictive
/// permissions the host supports (0o600 on POSIX). Returns the path written.
///
/// Writing to a file — instead of emitting the password to stdout — prevents
/// container log pipelines (Docker / k8s / systemd / CI) from capturing the
/// cleartext password. The operator must read and delete the file on first
/// login; the file is also added to `.gitignore` / `.dockerignore`.
pub(crate) fn persist_first_run_credentials(
    data_dir: &Path,
    username: &str,
    password: &str,
    was_generated: bool,
) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(data_dir)?;
    let path = data_dir.join(".root_credentials");

    // Build the file body. Keep it short so it fits on a single screen and
    // operators notice all three lines of guidance.
    let body = format!(
        "# Vectorizer first-run root credentials\n\
         # Written: {now}\n\
         # Generated: {generated}\n\
         # READ ONCE AND DELETE. Rotate via the dashboard or /auth API.\n\
         username={username}\n\
         password={password}\n",
        now = chrono::Utc::now().to_rfc3339(),
        generated = was_generated,
        username = username,
        password = password,
    );

    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    let mut f = opts.open(&path)?;
    f.write_all(body.as_bytes())?;
    f.flush()?;
    Ok(path)
}

/// Login attempt tracking for rate limiting
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    /// Number of failed attempts
    pub count: u32,
    /// Timestamp of first failed attempt in this window
    pub window_start: std::time::Instant,
    /// Timestamp when lockout expires (if locked)
    pub locked_until: Option<std::time::Instant>,
}

/// Rate limit configuration
pub(super) const MAX_LOGIN_ATTEMPTS: u32 = 5;
pub(super) const LOGIN_WINDOW_SECONDS: u64 = 300; // 5 minutes
pub(super) const LOCKOUT_SECONDS: u64 = 900; // 15 minutes lockout after max attempts

/// Shared state for auth handlers
#[derive(Clone)]
pub struct AuthHandlerState {
    /// Authentication manager
    pub auth_manager: Arc<AuthManager>,
    /// User store (in-memory cache, backed by disk persistence)
    pub users: Arc<tokio::sync::RwLock<HashMap<String, UserRecord>>>,
    /// Persistence manager for saving/loading from disk
    pub persistence: Arc<AuthPersistence>,
    /// Token blacklist for logout (tokens that have been invalidated)
    pub token_blacklist: Arc<tokio::sync::RwLock<HashSet<String>>>,
    /// Login attempt tracking for rate limiting (by IP or username)
    pub login_attempts: Arc<tokio::sync::RwLock<HashMap<String, LoginAttempt>>>,
}

/// User record for authentication
#[derive(Debug, Clone)]
pub struct UserRecord {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Password hash (bcrypt). Redacting wrapper — plaintext bcrypt hash only
    /// reaches `bcrypt::verify` through `.expose_secret()`.
    pub password_hash: vectorizer::auth::Secret<String>,
    /// User roles
    pub roles: Vec<Role>,
}

impl AuthHandlerState {
    /// Create a new auth handler state
    pub fn new(auth_manager: Arc<AuthManager>) -> Self {
        let users = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let persistence = Arc::new(AuthPersistence::with_default_dir());
        let token_blacklist = Arc::new(tokio::sync::RwLock::new(HashSet::new()));
        let login_attempts = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        Self {
            auth_manager,
            users,
            persistence,
            token_blacklist,
            login_attempts,
        }
    }

    /// Create with persistence - loads from disk or creates default admin
    pub async fn new_with_default_admin(auth_manager: Arc<AuthManager>) -> Self {
        Self::new_with_root_user(auth_manager, None, None).await
    }

    /// Create with persistence and optional root user configuration
    ///
    /// # Arguments
    /// * `auth_manager` - The authentication manager
    /// * `root_user` - Optional root username (defaults to "root")
    /// * `root_password` - Optional root password (generates random if not provided)
    pub async fn new_with_root_user(
        auth_manager: Arc<AuthManager>,
        root_user: Option<String>,
        root_password: Option<String>,
    ) -> Self {
        let persistence = Arc::new(AuthPersistence::with_default_dir());
        let users = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // Try to load users and API keys from disk
        let mut loaded_users = HashMap::new();
        let mut loaded_api_keys = Vec::new();
        match persistence.load() {
            Ok(data) => {
                if !data.users.is_empty() {
                    info!("Loaded {} users from disk", data.users.len());
                    for (username, persisted) in data.users {
                        loaded_users.insert(
                            username,
                            UserRecord {
                                user_id: persisted.user_id,
                                username: persisted.username,
                                password_hash: persisted.password_hash,
                                roles: persisted.roles,
                            },
                        );
                    }
                }
                // Load API keys
                if !data.api_keys.is_empty() {
                    info!("Loaded {} API keys from disk", data.api_keys.len());
                    loaded_api_keys = data.api_keys.into_values().collect();
                }
            }
            Err(e) => {
                warn!("Failed to load auth data from disk: {}", e);
            }
        }

        // Register loaded API keys with the auth manager
        for persisted_key in loaded_api_keys {
            let api_key = vectorizer::auth::ApiKey {
                id: persisted_key.id,
                name: persisted_key.name,
                key_hash: persisted_key.key_hash,
                user_id: persisted_key.user_id,
                permissions: persisted_key.permissions,
                created_at: persisted_key.created_at,
                last_used: persisted_key.last_used,
                expires_at: persisted_key.expires_at,
                active: persisted_key.active,
            };
            if let Err(e) = auth_manager.register_api_key(api_key).await {
                warn!("Failed to register API key from disk: {}", e);
            }
        }

        // Check if any admin user exists
        let has_admin = loaded_users
            .values()
            .any(|u| u.roles.contains(&Role::Admin));

        // Create root admin if no admin users exist
        if !has_admin {
            let username = root_user.unwrap_or_else(|| "root".to_string());
            let (password, was_generated) = match root_password {
                Some(pwd) => (pwd, false),
                None => (Self::generate_secure_password(), true),
            };

            info!(
                "No admin users found, creating root admin user '{}'",
                username
            );

            let password_hash = vectorizer::auth::Secret::new(
                bcrypt::hash(&password, bcrypt::DEFAULT_COST)
                    .unwrap_or_else(|_| "invalid".to_string()),
            );

            let admin = UserRecord {
                user_id: username.clone(),
                username: username.clone(),
                password_hash: password_hash.clone(),
                roles: vec![Role::Admin],
            };

            loaded_users.insert(username.clone(), admin);

            // Save to disk
            let persisted_admin = PersistedUser {
                user_id: username.clone(),
                username: username.clone(),
                password_hash,
                roles: vec![Role::Admin],
                created_at: chrono::Utc::now().timestamp() as u64,
                last_login: None,
            };

            if let Err(e) = persistence.save_user(persisted_admin) {
                error!("Failed to save root admin to disk: {}", e);
            }

            // Write the one-time root credentials to a 0o600 file in the
            // auth data directory. We deliberately do NOT print the password
            // to stdout — container log pipelines (Docker, k8s, systemd, CI)
            // would otherwise capture it permanently.
            let data_dir = AuthPersistence::get_data_dir();
            let cred_path =
                match persist_first_run_credentials(&data_dir, &username, &password, was_generated)
                {
                    Ok(p) => p,
                    Err(e) => {
                        error!(
                            "Failed to persist root credentials to {:?}: {}. The \
                         password was generated but could not be saved; abort \
                         and fix the filesystem before retrying.",
                            data_dir, e
                        );
                        data_dir.join(".root_credentials")
                    }
                };

            // Log only the username and the path. The path is safe to show
            // in any log shipper; the password stays on disk under 0o600.
            warn!(
                "Root admin user '{}' created. Credentials written to {:?} — \
                 READ ONCE AND DELETE. Rotate via dashboard or /auth API.",
                username, cred_path
            );
            if was_generated {
                warn!(
                    "Root password was auto-generated. The file at {:?} is the \
                     only copy; nothing was echoed to stdout.",
                    cred_path
                );
            }
        }

        *users.write().await = loaded_users;

        let token_blacklist = Arc::new(tokio::sync::RwLock::new(HashSet::new()));
        let login_attempts = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        Self {
            auth_manager,
            users,
            persistence,
            token_blacklist,
            login_attempts,
        }
    }

    /// Generate a secure random password
    fn generate_secure_password() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*";
        let mut rng = rand::thread_rng();
        let password: String = (0..24)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        password
    }

    /// Check if a token is blacklisted (logged out)
    pub async fn is_token_blacklisted(&self, token: &str) -> bool {
        self.token_blacklist.read().await.contains(token)
    }

    /// Add a token to the blacklist (logout)
    pub async fn blacklist_token(&self, token: String) {
        self.token_blacklist.write().await.insert(token);
    }

    /// Clean up expired tokens from blacklist (should be called periodically)
    pub async fn cleanup_expired_tokens(&self) {
        let mut blacklist = self.token_blacklist.write().await;
        let before_count = blacklist.len();

        // Remove tokens that have expired (can no longer be used anyway)
        blacklist.retain(|token| {
            // Try to decode without validation to check expiry
            match self.auth_manager.validate_jwt(token) {
                Ok(_) => true,   // Still valid, keep in blacklist
                Err(_) => false, // Expired or invalid, remove from blacklist
            }
        });

        let removed = before_count - blacklist.len();
        if removed > 0 {
            debug!("Cleaned up {} expired tokens from blacklist", removed);
        }
    }

    /// Check if login is rate limited for a given key (username or IP)
    pub async fn check_login_rate_limit(&self, key: &str) -> Result<(), (u32, u64)> {
        let attempts = self.login_attempts.read().await;

        if let Some(attempt) = attempts.get(key) {
            // Check if currently locked out
            if let Some(locked_until) = attempt.locked_until {
                if locked_until > std::time::Instant::now() {
                    let remaining = locked_until
                        .duration_since(std::time::Instant::now())
                        .as_secs();
                    return Err((attempt.count, remaining));
                }
            }

            // Check if within window and exceeded attempts
            let window_elapsed = attempt.window_start.elapsed().as_secs();
            if window_elapsed < LOGIN_WINDOW_SECONDS && attempt.count >= MAX_LOGIN_ATTEMPTS {
                let remaining = LOGIN_WINDOW_SECONDS - window_elapsed;
                return Err((attempt.count, remaining));
            }
        }

        Ok(())
    }

    /// Record a failed login attempt
    pub async fn record_failed_login(&self, key: &str) {
        let mut attempts = self.login_attempts.write().await;
        let now = std::time::Instant::now();

        let attempt = attempts.entry(key.to_string()).or_insert(LoginAttempt {
            count: 0,
            window_start: now,
            locked_until: None,
        });

        // Reset window if expired
        if attempt.window_start.elapsed().as_secs() >= LOGIN_WINDOW_SECONDS {
            attempt.count = 0;
            attempt.window_start = now;
            attempt.locked_until = None;
        }

        attempt.count += 1;

        // Lock out if exceeded max attempts
        if attempt.count >= MAX_LOGIN_ATTEMPTS {
            attempt.locked_until = Some(now + std::time::Duration::from_secs(LOCKOUT_SECONDS));
            warn!(
                "Account '{}' locked out for {} seconds after {} failed login attempts",
                key, LOCKOUT_SECONDS, attempt.count
            );
        }
    }

    /// Clear login attempts on successful login
    pub async fn clear_login_attempts(&self, key: &str) {
        let mut attempts = self.login_attempts.write().await;
        attempts.remove(key);
    }

    /// Clean up expired login attempt records
    pub async fn cleanup_expired_login_attempts(&self) {
        let mut attempts = self.login_attempts.write().await;
        let before_count = attempts.len();

        attempts.retain(|_, attempt| {
            // Keep if within window or still locked
            let window_active = attempt.window_start.elapsed().as_secs() < LOGIN_WINDOW_SECONDS;
            let still_locked = attempt
                .locked_until
                .is_some_and(|until| until > std::time::Instant::now());
            window_active || still_locked
        });

        let removed = before_count - attempts.len();
        if removed > 0 {
            debug!("Cleaned up {} expired login attempt records", removed);
        }
    }

    /// Save current users to disk
    pub async fn save_users_to_disk(&self) -> Result<(), String> {
        let users = self.users.read().await;
        let mut data = self.persistence.load().unwrap_or_default();

        data.users.clear();
        for (username, record) in users.iter() {
            data.users.insert(
                username.clone(),
                PersistedUser {
                    user_id: record.user_id.clone(),
                    username: record.username.clone(),
                    password_hash: record.password_hash.clone(),
                    roles: record.roles.clone(),
                    created_at: chrono::Utc::now().timestamp() as u64,
                    last_login: None,
                },
            );
        }

        self.persistence.save(&data)
    }
}

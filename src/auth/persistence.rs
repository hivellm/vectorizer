//! Authentication persistence module
//!
//! Handles saving and loading users and API keys to/from disk with AES-GCM encryption.

use std::collections::HashMap;
use std::path::PathBuf;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::ApiKey;
use super::roles::{Permission, Role};

/// Encryption key length for AES-256
const KEY_LENGTH: usize = 32;
/// Nonce length for AES-GCM
const NONCE_LENGTH: usize = 12;

/// Persisted user data (password hash, not plaintext)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedUser {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Password hash (bcrypt)
    pub password_hash: String,
    /// User roles
    pub roles: Vec<Role>,
    /// Created timestamp
    pub created_at: u64,
    /// Last login timestamp
    pub last_login: Option<u64>,
}

/// Persisted API key data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedApiKey {
    /// API key ID
    pub id: String,
    /// API key name/description
    pub name: String,
    /// API key value hash (not the raw key)
    pub key_hash: String,
    /// Associated user ID
    pub user_id: String,
    /// Permissions for this API key
    pub permissions: Vec<Permission>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used: Option<u64>,
    /// Expiration timestamp (None = never expires)
    pub expires_at: Option<u64>,
    /// Whether the key is active
    pub active: bool,
}

/// Auth data store structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthDataStore {
    /// Version for future migrations
    pub version: u32,
    /// Users map (username -> user)
    pub users: HashMap<String, PersistedUser>,
    /// API keys map (key_id -> key)
    pub api_keys: HashMap<String, PersistedApiKey>,
}

impl AuthDataStore {
    /// Create a new empty auth data store
    pub fn new() -> Self {
        Self {
            version: 1,
            users: HashMap::new(),
            api_keys: HashMap::new(),
        }
    }
}

/// Auth persistence manager with AES-GCM encryption
pub struct AuthPersistence {
    /// Path to the encrypted auth data file
    data_path: PathBuf,
    /// Path to the encryption key file
    key_path: PathBuf,
    /// Encryption cipher
    cipher: Aes256Gcm,
}

impl AuthPersistence {
    /// Create a new auth persistence manager
    pub fn new(data_dir: &PathBuf) -> Self {
        let data_path = data_dir.join("auth.enc");
        let key_path = data_dir.join(".auth.key");

        // Load or generate encryption key
        let key = Self::load_or_generate_key(&key_path, data_dir);
        let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length for AES-256-GCM");

        Self {
            data_path,
            key_path,
            cipher,
        }
    }

    /// Load existing key or generate a new one
    fn load_or_generate_key(key_path: &PathBuf, data_dir: &PathBuf) -> [u8; KEY_LENGTH] {
        // Ensure data directory exists
        if !data_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(data_dir) {
                error!("Failed to create data directory: {}", e);
            }
        }

        // Try to load existing key
        if key_path.exists() {
            if let Ok(key_data) = std::fs::read(key_path) {
                if key_data.len() == KEY_LENGTH {
                    let mut key = [0u8; KEY_LENGTH];
                    key.copy_from_slice(&key_data);
                    debug!("Loaded existing encryption key");
                    return key;
                }
            }
        }

        // Generate new key
        let mut key = [0u8; KEY_LENGTH];
        OsRng.fill_bytes(&mut key);

        // Save the key with restricted permissions
        if let Err(e) = std::fs::write(key_path, &key) {
            error!("Failed to save encryption key: {}", e);
        } else {
            // Set restrictive permissions on key file (Unix-like systems)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Err(e) =
                    std::fs::set_permissions(key_path, std::fs::Permissions::from_mode(0o600))
                {
                    warn!("Failed to set key file permissions: {}", e);
                }
            }
            info!("Generated new encryption key for auth persistence");
        }

        key
    }

    /// Get the default data directory
    pub fn get_data_dir() -> PathBuf {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join("data")
    }

    /// Create with default data directory
    pub fn with_default_dir() -> Self {
        Self::new(&Self::get_data_dir())
    }

    /// Encrypt data using AES-GCM
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt data using AES-GCM
    fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, String> {
        if encrypted.len() < NONCE_LENGTH {
            return Err("Encrypted data too short".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))
    }

    /// Migrate from unencrypted to encrypted storage
    fn migrate_from_unencrypted(&self, data_dir: &PathBuf) -> Option<AuthDataStore> {
        let old_path = data_dir.join("auth.json");
        if old_path.exists() {
            info!("Found unencrypted auth.json, migrating to encrypted storage...");
            if let Ok(content) = std::fs::read_to_string(&old_path) {
                if let Ok(data) = serde_json::from_str::<AuthDataStore>(&content) {
                    // Save to encrypted format
                    if self.save(&data).is_ok() {
                        // Remove old unencrypted file
                        if let Err(e) = std::fs::remove_file(&old_path) {
                            warn!("Failed to remove old auth.json: {}", e);
                        } else {
                            info!("Successfully migrated auth data to encrypted storage");
                        }
                        return Some(data);
                    }
                }
            }
        }
        None
    }

    /// Load auth data from disk (encrypted)
    pub fn load(&self) -> Result<AuthDataStore, String> {
        // First check for migration from unencrypted format
        let data_dir = self
            .data_path
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        if !self.data_path.exists() {
            if let Some(data) = self.migrate_from_unencrypted(&data_dir) {
                return Ok(data);
            }
            debug!("Auth data file does not exist, returning empty store");
            return Ok(AuthDataStore::new());
        }

        let encrypted = std::fs::read(&self.data_path)
            .map_err(|e| format!("Failed to read auth data file: {}", e))?;

        let decrypted = self.decrypt(&encrypted)?;

        let content = String::from_utf8(decrypted)
            .map_err(|e| format!("Invalid UTF-8 in auth data: {}", e))?;

        let data: AuthDataStore = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse auth data: {}", e))?;

        info!(
            "Loaded {} users and {} API keys from encrypted storage",
            data.users.len(),
            data.api_keys.len()
        );

        Ok(data)
    }

    /// Save auth data to disk (encrypted)
    pub fn save(&self, data: &AuthDataStore) -> Result<(), String> {
        // Ensure data directory exists
        if let Some(parent) = self.data_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create data directory: {}", e))?;
            }
        }

        let content = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize auth data: {}", e))?;

        let encrypted = self.encrypt(content.as_bytes())?;

        // Write to temp file first, then rename (atomic write)
        let temp_path = self.data_path.with_extension("enc.tmp");
        std::fs::write(&temp_path, &encrypted)
            .map_err(|e| format!("Failed to write auth data: {}", e))?;

        std::fs::rename(&temp_path, &self.data_path)
            .map_err(|e| format!("Failed to rename auth data file: {}", e))?;

        debug!(
            "Saved {} users and {} API keys to encrypted storage",
            data.users.len(),
            data.api_keys.len()
        );

        Ok(())
    }

    /// Add or update a user
    pub fn save_user(&self, user: PersistedUser) -> Result<(), String> {
        let mut data = self.load()?;
        data.users.insert(user.username.clone(), user);
        self.save(&data)
    }

    /// Remove a user
    pub fn remove_user(&self, username: &str) -> Result<(), String> {
        let mut data = self.load()?;
        data.users.remove(username);
        self.save(&data)
    }

    /// Add or update an API key
    pub fn save_api_key(&self, key: PersistedApiKey) -> Result<(), String> {
        let mut data = self.load()?;
        data.api_keys.insert(key.id.clone(), key);
        self.save(&data)
    }

    /// Remove an API key
    pub fn remove_api_key(&self, key_id: &str) -> Result<(), String> {
        let mut data = self.load()?;
        data.api_keys.remove(key_id);
        self.save(&data)
    }

    /// Get all users
    pub fn get_users(&self) -> Result<Vec<PersistedUser>, String> {
        let data = self.load()?;
        Ok(data.users.into_values().collect())
    }

    /// Get a user by username
    pub fn get_user(&self, username: &str) -> Result<Option<PersistedUser>, String> {
        let data = self.load()?;
        Ok(data.users.get(username).cloned())
    }

    /// Get all API keys for a user
    pub fn get_api_keys_for_user(&self, user_id: &str) -> Result<Vec<PersistedApiKey>, String> {
        let data = self.load()?;
        Ok(data
            .api_keys
            .into_values()
            .filter(|k| k.user_id == user_id)
            .collect())
    }

    /// Update API key last used timestamp
    pub fn update_api_key_last_used(&self, key_id: &str) -> Result<(), String> {
        let mut data = self.load()?;
        if let Some(key) = data.api_keys.get_mut(key_id) {
            key.last_used = Some(chrono::Utc::now().timestamp() as u64);
            self.save(&data)?;
        }
        Ok(())
    }

    /// Check if data file exists
    pub fn exists(&self) -> bool {
        self.data_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_auth_persistence_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AuthPersistence::new(&temp_dir.path().to_path_buf());

        // Create test data
        let mut data = AuthDataStore::new();
        data.users.insert(
            "testuser".to_string(),
            PersistedUser {
                user_id: "user1".to_string(),
                username: "testuser".to_string(),
                password_hash: "hash123".to_string(),
                roles: vec![Role::User],
                created_at: 1234567890,
                last_login: None,
            },
        );

        // Save
        persistence.save(&data).unwrap();

        // Load
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.users.len(), 1);
        assert_eq!(loaded.users.get("testuser").unwrap().user_id, "user1");
    }

    #[test]
    fn test_auth_persistence_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AuthPersistence::new(&temp_dir.path().to_path_buf());

        // Load from non-existent file should return empty store
        let data = persistence.load().unwrap();
        assert!(data.users.is_empty());
        assert!(data.api_keys.is_empty());
    }

    #[test]
    fn test_save_user() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AuthPersistence::new(&temp_dir.path().to_path_buf());

        let user = PersistedUser {
            user_id: "user1".to_string(),
            username: "admin".to_string(),
            password_hash: "hash".to_string(),
            roles: vec![Role::Admin],
            created_at: 1234567890,
            last_login: None,
        };

        persistence.save_user(user).unwrap();

        let loaded = persistence.get_user("admin").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().user_id, "user1");
    }

    #[test]
    fn test_save_api_key() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AuthPersistence::new(&temp_dir.path().to_path_buf());

        let key = PersistedApiKey {
            id: "key1".to_string(),
            name: "test key".to_string(),
            key_hash: "hash".to_string(),
            user_id: "user1".to_string(),
            permissions: vec![Permission::Read],
            created_at: 1234567890,
            last_used: None,
            expires_at: None,
            active: true,
        };

        persistence.save_api_key(key).unwrap();

        let keys = persistence.get_api_keys_for_user("user1").unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].name, "test key");
    }
}

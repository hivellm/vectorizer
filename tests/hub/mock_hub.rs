//! Mock HiveHub API for testing
//!
//! Provides a mock implementation of the HiveHub API for integration testing
//! without requiring a real HiveHub Cloud connection.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

/// Mock user record
#[derive(Debug, Clone)]
pub struct MockUser {
    pub id: Uuid,
    pub username: String,
    pub api_key: String,
    pub collections: Vec<Uuid>,
    pub quota_collections: u64,
    pub quota_vectors: u64,
    pub quota_storage: u64,
    pub used_collections: u64,
    pub used_vectors: u64,
    pub used_storage: u64,
}

impl MockUser {
    pub fn new(username: &str) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            username: username.to_string(),
            api_key: format!("hh_test_{}", id.to_string().replace('-', "")),
            collections: Vec::new(),
            quota_collections: 10,
            quota_vectors: 100_000,
            quota_storage: 1_000_000_000, // 1GB
            used_collections: 0,
            used_vectors: 0,
            used_storage: 0,
        }
    }

    pub fn with_quota(mut self, collections: u64, vectors: u64, storage: u64) -> Self {
        self.quota_collections = collections;
        self.quota_vectors = vectors;
        self.quota_storage = storage;
        self
    }
}

/// Mock collection record
#[derive(Debug, Clone)]
pub struct MockCollection {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    #[allow(dead_code)]
    pub vector_count: u64,
    #[allow(dead_code)]
    pub storage_bytes: u64,
}

/// Mock HiveHub API server
#[derive(Debug)]
pub struct MockHubApi {
    users: Arc<RwLock<HashMap<Uuid, MockUser>>>,
    users_by_api_key: Arc<RwLock<HashMap<String, Uuid>>>,
    collections: Arc<RwLock<HashMap<Uuid, MockCollection>>>,
}

impl MockHubApi {
    /// Create a new mock Hub API
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            users_by_api_key: Arc::new(RwLock::new(HashMap::new())),
            collections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a user to the mock API
    pub fn add_user(&self, user: MockUser) -> Uuid {
        let id = user.id;
        let api_key = user.api_key.clone();

        self.users_by_api_key.write().insert(api_key, id);
        self.users.write().insert(id, user);

        id
    }

    /// Create a test user with default settings
    pub fn create_test_user(&self, username: &str) -> MockUser {
        let user = MockUser::new(username);
        self.add_user(user.clone());
        user
    }

    /// Validate an API key
    pub fn validate_api_key(&self, api_key: &str) -> Option<MockUser> {
        let users_by_key = self.users_by_api_key.read();
        let user_id = users_by_key.get(api_key)?;

        let users = self.users.read();
        users.get(user_id).cloned()
    }

    /// Create a collection for a user
    pub fn create_collection(&self, user_id: Uuid, name: &str) -> Result<MockCollection, String> {
        let mut users = self.users.write();
        let user = users.get_mut(&user_id).ok_or("User not found")?;

        // Check quota
        if user.used_collections >= user.quota_collections {
            return Err("Collection quota exceeded".to_string());
        }

        let collection = MockCollection {
            id: Uuid::new_v4(),
            name: name.to_string(),
            owner_id: user_id,
            vector_count: 0,
            storage_bytes: 0,
        };

        user.collections.push(collection.id);
        user.used_collections += 1;

        self.collections
            .write()
            .insert(collection.id, collection.clone());

        Ok(collection)
    }

    /// Get collections for a user
    pub fn get_user_collections(&self, user_id: Uuid) -> Vec<MockCollection> {
        let users = self.users.read();
        let Some(user) = users.get(&user_id) else {
            return Vec::new();
        };

        let collections = self.collections.read();
        user.collections
            .iter()
            .filter_map(|id| collections.get(id).cloned())
            .collect()
    }

    /// Validate collection ownership
    pub fn validate_collection(&self, collection_id: Uuid, user_id: Uuid) -> bool {
        let collections = self.collections.read();
        match collections.get(&collection_id) {
            Some(c) => c.owner_id == user_id,
            None => false,
        }
    }

    /// Check quota
    pub fn check_quota(
        &self,
        user_id: Uuid,
        quota_type: &str,
        requested: u64,
    ) -> Result<bool, String> {
        let users = self.users.read();
        let user = users.get(&user_id).ok_or("User not found")?;

        let allowed = match quota_type {
            "collections" => user.used_collections + requested <= user.quota_collections,
            "vectors" => user.used_vectors + requested <= user.quota_vectors,
            "storage" => user.used_storage + requested <= user.quota_storage,
            _ => false,
        };

        Ok(allowed)
    }

    /// Record usage
    pub fn record_usage(&self, user_id: Uuid, vectors: u64, storage: u64) -> Result<(), String> {
        let mut users = self.users.write();
        let user = users.get_mut(&user_id).ok_or("User not found")?;

        user.used_vectors += vectors;
        user.used_storage += storage;

        Ok(())
    }

    /// Get user quota info
    pub fn get_quota_info(&self, user_id: Uuid) -> Option<QuotaInfo> {
        let users = self.users.read();
        let user = users.get(&user_id)?;

        Some(QuotaInfo {
            collections_limit: user.quota_collections,
            collections_used: user.used_collections,
            vectors_limit: user.quota_vectors,
            vectors_used: user.used_vectors,
            storage_limit: user.quota_storage,
            storage_used: user.used_storage,
        })
    }

    /// Delete a collection
    pub fn delete_collection(&self, collection_id: Uuid, user_id: Uuid) -> Result<(), String> {
        // Verify ownership
        if !self.validate_collection(collection_id, user_id) {
            return Err("Collection not found or not owned by user".to_string());
        }

        // Remove collection
        self.collections.write().remove(&collection_id);

        // Update user
        let mut users = self.users.write();
        if let Some(user) = users.get_mut(&user_id) {
            user.collections.retain(|&id| id != collection_id);
            user.used_collections = user.used_collections.saturating_sub(1);
        }

        Ok(())
    }

    /// Reset all data (for test cleanup)
    pub fn reset(&self) {
        self.users.write().clear();
        self.users_by_api_key.write().clear();
        self.collections.write().clear();
    }
}

impl Default for MockHubApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Quota information returned by mock API
#[derive(Debug, Clone)]
pub struct QuotaInfo {
    #[allow(dead_code)]
    pub collections_limit: u64,
    pub collections_used: u64,
    #[allow(dead_code)]
    pub vectors_limit: u64,
    pub vectors_used: u64,
    #[allow(dead_code)]
    pub storage_limit: u64,
    pub storage_used: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_hub_create_user() {
        let mock = MockHubApi::new();
        let user = mock.create_test_user("testuser");

        assert!(!user.api_key.is_empty());
        assert_eq!(user.username, "testuser");
    }

    #[test]
    fn test_mock_hub_validate_api_key() {
        let mock = MockHubApi::new();
        let user = mock.create_test_user("testuser");

        let validated = mock.validate_api_key(&user.api_key);
        assert!(validated.is_some());
        assert_eq!(validated.unwrap().id, user.id);

        let invalid = mock.validate_api_key("invalid_key");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_mock_hub_create_collection() {
        let mock = MockHubApi::new();
        let user = mock.create_test_user("testuser");

        let collection = mock.create_collection(user.id, "test_collection");
        assert!(collection.is_ok());

        let collections = mock.get_user_collections(user.id);
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].name, "test_collection");
    }

    #[test]
    fn test_mock_hub_collection_quota() {
        let mock = MockHubApi::new();
        let user = MockUser::new("testuser").with_quota(2, 1000, 1_000_000);
        mock.add_user(user.clone());

        // Create 2 collections (should succeed)
        assert!(mock.create_collection(user.id, "col1").is_ok());
        assert!(mock.create_collection(user.id, "col2").is_ok());

        // Third should fail
        let result = mock.create_collection(user.id, "col3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("quota exceeded"));
    }

    #[test]
    fn test_mock_hub_validate_collection() {
        let mock = MockHubApi::new();
        let user1 = mock.create_test_user("user1");
        let user2 = mock.create_test_user("user2");

        let collection = mock.create_collection(user1.id, "my_collection").unwrap();

        // User 1 owns it
        assert!(mock.validate_collection(collection.id, user1.id));

        // User 2 doesn't own it
        assert!(!mock.validate_collection(collection.id, user2.id));
    }

    #[test]
    fn test_mock_hub_check_quota() {
        let mock = MockHubApi::new();
        let user = MockUser::new("testuser").with_quota(10, 100, 1000);
        mock.add_user(user.clone());

        // Within quota
        assert!(mock.check_quota(user.id, "vectors", 50).unwrap());

        // Exceeds quota
        assert!(!mock.check_quota(user.id, "vectors", 150).unwrap());
    }

    #[test]
    fn test_mock_hub_record_usage() {
        let mock = MockHubApi::new();
        let user = mock.create_test_user("testuser");

        mock.record_usage(user.id, 100, 5000).unwrap();

        let quota = mock.get_quota_info(user.id).unwrap();
        assert_eq!(quota.vectors_used, 100);
        assert_eq!(quota.storage_used, 5000);
    }

    #[test]
    fn test_mock_hub_delete_collection() {
        let mock = MockHubApi::new();
        let user = mock.create_test_user("testuser");

        let collection = mock.create_collection(user.id, "to_delete").unwrap();
        assert_eq!(mock.get_user_collections(user.id).len(), 1);

        mock.delete_collection(collection.id, user.id).unwrap();
        assert_eq!(mock.get_user_collections(user.id).len(), 0);
    }
}

//! HiveHub surface.
//!
//! Covers user-scoped backup management (`/hub/backups/*`), usage
//! statistics (`/hub/usage/*`), and API key validation
//! (`/hub/validate-key`).
//!
//! These endpoints are only meaningful when the server is running
//! in HiveHub cluster mode (hub integration enabled in `config.yml`).
//! Calling them on a standalone instance returns a 503 that the SDK
//! surfaces as a [`VectorizerError`].

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::{
    CreateUserBackupRequest, HubApiKeyValidation, QuotaInfo, RestoreUserBackupRequest,
    UploadUserBackupRequest, UsageStatistics, UserBackup,
};

impl VectorizerClient {
    /// List all backups owned by a user.
    ///
    /// Calls `GET /hub/backups?user_id={user_id}`.
    pub async fn list_user_backups(&self, user_id: &str) -> Result<Vec<UserBackup>> {
        let endpoint = format!("/hub/backups?user_id={user_id}");
        let response = self.make_request("GET", &endpoint, None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_user_backups response: {e}"))
        })?;
        let arr = val
            .get("backups")
            .and_then(|b| b.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse user backup entry: {e}"))
                })
            })
            .collect()
    }

    /// Create a new backup for a user.
    ///
    /// Calls `POST /hub/backups` with `{user_id, name, description?, collections?}`.
    pub async fn create_user_backup(&self, request: CreateUserBackupRequest) -> Result<UserBackup> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to serialize create_user_backup request: {e}"
            ))
        })?;
        let response = self
            .make_request("POST", "/hub/backups", Some(payload))
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse create_user_backup response: {e}"))
        })?;
        serde_json::from_value(val.get("backup").cloned().unwrap_or(val)).map_err(|e| {
            VectorizerError::server(format!("Failed to parse user backup from response: {e}"))
        })
    }

    /// Restore a previously created user backup.
    ///
    /// Calls `POST /hub/backups/restore`.
    pub async fn restore_user_backup(&self, request: RestoreUserBackupRequest) -> Result<()> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to serialize restore_user_backup request: {e}"
            ))
        })?;
        self.make_request("POST", "/hub/backups/restore", Some(payload))
            .await?;
        Ok(())
    }

    /// Upload a backup file (raw bytes).
    ///
    /// Calls `POST /hub/backups/upload?user_id={user_id}&name={name}`.
    ///
    /// The request body is sent as raw bytes via a POST. The SDK sends
    /// the binary data as a JSON-encoded base64 string because the
    /// underlying `Transport::post` takes `Option<&serde_json::Value>`.
    /// Callers that need true multipart uploads should use the HTTP
    /// transport directly via `reqwest`.
    pub async fn upload_user_backup(&self, request: UploadUserBackupRequest) -> Result<UserBackup> {
        let mut qs = format!("user_id={}", request.user_id);
        if let Some(name) = &request.name {
            qs.push_str(&format!("&name={name}"));
        }
        let endpoint = format!("/hub/backups/upload?{qs}");
        // Encode binary data as a JSON value so the transport layer can
        // forward it. The server accepts raw bytes; this is a best-effort
        // path. For production uploads use the raw HTTP client.
        let payload = serde_json::json!({ "data": request.data });
        let response = self.make_request("POST", &endpoint, Some(payload)).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse upload_user_backup response: {e}"))
        })?;
        serde_json::from_value(val.get("backup").cloned().unwrap_or(val)).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse user backup from upload response: {e}"
            ))
        })
    }

    /// Fetch metadata for a single backup.
    ///
    /// Calls `GET /hub/backups/{backup_id}?user_id={user_id}`.
    pub async fn get_user_backup(&self, user_id: &str, backup_id: &str) -> Result<UserBackup> {
        let endpoint = format!("/hub/backups/{backup_id}?user_id={user_id}");
        let response = self.make_request("GET", &endpoint, None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_user_backup response: {e}"))
        })?;
        serde_json::from_value(val.get("backup").cloned().unwrap_or(val)).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse user backup from get response: {e}"
            ))
        })
    }

    /// Delete a user backup by id.
    ///
    /// Calls `DELETE /hub/backups/{backup_id}?user_id={user_id}`.
    pub async fn delete_user_backup(&self, user_id: &str, backup_id: &str) -> Result<()> {
        let endpoint = format!("/hub/backups/{backup_id}?user_id={user_id}");
        self.make_request("DELETE", &endpoint, None).await?;
        Ok(())
    }

    /// Download the raw binary data for a backup.
    ///
    /// Calls `GET /hub/backups/{backup_id}/download?user_id={user_id}`.
    ///
    /// The transport layer returns the response body as a `String`;
    /// the SDK re-encodes as UTF-8 bytes. For compressed binary
    /// backups the caller should use the raw HTTP client.
    pub async fn download_user_backup(&self, user_id: &str, backup_id: &str) -> Result<Vec<u8>> {
        let endpoint = format!("/hub/backups/{backup_id}/download?user_id={user_id}");
        let response = self.make_request("GET", &endpoint, None).await?;
        Ok(response.into_bytes())
    }

    /// Get aggregate usage statistics for a user.
    ///
    /// Calls `GET /hub/usage/statistics?user_id={user_id}`.
    pub async fn get_usage_statistics(&self, user_id: &str) -> Result<UsageStatistics> {
        let endpoint = format!("/hub/usage/statistics?user_id={user_id}");
        let response = self.make_request("GET", &endpoint, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse get_usage_statistics response: {e}"
            ))
        })
    }

    /// Get quota information for a user.
    ///
    /// Calls `GET /hub/usage/quota?user_id={user_id}`.
    pub async fn get_quota_info(&self, user_id: &str) -> Result<QuotaInfo> {
        let endpoint = format!("/hub/usage/quota?user_id={user_id}");
        let response = self.make_request("GET", &endpoint, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_quota_info response: {e}"))
        })
    }

    /// Validate a HiveHub API key.
    ///
    /// Calls `POST /hub/validate-key`. The key is sent in the
    /// `Authorization: Bearer <key>` header by the transport layer
    /// when `api_key` is configured on the client; the `key` parameter
    /// here is forwarded in the request body for callers that need to
    /// validate a *different* key than the one on the client.
    pub async fn validate_hub_api_key(&self, key: &str) -> Result<HubApiKeyValidation> {
        let payload = serde_json::json!({ "key": key });
        let response = self
            .make_request("POST", "/hub/validate-key", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse validate_hub_api_key response: {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use crate::models::{
        CreateUserBackupRequest, HubApiKeyValidation, QuotaInfo, RestoreUserBackupRequest,
        UsageStatistics, UserBackup,
    };

    #[test]
    fn user_backup_deserializes() {
        let raw = json!({
            "id": "b-1",
            "user_id": "u-1",
            "name": "my-backup",
            "collections": ["docs"],
            "created_at": "2026-05-02T00:00:00Z",
            "size": 8192u64,
            "status": "active"
        });
        let b: UserBackup = serde_json::from_value(raw).unwrap();
        assert_eq!(b.id, "b-1");
        assert_eq!(b.status, "active");
        assert_eq!(b.size, 8192);
    }

    #[test]
    fn user_backup_round_trip() {
        let b = UserBackup {
            id: "b-2".into(),
            user_id: "u-2".into(),
            name: "weekly".into(),
            description: Some("desc".into()),
            collections: vec!["col1".into()],
            created_at: "2026-05-02T00:00:00Z".into(),
            size: 1024,
            status: "active".into(),
        };
        let serialized = serde_json::to_value(&b).unwrap();
        let parsed: UserBackup = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed.id, "b-2");
        assert_eq!(parsed.description.as_deref(), Some("desc"));
    }

    #[test]
    fn create_user_backup_request_serializes() {
        let req = CreateUserBackupRequest {
            user_id: "u-1".into(),
            name: "nightly".into(),
            description: None,
            collections: Some(vec!["code".into()]),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["user_id"], "u-1");
        assert_eq!(v["name"], "nightly");
        assert_eq!(v["collections"][0], "code");
        // description should be absent
        assert!(v.get("description").is_none());
    }

    #[test]
    fn restore_user_backup_request_serializes() {
        let req = RestoreUserBackupRequest {
            user_id: "u-1".into(),
            backup_id: "b-99".into(),
            overwrite: true,
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["backup_id"], "b-99");
        assert!(v["overwrite"].as_bool().unwrap());
    }

    #[test]
    fn usage_statistics_deserializes() {
        let raw = json!({
            "success": true,
            "message": "ok",
            "stats": {
                "user_id": "u-1",
                "total_collections": 3,
                "total_vectors": 500u64,
                "total_storage": 512000u64
            }
        });
        let us: UsageStatistics = serde_json::from_value(raw).unwrap();
        assert!(us.success);
        assert!(us.stats.is_some());
    }

    #[test]
    fn quota_info_deserializes() {
        let raw = json!({
            "success": true,
            "message": "ok",
            "quota": {
                "tenant_id": "t-1",
                "storage": { "limit": 1_000_000u64, "used": 50_000u64 }
            }
        });
        let qi: QuotaInfo = serde_json::from_value(raw).unwrap();
        assert!(qi.success);
        assert!(qi.quota.is_some());
    }

    #[test]
    fn hub_api_key_validation_deserializes_valid() {
        let raw = json!({
            "valid": true,
            "tenant_id": "t-abc",
            "tenant_name": "Acme",
            "permissions": ["Read", "Write"],
            "validated_at": "2026-05-02T00:00:00Z"
        });
        let v: HubApiKeyValidation = serde_json::from_value(raw).unwrap();
        assert!(v.valid);
        assert_eq!(v.tenant_id, "t-abc");
        assert_eq!(v.permissions.len(), 2);
    }

    #[test]
    fn hub_api_key_validation_deserializes_invalid() {
        let raw = json!({
            "valid": false,
            "tenant_id": "",
            "tenant_name": "",
            "permissions": [],
            "validated_at": ""
        });
        let v: HubApiKeyValidation = serde_json::from_value(raw).unwrap();
        assert!(!v.valid);
    }
}

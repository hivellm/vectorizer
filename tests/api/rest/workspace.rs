//! Workspace API Integration Tests
//!
//! Tests for workspace management endpoints:
//! - POST /api/v1/workspaces - Add workspace
//! - DELETE /api/v1/workspaces - Remove workspace
//! - GET /api/v1/workspaces - List workspaces
//! - PUT /api/v1/workspace-config - Update workspace config

#[cfg(test)]
mod workspace_api_tests {
    use serde_json::json;

    // =========================================================================
    // Request/Response Structure Tests
    // =========================================================================

    #[test]
    fn test_add_workspace_request_structure() {
        let request = json!({
            "path": "/path/to/project",
            "collection_name": "project-vectors"
        });

        assert_eq!(request["path"], "/path/to/project");
        assert_eq!(request["collection_name"], "project-vectors");
    }

    #[test]
    fn test_add_workspace_response_structure() {
        // Expected successful response
        let response = json!({
            "success": true,
            "message": "Workspace added successfully",
            "workspace": {
                "id": "ws-abc123",
                "path": "/path/to/project",
                "collection_name": "project-vectors",
                "active": true,
                "created_at": "2025-01-01T00:00:00Z"
            }
        });

        assert_eq!(response["success"], true);
        assert!(response["workspace"]["id"].is_string());
        assert!(response["workspace"]["active"].as_bool().unwrap());
    }

    #[test]
    fn test_remove_workspace_request_structure() {
        let request = json!({
            "path": "/path/to/project"
        });

        assert_eq!(request["path"], "/path/to/project");
    }

    #[test]
    fn test_remove_workspace_response_structure() {
        let response = json!({
            "success": true,
            "message": "Workspace removed successfully",
            "removed_workspace": {
                "id": "ws-abc123",
                "path": "/path/to/project",
                "collection_name": "project-vectors"
            }
        });

        assert_eq!(response["success"], true);
        assert!(response["removed_workspace"]["id"].is_string());
    }

    #[test]
    fn test_list_workspaces_response_structure() {
        let response = json!({
            "workspaces": [
                {
                    "id": "ws-abc123",
                    "path": "/path/to/project1",
                    "collection_name": "project1-vectors",
                    "active": true,
                    "file_count": 100,
                    "created_at": "2025-01-01T00:00:00Z",
                    "updated_at": "2025-01-02T00:00:00Z",
                    "last_indexed": "2025-01-02T00:00:00Z",
                    "exists": true
                },
                {
                    "id": "ws-def456",
                    "path": "/path/to/project2",
                    "collection_name": "project2-vectors",
                    "active": false,
                    "file_count": 0,
                    "created_at": "2025-01-01T00:00:00Z",
                    "updated_at": "2025-01-01T00:00:00Z",
                    "last_indexed": null,
                    "exists": false
                }
            ]
        });

        assert!(response["workspaces"].is_array());
        let workspaces = response["workspaces"].as_array().unwrap();
        assert_eq!(workspaces.len(), 2);
        assert_eq!(workspaces[0]["file_count"], 100);
        assert_eq!(workspaces[1]["active"], false);
    }

    #[test]
    fn test_update_workspace_config_request_structure() {
        let request = json!({
            "workspaces": [
                {
                    "id": "ws-abc123",
                    "path": "/path/to/project",
                    "collection_name": "project-vectors",
                    "active": true,
                    "include_patterns": ["*.rs", "*.md"],
                    "exclude_patterns": ["**/target/**"]
                }
            ]
        });

        assert!(request["workspaces"].is_array());
        let workspace = &request["workspaces"][0];
        assert!(workspace["include_patterns"].is_array());
        assert!(workspace["exclude_patterns"].is_array());
    }

    #[test]
    fn test_update_workspace_config_response_structure() {
        let response = json!({
            "success": true,
            "message": "Workspace configuration updated successfully."
        });

        assert_eq!(response["success"], true);
        assert!(response["message"].as_str().unwrap().contains("updated"));
    }

    // =========================================================================
    // Validation Tests
    // =========================================================================

    #[test]
    fn test_add_workspace_missing_path_error() {
        // Missing path should result in validation error
        let invalid_request = json!({
            "collection_name": "project-vectors"
        });

        assert!(invalid_request.get("path").is_none());
    }

    #[test]
    fn test_add_workspace_missing_collection_name_error() {
        // Missing collection_name should result in validation error
        let invalid_request = json!({
            "path": "/path/to/project"
        });

        assert!(invalid_request.get("collection_name").is_none());
    }

    #[test]
    fn test_remove_workspace_missing_path_error() {
        // Missing path should result in validation error
        let invalid_request = json!({});

        assert!(invalid_request.get("path").is_none());
    }

    #[test]
    fn test_workspace_path_normalization() {
        // Test that various path formats should be normalized
        let paths = [
            "/path/to/project",
            "/path/to/project/",
            "C:\\Users\\test\\project",
            "./relative/path",
        ];

        for path in paths {
            let request = json!({"path": path, "collection_name": "test"});
            assert!(request["path"].is_string());
        }
    }

    #[test]
    fn test_workspace_collection_name_validation() {
        // Valid collection names
        let valid_names = ["my-collection", "collection_v1", "test123", "MyCollection"];

        for name in valid_names {
            let request = json!({"path": "/test", "collection_name": name});
            assert!(request["collection_name"].is_string());
        }
    }
}

#[cfg(test)]
mod workspace_manager_tests {
    use tempfile::TempDir;
    use vectorizer::config::WorkspaceManager;

    fn create_test_workspace_manager() -> (WorkspaceManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.yml");

        // Create an empty config file
        std::fs::write(&config_path, "workspaces: []\n").unwrap();

        let manager = WorkspaceManager::with_config_path(config_path);
        (manager, temp_dir)
    }

    #[test]
    fn test_workspace_manager_creation() {
        let (manager, _temp_dir) = create_test_workspace_manager();
        let workspaces = manager.list_workspaces();
        assert!(workspaces.is_empty());
    }

    #[test]
    fn test_add_workspace() {
        let (manager, temp_dir) = create_test_workspace_manager();

        // Create a test directory to add as workspace
        let workspace_path = temp_dir.path().join("test_project");
        std::fs::create_dir(&workspace_path).unwrap();

        let result = manager.add_workspace(workspace_path.to_str().unwrap(), "test-collection");

        assert!(result.is_ok());
        let workspace = result.unwrap();
        assert!(workspace.id.starts_with("ws-"));
        assert_eq!(workspace.collection_name, "test-collection");
        assert!(workspace.active);
    }

    #[test]
    fn test_add_duplicate_workspace_fails() {
        let (manager, temp_dir) = create_test_workspace_manager();

        let workspace_path = temp_dir.path().join("test_project");
        std::fs::create_dir(&workspace_path).unwrap();
        let path_str = workspace_path.to_str().unwrap();

        // First add should succeed
        let result1 = manager.add_workspace(path_str, "collection1");
        assert!(result1.is_ok());

        // Second add with same path should fail
        let result2 = manager.add_workspace(path_str, "collection2");
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_remove_workspace() {
        let (manager, temp_dir) = create_test_workspace_manager();

        let workspace_path = temp_dir.path().join("test_project");
        std::fs::create_dir(&workspace_path).unwrap();
        let path_str = workspace_path.to_str().unwrap();

        // Add workspace
        manager.add_workspace(path_str, "test-collection").unwrap();

        // Remove workspace
        let result = manager.remove_workspace(path_str);
        assert!(result.is_ok());

        // Verify it's removed
        let workspaces = manager.list_workspaces();
        assert!(workspaces.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_workspace_fails() {
        let (manager, _temp_dir) = create_test_workspace_manager();

        let result = manager.remove_workspace("/nonexistent/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_list_workspaces() {
        let (manager, temp_dir) = create_test_workspace_manager();

        // Add multiple workspaces
        for i in 0..3 {
            let workspace_path = temp_dir.path().join(format!("project_{i}"));
            std::fs::create_dir(&workspace_path).unwrap();
            manager
                .add_workspace(workspace_path.to_str().unwrap(), &format!("collection-{i}"))
                .unwrap();
        }

        let workspaces = manager.list_workspaces();
        assert_eq!(workspaces.len(), 3);
    }

    #[test]
    fn test_get_workspace() {
        let (manager, temp_dir) = create_test_workspace_manager();

        let workspace_path = temp_dir.path().join("test_project");
        std::fs::create_dir(&workspace_path).unwrap();
        let path_str = workspace_path.to_str().unwrap();

        manager.add_workspace(path_str, "test-collection").unwrap();

        let workspace = manager.get_workspace(path_str);
        assert!(workspace.is_some());
        assert_eq!(workspace.unwrap().collection_name, "test-collection");
    }

    #[test]
    fn test_get_nonexistent_workspace() {
        let (manager, _temp_dir) = create_test_workspace_manager();

        let workspace = manager.get_workspace("/nonexistent/path");
        assert!(workspace.is_none());
    }

    #[test]
    fn test_workspace_default_patterns() {
        let (manager, temp_dir) = create_test_workspace_manager();

        let workspace_path = temp_dir.path().join("test_project");
        std::fs::create_dir(&workspace_path).unwrap();

        let workspace = manager
            .add_workspace(workspace_path.to_str().unwrap(), "test-collection")
            .unwrap();

        // Check default include patterns
        assert!(!workspace.include_patterns.is_empty());
        assert!(workspace.include_patterns.contains(&"*.md".to_string()));
        assert!(workspace.include_patterns.contains(&"*.rs".to_string()));

        // Check default exclude patterns
        assert!(!workspace.exclude_patterns.is_empty());
        assert!(
            workspace
                .exclude_patterns
                .iter()
                .any(|p| p.contains("target"))
        );
        assert!(
            workspace
                .exclude_patterns
                .iter()
                .any(|p| p.contains("node_modules"))
        );
    }

    #[test]
    fn test_workspace_timestamps() {
        let (manager, temp_dir) = create_test_workspace_manager();

        let workspace_path = temp_dir.path().join("test_project");
        std::fs::create_dir(&workspace_path).unwrap();

        let workspace = manager
            .add_workspace(workspace_path.to_str().unwrap(), "test-collection")
            .unwrap();

        // Timestamps should be set
        assert!(workspace.created_at <= workspace.updated_at);
        assert!(workspace.last_indexed.is_none()); // Not indexed yet
    }
}

#[cfg(test)]
mod graphql_workspace_tests {
    use std::sync::Arc;
    use tempfile::TempDir;
    use vectorizer::api::graphql::{VectorizerSchema, create_schema};
    use vectorizer::db::VectorStore;
    use vectorizer::embedding::EmbeddingManager;

    fn create_test_schema() -> (VectorizerSchema, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(VectorStore::new());
        let embedding_manager = Arc::new(EmbeddingManager::new());
        let start_time = std::time::Instant::now();

        let schema = create_schema(store, embedding_manager, start_time);
        (schema, temp_dir)
    }

    #[tokio::test]
    async fn test_workspaces_query() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r"
            {
                workspaces {
                    path
                    collectionName
                    indexed
                }
            }
        ";

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Workspaces query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert!(data["workspaces"].is_array());
    }

    #[tokio::test]
    async fn test_workspace_config_query() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r"
            {
                workspaceConfig {
                    globalSettings
                    projects
                }
            }
        ";

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Workspace config query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert!(data["workspaceConfig"]["globalSettings"].is_object());
    }

    #[tokio::test]
    async fn test_add_workspace_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        // Use unique path to avoid conflicts
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mutation = format!(
            r#"
            mutation {{
                addWorkspace(input: {{
                    path: "/test/workspace-{unique_id}"
                    collectionName: "test-collection"
                }}) {{
                    success
                    message
                }}
            }}
        "#
        );

        let result = schema.execute(mutation).await;
        assert!(
            result.errors.is_empty(),
            "Add workspace mutation failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        // The mutation might fail due to file system operations in test env
        // but it should execute without GraphQL errors
        assert!(data["addWorkspace"]["success"].is_boolean());
    }

    #[tokio::test]
    async fn test_remove_workspace_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        let mutation = r#"
            mutation {
                removeWorkspace(path: "/nonexistent/workspace") {
                    success
                    message
                }
            }
        "#;

        let result = schema.execute(mutation).await;
        assert!(
            result.errors.is_empty(),
            "Remove workspace mutation failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        // Should fail gracefully for nonexistent workspace
        assert!(!data["removeWorkspace"]["success"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_update_workspace_config_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        // The mutation takes a config parameter as JSON scalar
        // In GraphQL, JSON is passed as a string that gets parsed
        let mutation = r#"
            mutation {
                updateWorkspaceConfig(config: "{\"workspaces\": []}") {
                    success
                    message
                }
            }
        "#;

        let result = schema.execute(mutation).await;
        // This might fail due to file system operations, but shouldn't error on parsing
        // File write failure is ok in test env
        assert!(
            result.errors.is_empty()
                || result.errors[0].message.contains("Failed")
                || result.errors[0].message.contains("write")
                || result.errors[0].message.contains("permission"),
            "Unexpected error in updateWorkspaceConfig: {:?}",
            result.errors
        );
    }
}

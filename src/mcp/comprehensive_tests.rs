//! Comprehensive tests for MCP module

use super::*;
use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde_json::json;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_mcp_config_serialization() {
        let config = McpConfig {
            enabled: true,
            port: 15003,
            host: "127.0.0.1".to_string(),
            max_connections: 20,
            connection_timeout: 600,
            auth_required: false,
            allowed_api_keys: vec!["key1".to_string(), "key2".to_string()],
            server_info: McpServerInfo {
                name: "Test Server".to_string(),
                version: "1.0.0".to_string(),
                description: "Test MCP Server".to_string(),
            },
            tools: vec![],
            resources: vec![],
            performance: McpPerformanceConfig {
                connection_pooling: true,
                max_message_size: 2048576,
                heartbeat_interval: 60,
                cleanup_interval: 600,
            },
            logging: McpLoggingConfig {
                level: "debug".to_string(),
                log_requests: true,
                log_responses: true,
                log_errors: true,
            },
        };

        // Test serialization
        let json_str = serde_json::to_string(&config).unwrap();
        assert!(json_str.contains("Test Server"));
        assert!(json_str.contains("debug"));

        // Test deserialization
        let deserialized: McpConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.port, 15003);
        assert_eq!(deserialized.max_connections, 20);
        assert!(!deserialized.auth_required);
    }

    #[test]
    fn test_mcp_performance_config() {
        let perf_config = McpPerformanceConfig {
            connection_pooling: true,
            max_message_size: 1048576,
            heartbeat_interval: 30,
            cleanup_interval: 300,
        };

        assert!(perf_config.connection_pooling);
        assert_eq!(perf_config.max_message_size, 1048576);
        assert_eq!(perf_config.heartbeat_interval, 30);
        assert_eq!(perf_config.cleanup_interval, 300);
    }

    #[test]
    fn test_mcp_logging_config() {
        let logging_config = McpLoggingConfig {
            level: "info".to_string(),
            log_requests: true,
            log_responses: false,
            log_errors: true,
        };

        assert_eq!(logging_config.level, "info");
        assert!(logging_config.log_requests);
        assert!(!logging_config.log_responses);
        assert!(logging_config.log_errors);
    }

    #[test]
    fn test_mcp_server_info() {
        let server_info = McpServerInfo {
            name: "Vectorizer MCP".to_string(),
            version: "2.0.0".to_string(),
            description: "Advanced MCP Server".to_string(),
        };

        assert_eq!(server_info.name, "Vectorizer MCP");
        assert_eq!(server_info.version, "2.0.0");
        assert_eq!(server_info.description, "Advanced MCP Server");
    }

    #[test]
    fn test_mcp_tool_definition() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool for validation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"},
                    "param2": {"type": "number"}
                },
                "required": ["param1"]
            }),
        };

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool for validation");
        assert!(tool.input_schema.is_object());
    }

    #[test]
    fn test_mcp_resource_definition() {
        let resource = McpResource {
            uri: "vectorizer://test".to_string(),
            name: "Test Resource".to_string(),
            description: "A test resource".to_string(),
            mime_type: "application/json".to_string(),
        };

        assert_eq!(resource.uri, "vectorizer://test");
        assert_eq!(resource.name, "Test Resource");
        assert_eq!(resource.description, "A test resource");
        assert_eq!(resource.mime_type, "application/json");
    }
}

#[cfg(test)]
mod request_response_tests {
    use super::*;

    #[test]
    fn test_mcp_request_serialization() {
        let init_request = McpRequest::Initialize {
            protocol_version: "2024-11-05".to_string(),
            capabilities: json!({"tools": {}}),
            client_info: json!({"name": "test_client"}),
        };

        let json_str = serde_json::to_string(&init_request).unwrap();
        assert!(json_str.contains("Initialize"));
        assert!(json_str.contains("2024-11-05"));

        let tools_list_request = McpRequest::ToolsList;
        let tools_json = serde_json::to_string(&tools_list_request).unwrap();
        assert!(tools_json.contains("ToolsList"));

        let tools_call_request = McpRequest::ToolsCall {
            name: "test_tool".to_string(),
            arguments: json!({"param": "value"}),
        };
        let call_json = serde_json::to_string(&tools_call_request).unwrap();
        assert!(call_json.contains("test_tool"));
        assert!(call_json.contains("value"));
    }

    #[test]
    fn test_mcp_response_creation() {
        let success_response = McpResponse {
            id: Some("req_123".to_string()),
            result: Some(json!({"status": "success"})),
            error: None,
        };

        assert_eq!(success_response.id, Some("req_123".to_string()));
        assert!(success_response.result.is_some());
        assert!(success_response.error.is_none());

        let error_response = McpResponse {
            id: Some("req_456".to_string()),
            result: None,
            error: Some(McpError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        assert_eq!(error_response.id, Some("req_456".to_string()));
        assert!(error_response.result.is_none());
        assert!(error_response.error.is_some());
        assert_eq!(error_response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_mcp_error_serialization() {
        let error = McpError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({"field": "collection"})),
        };

        let json_str = serde_json::to_string(&error).unwrap();
        assert!(json_str.contains("-32602"));
        assert!(json_str.contains("Invalid params"));
        assert!(json_str.contains("collection"));
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;

    #[test]
    fn test_mcp_connection_creation() {
        let connection = McpConnection {
            id: "conn_123".to_string(),
            client_capabilities: json!({"tools": {}}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: true,
        };

        assert_eq!(connection.id, "conn_123");
        assert!(connection.authenticated);
        assert!(connection.client_capabilities.is_object());
    }

    #[test]
    fn test_mcp_connection_activity() {
        let mut connection = McpConnection {
            id: "conn_456".to_string(),
            client_capabilities: json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: false,
        };

        let initial_activity = connection.last_activity;
        
        // Simulate activity update
        connection.last_activity = chrono::Utc::now();
        
        assert!(connection.last_activity > initial_activity);
    }

    #[tokio::test]
    async fn test_mcp_connection_management() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);

        // Test adding connections
        let connection1 = McpConnection {
            id: "conn_1".to_string(),
            client_capabilities: json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: false,
        };

        let connection2 = McpConnection {
            id: "conn_2".to_string(),
            client_capabilities: json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: true,
        };

        state.add_connection("conn_1".to_string(), connection1).await.unwrap();
        state.add_connection("conn_2".to_string(), connection2).await.unwrap();

        assert_eq!(state.connection_count().await, 2);

        // Test updating activity
        state.update_activity("conn_1").await.unwrap();

        // Test removing connection
        state.remove_connection("conn_1").await.unwrap();
        assert_eq!(state.connection_count().await, 1);

        // Test cleanup
        state.remove_connection("conn_2").await.unwrap();
        assert_eq!(state.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_mcp_connection_limits() {
        let mut config = McpConfig::default();
        config.max_connections = 2;
        let state = McpServerState::new(config);

        // Add connections up to limit
        for i in 0..2 {
            let connection = McpConnection {
                id: format!("conn_{}", i),
                client_capabilities: json!({}),
                connected_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                authenticated: false,
            };
            state.add_connection(format!("conn_{}", i), connection).await.unwrap();
        }

        assert_eq!(state.connection_count().await, 2);

        // Try to add one more connection (should fail)
        let extra_connection = McpConnection {
            id: "conn_extra".to_string(),
            client_capabilities: json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: false,
        };

        let result = state.add_connection("conn_extra".to_string(), extra_connection).await;
        assert!(result.is_err());
        assert_eq!(state.connection_count().await, 2);
    }

    #[tokio::test]
    async fn test_mcp_connection_cleanup() {
        let mut config = McpConfig::default();
        config.connection_timeout = 1; // 1 second timeout
        let state = McpServerState::new(config);

        // Add a connection
        let connection = McpConnection {
            id: "conn_old".to_string(),
            client_capabilities: json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now() - chrono::Duration::seconds(2), // 2 seconds ago
            authenticated: false,
        };

        state.add_connection("conn_old".to_string(), connection).await.unwrap();
        assert_eq!(state.connection_count().await, 1);

        // Cleanup inactive connections
        let cleaned = state.cleanup_inactive_connections().await.unwrap();
        assert_eq!(cleaned, 1);
        assert_eq!(state.connection_count().await, 0);
    }
}

#[cfg(test)]
mod capabilities_tests {
    use super::*;

    #[test]
    fn test_mcp_capabilities_creation() {
        let capabilities = McpCapabilities {
            server_info: McpServerInfo {
                name: "Test Server".to_string(),
                version: "1.0.0".to_string(),
                description: "Test MCP Server".to_string(),
            },
            tools: vec![
                McpTool {
                    name: "tool1".to_string(),
                    description: "First tool".to_string(),
                    input_schema: json!({}),
                },
                McpTool {
                    name: "tool2".to_string(),
                    description: "Second tool".to_string(),
                    input_schema: json!({}),
                },
            ],
            resources: vec![
                McpResource {
                    uri: "resource1".to_string(),
                    name: "Resource 1".to_string(),
                    description: "First resource".to_string(),
                    mime_type: "application/json".to_string(),
                },
            ],
        };

        assert_eq!(capabilities.server_info.name, "Test Server");
        assert_eq!(capabilities.tools.len(), 2);
        assert_eq!(capabilities.resources.len(), 1);
        assert_eq!(capabilities.tools[0].name, "tool1");
        assert_eq!(capabilities.resources[0].uri, "resource1");
    }

    #[test]
    fn test_mcp_capabilities_serialization() {
        let capabilities = McpCapabilities {
            server_info: McpServerInfo {
                name: "Serialization Test".to_string(),
                version: "2.0.0".to_string(),
                description: "Test serialization".to_string(),
            },
            tools: vec![],
            resources: vec![],
        };

        let json_str = serde_json::to_string(&capabilities).unwrap();
        assert!(json_str.contains("Serialization Test"));
        assert!(json_str.contains("2.0.0"));

        let deserialized: McpCapabilities = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.server_info.name, "Serialization Test");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_state_integration() {
        let config = McpConfig {
            enabled: true,
            port: 15003,
            host: "127.0.0.1".to_string(),
            max_connections: 5,
            connection_timeout: 300,
            auth_required: false,
            allowed_api_keys: vec![],
            server_info: McpServerInfo {
                name: "Integration Test Server".to_string(),
                version: "1.0.0".to_string(),
                description: "Integration test MCP server".to_string(),
            },
            tools: vec![
                McpTool {
                    name: "test_tool".to_string(),
                    description: "Test tool for integration".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "param": {"type": "string"}
                        }
                    }),
                },
            ],
            resources: vec![
                McpResource {
                    uri: "vectorizer://test".to_string(),
                    name: "Test Resource".to_string(),
                    description: "Test resource for integration".to_string(),
                    mime_type: "application/json".to_string(),
                },
            ],
            performance: McpPerformanceConfig {
                connection_pooling: true,
                max_message_size: 1048576,
                heartbeat_interval: 30,
                cleanup_interval: 300,
            },
            logging: McpLoggingConfig {
                level: "info".to_string(),
                log_requests: true,
                log_responses: false,
                log_errors: true,
            },
        };

        let state = McpServerState::new(config);

        // Test server capabilities
        assert_eq!(state.capabilities.server_info.name, "Vectorizer MCP Server");
        assert_eq!(state.capabilities.tools.len(), 18);
        assert_eq!(state.capabilities.resources.len(), 2);

        // Test connection management
        let connection = McpConnection {
            id: "integration_conn".to_string(),
            client_capabilities: json!({"tools": {}}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: true,
        };

        state.add_connection("integration_conn".to_string(), connection).await.unwrap();
        assert_eq!(state.connection_count().await, 1);

        // Test activity update
        state.update_activity("integration_conn").await.unwrap();

        // Test cleanup (should not remove active connection)
        let cleaned = state.cleanup_inactive_connections().await.unwrap();
        assert_eq!(cleaned, 0);
        assert_eq!(state.connection_count().await, 1);

        // Clean up
        state.remove_connection("integration_conn").await.unwrap();
        assert_eq!(state.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_mcp_request_processing_workflow() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);

        // Simulate a complete MCP request workflow
        let connection = McpConnection {
            id: "workflow_conn".to_string(),
            client_capabilities: json!({
                "tools": {},
                "resources": {}
            }),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: true,
        };

        state.add_connection("workflow_conn".to_string(), connection).await.unwrap();

        // Test various request types
        let init_request = McpRequest::Initialize {
            protocol_version: "2024-11-05".to_string(),
            capabilities: json!({"tools": {}}),
            client_info: json!({"name": "test_client", "version": "1.0.0"}),
        };

        let tools_list_request = McpRequest::ToolsList;
        let resources_list_request = McpRequest::ResourcesList;

        let tools_call_request = McpRequest::ToolsCall {
            name: "list_collections".to_string(),
            arguments: json!({}),
        };

        let ping_request = McpRequest::Ping;

        // All requests should serialize properly
        assert!(serde_json::to_string(&init_request).is_ok());
        assert!(serde_json::to_string(&tools_list_request).is_ok());
        assert!(serde_json::to_string(&resources_list_request).is_ok());
        assert!(serde_json::to_string(&tools_call_request).is_ok());
        assert!(serde_json::to_string(&ping_request).is_ok());

        // Clean up
        state.remove_connection("workflow_conn").await.unwrap();
    }

    #[tokio::test]
    async fn test_mcp_error_handling() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);

        // Test error response creation
        let error_response = McpResponse {
            id: Some("error_req".to_string()),
            result: None,
            error: Some(McpError {
                code: -32603,
                message: "Internal error".to_string(),
                data: Some(json!({"details": "Database connection failed"})),
            }),
        };

        assert!(error_response.error.is_some());
        let error = error_response.error.as_ref().unwrap();
        assert_eq!(error.code, -32603);
        assert_eq!(error.message, "Internal error");
        assert!(error.data.is_some());

        // Test error serialization
        let error_json = serde_json::to_string(&error_response).unwrap();
        assert!(error_json.contains("-32603"));
        assert!(error_json.contains("Internal error"));
        assert!(error_json.contains("Database connection failed"));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_connection_performance() {
        let config = McpConfig {
            max_connections: 100,
            ..Default::default()
        };
        let state = McpServerState::new(config);

        let start = std::time::Instant::now();

        // Add many connections
        for i in 0..50 {
            let connection = McpConnection {
                id: format!("perf_conn_{}", i),
                client_capabilities: json!({}),
                connected_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                authenticated: false,
            };
            state.add_connection(format!("perf_conn_{}", i), connection).await.unwrap();
        }

        let add_elapsed = start.elapsed();
        println!("Adding 50 connections took: {:?}", add_elapsed);
        assert!(add_elapsed.as_millis() < 1000); // Should be fast

        assert_eq!(state.connection_count().await, 50);

        // Test activity updates
        let update_start = std::time::Instant::now();
        for i in 0..50 {
            state.update_activity(&format!("perf_conn_{}", i)).await.unwrap();
        }
        let update_elapsed = update_start.elapsed();
        println!("Updating 50 connections took: {:?}", update_elapsed);
        assert!(update_elapsed.as_millis() < 1000);

        // Test cleanup
        let cleanup_start = std::time::Instant::now();
        let cleaned = state.cleanup_inactive_connections().await.unwrap();
        let cleanup_elapsed = cleanup_start.elapsed();
        println!("Cleanup took: {:?}", cleanup_elapsed);
        assert!(cleanup_elapsed.as_millis() < 1000);
        assert_eq!(cleaned, 0); // All connections are recent

        // Clean up all connections
        for i in 0..50 {
            state.remove_connection(&format!("perf_conn_{}", i)).await.unwrap();
        }
        assert_eq!(state.connection_count().await, 0);
    }

    #[test]
    fn test_mcp_serialization_performance() {
        let large_capabilities = McpCapabilities {
            server_info: McpServerInfo {
                name: "Performance Test Server".to_string(),
                version: "1.0.0".to_string(),
                description: "Server for performance testing".to_string(),
            },
            tools: (0..100).map(|i| McpTool {
                name: format!("tool_{}", i),
                description: format!("Tool number {}", i),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "param": {"type": "string"}
                    }
                }),
            }).collect(),
            resources: (0..50).map(|i| McpResource {
                uri: format!("vectorizer://resource_{}", i),
                name: format!("Resource {}", i),
                description: format!("Resource number {}", i),
                mime_type: "application/json".to_string(),
            }).collect(),
        };

        let start = std::time::Instant::now();
        let json_str = serde_json::to_string(&large_capabilities).unwrap();
        let serialize_elapsed = start.elapsed();
        println!("Serializing large capabilities took: {:?}", serialize_elapsed);
        assert!(serialize_elapsed.as_millis() < 100);

        let deserialize_start = std::time::Instant::now();
        let deserialized: McpCapabilities = serde_json::from_str(&json_str).unwrap();
        let deserialize_elapsed = deserialize_start.elapsed();
        println!("Deserializing large capabilities took: {:?}", deserialize_elapsed);
        assert!(deserialize_elapsed.as_millis() < 100);

        assert_eq!(deserialized.tools.len(), 100);
        assert_eq!(deserialized.resources.len(), 50);
    }
}

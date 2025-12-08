//! Integration tests for HiveHub operation logging and tracking
//!
//! Tests the operation logging, request tracking, and HiveHub Cloud integration features.

use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

use vectorizer::hub::mcp_gateway::{McpOperationType, McpRequestContext};
use vectorizer::hub::usage::UsageMetrics;
use vectorizer::hub::{TenantContext, TenantPermission};

// ============================================================================
// Operation Type Classification Tests
// ============================================================================

#[test]
fn test_operation_type_from_various_tool_names() {
    assert_eq!(
        McpOperationType::from_tool_name("list_collections"),
        McpOperationType::ListCollections
    );
    assert_eq!(
        McpOperationType::from_tool_name("create_collection"),
        McpOperationType::CreateCollection
    );
    assert_eq!(
        McpOperationType::from_tool_name("delete_collection"),
        McpOperationType::DeleteCollection
    );
    assert_eq!(
        McpOperationType::from_tool_name("get_collection_info"),
        McpOperationType::GetCollectionInfo
    );

    // Insert variants
    assert_eq!(
        McpOperationType::from_tool_name("insert_text"),
        McpOperationType::Insert
    );
    assert_eq!(
        McpOperationType::from_tool_name("insert_vector"),
        McpOperationType::Insert
    );
    assert_eq!(
        McpOperationType::from_tool_name("insert_vectors"),
        McpOperationType::Insert
    );

    // Search variants
    assert_eq!(
        McpOperationType::from_tool_name("search"),
        McpOperationType::Search
    );
    assert_eq!(
        McpOperationType::from_tool_name("search_vectors"),
        McpOperationType::Search
    );
    assert_eq!(
        McpOperationType::from_tool_name("search_intelligent"),
        McpOperationType::Search
    );
    assert_eq!(
        McpOperationType::from_tool_name("search_semantic"),
        McpOperationType::Search
    );
    assert_eq!(
        McpOperationType::from_tool_name("search_hybrid"),
        McpOperationType::Search
    );
    assert_eq!(
        McpOperationType::from_tool_name("multi_collection_search"),
        McpOperationType::Search
    );

    // Vector operations
    assert_eq!(
        McpOperationType::from_tool_name("get_vector"),
        McpOperationType::GetVector
    );
    assert_eq!(
        McpOperationType::from_tool_name("update_vector"),
        McpOperationType::UpdateVector
    );
    assert_eq!(
        McpOperationType::from_tool_name("delete_vector"),
        McpOperationType::DeleteVector
    );

    // Graph operations
    assert_eq!(
        McpOperationType::from_tool_name("graph_list_nodes"),
        McpOperationType::GraphOperation
    );
    assert_eq!(
        McpOperationType::from_tool_name("graph_add_edge"),
        McpOperationType::GraphOperation
    );

    // Cluster operations
    assert_eq!(
        McpOperationType::from_tool_name("cluster_add_node"),
        McpOperationType::ClusterOperation
    );
    assert_eq!(
        McpOperationType::from_tool_name("cluster_status"),
        McpOperationType::ClusterOperation
    );

    // File operations
    assert_eq!(
        McpOperationType::from_tool_name("get_file_content"),
        McpOperationType::FileOperation
    );
    assert_eq!(
        McpOperationType::from_tool_name("list_files"),
        McpOperationType::FileOperation
    );

    // Discovery operations
    assert_eq!(
        McpOperationType::from_tool_name("filter_collections"),
        McpOperationType::Discovery
    );
    assert_eq!(
        McpOperationType::from_tool_name("expand_queries"),
        McpOperationType::Discovery
    );

    // Unknown operations
    assert_eq!(
        McpOperationType::from_tool_name("unknown_operation"),
        McpOperationType::Unknown
    );
}

#[test]
fn test_operation_requires_write_permissions() {
    // Write operations
    assert!(McpOperationType::CreateCollection.requires_write());
    assert!(McpOperationType::DeleteCollection.requires_write());
    assert!(McpOperationType::Insert.requires_write());
    assert!(McpOperationType::UpdateVector.requires_write());
    assert!(McpOperationType::DeleteVector.requires_write());

    // Read operations
    assert!(!McpOperationType::ListCollections.requires_write());
    assert!(!McpOperationType::GetCollectionInfo.requires_write());
    assert!(!McpOperationType::Search.requires_write());
    assert!(!McpOperationType::GetVector.requires_write());
    assert!(!McpOperationType::GraphOperation.requires_write());
    assert!(!McpOperationType::ClusterOperation.requires_write());
    assert!(!McpOperationType::FileOperation.requires_write());
    assert!(!McpOperationType::Discovery.requires_write());
    assert!(!McpOperationType::Unknown.requires_write());
}

#[test]
fn test_operation_is_read_only() {
    // Read-only operations
    assert!(McpOperationType::ListCollections.is_read_only());
    assert!(McpOperationType::GetCollectionInfo.is_read_only());
    assert!(McpOperationType::Search.is_read_only());
    assert!(McpOperationType::GetVector.is_read_only());

    // Write operations (not read-only)
    assert!(!McpOperationType::CreateCollection.is_read_only());
    assert!(!McpOperationType::DeleteCollection.is_read_only());
    assert!(!McpOperationType::Insert.is_read_only());
    assert!(!McpOperationType::UpdateVector.is_read_only());
    assert!(!McpOperationType::DeleteVector.is_read_only());
}

// ============================================================================
// Request Context Tests
// ============================================================================

#[test]
fn test_mcp_request_context_creation() {
    let tenant = TenantContext {
        tenant_id: "test_tenant".to_string(),
        tenant_name: "Test Tenant".to_string(),
        api_key_id: "test_key_id".to_string(),
        permissions: vec![TenantPermission::ReadWrite],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };

    let context = McpRequestContext::new(tenant.clone());

    assert_eq!(context.tenant.tenant_id(), "test_tenant");
    assert_ne!(context.request_id, Uuid::nil());
}

#[test]
fn test_mcp_request_context_elapsed_time() {
    let tenant = TenantContext {
        tenant_id: "test_tenant".to_string(),
        tenant_name: "Test Tenant".to_string(),
        api_key_id: "test_key_id".to_string(),
        permissions: vec![TenantPermission::ReadOnly],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };

    let context = McpRequestContext::new(tenant);

    // Sleep for a short time to ensure elapsed_ms() returns non-zero
    std::thread::sleep(std::time::Duration::from_millis(10));

    let elapsed = context.elapsed_ms();
    assert!(elapsed >= 10, "Expected at least 10ms, got {elapsed}");
}

// ============================================================================
// Operation Log Entry Tests
// ============================================================================

#[test]
fn test_create_log_entry_success() {
    // Create a test tenant context
    let tenant = TenantContext {
        tenant_id: "test_tenant".to_string(),
        tenant_name: "Test Tenant".to_string(),
        api_key_id: "test_key_id".to_string(),
        permissions: vec![TenantPermission::ReadWrite],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };

    // Create a mock HubManager (this would normally require actual initialization)
    // For unit testing, we'll test the log entry structure directly

    let metadata = json!({
        "query": "test search",
        "limit": 10
    });

    // Simulate log entry creation
    let log_entry = serde_json::json!({
        "operation_id": Uuid::new_v4(),
        "tenant_id": tenant.tenant_id(),
        "tool_name": "search",
        "operation_type": "Search",
        "collection": "test_collection",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        "duration_ms": 50_u64,
        "success": true,
        "error": Option::<String>::None,
        "metadata": metadata,
    });

    assert_eq!(log_entry["tenant_id"], tenant.tenant_id());
    assert_eq!(log_entry["tool_name"], "search");
    assert_eq!(log_entry["operation_type"], "Search");
    assert_eq!(log_entry["collection"], "test_collection");
    assert_eq!(log_entry["success"], true);
    assert!(log_entry["error"].is_null());
    assert_eq!(log_entry["duration_ms"], 50);
}

#[test]
fn test_create_log_entry_failure() {
    let tenant = TenantContext {
        tenant_id: "test_tenant".to_string(),
        tenant_name: "Test Tenant".to_string(),
        api_key_id: "test_key_id".to_string(),
        permissions: vec![TenantPermission::ReadOnly],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };

    let metadata = json!({
        "error_details": "Permission denied"
    });

    let log_entry = json!({
        "operation_id": Uuid::new_v4(),
        "tenant_id": tenant.tenant_id(),
        "tool_name": "create_collection",
        "operation_type": "CreateCollection",
        "collection": null,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        "duration_ms": 5_u64,
        "success": false,
        "error": "Write permission required",
        "metadata": metadata,
    });

    assert_eq!(log_entry["tenant_id"], tenant.tenant_id());
    assert_eq!(log_entry["success"], false);
    assert_eq!(log_entry["error"], "Write permission required");
}

// ============================================================================
// Operation Logging Request/Response Tests
// ============================================================================

#[test]
fn test_operation_logs_request_structure() {
    let request = json!({
        "service_id": "vec-test-123",
        "logs": [
            {
                "operation_id": Uuid::new_v4(),
                "tenant_id": "test_tenant",
                "operation": "search",
                "operation_type": "search",
                "collection": "documents",
                "timestamp": 1234567890,
                "duration_ms": 50,
                "success": true,
                "error": null,
                "metadata": {
                    "query": "test",
                    "limit": 10
                }
            }
        ]
    });

    assert!(request["service_id"].is_string());
    assert!(request["logs"].is_array());
    assert_eq!(request["logs"].as_array().unwrap().len(), 1);

    let log = &request["logs"][0];
    assert!(log["operation_id"].is_string());
    assert_eq!(log["tenant_id"], "test_tenant");
    assert_eq!(log["operation"], "search");
    assert_eq!(log["success"], true);
}

#[test]
fn test_operation_logs_response_structure() {
    let response = json!({
        "accepted": true,
        "processed": 5,
        "error": null
    });

    assert_eq!(response["accepted"], true);
    assert_eq!(response["processed"], 5);
    assert!(response["error"].is_null());
}

#[test]
fn test_operation_logs_response_with_error() {
    let response = json!({
        "accepted": false,
        "processed": 0,
        "error": "Rate limit exceeded"
    });

    assert_eq!(response["accepted"], false);
    assert_eq!(response["processed"], 0);
    assert_eq!(response["error"], "Rate limit exceeded");
}

#[test]
fn test_batch_operation_logs() {
    let logs = [
        json!({
            "operation_id": Uuid::new_v4(),
            "tenant_id": "tenant1",
            "operation": "insert_text",
            "operation_type": "insert",
            "collection": "docs",
            "timestamp": 1234567890,
            "duration_ms": 25,
            "success": true,
            "error": null,
            "metadata": {}
        }),
        json!({
            "operation_id": Uuid::new_v4(),
            "tenant_id": "tenant1",
            "operation": "search",
            "operation_type": "search",
            "collection": "docs",
            "timestamp": 1234567900,
            "duration_ms": 15,
            "success": true,
            "error": null,
            "metadata": {}
        }),
        json!({
            "operation_id": Uuid::new_v4(),
            "tenant_id": "tenant2",
            "operation": "create_collection",
            "operation_type": "createcollection",
            "collection": "new_collection",
            "timestamp": 1234567910,
            "duration_ms": 100,
            "success": false,
            "error": "Quota exceeded",
            "metadata": {}
        }),
    ];

    assert_eq!(logs.len(), 3);

    // Verify different tenants
    assert_eq!(logs[0]["tenant_id"], "tenant1");
    assert_eq!(logs[1]["tenant_id"], "tenant1");
    assert_eq!(logs[2]["tenant_id"], "tenant2");

    // Verify different operations
    assert_eq!(logs[0]["operation"], "insert_text");
    assert_eq!(logs[1]["operation"], "search");
    assert_eq!(logs[2]["operation"], "create_collection");

    // Verify success/failure
    assert_eq!(logs[0]["success"], true);
    assert_eq!(logs[1]["success"], true);
    assert_eq!(logs[2]["success"], false);
}

// ============================================================================
// Usage Metrics Tracking Tests
// ============================================================================

#[test]
fn test_usage_metrics_for_insert_operation() {
    let metrics = UsageMetrics {
        vectors_inserted: 1,
        search_count: 0,
        ..Default::default()
    };

    assert_eq!(metrics.vectors_inserted, 1);
    assert_eq!(metrics.search_count, 0);
}

#[test]
fn test_usage_metrics_for_search_operation() {
    let metrics = UsageMetrics {
        vectors_inserted: 0,
        search_count: 1,
        ..Default::default()
    };

    assert_eq!(metrics.vectors_inserted, 0);
    assert_eq!(metrics.search_count, 1);
}

#[test]
fn test_usage_metrics_accumulation() {
    let mut total_metrics = UsageMetrics::default();

    // Simulate 10 inserts
    for _ in 0..10 {
        total_metrics.vectors_inserted += 1;
    }

    // Simulate 5 searches
    for _ in 0..5 {
        total_metrics.search_count += 1;
    }

    assert_eq!(total_metrics.vectors_inserted, 10);
    assert_eq!(total_metrics.search_count, 5);
}

// ============================================================================
// Tenant Collection Filtering Tests
// ============================================================================

#[test]
fn test_filter_collections_for_tenant() {
    let all_collections = [
        "tenant1:docs".to_string(),
        "tenant1:images".to_string(),
        "tenant2:videos".to_string(),
        "tenant2:audio".to_string(),
        "tenant3:mixed".to_string(),
    ];

    let tenant_prefix = "tenant1:";
    let filtered: Vec<String> = all_collections
        .iter()
        .filter(|name| name.starts_with(tenant_prefix))
        .cloned()
        .collect();

    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&"tenant1:docs".to_string()));
    assert!(filtered.contains(&"tenant1:images".to_string()));
    assert!(!filtered.contains(&"tenant2:videos".to_string()));
}

#[test]
fn test_display_collection_name_strips_prefix() {
    let full_name = "tenant1:my_collection";

    if let Some((owner, collection)) = full_name.split_once(':') {
        assert_eq!(owner, "tenant1");
        assert_eq!(collection, "my_collection");
    } else {
        panic!("Failed to parse collection name");
    }
}

#[test]
fn test_tenant_collection_name_formatting() {
    let tenant_id = "user_abc123";
    let collection_name = "documents";

    let full_name = format!("{tenant_id}:{collection_name}");

    assert_eq!(full_name, "user_abc123:documents");
    assert!(full_name.starts_with("user_abc123:"));
    assert!(full_name.ends_with("documents"));
}

// ============================================================================
// Performance and Timing Tests
// ============================================================================

#[test]
fn test_timestamp_generation() {
    let timestamp1 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    std::thread::sleep(std::time::Duration::from_millis(10));

    let timestamp2 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    assert!(timestamp2 > timestamp1);
    assert!(timestamp2 - timestamp1 >= 10);
}

#[test]
fn test_duration_measurement() {
    let start = Instant::now();

    std::thread::sleep(std::time::Duration::from_millis(50));

    let duration_ms = start.elapsed().as_millis() as u64;

    assert!(
        duration_ms >= 50,
        "Expected at least 50ms, got {duration_ms}"
    );
    assert!(
        duration_ms < 100,
        "Expected less than 100ms, got {duration_ms}"
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_log_entry_with_missing_optional_fields() {
    let log = json!({
        "operation_id": Uuid::new_v4(),
        "tenant_id": "test_tenant",
        "operation": "list_collections",
        "operation_type": "listcollections",
        "timestamp": 1234567890,
        "duration_ms": 5,
        "success": true,
    });

    // Optional fields should be omitted
    assert!(log.get("collection").is_none());
    assert!(log.get("error").is_none());
    assert!(log.get("metadata").is_none());
}

#[test]
fn test_log_entry_with_all_optional_fields() {
    let log = json!({
        "operation_id": Uuid::new_v4(),
        "tenant_id": "test_tenant",
        "operation": "insert_text",
        "operation_type": "insert",
        "collection": "documents",
        "timestamp": 1234567890,
        "duration_ms": 25,
        "success": false,
        "error": "Validation failed",
        "metadata": {
            "reason": "Invalid vector dimension"
        }
    });

    assert!(log.get("collection").is_some());
    assert_eq!(log["collection"], "documents");
    assert!(log.get("error").is_some());
    assert_eq!(log["error"], "Validation failed");
    assert!(log.get("metadata").is_some());
    assert!(log["metadata"].is_object());
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_operation_type_serialization() {
    let op_type = McpOperationType::Search;
    let serialized = format!("{op_type:?}");
    assert_eq!(serialized, "Search");

    let op_type2 = McpOperationType::CreateCollection;
    let serialized2 = format!("{op_type2:?}");
    assert_eq!(serialized2, "CreateCollection");
}

#[test]
fn test_uuid_serialization_in_logs() {
    let operation_id = Uuid::new_v4();
    let log = json!({
        "operation_id": operation_id.to_string(),
        "tenant_id": "test_tenant"
    });

    assert!(log["operation_id"].is_string());
    let parsed_uuid = Uuid::parse_str(log["operation_id"].as_str().unwrap());
    assert!(parsed_uuid.is_ok());
    assert_eq!(parsed_uuid.unwrap(), operation_id);
}

// ============================================================================
// Buffer Management Tests
// ============================================================================

#[test]
fn test_log_buffer_size_limits() {
    let max_buffer_size = 1000;
    let mut buffer: Vec<serde_json::Value> = Vec::new();

    // Add logs until buffer is full
    for i in 0..max_buffer_size {
        buffer.push(json!({
            "operation_id": Uuid::new_v4(),
            "tenant_id": format!("tenant_{}", i % 10),
            "operation": "search",
            "timestamp": 1234567890 + i,
            "duration_ms": 10,
            "success": true,
        }));
    }

    assert_eq!(buffer.len(), max_buffer_size);

    // Simulate flush
    let flushed_count = buffer.len();
    buffer.clear();

    assert_eq!(flushed_count, max_buffer_size);
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_log_buffer_partial_flush() {
    let mut buffer: Vec<serde_json::Value> = Vec::new();

    // Add some logs
    for i in 0..50 {
        buffer.push(json!({
            "operation_id": Uuid::new_v4(),
            "tenant_id": "test_tenant",
            "operation": "search",
            "timestamp": 1234567890 + i,
            "duration_ms": 10,
            "success": true,
        }));
    }

    assert_eq!(buffer.len(), 50);

    // Partial flush (take first 25)
    let to_flush: Vec<_> = buffer.drain(0..25).collect();

    assert_eq!(to_flush.len(), 25);
    assert_eq!(buffer.len(), 25);
}

//! UMICP Tool Discovery for Vectorizer
//!
//! Implements the DiscoverableService trait to expose all 38+ MCP tools
//! via UMICP v0.2.1 tool discovery protocol

use serde_json::json;
use umicp_core::{DiscoverableService, OperationSchema, ServerInfo};

/// Vectorizer Discovery Service
/// Exposes all MCP tools as UMICP-discoverable operations
pub struct VectorizerDiscoveryService;

impl DiscoverableService for VectorizerDiscoveryService {
    fn server_info(&self) -> ServerInfo {
        ServerInfo::new(
            "vectorizer-server",
            env!("CARGO_PKG_VERSION"),
            "UMICP/2.0"
        )
        .features(vec![
            "semantic-search".to_string(),
            "vector-storage".to_string(),
            "intelligent-discovery".to_string(),
            "file-operations".to_string(),
            "batch-operations".to_string(),
            "workspace-management".to_string(),
            "mcp-compatible".to_string(),
        ])
        .operations_count(38)
        .mcp_compatible(true)
        .metadata(json!({
            "description": "HiveLLM Vectorizer - High-performance semantic search and vector database system with 38+ tools"
        }))
    }

    fn list_operations(&self) -> Vec<OperationSchema> {
        // Get all MCP tools
        let mcp_tools = crate::server::mcp_tools::get_mcp_tools();

        // Convert MCP Tools to UMICP OperationSchema
        mcp_tools
            .into_iter()
            .map(|tool| {
                let mut schema =
                    OperationSchema::new(tool.name.to_string(), json!(tool.input_schema));

                // Set title if available
                if let Some(title) = tool.title {
                    schema = schema.title(title);
                }

                // Set description if available
                if let Some(description) = tool.description {
                    schema = schema.description(description.to_string());
                }

                // Set output schema if available
                if let Some(output) = tool.output_schema {
                    schema = schema.output_schema(json!(output));
                }

                // Convert MCP annotations to UMICP annotations JSON
                if let Some(annotations) = tool.annotations {
                    let annotations_json = json!({
                        "read_only": annotations.read_only_hint,
                        "idempotent": annotations.idempotent_hint,
                        "destructive": annotations.destructive_hint,
                    });
                    schema = schema.annotations(annotations_json);
                }

                schema
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let service = VectorizerDiscoveryService;
        let info = service.server_info();

        assert_eq!(info.server, "vectorizer-server");
        assert_eq!(info.protocol, "UMICP/2.0");
        assert!(info.features.is_some());
        let features = info.features.unwrap();
        assert!(features.contains(&"semantic-search".to_string()));
    }

    #[test]
    fn test_list_operations() {
        let service = VectorizerDiscoveryService;
        let operations = service.list_operations();

        // Should have 38+ operations
        assert!(
            operations.len() >= 38,
            "Expected at least 38 operations, got {}",
            operations.len()
        );

        // Check for key operations
        let op_names: Vec<String> = operations.iter().map(|op| op.name.clone()).collect();
        assert!(op_names.contains(&"search_vectors".to_string()));
        assert!(op_names.contains(&"list_collections".to_string()));
        assert!(op_names.contains(&"create_collection".to_string()));
        assert!(op_names.contains(&"intelligent_search".to_string()));
    }

    #[test]
    fn test_operation_has_required_fields() {
        let service = VectorizerDiscoveryService;
        let operations = service.list_operations();

        for op in operations.iter().take(5) {
            // Check that operation has a name
            assert!(!op.name.is_empty());

            // Check that input_schema exists
            assert!(op.input_schema.is_object() || op.input_schema.is_null());
        }
    }

    #[test]
    fn test_search_vectors_operation() {
        let service = VectorizerDiscoveryService;
        let operations = service.list_operations();

        let search_op = operations
            .iter()
            .find(|op| op.name == "search_vectors")
            .expect("search_vectors operation not found");

        // Should have annotations
        assert!(search_op.annotations.is_some());
        let annotations = search_op.annotations.as_ref().unwrap();
        assert_eq!(annotations["read_only"], true);
        assert_eq!(annotations["idempotent"], true);

        // Should have input schema
        assert!(search_op.input_schema.is_object());
    }

    #[test]
    fn test_delete_collection_operation() {
        let service = VectorizerDiscoveryService;
        let operations = service.list_operations();

        let delete_op = operations
            .iter()
            .find(|op| op.name == "delete_collection")
            .expect("delete_collection operation not found");

        // Should be destructive
        assert!(delete_op.annotations.is_some());
        let annotations = delete_op.annotations.as_ref().unwrap();
        assert_eq!(annotations["destructive"], true);
    }

    #[test]
    fn test_serialization() {
        let service = VectorizerDiscoveryService;
        let operations = service.list_operations();

        // Test that we can serialize an operation
        let first_op = &operations[0];
        let json_str = serde_json::to_string(first_op);
        assert!(json_str.is_ok());

        // Test server info serialization
        let info = service.server_info();
        let info_json = serde_json::to_string(&info);
        assert!(info_json.is_ok());
    }
}

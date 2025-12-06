//! UMICP Tool Discovery for Vectorizer
//!
//! Implements the DiscoverableService trait to expose all 32 MCP tools
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
        .operations_count(26)
        .mcp_compatible(true)
        .metadata(json!({
            "description": "HiveLLM Vectorizer - High-performance semantic search and vector database system with 26 tools"
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

        // Should have 26 individual focused operations
        // Note: 6 cluster tools were removed from MCP for security (v1.8.5)
        assert_eq!(
            operations.len(),
            26,
            "Expected 26 individual operations, got {}",
            operations.len()
        );

        // Check for key individual operations
        let op_names: Vec<String> = operations.iter().map(|op| op.name.clone()).collect();

        // Core operations
        assert!(op_names.contains(&"list_collections".to_string()));
        assert!(op_names.contains(&"create_collection".to_string()));
        assert!(op_names.contains(&"get_collection_info".to_string()));
        assert!(op_names.contains(&"insert_text".to_string()));
        assert!(op_names.contains(&"get_vector".to_string()));
        assert!(op_names.contains(&"update_vector".to_string()));
        assert!(op_names.contains(&"delete_vector".to_string()));
        assert!(op_names.contains(&"search".to_string()));
        assert!(op_names.contains(&"multi_collection_search".to_string()));

        // Search operations
        assert!(op_names.contains(&"search_intelligent".to_string()));
        assert!(op_names.contains(&"search_semantic".to_string()));
        assert!(op_names.contains(&"search_extra".to_string()));

        // Discovery operations
        assert!(op_names.contains(&"filter_collections".to_string()));
        assert!(op_names.contains(&"expand_queries".to_string()));

        // File operations
        assert!(op_names.contains(&"get_file_content".to_string()));
        assert!(op_names.contains(&"list_files".to_string()));
        assert!(op_names.contains(&"get_file_chunks".to_string()));
        assert!(op_names.contains(&"get_project_outline".to_string()));
        assert!(op_names.contains(&"get_related_files".to_string()));
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
            .find(|op| op.name == "search")
            .expect("search operation not found");

        // Should have annotations
        assert!(search_op.annotations.is_some());
        let annotations = search_op.annotations.as_ref().unwrap();
        assert_eq!(annotations.get("read_only"), Some(&serde_json::json!(true)));
        assert_eq!(
            annotations.get("idempotent"),
            Some(&serde_json::json!(true))
        );

        // Should have input schema with query and collection fields (no search_type enum)
        assert!(search_op.input_schema.is_object());
        let schema = search_op.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));

        // Verify required fields
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("query"));
        assert!(properties.contains_key("collection"));
    }

    #[test]
    fn test_get_collection_info_operation() {
        let service = VectorizerDiscoveryService;
        let operations = service.list_operations();

        let collection_op = operations
            .iter()
            .find(|op| op.name == "get_collection_info")
            .expect("get_collection_info operation not found");

        // get_collection_info should be read-only and idempotent
        assert!(collection_op.annotations.is_some());
        let annotations = collection_op.annotations.as_ref().unwrap();
        assert_eq!(annotations.get("read_only"), Some(&serde_json::json!(true)));
        assert_eq!(
            annotations.get("idempotent"),
            Some(&serde_json::json!(true))
        );

        // Should have input schema with name field
        assert!(collection_op.input_schema.is_object());
        let schema = collection_op.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));
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

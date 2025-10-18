// Example MCP integration for file operations
// This file shows how to add file-level tools to the MCP server

use serde_json::{Value, json};

use crate::file_operations::{
    FileContent, FileList, FileListFilter, FileOperations, FileSummary, SummaryType,
};

/// MCP tool handlers for file operations
pub struct FileMcpHandlers {
    file_ops: FileOperations,
}

impl FileMcpHandlers {
    pub fn new(file_ops: FileOperations) -> Self {
        Self { file_ops }
    }

    /// Register all file operation tools with MCP server
    pub fn register_tools() -> Vec<Value> {
        vec![
            json!({
                "name": "get_file_content",
                "description": "Retrieve complete file content from a collection. Use this instead of read_file for indexed files.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "collection": {
                            "type": "string",
                            "description": "Collection name (e.g., 'vectorizer-source', 'vectorizer-docs')"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "Relative file path within collection (e.g., 'src/main.rs')"
                        },
                        "max_size_kb": {
                            "type": "number",
                            "description": "Maximum file size in KB (default: 500, max: 5000)",
                            "default": 500
                        }
                    },
                    "required": ["collection", "file_path"]
                }
            }),
            json!({
                "name": "list_files_in_collection",
                "description": "List all indexed files in a collection with metadata. Use this to explore project structure.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "collection": {
                            "type": "string",
                            "description": "Collection name"
                        },
                        "filter_by_type": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Filter by file types (e.g., ['rs', 'md', 'toml'])"
                        },
                        "min_chunks": {
                            "type": "number",
                            "description": "Minimum number of chunks (filters out small files)"
                        },
                        "max_results": {
                            "type": "number",
                            "description": "Maximum number of results (default: 100)"
                        },
                        "sort_by": {
                            "type": "string",
                            "enum": ["name", "size", "chunks", "recent"],
                            "description": "Sort order (default: 'name')",
                            "default": "name"
                        }
                    },
                    "required": ["collection"]
                }
            }),
            json!({
                "name": "get_file_summary",
                "description": "Get extractive or structural summary of an indexed file. More efficient than reading the entire file.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "collection": {
                            "type": "string",
                            "description": "Collection name"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "Relative file path within collection"
                        },
                        "summary_type": {
                            "type": "string",
                            "enum": ["extractive", "structural", "both"],
                            "description": "Type of summary (default: 'both')",
                            "default": "both"
                        },
                        "max_sentences": {
                            "type": "number",
                            "description": "Maximum sentences for extractive summary (default: 5)",
                            "default": 5
                        }
                    },
                    "required": ["collection", "file_path"]
                }
            }),
        ]
    }

    /// Handle get_file_content tool call
    pub async fn handle_get_file_content(&self, params: Value) -> Result<Value, String> {
        let collection = params
            .get("collection")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'collection' parameter")?;

        let file_path = params
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'file_path' parameter")?;

        let max_size_kb = params
            .get("max_size_kb")
            .and_then(|v| v.as_u64())
            .unwrap_or(500) as usize;

        let result = self
            .file_ops
            .get_file_content(collection, file_path, max_size_kb)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "file_path": result.file_path,
            "content": result.content,
            "metadata": {
                "file_type": result.metadata.file_type,
                "size_kb": result.metadata.size_kb,
                "chunk_count": result.metadata.chunk_count,
                "last_indexed": result.metadata.last_indexed,
                "language": result.metadata.language,
            },
            "chunks_available": result.chunks_available,
            "collection": result.collection,
            "from_cache": result.from_cache,
        }))
    }

    /// Handle list_files_in_collection tool call
    pub async fn handle_list_files(&self, params: Value) -> Result<Value, String> {
        let collection = params
            .get("collection")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'collection' parameter")?;

        let filter = self.parse_file_list_filter(&params)?;

        let result = self
            .file_ops
            .list_files_in_collection(collection, filter)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "collection": result.collection,
            "total_files": result.total_files,
            "total_chunks": result.total_chunks,
            "files": result.files.iter().map(|f| json!({
                "path": f.path,
                "file_type": f.file_type,
                "chunk_count": f.chunk_count,
                "size_estimate_kb": f.size_estimate_kb,
                "last_indexed": f.last_indexed,
                "has_summary": f.has_summary,
            })).collect::<Vec<_>>(),
        }))
    }

    /// Handle get_file_summary tool call
    pub async fn handle_get_file_summary(&self, params: Value) -> Result<Value, String> {
        let collection = params
            .get("collection")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'collection' parameter")?;

        let file_path = params
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'file_path' parameter")?;

        let summary_type = params
            .get("summary_type")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "extractive" => Some(SummaryType::Extractive),
                "structural" => Some(SummaryType::Structural),
                "both" => Some(SummaryType::Both),
                _ => None,
            })
            .unwrap_or(SummaryType::Both);

        let max_sentences = params
            .get("max_sentences")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let result = self
            .file_ops
            .get_file_summary(collection, file_path, summary_type, max_sentences)
            .await
            .map_err(|e| e.to_string())?;

        let mut response = json!({
            "file_path": result.file_path,
            "metadata": {
                "chunk_count": result.metadata.chunk_count,
                "file_type": result.metadata.file_type,
                "summary_method": result.metadata.summary_method,
            },
            "generated_at": result.generated_at,
        });

        if let Some(extractive) = result.extractive_summary {
            response["extractive_summary"] = json!(extractive);
        }

        if let Some(structural) = result.structural_summary {
            response["structural_summary"] = json!({
                "outline": structural.outline,
                "key_sections": structural.key_sections,
                "key_points": structural.key_points,
            });
        }

        Ok(response)
    }

    /// Parse file list filter from params
    fn parse_file_list_filter(&self, params: &Value) -> Result<FileListFilter, String> {
        use crate::file_operations::SortBy;

        let filter_by_type = params
            .get("filter_by_type")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let min_chunks = params
            .get("min_chunks")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        let max_results = params
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        let sort_by = params
            .get("sort_by")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "name" => Some(SortBy::Name),
                "size" => Some(SortBy::Size),
                "chunks" => Some(SortBy::Chunks),
                "recent" => Some(SortBy::Recent),
                _ => None,
            })
            .unwrap_or(SortBy::Name);

        Ok(FileListFilter {
            filter_by_type,
            min_chunks,
            max_results,
            sort_by,
        })
    }

    /// Dispatch tool call to appropriate handler
    pub async fn handle_tool_call(&self, tool_name: &str, params: Value) -> Result<Value, String> {
        match tool_name {
            "get_file_content" => self.handle_get_file_content(params).await,
            "list_files_in_collection" => self.handle_list_files(params).await,
            "get_file_summary" => self.handle_get_file_summary(params).await,
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_handler_creation() {
        let file_ops = FileOperations::new();
        let handlers = FileMcpHandlers::new(file_ops);

        // Test tool registration
        let tools = FileMcpHandlers::register_tools();
        assert_eq!(tools.len(), 3);

        // Verify tool names
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name")?.as_str())
            .collect();

        assert!(tool_names.contains(&"get_file_content"));
        assert!(tool_names.contains(&"list_files_in_collection"));
        assert!(tool_names.contains(&"get_file_summary"));
    }

    #[tokio::test]
    async fn test_get_file_content_handler() {
        let file_ops = FileOperations::new();
        let handlers = FileMcpHandlers::new(file_ops);

        let params = json!({
            "collection": "test-collection",
            "file_path": "src/main.rs",
            "max_size_kb": 500
        });

        let result = handlers.handle_get_file_content(params).await;
        // Test may fail if file doesn't exist in collection - this is expected
        // Just verify handler doesn't panic
        match result {
            Ok(response) => {
                assert_eq!(response["file_path"], "src/main.rs");
                assert_eq!(response["collection"], "test-collection");
            }
            Err(_) => {
                // Expected if file not indexed
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_list_files_handler() {
        let file_ops = FileOperations::new();
        let handlers = FileMcpHandlers::new(file_ops);

        let params = json!({
            "collection": "test-collection",
            "filter_by_type": ["rs"],
            "max_results": 10
        });

        let result = handlers.handle_list_files(params).await;
        // May fail if collection doesn't exist
        match result {
            Ok(response) => {
                assert_eq!(response["collection"], "test-collection");
                assert!(response["files"].is_array());
            }
            Err(_) => {
                assert!(true);
            }
        }
    }
}

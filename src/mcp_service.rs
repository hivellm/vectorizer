use rmcp::model::{
    CallToolRequestParam, CallToolResult, ErrorData, Implementation, ListToolsResult,
    PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::json;
use std::borrow::Cow;
use std::future::Future;
use std::sync::Arc;
use crate::grpc::client::VectorizerGrpcClient;
use crate::config::GrpcConfig;

#[derive(Clone)]
pub struct VectorizerService {
    grpc_server_url: String,
}

impl VectorizerService {
    pub fn new(_grpc_server_url: String) -> Self {
        Self {
            grpc_server_url: String::new(), // Not used anymore, config handles this
        }
    }

    async fn get_grpc_client(&self) -> Result<VectorizerGrpcClient, ErrorData> {
        let config = GrpcConfig::from_env();
        VectorizerGrpcClient::new(config.client)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to create GRPC client: {}", e), None))
    }
}

impl ServerHandler for VectorizerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "vectorizer-mcp-server".to_string(),
                title: Some("HiveLLM Vectorizer MCP Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: Some("https://github.com/hivellm/hivellm".to_string()),
                icons: None,
            },
            instructions: Some("This is the HiveLLM Vectorizer MCP Server - a high-performance semantic search and vector database system. It provides comprehensive capabilities for:\n\n🔍 SEMANTIC SEARCH: Search for content by meaning using natural language queries, finding relevant information even when exact keywords don't match.\n\n📚 COLLECTION MANAGEMENT: Create, manage, and organize multiple isolated vector collections, each optimized for different domains or use cases.\n\n💾 VECTOR OPERATIONS: Insert, update, retrieve, and delete vectors with automatic embedding generation using BM25 or custom embedding models.\n\n⚡ BATCH PROCESSING: Efficiently process large volumes of data with batch operations for inserting, searching, updating, and deleting multiple items at once.\n\n📊 SUMMARIZATION: Generate intelligent summaries of text content and project context using multiple methods (extractive, keyword, sentence, abstractive), perfect for condensing documentation or helping AI models understand codebases.\n\n📈 MONITORING: Track indexing progress, check service health, and monitor system performance.\n\nAll operations use GRPC for high performance and reliability. The default embedding model is BM25, which provides fast, accurate semantic matching without requiring external API calls.".to_string()),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            let tools = vec![
                // Core vector operations
                Tool {
                    name: Cow::Borrowed("search_vectors"),
                    title: Some("Search Vectors".to_string()),
                    description: Some(Cow::Borrowed("Search for semantically similar content in a vector collection using text queries. This tool performs semantic search by converting your query text into an embedding and finding the most similar vectors in the specified collection. Returns ranked results with similarity scores, content, and metadata. Useful for finding relevant code snippets, documentation, or any indexed content based on meaning rather than exact text matches.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query text"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results",
                                "default": 10
                            }
                        },
                        "required": ["collection", "query"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "results": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string"},
                                        "content": {"type": "string"},
                                        "score": {"type": "number"},
                                        "metadata": {"type": "object"}
                                    }
                                }
                            },
                            "total_found": {"type": "integer"},
                            "search_time_ms": {"type": "number"}
                        }
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("list_collections"),
                    title: Some("List Collections".to_string()),
                    description: Some(Cow::Borrowed("List all available vector collections in the database. Returns comprehensive information about each collection including name, vector count, dimension size, similarity metric used, current status, and last update timestamp. Use this to discover what collections are available for searching or to check the status of indexed collections. Essential for understanding what data is available in the vectorizer system.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "collections": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "name": {"type": "string"},
                                        "vector_count": {"type": "integer"},
                                        "dimension": {"type": "integer"},
                                        "similarity_metric": {"type": "string"},
                                        "status": {"type": "string"},
                                        "last_updated": {"type": "string"},
                                        "error_message": {"type": "string"}
                                    }
                                }
                            },
                            "total_collections": {"type": "integer"}
                        }
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("get_collection_info"),
                    title: Some("Get Collection Info".to_string()),
                    description: Some(Cow::Borrowed("Get detailed information about a specific vector collection. Returns comprehensive metadata including collection name, total vector count, document count, vector dimension, similarity metric (cosine, euclidean, etc.), current indexing status, last update timestamp, and any error messages. Use this to verify collection properties before searching or inserting data, or to monitor the health and status of a specific collection.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            }
                        },
                        "required": ["collection"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "vector_count": {"type": "integer"},
                            "document_count": {"type": "integer"},
                            "dimension": {"type": "integer"},
                            "similarity_metric": {"type": "string"},
                            "status": {"type": "string"},
                            "last_updated": {"type": "string"},
                            "error_message": {"type": "string"}
                        },
                        "required": ["name", "vector_count", "dimension", "similarity_metric", "status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("embed_text"),
                    title: Some("Embed Text".to_string()),
                    description: Some(Cow::Borrowed("Generate vector embeddings for input text using the configured embedding model (default: BM25). Converts text into a numerical vector representation that captures semantic meaning. Returns the embedding vector, dimension size, and provider used. This is useful for understanding how text is represented in vector space, testing embedding quality, or manually creating embeddings for custom operations. The embeddings can be used for similarity comparisons or direct vector operations.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to embed"
                            }
                        },
                        "required": ["text"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "embedding": {
                                "type": "array",
                                "items": {"type": "number"},
                                "description": "Vector embedding"
                            },
                            "dimension": {"type": "integer"},
                            "provider": {"type": "string"}
                        },
                        "required": ["embedding", "dimension", "provider"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                // Collection management
                Tool {
                    name: Cow::Borrowed("create_collection"),
                    title: Some("Create Collection".to_string()),
                    description: Some(Cow::Borrowed("Create a new vector collection with specified configuration. Requires a unique collection name and allows customization of vector dimension (default: 384) and distance metric (cosine, euclidean, or dot_product). The collection will be initialized and ready to receive vectors. Use this when you need to create a new semantic search index for a specific domain, project, or type of content. Each collection is isolated and can have different configurations optimized for different use cases.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "dimension": {
                                "type": "integer",
                                "description": "Vector dimension",
                                "default": 384
                            },
                            "metric": {
                                "type": "string",
                                "description": "Distance metric",
                                "default": "cosine",
                                "enum": ["cosine", "euclidean", "dot_product"]
                            }
                        },
                        "required": ["name"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "dimension": {"type": "integer"},
                            "similarity_metric": {"type": "string"},
                            "status": {"type": "string"},
                            "message": {"type": "string"}
                        }
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(false)
                        .idempotent(false)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("delete_collection"),
                    title: Some("Delete Collection".to_string()),
                    description: Some(Cow::Borrowed("Permanently delete a vector collection and all its contents. This operation removes all vectors, metadata, and index structures associated with the collection. Use with caution as this action is irreversible and will permanently delete all data in the collection. Returns status confirmation and message. Useful for cleanup, removing outdated collections, or freeing up storage space when a collection is no longer needed.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Collection name"
                            }
                        },
                        "required": ["name"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "collection_name": {"type": "string"},
                            "status": {"type": "string"},
                            "message": {"type": "string"}
                        }
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                // Vector operations
                Tool {
                    name: Cow::Borrowed("insert_texts"),
                    title: Some("Insert Texts".to_string()),
                    description: Some(Cow::Borrowed("Insert text content into a collection with automatic embedding generation. Accepts an array of text items, each with a unique ID, text content, and optional metadata. The system automatically generates vector embeddings for each text using the configured embedding model (BM25 by default). This is the primary method for indexing new content for semantic search. Supports custom metadata for filtering and context. Returns insertion status and count of successfully inserted items.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vectors": {
                                "type": "array",
                                "description": "Array of vectors to insert",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Vector ID"
                                        },
                                        "data": {
                                            "type": "array",
                                            "description": "Vector data",
                                            "items": {"type": "number"}
                                        },
                                        "metadata": {
                                            "type": "object",
                                            "description": "Optional metadata"
                                        }
                                    },
                                    "required": ["id", "data"]
                                }
                            }
                        },
                        "required": ["collection", "vectors"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "collection": {"type": "string"},
                            "inserted_count": {"type": "integer"},
                            "status": {"type": "string"},
                            "message": {"type": "string"}
                        },
                        "required": ["collection", "inserted_count", "status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(false)
                        .idempotent(false)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("delete_vectors"),
                    title: Some("Delete Vectors".to_string()),
                    description: Some(Cow::Borrowed("Delete specific vectors from a collection by their IDs. Accepts an array of vector IDs to remove. This operation permanently removes the specified vectors and their associated metadata from the collection without affecting other vectors. Useful for removing outdated content, cleaning up test data, or maintaining data freshness by removing obsolete entries. Returns deletion status and count of successfully deleted vectors.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vector_ids": {
                                "type": "array",
                                "description": "Array of vector IDs to delete",
                                "items": {"type": "string"},
                                "minItems": 1
                            }
                        },
                        "required": ["collection", "vector_ids"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "collection": {"type": "string"},
                            "deleted_count": {"type": "integer"},
                            "status": {"type": "string"},
                            "message": {"type": "string"}
                        },
                        "required": ["collection", "deleted_count", "status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("get_vector"),
                    title: Some("Get Vector".to_string()),
                    description: Some(Cow::Borrowed("Retrieve a specific vector and its metadata from a collection using its unique ID. Returns the complete vector data including ID, embedding values, metadata, collection name, and status. Useful for verifying that specific content was indexed correctly, retrieving stored metadata, or debugging indexing issues. This allows direct access to individual vectors without performing a search query.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vector_id": {
                                "type": "string",
                                "description": "Vector ID"
                            }
                        },
                        "required": ["collection", "vector_id"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "data": {
                                "type": "array",
                                "items": {"type": "number"}
                            },
                            "metadata": {"type": "object"},
                            "collection": {"type": "string"},
                            "status": {"type": "string"}
                        },
                        "required": ["id", "data", "collection", "status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                // Monitoring
                Tool {
                    name: Cow::Borrowed("get_indexing_progress"),
                    title: Some("Get Indexing Progress".to_string()),
                    description: Some(Cow::Borrowed("Monitor the progress of ongoing indexing operations across all collections. Returns detailed status for each collection including indexing state, progress percentage, current vector count, error messages if any, and last update timestamp. Also provides overall system status indicating if any indexing is currently in progress. Essential for monitoring large indexing jobs, troubleshooting indexing issues, and understanding system load.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "collections": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "collection_name": {"type": "string"},
                                        "status": {"type": "string"},
                                        "progress": {"type": "number", "minimum": 0, "maximum": 1},
                                        "vector_count": {"type": "integer"},
                                        "error_message": {"type": "string"},
                                        "last_updated": {"type": "string"}
                                    }
                                }
                            },
                            "is_indexing": {"type": "boolean"},
                            "overall_status": {"type": "string"}
                        },
                        "required": ["collections", "is_indexing", "overall_status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("health_check"),
                    title: Some("Health Check".to_string()),
                    description: Some(Cow::Borrowed("Check the health and status of the vectorizer service. Returns current service status, service name, version information, current timestamp, and any error messages if the service is experiencing issues. Use this to verify the service is running correctly, check version compatibility, diagnose connectivity problems, or monitor service availability in production environments. Essential for system monitoring and troubleshooting.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "status": {"type": "string", "enum": ["healthy", "degraded", "unhealthy"]},
                            "service": {"type": "string"},
                            "version": {"type": "string"},
                            "timestamp": {"type": "string"},
                            "error_message": {"type": "string"}
                        },
                        "required": ["status", "service", "version", "timestamp"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                // Batch operations
                Tool {
                    name: Cow::Borrowed("batch_insert_texts"),
                    title: Some("Batch Insert Texts".to_string()),
                    description: Some(Cow::Borrowed("Efficiently insert multiple text documents into a collection in a single operation with automatic embedding generation. Optimized for bulk indexing operations, this tool processes large batches of texts more efficiently than individual inserts. Each text entry requires a unique ID, text content, and can include optional metadata. Specify the embedding provider (default: BM25) for consistent embedding generation. Returns insertion statistics including total inserted count and status. Ideal for initial data loading, bulk imports, or batch updates of large document sets.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "texts": {
                                "type": "array",
                                "description": "Array of texts to insert",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Text ID"
                                        },
                                        "text": {
                                            "type": "string",
                                            "description": "Text content"
                                        },
                                        "metadata": {
                                            "type": "object",
                                            "description": "Optional metadata"
                                        }
                                    },
                                    "required": ["id", "text"]
                                }
                            },
                            "provider": {
                                "type": "string",
                                "description": "Embedding provider",
                                "default": "bm25",
                                "enum": ["bm25", "native_bow", "native_hash", "native_ngram"]
                            }
                        },
                        "required": ["collection", "texts"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "success": {"type": "boolean"},
                            "collection": {"type": "string"},
                            "inserted_count": {"type": "integer"},
                            "status": {"type": "string"},
                            "message": {"type": "string"},
                            "operation": {"type": "string"}
                        }
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(false)
                        .idempotent(false)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("batch_search_vectors"),
                    title: Some("Batch Search Vectors".to_string()),
                    description: Some(Cow::Borrowed("Execute multiple semantic search queries in a single batch operation for improved efficiency. Accepts an array of query objects, each with query text, optional result limit, and optional score threshold for filtering results. Returns comprehensive results for each query including matched vectors, similarity scores, metadata, and search timing. Specify the embedding provider (default: BM25) for consistent query processing. Ideal for multi-faceted searches, comparative analysis, or when you need to find results for multiple related queries simultaneously. More efficient than multiple individual search calls.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "queries": {
                                "type": "array",
                                "description": "Array of search queries",
                                "minItems": 1,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "query": {
                                            "type": "string",
                                            "description": "Search query text"
                                        },
                                        "limit": {
                                            "type": "integer",
                                            "description": "Maximum number of results",
                                            "default": 10,
                                            "minimum": 1,
                                            "maximum": 100
                                        },
                                        "score_threshold": {
                                            "type": "number",
                                            "description": "Minimum score threshold",
                                            "minimum": 0,
                                            "maximum": 1
                                        }
                                    },
                                    "required": ["query"]
                                }
                            },
                            "provider": {
                                "type": "string",
                                "description": "Embedding provider",
                                "default": "bm25",
                                "enum": ["bm25", "native_bow", "native_hash", "native_ngram"]
                            }
                        },
                        "required": ["collection", "queries"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "success": {"type": "boolean"},
                            "collection": {"type": "string"},
                            "total_queries": {"type": "integer"},
                            "batch_results": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "query": {"type": "string"},
                                        "query_index": {"type": "integer"},
                                        "results": {
                                            "type": "array",
                                            "items": {
                                                "type": "object",
                                                "properties": {
                                                    "id": {"type": "string"},
                                                    "content": {"type": "string"},
                                                    "score": {"type": "number"},
                                                    "metadata": {"type": "object"}
                                                }
                                            }
                                        },
                                        "total_found": {"type": "integer"},
                                        "search_time_ms": {"type": "number"},
                                        "error": {"type": "string"}
                                    }
                                }
                            },
                            "operation": {"type": "string"}
                        },
                        "required": ["success", "collection", "total_queries", "batch_results"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("batch_update_vectors"),
                    title: Some("Batch Update Vectors".to_string()),
                    description: Some(Cow::Borrowed("Update multiple existing vectors in a collection efficiently in a single batch operation. For each update, provide the vector ID and optionally new text content (which triggers re-embedding) and/or updated metadata. When new text is provided, the system automatically regenerates embeddings using the specified provider (default: BM25). Returns detailed results for each update attempt including success/failure status and messages. Use this to keep indexed content up-to-date, correct errors in bulk, or refresh embeddings after content modifications. More efficient than individual update operations.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "updates": {
                                "type": "array",
                                "description": "Array of vector updates",
                                "minItems": 1,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Vector ID"
                                        },
                                        "text": {
                                            "type": "string",
                                            "description": "New text content (optional)"
                                        },
                                        "metadata": {
                                            "type": "object",
                                            "description": "New metadata (optional)"
                                        }
                                    },
                                    "required": ["id"]
                                }
                            },
                            "provider": {
                                "type": "string",
                                "description": "Embedding provider",
                                "default": "bm25",
                                "enum": ["bm25", "native_bow", "native_hash", "native_ngram"]
                            }
                        },
                        "required": ["collection", "updates"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "success": {"type": "boolean"},
                            "collection": {"type": "string"},
                            "total_updates": {"type": "integer"},
                            "successful_updates": {"type": "integer"},
                            "failed_updates": {"type": "integer"},
                            "batch_results": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string"},
                                        "update_index": {"type": "integer"},
                                        "status": {"type": "string", "enum": ["success", "error", "skipped"]},
                                        "message": {"type": "string"},
                                        "error": {"type": "string"}
                                    }
                                }
                            },
                            "operation": {"type": "string"}
                        },
                        "required": ["success", "collection", "total_updates", "batch_results"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(false)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("batch_delete_vectors"),
                    title: Some("Batch Delete Vectors".to_string()),
                    description: Some(Cow::Borrowed("Delete multiple vectors from a collection efficiently in a single batch operation. Accepts an array of vector IDs to remove permanently. This operation is more efficient than individual delete calls when removing multiple vectors. Useful for bulk cleanup operations, removing batches of outdated content, clearing test data, or maintaining collection hygiene by removing obsolete entries in bulk. Returns deletion statistics including total deleted count and operation status. All specified vectors are removed atomically.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vector_ids": {
                                "type": "array",
                                "description": "Array of vector IDs to delete",
                                "items": {"type": "string"},
                                "minItems": 1
                            }
                        },
                        "required": ["collection", "vector_ids"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "success": {"type": "boolean"},
                            "collection": {"type": "string"},
                            "deleted_count": {"type": "integer"},
                            "status": {"type": "string"},
                            "message": {"type": "string"},
                            "operation": {"type": "string"}
                        },
                        "required": ["success", "collection", "deleted_count", "status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(false)
                        .destructive(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                // Summarization tools
                Tool {
                    name: Cow::Borrowed("summarize_text"),
                    title: Some("Summarize Text".to_string()),
                    description: Some(Cow::Borrowed("Generate intelligent summaries of text content using multiple summarization methods. Supports four methods: 'extractive' (extracts key sentences), 'keyword' (identifies main keywords), 'sentence' (selects representative sentences), and 'abstractive' (generates new summary text). Configure summary length with max_length or compression_ratio (0.1-0.9). Optionally specify language for better processing. Returns the generated summary, original text, length statistics, compression ratio achieved, and a unique summary ID for later retrieval. Ideal for condensing documentation, creating TL;DR versions, extracting key points from long content, or generating concise overviews. Stored summaries can be retrieved later using the summary ID.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to summarize",
                                "minLength": 1
                            },
                            "method": {
                                "type": "string",
                                "description": "Summarization method",
                                "enum": ["extractive", "keyword", "sentence", "abstractive"],
                                "default": "extractive"
                            },
                            "max_length": {
                                "type": "integer",
                                "description": "Maximum summary length (optional)",
                                "minimum": 10
                            },
                            "compression_ratio": {
                                "type": "number",
                                "description": "Compression ratio (0.1-0.9, optional)",
                                "minimum": 0.1,
                                "maximum": 0.9
                            },
                            "language": {
                                "type": "string",
                                "description": "Language code (optional)",
                                "examples": ["en", "pt", "es", "fr"]
                            }
                        },
                        "required": ["text"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "summary_id": {"type": "string"},
                            "original_text": {"type": "string"},
                            "summary": {"type": "string"},
                            "method": {"type": "string"},
                            "original_length": {"type": "integer"},
                            "summary_length": {"type": "integer"},
                            "compression_ratio": {"type": "number"},
                            "language": {"type": "string"},
                            "status": {"type": "string"},
                            "message": {"type": "string"},
                            "metadata": {"type": "object"}
                        },
                        "required": ["summary_id", "summary", "method", "original_length", "summary_length"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("summarize_context"),
                    title: Some("Summarize Context".to_string()),
                    description: Some(Cow::Borrowed("Generate concise summaries of project context specifically optimized for AI model consumption and understanding. This specialized summarization tool helps AI assistants quickly grasp project structure, codebase organization, documentation, and key concepts without processing entire files. Supports multiple methods (extractive, keyword, sentence, abstractive) and allows customization via max_length or compression_ratio. Returns a focused summary that captures essential project information, patterns, and architecture. Particularly useful for helping AI models understand large codebases, providing context for code generation, or creating high-level project overviews. The generated summaries are optimized for AI comprehension and can be stored for reuse.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "context": {
                                "type": "string",
                                "description": "Context to summarize",
                                "minLength": 1
                            },
                            "method": {
                                "type": "string",
                                "description": "Summarization method",
                                "enum": ["extractive", "keyword", "sentence", "abstractive"],
                                "default": "extractive"
                            },
                            "max_length": {
                                "type": "integer",
                                "description": "Maximum summary length (optional)",
                                "minimum": 10
                            },
                            "compression_ratio": {
                                "type": "number",
                                "description": "Compression ratio (0.1-0.9, optional)",
                                "minimum": 0.1,
                                "maximum": 0.9
                            },
                            "language": {
                                "type": "string",
                                "description": "Language code (optional)",
                                "examples": ["en", "pt", "es", "fr"]
                            }
                        },
                        "required": ["context"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "summary_id": {"type": "string"},
                            "original_context": {"type": "string"},
                            "summary": {"type": "string"},
                            "method": {"type": "string"},
                            "original_length": {"type": "integer"},
                            "summary_length": {"type": "integer"},
                            "compression_ratio": {"type": "number"},
                            "language": {"type": "string"},
                            "status": {"type": "string"},
                            "message": {"type": "string"},
                            "metadata": {"type": "object"}
                        },
                        "required": ["summary_id", "summary", "method", "original_length", "summary_length"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("get_summary"),
                    title: Some("Get Summary".to_string()),
                    description: Some(Cow::Borrowed("Retrieve a previously generated summary by its unique ID. Returns complete summary information including the original text, generated summary, summarization method used, length statistics, compression ratio, language, creation timestamp, and any associated metadata. Use this to retrieve summaries created earlier, share summaries across sessions, or access historical summarization results. Enables reuse of computational effort by retrieving cached summaries instead of re-generating them. Particularly useful for frequently accessed summaries or when working with the same content across multiple sessions.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "summary_id": {
                                "type": "string",
                                "description": "Summary ID"
                            }
                        },
                        "required": ["summary_id"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "summary_id": {"type": "string"},
                            "original_text": {"type": "string"},
                            "summary": {"type": "string"},
                            "method": {"type": "string"},
                            "original_length": {"type": "integer"},
                            "summary_length": {"type": "integer"},
                            "compression_ratio": {"type": "number"},
                            "language": {"type": "string"},
                            "created_at": {"type": "string"},
                            "metadata": {"type": "object"},
                            "status": {"type": "string"}
                        },
                        "required": ["summary_id", "original_text", "summary", "method"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
                Tool {
                    name: Cow::Borrowed("list_summaries"),
                    title: Some("List Summaries".to_string()),
                    description: Some(Cow::Borrowed("List all available summaries with powerful filtering and pagination capabilities. Filter summaries by summarization method (extractive, keyword, sentence, abstractive) or language. Use limit and offset parameters for pagination when dealing with large numbers of summaries. Returns summary metadata including ID, method, language, length statistics, compression ratios, and creation timestamps for each summary. Useful for discovering available summaries, auditing summarization history, finding summaries by specific criteria, or managing summary storage. The total count helps with pagination planning.")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "method": {
                                "type": "string",
                                "description": "Filter by summarization method (optional)",
                                "enum": ["extractive", "keyword", "sentence", "abstractive"]
                            },
                            "language": {
                                "type": "string",
                                "description": "Filter by language (optional)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of summaries to return (optional)",
                                "minimum": 1,
                                "maximum": 1000,
                                "default": 100
                            },
                            "offset": {
                                "type": "integer",
                                "description": "Offset for pagination (optional)",
                                "minimum": 0,
                                "default": 0
                            }
                        }
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "summaries": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "summary_id": {"type": "string"},
                                        "method": {"type": "string"},
                                        "language": {"type": "string"},
                                        "original_length": {"type": "integer"},
                                        "summary_length": {"type": "integer"},
                                        "compression_ratio": {"type": "number"},
                                        "created_at": {"type": "string"},
                                        "metadata": {"type": "object"}
                                    }
                                }
                            },
                            "total_count": {"type": "integer"},
                            "status": {"type": "string"}
                        },
                        "required": ["summaries", "total_count", "status"]
                    }).as_object().unwrap().clone().into()),
                    icons: None,
                    annotations: Some(ToolAnnotations::new()
                        .read_only(true)
                        .idempotent(true)
                        .open_world(false)),
                },
            ];

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            match request.name.as_ref() {
                "search_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection =
                        args.get("collection")
                            .and_then(|c| c.as_str())
                            .ok_or_else(|| {
                                ErrorData::invalid_params("Missing collection parameter", None)
                            })?;

                    let query = args.get("query").and_then(|q| q.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing query parameter", None)
                    })?;

                    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as i32;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let search_response = grpc_client
                        .search(collection.to_string(), query.to_string(), limit)
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC search failed: {}", e), None))?;

                    let results: Vec<serde_json::Value> = search_response.results
                        .into_iter()
                        .map(|result| json!({
                            "id": result.id,
                            "content": result.content,
                            "score": result.score,
                            "metadata": result.metadata
                        }))
                        .collect();

                    let result_text = json!({
                        "results": results,
                        "total_found": search_response.total_found,
                        "search_time_ms": search_response.search_time_ms
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "list_collections" => {
                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let collections_response = grpc_client
                        .list_collections()
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC list collections failed: {}", e), None))?;

                    let collections: Vec<serde_json::Value> = collections_response.collections
                        .into_iter()
                        .map(|collection| json!({
                            "name": collection.name,
                            "vector_count": collection.vector_count,
                            "dimension": collection.dimension,
                            "similarity_metric": collection.similarity_metric,
                            "status": collection.status,
                            "last_updated": collection.last_updated,
                            "error_message": collection.error_message
                        }))
                        .collect();

                    let result_text = json!({
                        "collections": collections,
                        "total_collections": collections_response.total_collections
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "embed_text" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let text = args
                        .get("text")
                        .and_then(|t| t.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing text parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let embed_response = grpc_client
                        .embed_text(text.to_string(), "bm25".to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC embed failed: {}", e), None))?;

                    let result_text = json!({
                        "embedding": embed_response.embedding,
                        "dimension": embed_response.dimension,
                        "provider": embed_response.provider
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_collection_info" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let collection_info = grpc_client
                        .get_collection_info(collection.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_collection_info failed: {}", e), None))?;

                    let result_text = json!({
                        "name": collection_info.name,
                        "vector_count": collection_info.vector_count,
                        "document_count": collection_info.document_count,
                        "dimension": collection_info.dimension,
                        "similarity_metric": collection_info.similarity_metric,
                        "status": collection_info.status,
                        "last_updated": collection_info.last_updated,
                        "error_message": collection_info.error_message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "create_collection" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let name = args
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing name parameter", None))?;

                    let dimension = args.get("dimension").and_then(|d| d.as_u64()).unwrap_or(384) as i32;
                    let metric = args.get("metric").and_then(|m| m.as_str()).unwrap_or("cosine");

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .create_collection(name.to_string(), dimension, metric.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC create_collection failed: {}", e), None))?;

                    let result_text = json!({
                        "name": response.name,
                        "dimension": response.dimension,
                        "similarity_metric": response.similarity_metric,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "delete_collection" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let name = args
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing name parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .delete_collection(name.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC delete_collection failed: {}", e), None))?;

                    let result_text = json!({
                        "collection_name": response.collection_name,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "insert_texts" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    // Parse vectors as texts for GRPC embedding
                    let parsed_texts: std::result::Result<
                        Vec<(String, String, Option<std::collections::HashMap<String, String>>)>,
                        &str,
                    > = args
                        .get("vectors")
                        .and_then(|v| v.as_array())
                        .ok_or("Missing vectors array")
                        .and_then(|vectors| {
                            vectors
                                .iter()
                                .map(|v| {
                                    let id = v
                                        .get("id")
                                        .and_then(|x| x.as_str())
                                        .ok_or("Missing vector id")
                                        .map(|s| s.to_string())?;
                                    
                                    // Extrair texto do payload ou usar dados como fallback
                                    // Primeiro tentar extrair texto diretamente
                                    let text = if let Some(text_str) = v.get("text").and_then(|x| x.as_str()) {
                                        text_str.to_string()
                                    } else {
                                        // Fallback: converter dados de vetor para string se texto não estiver disponível
                                        v.get("data")
                                            .and_then(|x| x.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|x| x.as_f64())
                                                    .map(|f| f.to_string())
                                                    .collect::<Vec<String>>()
                                                    .join(",")
                                            })
                                            .ok_or("Missing text or data for embedding")?
                                    };
                                    
                                    let metadata = v
                                        .get("metadata")
                                        .and_then(|x| x.as_object())
                                        .map(|obj| {
                                            obj.iter()
                                                .filter_map(|(k, v)| {
                                                    v.as_str().map(|s| (k.clone(), s.to_string()))
                                                })
                                                .collect()
                                        });
                                    Ok::<(String, String, Option<std::collections::HashMap<String, String>>), &str>((id, text, metadata))
                                })
                                .collect()
                        });

                    let texts_data = parsed_texts
                        .map_err(|e| ErrorData::invalid_params(format!("Failed to parse texts: {}", e), None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .insert_texts(collection.to_string(), texts_data, "bm25".to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC insert_texts failed: {}", e), None))?;

                    let result_text = json!({
                        "collection": response.collection,
                        "inserted_count": response.inserted_count,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "delete_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let vector_ids = args
                        .get("vector_ids")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing vector_ids parameter", None))?
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .delete_vectors(collection.to_string(), vector_ids)
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC delete_vectors failed: {}", e), None))?;

                    let result_text = json!({
                        "collection": response.collection,
                        "deleted_count": response.deleted_count,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_vector" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let vector_id = args
                        .get("vector_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing vector_id parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .get_vector(collection.to_string(), vector_id.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_vector failed: {}", e), None))?;

                    let result_text = json!({
                        "id": response.id,
                        "data": response.data,
                        "metadata": response.metadata,
                        "collection": response.collection,
                        "status": response.status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_indexing_progress" => {
                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .get_indexing_progress()
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_indexing_progress failed: {}", e), None))?;

                    let collections: Vec<serde_json::Value> = response.collections
                        .into_iter()
                        .map(|c| json!({
                            "collection_name": c.collection_name,
                            "status": c.status,
                            "progress": c.progress,
                            "vector_count": c.vector_count,
                            "error_message": c.error_message,
                            "last_updated": c.last_updated
                        }))
                        .collect();

                    let result_text = json!({
                        "collections": collections,
                        "is_indexing": response.is_indexing,
                        "overall_status": response.overall_status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "health_check" => {
                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .health_check()
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC health_check failed: {}", e), None))?;

                    let result_text = json!({
                        "status": response.status,
                        "service": response.service,
                        "version": response.version,
                        "timestamp": response.timestamp,
                        "error_message": response.error_message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_insert_texts" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let provider = args
                        .get("provider")
                        .and_then(|p| p.as_str())
                        .unwrap_or("bm25");

                    // Parse texts for GRPC
                    let parsed_texts: std::result::Result<
                        Vec<(String, String, Option<std::collections::HashMap<String, String>>)>,
                        &str,
                    > = args
                        .get("texts")
                        .and_then(|v| v.as_array())
                        .ok_or("Missing texts array")
                        .and_then(|texts| {
                            texts
                                .iter()
                                .map(|v| {
                                    let id = v
                                        .get("id")
                                        .and_then(|x| x.as_str())
                                        .ok_or("Missing text id")
                                        .map(|s| s.to_string())?;
                                    
                                    let text = v
                                        .get("text")
                                        .and_then(|x| x.as_str())
                                        .ok_or("Missing text content")
                                        .map(|s| s.to_string())?;
                                    
                                    let metadata = v
                                        .get("metadata")
                                        .and_then(|x| x.as_object())
                                        .map(|obj| {
                                            obj.iter()
                                                .filter_map(|(k, v)| {
                                                    v.as_str().map(|s| (k.clone(), s.to_string()))
                                                })
                                                .collect()
                                        });
                                    
                                    Ok::<(String, String, Option<std::collections::HashMap<String, String>>), &str>((id, text, metadata))
                                })
                                .collect()
                        });

                    let texts_data = parsed_texts
                        .map_err(|e| ErrorData::invalid_params(format!("Failed to parse texts: {}", e), None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .insert_texts(collection.to_string(), texts_data, provider.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC batch insert_texts failed: {}", e), None))?;

                    let result_text = json!({
                        "success": true,
                        "collection": response.collection,
                        "inserted_count": response.inserted_count,
                        "status": response.status,
                        "message": response.message,
                        "operation": "batch_insert_texts"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_search_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let provider = args
                        .get("provider")
                        .and_then(|p| p.as_str())
                        .unwrap_or("bm25");

                    // Parse queries
                    let queries = args
                        .get("queries")
                        .and_then(|q| q.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing queries array", None))?;

                    let mut grpc_client = self.get_grpc_client().await?;
                    let mut batch_results = Vec::new();

                    for (i, query_obj) in queries.iter().enumerate() {
                        let query_text = query_obj
                            .get("query")
                            .and_then(|q| q.as_str())
                            .ok_or_else(|| ErrorData::invalid_params(format!("Missing query in item {}", i), None))?;

                        let limit = query_obj
                            .get("limit")
                            .and_then(|l| l.as_u64())
                            .unwrap_or(10) as i32;

                        let score_threshold = query_obj
                            .get("score_threshold")
                            .and_then(|s| s.as_f64());

                        // Make GRPC search request
                        match grpc_client
                            .search(collection.to_string(), query_text.to_string(), limit)
                            .await
                        {
                            Ok(search_response) => {
                                let results: Vec<serde_json::Value> = search_response.results
                                    .into_iter()
                                    .filter(|result| {
                                        if let Some(threshold) = score_threshold {
                                            result.score >= threshold as f32
                                        } else {
                                            true
                                        }
                                    })
                                    .map(|result| json!({
                                        "id": result.id,
                                        "content": result.content,
                                        "score": result.score,
                                        "metadata": result.metadata
                                    }))
                                    .collect();

                                batch_results.push(json!({
                                    "query": query_text,
                                    "query_index": i,
                                    "results": results,
                                    "total_found": search_response.total_found,
                                    "search_time_ms": search_response.search_time_ms
                                }));
                            }
                            Err(e) => {
                                batch_results.push(json!({
                                    "query": query_text,
                                    "query_index": i,
                                    "error": format!("Search failed: {}", e),
                                    "results": [],
                                    "total_found": 0,
                                    "search_time_ms": 0
                                }));
                            }
                        }
                    }

                    let result_text = json!({
                        "success": true,
                        "collection": collection,
                        "total_queries": queries.len(),
                        "batch_results": batch_results,
                        "operation": "batch_search_vectors"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_update_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let provider = args
                        .get("provider")
                        .and_then(|p| p.as_str())
                        .unwrap_or("bm25");

                    // Parse updates
                    let updates = args
                        .get("updates")
                        .and_then(|u| u.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing updates array", None))?;

                    let mut grpc_client = self.get_grpc_client().await?;
                    let mut batch_results = Vec::new();
                    let mut successful_updates = 0;
                    let mut failed_updates = 0;

                    for (i, update_obj) in updates.iter().enumerate() {
                        let vector_id = update_obj
                            .get("id")
                            .and_then(|id| id.as_str())
                            .ok_or_else(|| ErrorData::invalid_params(format!("Missing id in update item {}", i), None))?;

                        // Check if we have new text to update
                        if let Some(text) = update_obj.get("text").and_then(|t| t.as_str()) {
                            // Parse metadata
                            let metadata = update_obj
                                .get("metadata")
                                .and_then(|x| x.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                });

                            // Create update data
                            let update_data = vec![(vector_id.to_string(), text.to_string(), metadata)];

                            // Make GRPC update request (using insert_texts as update mechanism)
                            match grpc_client
                                .insert_texts(collection.to_string(), update_data, provider.to_string())
                                .await
                            {
                                Ok(response) => {
                                    batch_results.push(json!({
                                        "id": vector_id,
                                        "update_index": i,
                                        "status": "success",
                                        "message": response.message
                                    }));
                                    successful_updates += 1;
                                }
                                Err(e) => {
                                    batch_results.push(json!({
                                        "id": vector_id,
                                        "update_index": i,
                                        "status": "error",
                                        "error": format!("Update failed: {}", e)
                                    }));
                                    failed_updates += 1;
                                }
                            }
                        } else {
                            // No text update, just metadata update (not supported in current GRPC)
                            batch_results.push(json!({
                                "id": vector_id,
                                "update_index": i,
                                "status": "skipped",
                                "message": "No text content provided for update"
                            }));
                        }
                    }

                    let result_text = json!({
                        "success": true,
                        "collection": collection,
                        "total_updates": updates.len(),
                        "successful_updates": successful_updates,
                        "failed_updates": failed_updates,
                        "batch_results": batch_results,
                        "operation": "batch_update_vectors"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_delete_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let vector_ids = args
                        .get("vector_ids")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing vector_ids parameter", None))?
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .delete_vectors(collection.to_string(), vector_ids)
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC batch delete_vectors failed: {}", e), None))?;

                    let result_text = json!({
                        "success": true,
                        "collection": response.collection,
                        "deleted_count": response.deleted_count,
                        "status": response.status,
                        "message": response.message,
                        "operation": "batch_delete_vectors"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                // Summarization tools
                "summarize_text" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let text = args.get("text").and_then(|t| t.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing text parameter", None)
                    })?;

                    let method = args.get("method").and_then(|m| m.as_str()).unwrap_or("extractive");
                    let max_length = args.get("max_length").and_then(|l| l.as_i64()).map(|l| l as i32);
                    let compression_ratio = args.get("compression_ratio").and_then(|r| r.as_f64()).map(|r| r as f32);
                    let language = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .summarize_text(crate::grpc::vectorizer::SummarizeTextRequest {
                            text: text.to_string(),
                            method: method.to_string(),
                            max_length,
                            compression_ratio,
                            language,
                            metadata: std::collections::HashMap::new(),
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC summarize_text failed: {}", e), None))?;

                    let result_text = json!({
                        "summary_id": response.summary_id,
                        "original_text": response.original_text,
                        "summary": response.summary,
                        "method": response.method,
                        "original_length": response.original_length,
                        "summary_length": response.summary_length,
                        "compression_ratio": response.compression_ratio,
                        "language": response.language,
                        "status": response.status,
                        "message": response.message,
                        "metadata": response.metadata
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "summarize_context" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let context = args.get("context").and_then(|c| c.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing context parameter", None)
                    })?;

                    let method = args.get("method").and_then(|m| m.as_str()).unwrap_or("extractive");
                    let max_length = args.get("max_length").and_then(|l| l.as_i64()).map(|l| l as i32);
                    let compression_ratio = args.get("compression_ratio").and_then(|r| r.as_f64()).map(|r| r as f32);
                    let language = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .summarize_context(crate::grpc::vectorizer::SummarizeContextRequest {
                            context: context.to_string(),
                            method: method.to_string(),
                            max_length,
                            compression_ratio,
                            language,
                            metadata: std::collections::HashMap::new(),
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC summarize_context failed: {}", e), None))?;

                    let result_text = json!({
                        "summary_id": response.summary_id,
                        "original_context": response.original_context,
                        "summary": response.summary,
                        "method": response.method,
                        "original_length": response.original_length,
                        "summary_length": response.summary_length,
                        "compression_ratio": response.compression_ratio,
                        "language": response.language,
                        "status": response.status,
                        "message": response.message,
                        "metadata": response.metadata
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_summary" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let summary_id = args.get("summary_id").and_then(|s| s.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing summary_id parameter", None)
                    })?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .get_summary(crate::grpc::vectorizer::GetSummaryRequest {
                            summary_id: summary_id.to_string(),
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_summary failed: {}", e), None))?;

                    let result_text = json!({
                        "summary_id": response.summary_id,
                        "original_text": response.original_text,
                        "summary": response.summary,
                        "method": response.method,
                        "original_length": response.original_length,
                        "summary_length": response.summary_length,
                        "compression_ratio": response.compression_ratio,
                        "language": response.language,
                        "created_at": response.created_at,
                        "metadata": response.metadata,
                        "status": response.status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "list_summaries" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let method = args.get("method").and_then(|m| m.as_str()).map(|s| s.to_string());
                    let language = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());
                    let limit = args.get("limit").and_then(|l| l.as_i64()).map(|l| l as i32);
                    let offset = args.get("offset").and_then(|o| o.as_i64()).map(|o| o as i32);

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .list_summaries(crate::grpc::vectorizer::ListSummariesRequest {
                            method,
                            language,
                            limit,
                            offset,
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC list_summaries failed: {}", e), None))?;

                    let summaries: Vec<serde_json::Value> = response.summaries
                        .into_iter()
                        .map(|summary| json!({
                            "summary_id": summary.summary_id,
                            "method": summary.method,
                            "language": summary.language,
                            "original_length": summary.original_length,
                            "summary_length": summary.summary_length,
                            "compression_ratio": summary.compression_ratio,
                            "created_at": summary.created_at,
                            "metadata": summary.metadata
                        }))
                        .collect();

                    let result_text = json!({
                        "summaries": summaries,
                        "total_count": response.total_count,
                        "status": response.status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                _ => Err(ErrorData::invalid_params("Unknown tool", None)),
            }
        }
    }
}

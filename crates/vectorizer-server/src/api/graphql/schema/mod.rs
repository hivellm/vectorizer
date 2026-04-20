//! GraphQL Schema and Resolvers for Vectorizer
//!
//! This module defines the GraphQL schema including Query and Mutation types.

use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Schema};
use tracing::{error, info, warn};

use super::types::*;
use vectorizer::config::FileUploadConfig;
use vectorizer::db::VectorStore;
use vectorizer::db::auto_save::AutoSaveManager;
use vectorizer::db::graph::{Edge, Node, RelationshipType};
use vectorizer::embedding::EmbeddingManager;
use vectorizer::file_loader::chunker::Chunker;
use vectorizer::file_loader::config::LoaderConfig;
use vectorizer::hub::auth::TenantContext;
use vectorizer::hub::quota::QuotaManager;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector,
};

/// GraphQL context containing shared state
pub struct GraphQLContext {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
    /// Optional tenant context for multi-tenant mode
    pub tenant_context: Option<TenantContext>,
    /// Optional quota manager for multi-tenant mode
    pub quota_manager: Option<Arc<QuotaManager>>,
    /// Optional auto-save manager for persistence
    pub auto_save_manager: Option<Arc<AutoSaveManager>>,
}

/// The GraphQL schema type
pub type VectorizerSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Create the GraphQL schema with the given context
///
/// Schema includes:
/// - Query depth limit of 10 (prevents deeply nested queries)
/// - Query complexity limit of 1000 (prevents expensive queries)
pub fn create_schema(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    start_time: std::time::Instant,
) -> VectorizerSchema {
    let ctx = GraphQLContext {
        store,
        embedding_manager,
        start_time,
        tenant_context: None,
        quota_manager: None,
        auto_save_manager: None,
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        // Limit query depth to prevent deeply nested queries
        .limit_depth(10)
        // Limit query complexity to prevent expensive queries
        .limit_complexity(1000)
        .finish()
}

/// Create the GraphQL schema with auto-save support
pub fn create_schema_with_auto_save(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    start_time: std::time::Instant,
    auto_save_manager: Arc<AutoSaveManager>,
) -> VectorizerSchema {
    let ctx = GraphQLContext {
        store,
        embedding_manager,
        start_time,
        tenant_context: None,
        quota_manager: None,
        auto_save_manager: Some(auto_save_manager),
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .limit_depth(10)
        .limit_complexity(1000)
        .finish()
}

/// Create the GraphQL schema with multi-tenant support
pub fn create_schema_with_hub(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    start_time: std::time::Instant,
    quota_manager: Arc<QuotaManager>,
) -> VectorizerSchema {
    let ctx = GraphQLContext {
        store,
        embedding_manager,
        start_time,
        tenant_context: None, // Set per-request in handler
        quota_manager: Some(quota_manager),
        auto_save_manager: None,
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .limit_depth(10)
        .limit_complexity(1000)
        .finish()
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Verify collection ownership in multi-tenant mode
fn check_collection_ownership(
    store: &VectorStore,
    collection: &str,
    tenant_ctx: Option<&TenantContext>,
) -> async_graphql::Result<()> {
    if let Some(tenant) = tenant_ctx {
        // Parse tenant_id as UUID
        let tenant_uuid = uuid::Uuid::parse_str(&tenant.tenant_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid tenant ID: {e}")))?;

        if !store.is_collection_owned_by(collection, &tenant_uuid) {
            return Err(async_graphql::Error::new(
                "Collection not found or access denied",
            ));
        }
    }
    Ok(())
}

// ============================================================================
// Sub-modules — QueryRoot / MutationRoot (phase4_split-graphql-schema).
// The `#[Object] impl` blocks are too large to keep next to the schema
// builders. Helpers below (`check_collection_ownership`,
// `load_file_upload_config`, `base64_decode`, `is_binary_content`,
// `get_language_from_extension`) are shared by both roots and stay here;
// the sub-files import them via `use super::...`.
// ============================================================================

pub mod mutation;
pub mod query;

pub use mutation::MutationRoot;
pub use query::QueryRoot;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Load file upload configuration from config.yml
fn load_file_upload_config() -> FileUploadConfig {
    std::fs::read_to_string("config.yml")
        .ok()
        .and_then(|content| {
            serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                .ok()
                .map(|config| config.file_upload)
        })
        .unwrap_or_default()
}

/// Decode base64 string to bytes
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| e.to_string())
}

/// Check if content appears to be binary
fn is_binary_content(content: &[u8]) -> bool {
    let check_size = content.len().min(8192);
    let sample = &content[..check_size];

    let mut null_count = 0;
    let mut non_printable_count = 0;

    for &byte in sample {
        if byte == 0 {
            null_count += 1;
        } else if byte < 0x09 || (byte > 0x0D && byte < 0x20 && byte != 0x1B) {
            non_printable_count += 1;
        }
    }

    let null_ratio = null_count as f32 / check_size as f32;
    let non_printable_ratio = non_printable_count as f32 / check_size as f32;

    null_ratio > 0.01 || non_printable_ratio > 0.10
}

/// Get language from file extension
fn get_language_from_extension(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "rs" => "rust",
        "py" | "pyw" | "pyi" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "jsx" => "javascriptreact",
        "tsx" => "typescriptreact",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" | "rake" | "gemspec" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "r" => "r",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        "ps1" | "psm1" | "psd1" => "powershell",
        "bat" | "cmd" => "batch",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "ini" | "cfg" | "conf" => "ini",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        "md" | "markdown" => "markdown",
        "rst" => "restructuredtext",
        "txt" | "text" => "plaintext",
        "csv" => "csv",
        "log" => "log",
        _ => "plaintext",
    }
}

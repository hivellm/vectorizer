//! API route definitions for the Vectorizer REST API

use axum::{
    Router,
    routing::{delete, get, post},
};

use super::handlers::{
    AppState,
    // Batch handlers
    batch_delete_vectors,
    batch_insert_texts,
    batch_search_vectors,
    batch_update_vectors,
    create_collection,
    delete_collection,
    delete_vector,
    embed_text,
    get_collection,
    get_indexing_progress,
    get_memory_analysis,
    get_vector,
    health_check,
      get_stats,
    insert_texts,
    list_collections,
    list_embedding_providers,
    list_files,
    list_vectors,
    mcp_http_tools_call,
    // MCP handlers
    mcp_initialize,
    mcp_ping,
    mcp_sse,
    mcp_tools_call,
    mcp_tools_list,
    requantize_collection,
    generate_memory_profile,
    analyze_heap_memory,
    search_by_file,
    search_vectors,
    search_vectors_by_text,
    set_embedding_provider,
    stream_indexing_progress,
    update_indexing_progress,
    // Summarization handlers
    summarize_text,
    summarize_context,
    get_summary,
    list_summaries,
};

use super::metrics_handlers::metrics_handler;

use super::memory_handlers::{
    generate_memory_snapshot,
};

/// Create the main API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check and metrics
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))
        .route("/metrics", get(metrics_handler))
        .route("/memory-analysis", get(get_memory_analysis))
        // Indexing progress
        .route("/indexing/progress", get(get_indexing_progress))
        .route("/indexing/progress", post(update_indexing_progress))
        .route("/indexing/progress/stream", get(stream_indexing_progress))
        // Collection management
        .route("/collections", get(list_collections))
        .route("/collections", post(create_collection))
        .route("/collections/{collection_name}", get(get_collection))
        .route("/collections/{collection_name}", delete(delete_collection))
        .route("/collections/{collection_name}/requantize", post(requantize_collection))
        // Memory profiling
        .route("/memory-profile", get(generate_memory_profile))
        .route("/heap-analysis", get(analyze_heap_memory))
        // Memory snapshot endpoints
        // .route("/memory-snapshot", get(generate_memory_snapshot))
        // Vector operations
        .route(
            "/collections/{collection_name}/vectors",
            post(insert_texts).get(list_vectors),
        )
        .route(
            "/collections/{collection_name}/search",
            post(search_vectors),
        )
        .route(
            "/collections/{collection_name}/search/text",
            post(search_vectors_by_text),
        )
        .route(
            "/collections/{collection_name}/search/file",
            post(search_by_file),
        )
        .route("/collections/{collection_name}/files", post(list_files))
        .route(
            "/collections/{collection_name}/vectors/{vector_id}",
            get(get_vector),
        )
        .route(
            "/collections/{collection_name}/vectors/{vector_id}",
            delete(delete_vector),
        )
        // Embedding operations
        .route("/embed", post(embed_text))
        // Embedding provider management
        .route("/embedding/providers", get(list_embedding_providers))
        .route("/embedding/providers/set", post(set_embedding_provider))
        // Batch operations
        .route(
            "/collections/{collection_name}/batch/insert",
            post(batch_insert_texts),
        )
        .route(
            "/collections/{collection_name}/batch/update",
            post(batch_update_vectors),
        )
        .route(
            "/collections/{collection_name}/batch/delete",
            post(batch_delete_vectors),
        )
        .route(
            "/collections/{collection_name}/batch/search",
            post(batch_search_vectors),
        )
        // Summarization endpoints
        .route("/summarize/text", post(summarize_text))
        .route("/summarize/context", post(summarize_context))
        .route("/summaries/{summary_id}", get(get_summary))
        .route("/summaries", get(list_summaries))
        .with_state(state)
}

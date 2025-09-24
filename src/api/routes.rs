//! API route definitions for the Vectorizer REST API

use axum::{
    Router,
    routing::{delete, get, post},
};

use super::handlers::{
    AppState, create_collection, delete_collection, delete_vector, get_collection, get_vector,
    health_check, insert_vectors, list_collections, list_files, list_vectors, search_by_file,
    search_vectors, search_vectors_by_text, list_embedding_providers, set_embedding_provider,
};

/// Create the main API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Collection management
        .route("/collections", get(list_collections))
        .route("/collections", post(create_collection))
        .route("/collections/:collection_name", get(get_collection))
        .route("/collections/:collection_name", delete(delete_collection))
        // Vector operations
        .route(
            "/collections/:collection_name/vectors",
            post(insert_vectors),
        )
        .route(
            "/collections/:collection_name/vectors",
            get(list_vectors),
        )
        .route("/collections/:collection_name/search", post(search_vectors))
        .route(
            "/collections/:collection_name/search/text",
            post(search_vectors_by_text),
        )
        .route(
            "/collections/:collection_name/search/file",
            post(search_by_file),
        )
        .route(
            "/collections/:collection_name/files",
            post(list_files),
        )
        .route(
            "/collections/:collection_name/vectors/:vector_id",
            get(get_vector),
        )
        .route(
            "/collections/:collection_name/vectors/:vector_id",
            delete(delete_vector),
        )
        // Embedding provider management
        .route("/embedding/providers", get(list_embedding_providers))
        .route("/embedding/providers/set", post(set_embedding_provider))
        .with_state(state)
}

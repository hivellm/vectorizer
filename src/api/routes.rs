//! API route definitions for the Vectorizer REST API

use axum::{
    routing::{delete, get, post},
    Router,
};

use super::handlers::{
    create_collection, delete_collection, delete_vector, get_collection, get_vector, health_check,
    insert_vectors, list_collections, search_vectors, AppState,
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
        .route("/collections/:collection_name/vectors", post(insert_vectors))
        .route("/collections/:collection_name/search", post(search_vectors))
        .route("/collections/:collection_name/vectors/:vector_id", get(get_vector))
        .route("/collections/:collection_name/vectors/:vector_id", delete(delete_vector))
        
        .with_state(state)
}

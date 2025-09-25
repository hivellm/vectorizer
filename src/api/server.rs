//! HTTP server implementation for the Vectorizer API

use axum::{
    Router,
    routing::{get, post, delete},
    response::Json,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    services::ServeDir,
};
use tracing::info;

use crate::{VectorStore, embedding::EmbeddingManager};
use std::sync::Arc;

use super::{handlers::AppState, routes::create_router};

/// Vectorizer HTTP server
pub struct VectorizerServer {
    /// Server address
    addr: SocketAddr,
    /// Application state
    state: AppState,
}

impl VectorizerServer {
    /// Create a new server instance
    pub fn new(host: &str, port: u16, store: Arc<VectorStore>, embedding_manager: EmbeddingManager) -> Self {
        let addr = format!("{}:{}", host, port)
            .parse()
            .expect("Invalid host/port combination");

        let state = AppState::new(store, embedding_manager);

        Self { addr, state }
    }

    /// Create a new server instance with existing AppState
    pub fn new_with_state(host: &str, port: u16, state: AppState) -> Self {
        let addr = format!("{}:{}", host, port)
            .parse()
            .expect("Invalid host/port combination");

        Self { addr, state }
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Vectorizer HTTP server on {}", self.addr);

        // Create the main router
        let app = self.create_app();

        // Create TCP listener
        let listener = TcpListener::bind(self.addr).await?;
        info!("Server listening on {}", self.addr);

        // Start the server
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Create the Axum application with all middleware
    pub fn create_app(&self) -> Router {
        // Create the main router
        let api_router = create_router(self.state.clone());

        // Create dashboard router with static files only
        let dashboard_router = Router::new()
            .route("/", get(|| async { 
                axum::response::Redirect::permanent("/dashboard/static/index.html")
            }))
            .nest_service("/static", ServeDir::new("dashboard/public"));

        // Build the application with middleware
        Router::new()
            .route("/test", get(|| async {
                info!("Test endpoint called!");
                Json(serde_json::json!({"message": "Server test endpoint working!"}))
            }))
            .nest("/api/v1", api_router)
            .nest("/dashboard", dashboard_router)
            .fallback(not_found_handler)
    }
}

/// Handler for 404 Not Found responses
async fn not_found_handler() -> (axum::http::StatusCode, &'static str) {
    (axum::http::StatusCode::NOT_FOUND, "Not Found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let store = VectorStore::new();
        let server = VectorizerServer::new("127.0.0.1", 0, store);
        let app = server.create_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_found() {
        let store = VectorStore::new();
        let server = VectorizerServer::new("127.0.0.1", 0, store);
        let app = server.create_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

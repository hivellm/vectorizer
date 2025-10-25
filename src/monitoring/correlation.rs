//! Correlation ID Middleware
//!
//! This module provides correlation ID tracking for distributed request tracing.
//! Each request receives a unique ID that is propagated through all logs and traces.

use std::sync::Arc;

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::http::header::HeaderName;
use axum::middleware::Next;
use axum::response::Response;
use tokio::task_local;
use uuid::Uuid;

/// Header name for correlation ID
pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";

/// Task-local storage for correlation ID
task_local! {
    pub static CORRELATION_ID: Arc<String>;
}

/// Generate a new correlation ID
pub fn generate_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Middleware to add correlation ID to requests
pub async fn correlation_middleware(mut req: Request, next: Next) -> Response {
    // Extract or generate correlation ID
    let correlation_id = req
        .headers()
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(generate_correlation_id);

    let correlation_id_arc = Arc::new(correlation_id.clone());

    // Add correlation ID to request extensions for downstream handlers
    req.extensions_mut().insert(correlation_id_arc.clone());

    // Run the request with correlation ID in context
    let response = CORRELATION_ID
        .scope(correlation_id_arc, async move { next.run(req).await })
        .await;

    // Add correlation ID to response headers
    let mut response = response;
    if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-correlation-id"), header_value);
    }

    response
}

/// Get the current correlation ID from task-local storage
pub fn current_correlation_id() -> Option<String> {
    CORRELATION_ID.try_with(|id| (**id).clone()).ok()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use axum::middleware;
    use tower::ServiceExt;

    use super::*; // for `oneshot`

    #[test]
    fn test_generate_correlation_id() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2, "Each ID should be unique");
    }

    #[tokio::test]
    async fn test_correlation_middleware() {
        use axum::Router;
        use axum::routing::get;

        // Create a simple handler that returns OK
        async fn handler() -> &'static str {
            "ok"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(middleware::from_fn(correlation_middleware));

        // Test request without correlation ID
        let request = HttpRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should have correlation ID in response
        assert!(response.headers().get("x-correlation-id").is_some());
    }

    #[tokio::test]
    async fn test_correlation_id_propagation() {
        use axum::Router;
        use axum::routing::get;

        async fn handler() -> &'static str {
            // Get correlation ID from context
            let id = current_correlation_id();
            assert!(
                id.is_some(),
                "Correlation ID should be available in handler"
            );
            "ok"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(middleware::from_fn(correlation_middleware));

        let request = HttpRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let _response = app.oneshot(request).await.unwrap();
    }

    #[tokio::test]
    async fn test_existing_correlation_id() {
        use axum::Router;
        use axum::routing::get;

        let test_id = "test-correlation-id-12345";

        async fn handler() -> &'static str {
            "ok"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(middleware::from_fn(correlation_middleware));

        let request = HttpRequest::builder()
            .uri("/test")
            .header(CORRELATION_ID_HEADER, test_id)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should preserve the provided correlation ID
        let response_id = response
            .headers()
            .get("x-correlation-id")
            .and_then(|v| v.to_str().ok());

        assert_eq!(response_id, Some(test_id));
    }
}

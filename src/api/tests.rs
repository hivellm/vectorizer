//! API integration tests for REST endpoints

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use serde_json::json;
    use tower::ServiceExt;

    use crate::{api::server::VectorizerServer, VectorStore};

    fn app() -> Router {
        let store = VectorStore::new();
        let server = VectorizerServer::new("127.0.0.1", 0, store);
        server.create_app()
    }

    #[tokio::test]
    async fn create_and_get_collection() {
        let app = app();

        // Create collection
        let payload = json!({
            "name": "test_col",
            "dimension": 4,
            "metric": "cosine"
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/collections")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Get collection
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/collections/test_col")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn list_collections_after_creation() {
        let app = app();

        for i in 0..2 {
            let body = json!({
                "name": format!("col_{}", i),
                "dimension": 3,
                "metric": "cosine"
            });
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/collections")
                        .header("content-type", "application/json")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/collections")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn insert_get_search_and_delete_vector() {
        let app = app();

        // Create collection (dimension 4)
        let payload = json!({
            "name": "docs",
            "dimension": 4,
            "metric": "cosine"
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/collections")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Insert a vector
        let insert_body = json!({
            "vectors": [
                {
                    "id": "v1",
                    "vector": [0.1, 0.2, 0.3, 0.4],
                    "payload": {"source": "test"}
                }
            ]
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/collections/docs/vectors")
                    .header("content-type", "application/json")
                    .body(Body::from(insert_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Get the vector
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/collections/docs/vectors/v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Vector search
        let search_body = json!({
            "vector": [0.1, 0.21, 0.29, 0.41],
            "limit": 5
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/collections/docs/search")
                    .header("content-type", "application/json")
                    .body(Body::from(search_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Text search (may return zero results; just ensure 200)
        let text_body = json!({
            "query": "example query",
            "limit": 5
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/collections/docs/search/text")
                    .header("content-type", "application/json")
                    .body(Body::from(text_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Delete vector
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/collections/docs/vectors/v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Ensure it is gone
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/collections/docs/vectors/v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_collection_after_use() {
        let app = app();

        let payload = json!({
            "name": "to_delete",
            "dimension": 3,
            "metric": "cosine"
        });

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/collections")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/collections/to_delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/collections/to_delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}



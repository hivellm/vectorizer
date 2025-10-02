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

    use crate::{api::server::VectorizerServer, VectorStore, embedding::EmbeddingManager};
    use std::sync::Arc;

    fn app() -> Router {
        let store = Arc::new(VectorStore::new());
        let embedding_manager = EmbeddingManager::new();
        let server = VectorizerServer::new("127.0.0.1", 0, store, embedding_manager, None);
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
        assert_eq!(resp.status(), StatusCode::CREATED);

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

        // Create collection (dimension 512 to match embedding)
        let payload = json!({
            "name": "docs",
            "dimension": 512,
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

        // Insert a text (which will be automatically embedded)
        let insert_body = json!({
            "texts": [
                {
                    "id": "v1",
                    "text": "This is a test document for vector insertion",
                    "metadata": {"source": "test"}
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
        
        let status = resp.status();
        if status != StatusCode::OK && status != StatusCode::CREATED {
            let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
            let error_body = String::from_utf8_lossy(&body);
            panic!("Insert failed with status {}: {}", status, error_body);
        }

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
        let mut search_vector = Vec::new();
        for i in 0..512 {
            search_vector.push((i as f32) * 0.001);
        }
        let search_body = json!({
            "vector": search_vector,
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
        
        let status = resp.status();
        if status != StatusCode::OK {
            let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
            let error_body = String::from_utf8_lossy(&body);
            panic!("Vector search failed with status {}: {}", status, error_body);
        }

        // Text search (may return zero results; just ensure 200)
        let text_body = json!({
            "query": "test document",
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
        
        // Text search may fail if no vectors are indexed, so we'll just check it doesn't crash
        let status = resp.status();
        if status != StatusCode::OK && status != StatusCode::NOT_FOUND {
            let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
            let error_body = String::from_utf8_lossy(&body);
            panic!("Text search failed with status {}: {}", status, error_body);
        }

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
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

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
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

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

// Summarization tests
#[cfg(test)]
mod summarization_tests {
    use super::*;
    use crate::api::types::{
        SummarizeTextRequest, SummarizeTextResponse,
        SummarizeContextRequest, SummarizeContextResponse,
        GetSummaryResponse, ListSummariesResponse, SummaryInfo
    };
    use crate::api::handlers::{AppState, summarize_text, summarize_context, get_summary, list_summaries};
    use crate::summarization::{SummarizationManager, SummarizationConfig, MethodConfig, LanguageConfig, MetadataConfig};
    use crate::VectorStore;
    use crate::embedding::EmbeddingManager;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    async fn create_test_app_state() -> AppState {
        let mut methods = HashMap::new();
        methods.insert("extractive".to_string(), MethodConfig::default());
        methods.insert("keyword".to_string(), MethodConfig::default());
        
        let mut languages = HashMap::new();
        languages.insert("en".to_string(), LanguageConfig::default());
        languages.insert("pt".to_string(), LanguageConfig::default());
        
        let config = SummarizationConfig {
            enabled: true,
            auto_summarize: true,
            summary_collection: "test_summaries".to_string(),
            default_method: "extractive".to_string(),
            methods,
            languages,
            metadata: MetadataConfig::default(),
        };

        let summarization_manager = Arc::new(Mutex::new(SummarizationManager::new(config).unwrap()));
        
        AppState {
            store: Arc::new(VectorStore::new()),
            embedding_manager: Arc::new(Mutex::new(EmbeddingManager::new())),
            grpc_client: None,
            summarization_manager: Some(summarization_manager),
            start_time: std::time::Instant::now(),
            indexing_progress: crate::api::handlers::IndexingProgressState::from_map(HashMap::new()),
            workspace_collections: Vec::new(),
            file_watcher: None,
        }
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/api/v1/summarize/text", axum::routing::post(summarize_text))
            .route("/api/v1/summarize/context", axum::routing::post(summarize_context))
            .route("/api/v1/summaries/{summary_id}", axum::routing::get(get_summary))
            .route("/api/v1/summaries", axum::routing::get(list_summaries))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_summarize_text_rest() {
        let state = create_test_app_state().await;
        let app = create_test_router(state);

        let request_body = SummarizeTextRequest {
            text: "This is a long text that needs to be summarized using the REST API. It contains multiple sentences and should be compressed to a shorter version while maintaining the key information.".to_string(),
            method: "extractive".to_string(),
            max_length: Some(50),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: Some(HashMap::new()),
        };

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/summarize/text")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response: SummarizeTextResponse = serde_json::from_slice(&body).unwrap();

        assert!(!response.summary.is_empty());
        assert!(response.summary.len() < response.original_text.len());
        assert_eq!(response.method, "extractive");
        assert_eq!(response.language, "en");
        assert_eq!(response.status, "success");
        assert!(response.compression_ratio > 0.0);
        assert!(response.compression_ratio <= 1.0);
    }

    #[tokio::test]
    async fn test_summarize_context_rest() {
        let state = create_test_app_state().await;
        let app = create_test_router(state);

        let request_body = SummarizeContextRequest {
            context: "This is a comprehensive context about artificial intelligence and machine learning applications in various industries. Healthcare applications include diagnostic imaging, drug discovery, and personalized treatment plans. Financial services use AI for fraud detection, algorithmic trading, and risk assessment. The automotive industry leverages machine learning for autonomous driving, predictive maintenance, and supply chain optimization. These technologies are transforming how businesses operate and deliver value to customers. The integration of AI systems requires careful consideration of ethical implications, data privacy, and regulatory compliance. Organizations must invest in proper infrastructure, talent development, and change management to successfully implement these advanced technologies.".to_string(),
            method: "extractive".to_string(),
            max_length: Some(100),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: Some(HashMap::new()),
        };

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/summarize/context")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response: SummarizeContextResponse = serde_json::from_slice(&body).unwrap();

        assert!(!response.summary.is_empty());
        assert!(response.summary.len() < response.original_context.len());
        assert_eq!(response.method, "extractive");
        assert_eq!(response.language, "en");
        assert_eq!(response.status, "success");
    }

    #[tokio::test]
    async fn test_get_summary_rest() {
        let state = create_test_app_state().await;
        let app = create_test_router(state);

        // First create a summary
        let create_request_body = SummarizeTextRequest {
            text: "This is a test document for retrieval testing via REST API.".to_string(),
            method: "extractive".to_string(),
            max_length: Some(20),
            compression_ratio: Some(0.3),
            language: Some("en".to_string()),
            metadata: Some(HashMap::new()),
        };

        let create_request = Request::builder()
            .method("POST")
            .uri("/api/v1/summarize/text")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&create_request_body).unwrap()))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
        let create_response: SummarizeTextResponse = serde_json::from_slice(&create_body).unwrap();
        let summary_id = create_response.summary_id.clone();

        // Now retrieve it
        let get_request = Request::builder()
            .method("GET")
            .uri(&format!("/api/v1/summaries/{}", summary_id))
            .body(Body::empty())
            .unwrap();

        let get_response = app.oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let get_body = axum::body::to_bytes(get_response.into_body(), usize::MAX).await.unwrap();
        let get_response: GetSummaryResponse = serde_json::from_slice(&get_body).unwrap();

        assert_eq!(get_response.summary_id, summary_id);
        assert_eq!(get_response.summary, create_response.summary);
        assert_eq!(get_response.method, create_response.method);
        assert_eq!(get_response.language, create_response.language);
        assert_eq!(get_response.status, "success");
    }

    #[tokio::test]
    async fn test_list_summaries_rest() {
        let state = create_test_app_state().await;
        let app = create_test_router(state);

        // Create multiple summaries
        let texts = vec![
            "First document about technology and its applications in modern society. Technology has revolutionized how we communicate, work, and live our daily lives. From smartphones to artificial intelligence, technological advances continue to shape our future.",
            "Second document about science and research methodologies. Scientific research involves systematic investigation and experimentation to discover new knowledge. Researchers use various methods to test hypotheses and validate their findings through peer review.",
            "Third document about innovation and entrepreneurship in the digital age. Innovation drives economic growth and creates new opportunities for businesses and individuals. Entrepreneurs leverage technology to develop solutions that address market needs and challenges.",
        ];
        
        for text in texts {
            let request_body = SummarizeTextRequest {
                text: text.to_string(),
                method: "extractive".to_string(),
                max_length: Some(20),
                compression_ratio: Some(0.3),
                language: Some("en".to_string()),
                metadata: Some(HashMap::new()),
            };

            let request = Request::builder()
                .method("POST")
                .uri("/api/v1/summarize/text")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap();

            app.clone().oneshot(request).await.unwrap();
        }
        
        // List summaries
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/summaries?limit=10&offset=0")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response: ListSummariesResponse = serde_json::from_slice(&body).unwrap();

        assert!(response.summaries.len() >= 3);
        assert_eq!(response.status, "success");
    }
}



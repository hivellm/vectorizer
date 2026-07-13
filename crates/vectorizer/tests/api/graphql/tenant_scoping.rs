//! GraphQL multi-tenant collection-name consistency (phase40 §4).
//!
//! `create_collection` and `upload_file` used to build the tenant-scoped
//! collection name independently — `user_{id}:{name}` (colon) vs
//! `user_{id}_{name}` (underscore) — so an upload immediately after a
//! create silently landed in (and auto-created) a *different* collection
//! than the one the user had just made. Both resolvers now go through
//! the single `tenant_collection_name` helper in
//! `vectorizer-server/src/api/graphql/schema/mod.rs`.
//!
//! This test proves the fix end-to-end, in-process (no live HiveHub
//! server needed — `TenantContext` is injected directly into the
//! `async-graphql` request, exactly like `hub_auth_middleware` does for
//! a real request): create a collection as tenant A, then upload a file
//! into "the same" collection name as tenant A, and assert the vectors
//! actually landed in the collection `create_collection` made — not a
//! second, differently-prefixed collection.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use vectorizer::db::VectorStore;
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::hub::auth::TenantContext;
use vectorizer_server::api::graphql::create_schema;

/// Small seed corpus so BM25 produces non-zero embeddings for the
/// uploaded file's content (an unfitted/empty BM25 vocabulary would
/// silently embed everything to a zero vector, and `upload_file` skips
/// zero-vector chunks — see `mutation.rs`'s `upload_file`).
const SEED_CORPUS: &[&str] = &[
    "the quick brown fox jumps over the lazy dog",
    "vector databases store high dimensional embeddings",
    "semantic search finds documents by meaning not keywords",
];

fn build_embedding_manager() -> Arc<EmbeddingManager> {
    let mut bm25 = Bm25Embedding::new(64);
    bm25.build_vocabulary(
        &SEED_CORPUS
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
    );
    let mut manager = EmbeddingManager::new();
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager
        .set_default_provider("bm25")
        .expect("bm25 provider was just registered");
    Arc::new(manager)
}

fn test_tenant_context(tenant_id: &str) -> TenantContext {
    TenantContext {
        tenant_id: tenant_id.to_string(),
        tenant_name: "phase40 tenant scoping test".to_string(),
        api_key_id: "test-key".to_string(),
        permissions: vec![vectorizer::hub::auth::TenantPermission::ReadWrite],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    }
}

#[tokio::test]
async fn create_collection_then_upload_file_target_the_same_collection() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = build_embedding_manager();
    let start_time = std::time::Instant::now();
    let schema = create_schema(store.clone(), embedding_manager, start_time);

    let tenant_id = "tenant-phase40-scoping";
    let tenant_ctx = test_tenant_context(tenant_id);
    let collection_name = "phase40_scoping_collection";

    // The single source of truth for the format both resolvers must
    // agree on. Deliberately not calling the crate's own
    // `tenant_collection_name` helper (it's a private `mod schema` item,
    // unreachable from this integration test) — this is the *expected*
    // format both mutations should independently arrive at.
    let expected_prefixed_name = format!("user_{tenant_id}:{collection_name}");
    let stale_buggy_name = format!("user_{tenant_id}_{collection_name}");

    // 1. Create the collection as this tenant.
    let create_mutation = r"
        mutation($name: String!, $dimension: Int!) {
            createCollection(input: { name: $name, dimension: $dimension }) {
                name
            }
        }
    ";
    let create_request = async_graphql::Request::new(create_mutation)
        .variables(async_graphql::Variables::from_json(serde_json::json!({
            "name": collection_name,
            "dimension": 64,
        })))
        .data(tenant_ctx.clone());
    let create_response = schema.execute(create_request).await;
    assert!(
        create_response.errors.is_empty(),
        "createCollection GraphQL errors: {:?}",
        create_response.errors
    );
    let created_name = create_response
        .data
        .into_json()
        .expect("createCollection response must be JSON")["createCollection"]["name"]
        .as_str()
        .expect("createCollection.name must be a string")
        .to_string();
    assert_eq!(
        created_name, expected_prefixed_name,
        "create_collection must store the collection under the tenant-prefixed \
         (colon-separated) name"
    );

    // 2. Upload a file into "the same" collection, as the same tenant.
    let file_content = "vector databases store high dimensional embeddings for semantic search";
    let content_base64 = BASE64.encode(file_content.as_bytes());
    let upload_mutation = r"
        mutation($input: UploadFileInput!) {
            uploadFile(input: $input) {
                success
                vectorsCreated
                error
            }
        }
    ";
    let upload_request = async_graphql::Request::new(upload_mutation)
        .variables(async_graphql::Variables::from_json(serde_json::json!({
            "input": {
                "collectionName": collection_name,
                "filename": "notes.txt",
                "contentBase64": content_base64,
            }
        })))
        .data(tenant_ctx);
    let upload_response = schema.execute(upload_request).await;
    assert!(
        upload_response.errors.is_empty(),
        "uploadFile GraphQL errors: {:?}",
        upload_response.errors
    );
    let upload_json = upload_response
        .data
        .into_json()
        .expect("uploadFile response must be JSON");
    assert_eq!(
        upload_json["uploadFile"]["success"].as_bool(),
        Some(true),
        "uploadFile must succeed: {upload_json:?}"
    );
    let vectors_created = upload_json["uploadFile"]["vectorsCreated"]
        .as_i64()
        .expect("vectorsCreated must be a number");
    assert!(
        vectors_created > 0,
        "uploadFile must create at least one vector: {upload_json:?}"
    );

    // 3. The uploaded vectors must have landed in the SAME collection
    // `create_collection` made — not a second, differently-prefixed one.
    let metadata = store
        .get_collection_metadata(&expected_prefixed_name)
        .expect("the tenant-prefixed collection must exist and hold the uploaded vectors");
    assert_eq!(
        metadata.vector_count, vectors_created as usize,
        "vector count in the tenant-prefixed collection must match uploadFile's own count"
    );

    // 4. The old, buggy underscore-separated name must never have been
    // created — that's exactly the divergent-collection bug this fix
    // closes.
    assert!(
        store.get_collection_metadata(&stale_buggy_name).is_err(),
        "no collection should exist under the old buggy underscore-separated name"
    );
}

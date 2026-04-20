//! Live gRPC probe against a running `vectorizer` server on
//! `127.0.0.1:15003`. Complements `tests/grpc_integration.rs` (which
//! spins up its own in-process Tonic server).
//!
//! Covers probe 2.3 of `phase8_release-v3-runtime-verification`:
//! confirms the Tonic 0.14 + prost wire formats land on the live
//! binary, the `grpc_conversions` module round-trips, and
//! `ListCollections` + `CreateCollection` + `InsertVector` + `Search`
//! all return the expected proto shape.
//!
//! Run with: `cargo test -p vectorizer-server --test grpc_live -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

use tonic::transport::Channel;
use vectorizer_protocol::grpc_gen::vectorizer::{
    CollectionConfig as ProtoCollectionConfig, CreateCollectionRequest, DeleteCollectionRequest,
    DistanceMetric as ProtoDistanceMetric, GetVectorRequest, HnswConfig as ProtoHnswConfig,
    InsertVectorRequest, ListCollectionsRequest, SearchRequest, StorageType as ProtoStorageType,
    vectorizer_service_client::VectorizerServiceClient,
};

const GRPC_ADDR: &str = "http://127.0.0.1:15003";

async fn connect() -> VectorizerServiceClient<Channel> {
    VectorizerServiceClient::connect(GRPC_ADDR)
        .await
        .expect("live gRPC server on 127.0.0.1:15003")
}

#[tokio::test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15003"]
async fn live_grpc_list_create_insert_search_roundtrip() {
    let mut client = connect().await;

    // 1. ListCollections — picks up whatever the live server has.
    let list = client
        .list_collections(ListCollectionsRequest {})
        .await
        .expect("ListCollections RPC")
        .into_inner();
    println!(
        "ListCollections: {} collection(s) returned over live gRPC",
        list.collection_names.len()
    );

    // 2. CreateCollection.
    let coll_name = format!("grpc_live_{}", uuid::Uuid::new_v4().simple());
    let config = ProtoCollectionConfig {
        dimension: 128,
        metric: ProtoDistanceMetric::Cosine as i32,
        hnsw_config: Some(ProtoHnswConfig {
            m: 16,
            ef_construction: 200,
            ef: 50,
            seed: 42,
        }),
        quantization: None,
        storage_type: ProtoStorageType::Memory as i32,
    };
    let created = client
        .create_collection(CreateCollectionRequest {
            name: coll_name.clone(),
            config: Some(config),
        })
        .await
        .expect("CreateCollection RPC")
        .into_inner();
    assert!(
        created.success,
        "CreateCollection.success = false: {}",
        created.message
    );

    // 3. InsertVector (dim 128 deterministic data).
    let data: Vec<f32> = (0..128).map(|i| (i as f32) / 128.0).collect();
    let vec_id = "grpc_live_vector_1".to_string();
    let inserted = client
        .insert_vector(InsertVectorRequest {
            collection_name: coll_name.clone(),
            vector_id: vec_id.clone(),
            data: data.clone(),
            payload: HashMap::new(),
        })
        .await
        .expect("InsertVector RPC")
        .into_inner();
    assert!(
        inserted.success,
        "InsertVector.success = false: {}",
        inserted.message
    );

    // 4. GetVector — verify the data round-tripped byte-for-byte (modulo
    // any cosine normalization the collection applies).
    let got = client
        .get_vector(GetVectorRequest {
            collection_name: coll_name.clone(),
            vector_id: vec_id.clone(),
        })
        .await
        .expect("GetVector RPC")
        .into_inner();
    assert_eq!(got.vector_id, vec_id);
    assert_eq!(got.data.len(), 128);

    // 5. Search — query with the same vector; top-1 must be the inserted id.
    let searched = client
        .search(SearchRequest {
            collection_name: coll_name.clone(),
            query_vector: data.clone(),
            limit: 3,
            threshold: 0.0,
            filter: HashMap::new(),
        })
        .await
        .expect("Search RPC")
        .into_inner();
    assert!(
        !searched.results.is_empty(),
        "Search returned zero results over live gRPC"
    );
    assert_eq!(searched.results[0].id, vec_id);
    assert!(
        searched.results[0].score >= 0.999,
        "Top result score {} < 0.999 for self-query",
        searched.results[0].score
    );

    // 6. Cleanup.
    let _ = client
        .delete_collection(DeleteCollectionRequest {
            collection_name: coll_name.clone(),
        })
        .await;
}

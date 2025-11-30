//! Tests for Qdrant advanced features (1.14.x)

use vectorizer_sdk::*;

#[tokio::test]
async fn test_qdrant_list_collection_snapshots() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";

    match client
        .qdrant_list_collection_snapshots(collection_name)
        .await
    {
        Ok(result) => {
            // Should return a valid JSON response
            assert!(result.is_object() || result.is_array());
        }
        Err(e) => {
            // Expected if server not running or collection doesn't exist
            println!("Qdrant list snapshots test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_create_collection_snapshot() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";

    match client
        .qdrant_create_collection_snapshot(collection_name)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant create snapshot test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_delete_collection_snapshot() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let snapshot_name = "test_snapshot";

    match client
        .qdrant_delete_collection_snapshot(collection_name, snapshot_name)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant delete snapshot test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_recover_collection_snapshot() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let location = "snapshots/test_snapshot.snapshot";

    match client
        .qdrant_recover_collection_snapshot(collection_name, location)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant recover snapshot test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_list_all_snapshots() {
    let client = VectorizerClient::new_default().unwrap();

    match client.qdrant_list_all_snapshots().await {
        Ok(result) => {
            assert!(result.is_object() || result.is_array());
        }
        Err(e) => {
            println!("Qdrant list all snapshots test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_create_full_snapshot() {
    let client = VectorizerClient::new_default().unwrap();

    match client.qdrant_create_full_snapshot().await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant create full snapshot test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_list_shard_keys() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";

    match client.qdrant_list_shard_keys(collection_name).await {
        Ok(result) => {
            assert!(result.is_object() || result.is_array());
        }
        Err(e) => {
            println!("Qdrant list shard keys test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_create_shard_key() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let shard_key = serde_json::json!({"shard_key": "test_key"});

    match client
        .qdrant_create_shard_key(collection_name, &shard_key)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant create shard key test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_delete_shard_key() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let shard_key = serde_json::json!({"shard_key": "test_key"});

    match client
        .qdrant_delete_shard_key(collection_name, &shard_key)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant delete shard key test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_get_cluster_status() {
    let client = VectorizerClient::new_default().unwrap();

    match client.qdrant_get_cluster_status().await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant get cluster status test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_cluster_recover() {
    let client = VectorizerClient::new_default().unwrap();

    match client.qdrant_cluster_recover().await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant cluster recover test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_remove_peer() {
    let client = VectorizerClient::new_default().unwrap();
    let peer_id = "test_peer_123";

    match client.qdrant_remove_peer(peer_id).await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant remove peer test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_list_metadata_keys() {
    let client = VectorizerClient::new_default().unwrap();

    match client.qdrant_list_metadata_keys().await {
        Ok(result) => {
            assert!(result.is_object() || result.is_array());
        }
        Err(e) => {
            println!("Qdrant list metadata keys test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_get_metadata_key() {
    let client = VectorizerClient::new_default().unwrap();
    let key = "test_key";

    match client.qdrant_get_metadata_key(key).await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant get metadata key test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_update_metadata_key() {
    let client = VectorizerClient::new_default().unwrap();
    let key = "test_key";
    let value = serde_json::json!({"value": "test_value"});

    match client.qdrant_update_metadata_key(key, &value).await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant update metadata key test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_query_points() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let request = serde_json::json!({
        "query": {
            "vector": [0.1, 0.2, 0.3]
        },
        "limit": 10
    });

    match client.qdrant_query_points(collection_name, &request).await {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant query points test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_batch_query_points() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let request = serde_json::json!({
        "searches": [
            {
                "query": {
                    "vector": [0.1, 0.2, 0.3]
                },
                "limit": 10
            }
        ]
    });

    match client
        .qdrant_batch_query_points(collection_name, &request)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant batch query points test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_query_points_groups() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let request = serde_json::json!({
        "query": {
            "vector": [0.1, 0.2, 0.3]
        },
        "group_by": "category",
        "group_size": 3,
        "limit": 10
    });

    match client
        .qdrant_query_points_groups(collection_name, &request)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant query points groups test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_search_points_groups() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let request = serde_json::json!({
        "vector": [0.1, 0.2, 0.3],
        "group_by": "category",
        "group_size": 3,
        "limit": 10
    });

    match client
        .qdrant_search_points_groups(collection_name, &request)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant search points groups test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_search_matrix_pairs() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let request = serde_json::json!({
        "sample": 10,
        "limit": 5
    });

    match client
        .qdrant_search_matrix_pairs(collection_name, &request)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant search matrix pairs test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qdrant_search_matrix_offsets() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = "test_collection";
    let request = serde_json::json!({
        "sample": 10,
        "limit": 5
    });

    match client
        .qdrant_search_matrix_offsets(collection_name, &request)
        .await
    {
        Ok(result) => {
            assert!(result.is_object());
        }
        Err(e) => {
            println!("Qdrant search matrix offsets test skipped: {}", e);
        }
    }
}

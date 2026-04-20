//! Backup REST handlers (list / create / restore / directory).
//!
//! Backups are JSON files written under `./backups/` and contain the
//! serialized vector data for the requested collections. `restore_backup`
//! is admin-gated via
//! [`crate::server::auth_handlers::require_admin_for_rest`]; the other
//! three endpoints are GUI helpers that don't touch credentials.

use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{error, info};

use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_not_found_error, create_validation_error,
};

/// List backups (for GUI)
pub async fn list_backups() -> Json<Value> {
    let backup_dir = std::path::Path::new("./backups");
    let mut backups = Vec::new();

    if backup_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(backup_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("backup") {
                    // Read backup metadata
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(backup_data) = serde_json::from_str::<Value>(&content) {
                            backups.push(backup_data);
                        }
                    }
                }
            }
        }
    }

    // Sort by date (newest first)
    backups.sort_by(|a, b| {
        let a_date = a.get("date").and_then(|d| d.as_str()).unwrap_or("");
        let b_date = b.get("date").and_then(|d| d.as_str()).unwrap_or("");
        b_date.cmp(a_date)
    });

    Json(json!({
        "backups": backups
    }))
}

/// Create backup (for GUI)
pub async fn create_backup(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let name = payload
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| create_validation_error("name", "missing or invalid name parameter"))?;

    let collections = payload
        .get("collections")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    info!(
        "💾 Creating backup '{}' for collections: {:?}",
        name, collections
    );

    // Create backups directory if it doesn't exist
    let backup_dir = std::path::Path::new("./backups");
    if !backup_dir.exists() {
        std::fs::create_dir_all(backup_dir).map_err(|e| {
            create_bad_request_error(&format!("Failed to create backup directory: {}", e))
        })?;
    }

    // Generate backup ID and metadata
    let backup_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Create backup data structure
    let mut backup_data = json!({
        "id": backup_id.clone(),
        "name": name,
        "date": timestamp,
        "collections": collections.clone(),
        "size": 0,
        "data": {}
    });

    let mut total_size = 0u64;
    let mut backup_collections_data = serde_json::Map::new();

    // Backup each collection
    for collection_name in &collections {
        match state.store.get_collection(collection_name) {
            Ok(collection) => {
                // Get all vectors from collection
                let all_vectors = collection.get_all_vectors();

                let vectors: Vec<_> = all_vectors
                    .iter()
                    .map(|vector| {
                        json!({
                            "id": vector.id,
                            "vector": vector.data,
                            "metadata": vector.payload
                        })
                    })
                    .collect();

                let collection_size = std::mem::size_of_val(&vectors) as u64;
                total_size += collection_size;

                let config = collection.config();

                backup_collections_data.insert(
                    collection_name.clone(),
                    json!({
                        "vectors": vectors,
                        "dimension": config.dimension,
                        "metric": format!("{:?}", config.metric)
                    }),
                );

                info!(
                    "✅ Backed up collection '{}': {} vectors",
                    collection_name,
                    vectors.len()
                );
            }
            Err(e) => {
                error!("Failed to backup collection '{}': {}", collection_name, e);
            }
        }
    }

    backup_data["data"] = Value::Object(backup_collections_data);
    backup_data["size"] = json!(total_size);

    // Save backup to file
    let backup_file = backup_dir.join(format!("{}.backup", backup_id));
    let backup_json = serde_json::to_string_pretty(&backup_data).map_err(|e| {
        create_bad_request_error(&format!("Failed to serialize backup data: {}", e))
    })?;

    std::fs::write(&backup_file, backup_json)
        .map_err(|e| create_bad_request_error(&format!("Failed to write backup file: {}", e)))?;

    info!("💾 Backup created successfully: {}", backup_file.display());

    // Return metadata without full data
    Ok(Json(json!({
        "id": backup_id,
        "name": name,
        "date": timestamp,
        "size": total_size,
        "collections": collections
    })))
}

/// Restore backup (for GUI). Admin-only — gate enforced at the router
/// layer in `crate::server::core::routing`.
pub async fn restore_backup(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let backup_id = payload
        .get("backup_id")
        .and_then(|b| b.as_str())
        .ok_or_else(|| {
            create_validation_error("backup_id", "missing or invalid backup_id parameter")
        })?;

    info!("♻️ Restoring backup: {}", backup_id);

    // Load backup file
    let backup_file = std::path::Path::new("./backups").join(format!("{}.backup", backup_id));
    if !backup_file.exists() {
        error!("Backup file not found: {}", backup_file.display());
        return Err(create_not_found_error("backup", backup_id));
    }

    let backup_content = std::fs::read_to_string(&backup_file)
        .map_err(|e| create_bad_request_error(&format!("Failed to read backup file: {}", e)))?;

    let backup_data: Value = serde_json::from_str(&backup_content)
        .map_err(|e| create_bad_request_error(&format!("Failed to parse backup content: {}", e)))?;

    let collections_data = backup_data
        .get("data")
        .and_then(|d| d.as_object())
        .ok_or_else(|| create_bad_request_error("Missing 'data' field in backup content"))?;

    // Restore each collection
    for (collection_name, collection_data) in collections_data {
        let vectors = collection_data
            .get("vectors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                create_bad_request_error(&format!(
                    "Missing 'vectors' field for collection '{}'",
                    collection_name
                ))
            })?;

        let dimension = collection_data
            .get("dimension")
            .and_then(|d| d.as_u64())
            .ok_or_else(|| {
                create_bad_request_error(&format!(
                    "Missing 'dimension' field for collection '{}'",
                    collection_name
                ))
            })? as usize;

        info!(
            "🔄 Restoring collection '{}': {} vectors",
            collection_name,
            vectors.len()
        );

        // Create or get collection
        let collection_exists = state.store.get_collection(collection_name).is_ok();

        if !collection_exists {
            // Create new collection if it doesn't exist
            use vectorizer::models::{
                CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
            };

            let config = CollectionConfig {
                dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::default(),
                compression: CompressionConfig::default(),
                normalization: None,
                storage_type: Some(vectorizer::models::StorageType::Memory),
                sharding: None,
                graph: None,
                encryption: None,
            };

            state
                .store
                .create_collection(collection_name, config)
                .map_err(|e| ErrorResponse::from(e))?;
        }

        // Restore vectors
        let mut vectors_to_insert = Vec::new();

        for vector_data in vectors {
            let id = vector_data
                .get("id")
                .and_then(|i| i.as_str())
                .ok_or_else(|| {
                    create_bad_request_error(&format!("Missing 'id' field in vector data"))
                })?;

            let vector_array = vector_data
                .get("vector")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    create_bad_request_error(&format!("Missing 'vector' field for vector '{}'", id))
                })?;

            let vector: Vec<f32> = vector_array
                .iter()
                .filter_map(|f| f.as_f64())
                .map(|f| f as f32)
                .collect();

            let payload_value = vector_data.get("metadata").cloned();
            let payload = payload_value.map(|v| vectorizer::models::Payload { data: v });

            use vectorizer::models::Vector;
            let vec = Vector {
                id: id.to_string(),
                data: vector,
                sparse: None,
                payload,
                document_id: None,
            };

            vectors_to_insert.push(vec);
        }

        // Insert all vectors at once
        state
            .store
            .insert(collection_name, vectors_to_insert)
            .map_err(|e| ErrorResponse::from(e))?;

        let collection = state
            .store
            .get_collection(collection_name)
            .map_err(|e| ErrorResponse::from(e))?;

        info!(
            "✅ Restored collection '{}': {} vectors",
            collection_name,
            collection.vector_count()
        );
    }

    // Force save all collections
    state
        .store
        .force_save_all()
        .map_err(|e| ErrorResponse::from(e))?;

    info!("♻️ Backup restored successfully");

    Ok(Json(json!({
        "success": true,
        "message": "Backup restored successfully"
    })))
}

/// Get backup directory (for GUI)
pub async fn get_backup_directory() -> Json<Value> {
    Json(json!({
        "path": "./backups"
    }))
}

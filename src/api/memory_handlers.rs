use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use tracing::{error, info};

use crate::api::{
    types::ErrorResponse,
    handlers::AppState,
};

/// Generate comprehensive memory snapshot with real system data
pub async fn generate_memory_snapshot(
    State(mut state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("🔍 Generating comprehensive memory snapshot using GRPC data");
    
    // Use GRPC client to get collections from vzr (correct architecture)
    if let Some(grpc_client) = &mut state.grpc_client {
        info!("🔗 Using GRPC client to get collections from vzr");
        match grpc_client.list_collections().await {
            Ok(response) => {
                info!("✅ Retrieved {} collections from GRPC", response.collections.len());
                
                // Convert GRPC response to REST format for compatibility
                let collections_data = crate::api::types::ListCollectionsResponse {
                    collections: response.collections.into_iter().map(|grpc_collection| {
                        crate::api::types::CollectionInfo {
                            name: grpc_collection.name,
                            dimension: grpc_collection.dimension as usize,
                            metric: crate::api::types::DistanceMetric::Cosine, // Default
                            embedding_provider: "unknown".to_string(),
                            vector_count: grpc_collection.vector_count as usize,
                            document_count: grpc_collection.document_count as usize,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            updated_at: chrono::Utc::now().to_rfc3339(),
                            indexing_status: crate::api::types::IndexingStatus {
                                status: "completed".to_string(),
                                progress: 100.0,
                                total_documents: grpc_collection.document_count as usize,
                                processed_documents: grpc_collection.document_count as usize,
                                vector_count: grpc_collection.vector_count as usize,
                                estimated_time_remaining: None,
                                last_updated: chrono::Utc::now().to_rfc3339(),
                            },
                        }
                    }).collect(),
                };
                
                // Generate the comprehensive snapshot with GRPC data
                match crate::memory_snapshot::generate_memory_snapshot(&collections_data, &state).await {
                    Ok(snapshot) => {
                        info!("✅ Memory snapshot generated successfully with GRPC data");
                        Ok(Json(serde_json::to_value(snapshot).unwrap()))
                    },
                    Err(e) => {
                        error!("Failed to generate memory snapshot: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: format!("Failed to generate memory snapshot: {}", e),
                                code: "SNAPSHOT_GENERATION_ERROR".to_string(),
                                details: None,
                            }),
                        ))
                    }
                }
            },
            Err(e) => {
                error!("Failed to get collections from GRPC: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to get collections from GRPC: {}", e),
                        code: "GRPC_ERROR".to_string(),
                        details: None,
                    }),
                ))
            }
        }
    } else {
        error!("No GRPC client available");
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "No GRPC client available".to_string(),
                code: "NO_GRPC_CLIENT".to_string(),
                details: None,
            }),
        ))
    }
}

/// Export memory snapshot to file
pub async fn export_memory_snapshot(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let file_path = req.get("file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("memory_snapshot.json");
    
    info!("📁 Exporting memory snapshot to: {}", file_path);
    
    // Get collections data first
    let collections_response = crate::api::handlers::list_collections(State(state.clone())).await;
    let collections_data = collections_response.0;
    
    // Generate the snapshot
    match crate::memory_snapshot::generate_memory_snapshot(&collections_data, &state).await {
        Ok(snapshot) => {
            // Export to file
            match crate::memory_snapshot::export_snapshot_to_file(&snapshot, file_path).await {
                Ok(_) => {
                    info!("✅ Memory snapshot exported successfully");
                    Ok(Json(serde_json::json!({
                        "status": "success",
                        "message": format!("Snapshot exported to {}", file_path),
                        "file_path": file_path,
                        "timestamp": snapshot.timestamp
                    })))
                },
                Err(e) => {
                    error!("Failed to export snapshot: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to export snapshot: {}", e),
                            code: "EXPORT_ERROR".to_string(),
                            details: None,
                        }),
                    ))
                }
            }
        },
        Err(e) => {
            error!("Failed to generate snapshot for export: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to generate snapshot: {}", e),
                    code: "SNAPSHOT_GENERATION_ERROR".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

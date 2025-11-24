//! Qdrant alias management handlers

use axum::extract::{Path, State};
use axum::response::Json;
use serde_json::json;
use tracing::{debug, error, info};

use super::VectorizerServer;
use super::error_middleware::ErrorResponse;
use crate::models::qdrant::{
    QdrantAliasDescription, QdrantAliasOperations, QdrantChangeAliasesOperation,
    QdrantCollectionsAliasesResponse,
};
use crate::monitoring::metrics::METRICS;

/// Helper for recording alias metrics
fn record_alias_metric(operation: &str, status: &str) {
    METRICS
        .alias_operations_total
        .with_label_values(&[operation, status])
        .inc();
}

/// List all aliases and their target collections
pub async fn list_aliases(
    State(state): State<VectorizerServer>,
) -> Result<Json<QdrantCollectionsAliasesResponse>, ErrorResponse> {
    debug!("Listing all aliases");

    let aliases = state
        .store
        .list_aliases()
        .into_iter()
        .map(|(alias_name, collection_name)| QdrantAliasDescription {
            alias_name,
            collection_name,
        })
        .collect();

    Ok(Json(QdrantCollectionsAliasesResponse { aliases }))
}

/// List aliases that point to a specific collection
pub async fn list_collection_aliases(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<QdrantCollectionsAliasesResponse>, ErrorResponse> {
    debug!("Listing aliases for collection '{}'", collection_name);

    let collection_ref = state
        .store
        .get_collection(&collection_name)
        .map_err(ErrorResponse::from)?;
    let canonical_name = collection_ref.name().to_string();

    let aliases = state
        .store
        .list_aliases_for_collection(&collection_name)
        .map_err(ErrorResponse::from)?
        .into_iter()
        .map(|alias_name| QdrantAliasDescription {
            alias_name,
            collection_name: canonical_name.clone(),
        })
        .collect();

    Ok(Json(QdrantCollectionsAliasesResponse { aliases }))
}

/// Apply alias operations (create/delete/rename)
pub async fn update_aliases(
    State(state): State<VectorizerServer>,
    Json(payload): Json<QdrantChangeAliasesOperation>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    debug!("Applying {} alias operations", payload.actions.len());

    for action in payload.actions {
        match action {
            QdrantAliasOperations::Create(op) => {
                let alias = op.create_alias.alias_name;
                let target = op.create_alias.collection_name;
                match state.store.create_alias(&alias, &target) {
                    Ok(_) => {
                        record_alias_metric("create", "success");
                        info!("Alias '{}' -> '{}' created", alias, target);
                    }
                    Err(err) => {
                        record_alias_metric("create", "error");
                        error!(
                            "Failed to create alias '{}' for '{}': {}",
                            alias, target, err
                        );
                        return Err(ErrorResponse::from(err));
                    }
                }
            }
            QdrantAliasOperations::Delete(op) => {
                let alias = op.delete_alias.alias_name;
                match state.store.delete_alias(&alias) {
                    Ok(_) => {
                        record_alias_metric("delete", "success");
                        info!("Alias '{}' deleted", alias);
                    }
                    Err(err) => {
                        record_alias_metric("delete", "error");
                        error!("Failed to delete alias '{}': {}", alias, err);
                        return Err(ErrorResponse::from(err));
                    }
                }
            }
            QdrantAliasOperations::Rename(op) => {
                let old_alias = op.rename_alias.old_alias_name;
                let new_alias = op.rename_alias.new_alias_name;
                match state.store.rename_alias(&old_alias, &new_alias) {
                    Ok(_) => {
                        record_alias_metric("rename", "success");
                        info!("Alias '{}' renamed to '{}'", old_alias, new_alias);
                    }
                    Err(err) => {
                        record_alias_metric("rename", "error");
                        error!(
                            "Failed to rename alias '{}' to '{}': {}",
                            old_alias, new_alias, err
                        );
                        return Err(ErrorResponse::from(err));
                    }
                }
            }
        }
    }

    Ok(Json(json!({
        "status": "ok",
        "time": 0.0,
        "result": true
    })))
}

//! File-navigation REST handlers.
//!
//! These endpoints read payloads stored alongside vectors (file_path,
//! chunk_index, file_extension) to offer project-aware browsing on top
//! of the semantic index: listing files in a collection, summarising a
//! file, walking its chunks in order, building a project outline,
//! finding related files, and filtering search by file type.
//!
//! All handlers delegate to [`crate::file_operations::FileOperations`];
//! the handlers here only parse JSON, build the filter/config, and
//! marshal the response.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]
// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::error;

use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_validation_error,
};

pub async fn get_file_content(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::FileOperations;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let file_path = payload
        .get("file_path")
        .and_then(|f| f.as_str())
        .ok_or_else(|| {
            create_validation_error("file_path", "missing or invalid file_path parameter")
        })?;

    let max_size_kb = payload
        .get("max_size_kb")
        .and_then(|m| m.as_u64())
        .unwrap_or(500) as usize;

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops
        .get_file_content(collection, file_path, max_size_kb)
        .await
    {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get file content error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn list_files_in_collection(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::{FileListFilter, FileOperations, SortBy};

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let filter_by_type = payload
        .get("filter_by_type")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    let min_chunks = payload
        .get("min_chunks")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    let max_results = payload
        .get("max_results")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    let sort_by = payload
        .get("sort_by")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "name" => Some(SortBy::Name),
            "size" => Some(SortBy::Size),
            "chunks" => Some(SortBy::Chunks),
            "recent" => Some(SortBy::Recent),
            _ => None,
        })
        .unwrap_or(SortBy::Name);

    let filter = FileListFilter {
        filter_by_type,
        min_chunks,
        max_results,
        sort_by,
    };

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops.list_files_in_collection(collection, filter).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("List files error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn get_file_summary(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::{FileOperations, SummaryType};

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let file_path = payload
        .get("file_path")
        .and_then(|f| f.as_str())
        .ok_or_else(|| {
            create_validation_error("file_path", "missing or invalid file_path parameter")
        })?;

    let summary_type = payload
        .get("summary_type")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "extractive" => Some(SummaryType::Extractive),
            "structural" => Some(SummaryType::Structural),
            "both" => Some(SummaryType::Both),
            _ => None,
        })
        .unwrap_or(SummaryType::Both);

    let max_sentences = payload
        .get("max_sentences")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops
        .get_file_summary(collection, file_path, summary_type, max_sentences)
        .await
    {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get file summary error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn get_file_chunks_ordered(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::FileOperations;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let file_path = payload
        .get("file_path")
        .and_then(|f| f.as_str())
        .ok_or_else(|| {
            create_validation_error("file_path", "missing or invalid file_path parameter")
        })?;

    let start_chunk = payload
        .get("start_chunk")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let limit = payload.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let include_context = payload
        .get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops
        .get_file_chunks_ordered(collection, file_path, start_chunk, limit, include_context)
        .await
    {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get file chunks error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn get_project_outline(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::FileOperations;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let max_depth = payload
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;

    let include_summaries = payload
        .get("include_summaries")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let highlight_key_files = payload
        .get("highlight_key_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops
        .get_project_outline(
            collection,
            max_depth,
            include_summaries,
            highlight_key_files,
        )
        .await
    {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get project outline error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn get_related_files(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::FileOperations;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let file_path = payload
        .get("file_path")
        .and_then(|f| f.as_str())
        .ok_or_else(|| {
            create_validation_error("file_path", "missing or invalid file_path parameter")
        })?;

    let limit = payload.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

    let similarity_threshold = payload
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.6) as f32;

    let include_reason = payload
        .get("include_reason")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops
        .get_related_files(
            collection,
            file_path,
            limit,
            similarity_threshold,
            include_reason,
            &state.embedding_manager,
        )
        .await
    {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get related files error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn search_by_file_type(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::file_operations::FileOperations;

    let collection = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let file_types = payload
        .get("file_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| {
            create_validation_error("file_types", "missing or invalid file_types parameter")
        })?;

    let limit = payload.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let return_full_files = payload
        .get("return_full_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let file_ops = FileOperations::with_store(state.store.clone());

    match file_ops
        .search_by_file_type(
            collection,
            query,
            file_types,
            limit,
            return_full_files,
            &state.embedding_manager,
        )
        .await
    {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Search by file type error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

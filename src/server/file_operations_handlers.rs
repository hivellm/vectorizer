use std::sync::Arc;
use rmcp::model::{CallToolRequestParam, CallToolResult, Content, ErrorData};
use serde_json::json;
use crate::{VectorStore, embedding::EmbeddingManager};
use crate::file_operations::FileOperations;

pub async fn handle_get_file_chunks_ordered(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let file_path = args.get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing file_path", None))?;
    
    let start_chunk = args.get("start_chunk")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    
    let limit = args.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let include_context = args.get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Call actual implementation
    let file_ops = FileOperations::with_store(store);
    let result = file_ops.get_file_chunks_ordered(
        collection,
        file_path,
        start_chunk,
        limit,
        include_context
    ).await
    .map_err(|e| ErrorData::internal_error(format!("Failed to get file chunks: {}", e), None))?;
    
    let response = json!(result);
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

pub async fn handle_get_project_outline(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let max_depth = args.get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    let include_summaries = args.get("include_summaries")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let highlight_key_files = args.get("highlight_key_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    // Call actual implementation
    let file_ops = FileOperations::with_store(store);
    let result = file_ops.get_project_outline(
        collection,
        max_depth,
        include_summaries,
        highlight_key_files
    ).await
    .map_err(|e| ErrorData::internal_error(format!("Failed to get project outline: {}", e), None))?;
    
    let response = json!(result);
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

pub async fn handle_get_related_files(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let file_path = args.get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing file_path", None))?;
    
    let limit = args.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    let similarity_threshold = args.get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.6) as f32;
    
    let include_reason = args.get("include_reason")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    // Call actual implementation
    let file_ops = FileOperations::with_store(store);
    let result = file_ops.get_related_files(
        collection,
        file_path,
        limit,
        similarity_threshold,
        include_reason,
        &embedding_manager
    ).await
    .map_err(|e| ErrorData::internal_error(format!("Failed to get related files: {}", e), None))?;
    
    let response = json!(result);
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

pub async fn handle_search_by_file_type(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;
    
    let file_types = args.get("file_types")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>())
        .ok_or_else(|| ErrorData::invalid_params("Missing or invalid file_types", None))?;
    
    let limit = args.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let return_full_files = args.get("return_full_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Call actual implementation
    let file_ops = FileOperations::with_store(store);
    let result = file_ops.search_by_file_type(
        collection,
        query,
        file_types,
        limit,
        return_full_files,
        &embedding_manager
    ).await
    .map_err(|e| ErrorData::internal_error(format!("Failed to search by file type: {}", e), None))?;
    
    let response = json!(result);
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

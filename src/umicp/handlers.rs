//! UMICP Handlers - Wrapper for MCP tools
//! 
//! Converts UMICP Envelopes to MCP CallToolRequest and back
//! Updated for UMICP v0.2.1 - Native JSON types support

use umicp_core::{Envelope, OperationType};
use rmcp::model::{CallToolRequestParam, Content};
use tracing::{debug, error};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::Result;
use super::UmicpState;

/// Main UMICP request handler - routes to MCP handlers
pub async fn handle_umicp_request(
    state: UmicpState,
    envelope: Envelope,
) -> Result<Envelope> {
    debug!("📦 Processing UMICP envelope: {:?}", envelope.operation());
    
    // Extract capabilities
    let caps = envelope.capabilities()
        .ok_or_else(|| crate::error::VectorizerError::Other("Missing capabilities".to_string()))?;
    
    // Get operation name (now a Value, convert to string)
    let operation = caps.get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::VectorizerError::Other("Missing 'operation' in capabilities".to_string()))?;
    
    debug!("Operation: {}", operation);
    
    // Convert UMICP capabilities to MCP CallToolRequest
    let mcp_request = capabilities_to_mcp_request(operation, caps)?;
    
    // Call existing MCP handler
    let mcp_result = crate::server::mcp_handlers::handle_mcp_tool(
        mcp_request,
        state.store.clone(),
        state.embedding_manager.clone(),
    ).await;
    
    // Convert MCP result back to UMICP Envelope
    let response = match mcp_result {
        Ok(result) => {
            let result_json = content_to_json(&result.content);
            create_success_response(envelope, result_json)
        },
        Err(err) => {
            error!("MCP handler error: {:?}", err);
            create_error_response(envelope, &err.message)
        }
    }?;
    
    Ok(response)
}

/// Convert UMICP capabilities to MCP CallToolRequest
/// Now with native JSON types support (v0.2.1)
fn capabilities_to_mcp_request(
    tool_name: &str,
    caps: &HashMap<String, Value>,
) -> Result<CallToolRequestParam> {
    // Build arguments JSON from capabilities
    let mut args = serde_json::Map::new();
    
    for (key, value) in caps.iter() {
        if key == "operation" {
            continue; // Skip the operation field
        }
        
        // Direct use of Value - no parsing needed!
        args.insert(key.clone(), value.clone());
    }
    
    Ok(CallToolRequestParam {
        name: tool_name.to_string().into(),
        arguments: Some(args),
    })
}

/// Convert MCP Content to JSON
fn content_to_json(content: &[Content]) -> serde_json::Value {
    if content.is_empty() {
        return json!({"result": "ok"});
    }
    
    // Serialize content to JSON string and parse back
    // This is simpler than pattern matching on the enum variants
    let content_json = serde_json::to_string(content)
        .unwrap_or_else(|_| "[]".to_string());
    
    serde_json::from_str(&content_json)
        .unwrap_or_else(|_| json!({"content": content_json}))
}

/// Create success response envelope
/// Now with native JSON types (v0.2.1)
fn create_success_response(
    request: Envelope,
    result: Value,
) -> Result<Envelope> {
    let mut response_caps = HashMap::new();
    response_caps.insert("status".to_string(), json!("success"));
    response_caps.insert("result".to_string(), result); // Direct JSON value!
    response_caps.insert("original_message_id".to_string(), json!(request.message_id()));
    
    let response = Envelope::builder()
        .from(request.to())
        .to(request.from())
        .operation(OperationType::Data)
        .message_id(&format!("resp-{}", request.message_id()))
        .capabilities(response_caps)
        .build()?;
    
    Ok(response)
}

/// Create error response envelope
/// Now with native JSON types (v0.2.1)
fn create_error_response(
    request: Envelope,
    error_message: &str,
) -> Result<Envelope> {
    let mut response_caps = HashMap::new();
    response_caps.insert("status".to_string(), json!("error"));
    response_caps.insert("error".to_string(), json!(error_message));
    response_caps.insert("original_message_id".to_string(), json!(request.message_id()));
    
    let response = Envelope::builder()
        .from(request.to())
        .to(request.from())
        .operation(OperationType::Control)
        .message_id(&format!("err-{}", request.message_id()))
        .capabilities(response_caps)
        .build()?;
    
    Ok(response)
}

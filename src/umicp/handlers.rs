//! UMICP Handlers - Wrapper for MCP tools
//! 
//! Converts UMICP Envelopes to MCP CallToolRequest and back

use umicp_core::{Envelope, OperationType, Capabilities};
use rmcp::model::{CallToolRequestParam, Content};
use tracing::{debug, error};
use serde_json::json;

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
    
    // Get operation name
    let operation = caps.get("operation")
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
fn capabilities_to_mcp_request(
    tool_name: &str,
    caps: &std::collections::HashMap<String, String>,
) -> Result<CallToolRequestParam> {
    // Build arguments JSON from capabilities
    let mut args = serde_json::Map::new();
    
    for (key, value) in caps.iter() {
        if key == "operation" {
            continue; // Skip the operation field
        }
        
        // Try to parse as JSON, fallback to string
        let json_value = serde_json::from_str(value)
            .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
        
        args.insert(key.clone(), json_value);
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
fn create_success_response(
    request: Envelope,
    result: serde_json::Value,
) -> Result<Envelope> {
    let mut response_caps = Capabilities::new();
    response_caps.insert("status".to_string(), "success".to_string());
    response_caps.insert("result".to_string(), serde_json::to_string(&result)?);
    response_caps.insert("original_message_id".to_string(), request.message_id().to_string());
    
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
fn create_error_response(
    request: Envelope,
    error_message: &str,
) -> Result<Envelope> {
    let mut response_caps = Capabilities::new();
    response_caps.insert("status".to_string(), "error".to_string());
    response_caps.insert("error".to_string(), error_message.to_string());
    response_caps.insert("original_message_id".to_string(), request.message_id().to_string());
    
    let response = Envelope::builder()
        .from(request.to())
        .to(request.from())
        .operation(OperationType::Control)
        .message_id(&format!("err-{}", request.message_id()))
        .capabilities(response_caps)
        .build()?;
    
    Ok(response)
}

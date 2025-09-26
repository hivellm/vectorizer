use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Serialize, Deserialize, Debug)]
struct MCPRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct MCPResponse {
    jsonrpc: String,
    id: u64,
    result: Option<Value>,
    error: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MCPNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("VECTORIZER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("VECTORIZER_PORT").unwrap_or_else(|_| "15003".to_string());
    let url = format!("ws://{}:{}/mcp", host, port);

    println!("🔌 Conectando ao servidor MCP em {}", url);

    let (ws_stream, _) = connect_async(url).await?;
    println!("✅ Conectado ao servidor MCP");

    let (mut write, mut read) = ws_stream.split();

    // Initialize MCP
    let init_request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "initialize".to_string(),
        params: serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "Vectorizer MCP Client",
                "version": "1.0.0"
            }
        }),
    };

    let init_msg = serde_json::to_string(&init_request)?;
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            init_msg.into(),
        ))
        .await?;

    // Handle initialization response
    if let Some(message) = read.next().await {
        let message = message?;
        if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
            let response: MCPResponse = serde_json::from_str(&text)?;
            if let Some(result) = response.result {
                println!("🚀 MCP inicializado com sucesso");
                if let Some(server_info) = result.get("serverInfo") {
                    println!(
                        "Server Info: {}",
                        serde_json::to_string_pretty(server_info)?
                    );
                }
            }
        }
    }

    // Test basic operations
    println!("\n🧪 Iniciando testes MCP...");

    // Test 1: List collections
    println!("\n📁 Teste 1: Listando coleções...");
    let collections_request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: 2,
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "list_collections",
            "arguments": {}
        }),
    };

    let collections_msg = serde_json::to_string(&collections_request)?;
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            collections_msg.into(),
        ))
        .await?;

    // Handle response
    if let Some(message) = read.next().await {
        let message = message?;
        if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
            let response: MCPResponse = serde_json::from_str(&text)?;
            if let Some(result) = response.result {
                println!("✅ Coleções encontradas:");
                if let Some(collections) = result.get("collections") {
                    println!("{}", serde_json::to_string_pretty(collections)?);
                }
            }
        }
    }

    // Test 2: Search vectors
    println!("\n🔍 Teste 2: Busca semântica...");
    let search_request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: 3,
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "search_vectors",
            "arguments": {
                "collection": "documents",
                "query": "governance blockchain",
                "limit": 3
            }
        }),
    };

    let search_msg = serde_json::to_string(&search_request)?;
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            search_msg.into(),
        ))
        .await?;

    // Handle response
    if let Some(message) = read.next().await {
        let message = message?;
        if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
            let response: MCPResponse = serde_json::from_str(&text)?;
            if let Some(result) = response.result {
                println!("✅ Resultados da busca:");
                if let Some(results) = result.get("results") {
                    println!("{}", serde_json::to_string_pretty(results)?);
                }
            }
        }
    }

    // Test 3: Get database stats
    println!("\n📊 Teste 3: Estatísticas do banco...");
    let stats_request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: 4,
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "get_database_stats",
            "arguments": {}
        }),
    };

    let stats_msg = serde_json::to_string(&stats_request)?;
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            stats_msg.into(),
        ))
        .await?;

    // Handle response
    if let Some(message) = read.next().await {
        let message = message?;
        if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
            let response: MCPResponse = serde_json::from_str(&text)?;
            if let Some(result) = response.result {
                println!("✅ Estatísticas do banco:");
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
    }

    // Test 4: Embed text
    println!("\n📝 Teste 4: Geração de embedding...");
    let embed_request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: 5,
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "embed_text",
            "arguments": {
                "text": "artificial intelligence governance"
            }
        }),
    };

    let embed_msg = serde_json::to_string(&embed_request)?;
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            embed_msg.into(),
        ))
        .await?;

    // Handle response
    if let Some(message) = read.next().await {
        let message = message?;
        if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
            let response: MCPResponse = serde_json::from_str(&text)?;
            if let Some(result) = response.result {
                println!("✅ Embedding gerado:");
                println!(
                    "Dimensão: {}",
                    result.get("dimension").unwrap_or(&Value::Null)
                );
                println!(
                    "Provedor: {}",
                    result.get("provider").unwrap_or(&Value::Null)
                );
                if let Some(embedding) = result.get("embedding") {
                    if let Some(arr) = embedding.as_array() {
                        println!(
                            "Vetor (primeiros 5 valores): [{}, {}, {}, {}, {}]",
                            arr.get(0).unwrap_or(&Value::Null),
                            arr.get(1).unwrap_or(&Value::Null),
                            arr.get(2).unwrap_or(&Value::Null),
                            arr.get(3).unwrap_or(&Value::Null),
                            arr.get(4).unwrap_or(&Value::Null)
                        );
                    }
                }
            }
        }
    }

    println!("\n🎉 Todos os testes MCP concluídos com sucesso!");

    Ok(())
}

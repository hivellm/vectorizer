use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vectorizer::mcp_service::VectorizerService;
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use vectorizer::grpc::client::VectorizerGrpcClient;
use vectorizer::config::GrpcConfig;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use axum::Router;
use std::net::SocketAddr;


fn get_bind_address() -> String {
    let port = std::env::var("VECTORIZER_SERVER_PORT").unwrap_or_else(|_| "15002".to_string());
    format!("127.0.0.1:{}", port)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to file (unique per port to avoid conflicts)
    let port = std::env::var("VECTORIZER_SERVER_PORT").unwrap_or_else(|_| "15002".to_string());
    let log_filename = format!("vectorizer-mcp-server-{}.log", port);
    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_filename)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open log file {}: {}", log_filename, e);
            std::process::exit(1);
        }
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(move || log_file.try_clone().expect("Failed to clone log file")),
        )
        .init();

    // MCP server now acts as a GRPC client to the vzr GRPC server
    let grpc_server_url = std::env::var("VECTORIZER_GRPC_URL").unwrap_or_else(|_| "http://127.0.0.1:15003".to_string());
    
    tracing::info!("🚀 MCP server started - acting as GRPC client");
    tracing::info!("📡 Connecting to GRPC server at: {}", grpc_server_url);
    
    // Test connection to GRPC server
    let grpc_config = GrpcConfig::from_env();
    match VectorizerGrpcClient::new(grpc_config.client).await {
        Ok(mut client) => {
            match client.health_check().await {
                Ok(health) => {
                    tracing::info!("✅ Successfully connected to GRPC server: {}", health.status);
                }
                Err(e) => {
                    tracing::warn!("⚠️ GRPC server health check failed: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to connect to GRPC server: {}", e);
            tracing::info!("💡 Make sure the vzr GRPC server is running on {}", grpc_server_url);
        }
    }

    let bind_address = get_bind_address();
    let addr: SocketAddr = bind_address.parse()?;

    tracing::info!("🚀 Starting MCP server on {}", bind_address);
    tracing::info!("📡 Using GRPC server at: {}", grpc_server_url);
    tracing::info!("🔗 Connect your MCP client to: http://{}/sse", bind_address);

    // Create the VectorizerService that will make GRPC calls to vzr
    let service = VectorizerService::new(grpc_server_url.clone());

    // Create SSE server config
    let ct = CancellationToken::new();
    let config = SseServerConfig {
        bind: bind_address.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: ct.clone(),
        sse_keep_alive: Some(Duration::from_secs(30)),
    };

    // Create the SSE server and get the router
    let (sse_server, router) = SseServer::new(config);

    // Configure the service and get the axum router
    let _ct = sse_server.with_service(move || service.clone());

    // Use the router from rmcp with axum
    let app = router;

    // Start the axum server
    let server = axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app.into_make_service(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.unwrap();
        tracing::info!("Received shutdown signal");
    });

    tracing::info!("✅ MCP server ready on {}", bind_address);

    server.await?;

    Ok(())
}
use clap::Parser;
use std::sync::Arc;
use vectorizer::mcp_service::VectorizerService;
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use vectorizer::grpc::client::VectorizerGrpcClient;
use vectorizer::config::{GrpcConfig, VectorizerConfig};
use vectorizer::process_manager::{initialize_process_management, cleanup_process_management};
use vectorizer::logging;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use axum::Router;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "vectorizer-mcp-server")]
#[command(about = "Vectorizer MCP Server for IDE integration")]
struct Args {
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[arg(long, default_value = "15002")]
    port: u16,

    /// GRPC server URL to connect to
    #[arg(long, default_value = "http://127.0.0.1:15003")]
    grpc_url: String,

    /// Configuration file path
    #[arg(long, default_value = "config.yml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration from file
    let config = VectorizerConfig::from_yaml_file(&std::path::PathBuf::from(&args.config)).unwrap_or_else(|e| {
        VectorizerConfig::default()
    });

    // Initialize process management first
    let ports = vec![args.port];
    // Temporarily disabled to fix server startup issues
    // let _process_logger = initialize_process_management("vectorizer-mcp-server", &ports)
    //     .map_err(|e| anyhow::anyhow!("Process management initialization failed: {}", e))?;

    // Initialize centralized logging
    if let Err(e) = logging::init_logging("vectorizer-mcp-server") {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    // MCP server now acts as a GRPC client to the vzr GRPC server
    let grpc_server_url = &args.grpc_url;
    
    tracing::info!("🚀 MCP server started - acting as GRPC client");
    tracing::info!("📡 Connecting to GRPC server at: {}", grpc_server_url);
    
    // Test connection to GRPC server using config
    let grpc_config = GrpcConfig::default(); // Use default GRPC config for now
    match VectorizerGrpcClient::new(grpc_config.client.clone()).await {
        Ok(mut client) => {
            match client.health_check().await {
                Ok(health) => {
                    tracing::info!("✅ Successfully connected to GRPC server: {}", health.status);
                }
                Err(e) => {
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to connect to GRPC server: {}", e);
            tracing::info!("💡 Make sure the vzr GRPC server is running on {}", grpc_server_url);
        }
    }

    let bind_address = format!("{}:{}", args.host, args.port);
    let addr: SocketAddr = bind_address.parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse bind address: {}", e))?;

    tracing::info!("🚀 Starting MCP server on {}", bind_address);
    tracing::info!("📡 Using GRPC server at: {}", grpc_server_url);
    tracing::info!("🔗 Connect your MCP client to: http://{}/sse", bind_address);

    // Create the VectorizerService that will make GRPC calls to vzr
    let service = VectorizerService::new(grpc_server_url.clone());

    // Create SSE server config
    let ct = CancellationToken::new();
    let sse_config = SseServerConfig {
        bind: bind_address.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: ct.clone(),
        sse_keep_alive: Some(Duration::from_secs(30)),
    };

    // Create the SSE server and get the router
    let (sse_server, router) = SseServer::new(sse_config);

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

    // Setup cleanup on exit
    let cleanup_guard = scopeguard::guard((), |_| {
        cleanup_process_management("vectorizer-mcp-server");
    });
    
    let result = server.await;
    
    // Cleanup will be called automatically when cleanup_guard goes out of scope
    drop(cleanup_guard);

    result.map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
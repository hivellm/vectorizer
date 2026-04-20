//! HTTP router composition + shutdown orchestration.
//!
//! This file is all of `VectorizerServer::start` (route registration,
//! middleware layering, listener, graceful shutdown) plus
//! [`create_mcp_router`]. It's the long runtime payload of the server;
//! bootstrap decisions live in [`super::bootstrap`].

use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, post, put};
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

use super::helpers::{
    extract_auth_credentials, get_file_watcher_metrics, security_headers_middleware,
};
use super::mcp_service::VectorizerMcpService;
use crate::server::{
    ServerState, VectorizerServer, auth_handlers, embedded_assets, files, graphql_handlers,
    hub_handlers, qdrant, replication_handlers, rest_handlers, setup_handlers,
};

impl VectorizerServer {
    /// Start the server.
    ///
    /// Function-scoped allow: the unwraps below cover (a) static
    /// `Response::builder().body(...).unwrap()` calls where the body is a
    /// literal `&'static str` so the builder cannot fail, (b)
    /// `user_claims.unwrap()` on an `Option` whose `is_none()` branch was
    /// just early-returned, and (c) Ctrl-C / SIGTERM `expect("...")` calls
    /// at startup where a panic on signal-handler init is the correct
    /// failure mode. See phase4_enforce-no-unwrap-policy.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    pub async fn start(&self, host: &str, port: u16) -> anyhow::Result<()> {
        info!("🚀 Starting Vectorizer Server on {}:{}", host, port);

        // SECURITY CHECK: When binding to 0.0.0.0 (production), require authentication
        // Either standard auth or HiveHub integration must be enabled
        let is_production_bind = host == "0.0.0.0";
        if is_production_bind {
            let has_auth = self.auth_handler_state.is_some();
            let has_hub = self.hub_manager.is_some();

            if !has_auth && !has_hub {
                error!("❌ SECURITY ERROR: Cannot bind to 0.0.0.0 without authentication enabled!");
                error!(
                    "   When exposing the server to all network interfaces, authentication is required."
                );
                error!("   Please enable authentication in config.yml:");
                error!("   auth:");
                error!("     enabled: true");
                error!("     jwt_secret: \"your-secure-secret-key\"");
                error!("");
                error!("   Or enable HiveHub integration:");
                error!("   hub:");
                error!("     enabled: true");
                error!("");
                error!("   Or use --host 127.0.0.1 for local development only.");
                return Err(anyhow::anyhow!(
                    "Security: Authentication required when binding to 0.0.0.0"
                ));
            }

            if has_hub {
                info!("🌐 HiveHub integration enabled - accepting internal service requests");
            }
            if has_auth {
                warn!(
                    "🔐 Production mode detected (0.0.0.0) - Authentication is REQUIRED for all API requests"
                );
            }
        }

        // Start gRPC server in background
        let grpc_port = port + 1; // gRPC on next port
        let grpc_host = host.to_string();
        let grpc_store = self.store.clone();
        let grpc_cluster_manager = self.cluster_manager.clone();
        let grpc_snapshot_manager = self.snapshot_manager.clone();
        let grpc_raft_manager = self.raft_manager.clone();
        let grpc_handle = tokio::spawn(async move {
            if let Err(e) = Self::start_grpc_server(
                &grpc_host,
                grpc_port,
                grpc_store,
                grpc_cluster_manager,
                grpc_snapshot_manager,
                grpc_raft_manager,
            )
            .await
            {
                error!("❌ gRPC server failed: {}", e);
            }
        });
        // Store gRPC handle for shutdown
        *self.grpc_task.lock().await = Some(grpc_handle);
        info!("✅ gRPC server task spawned");

        // Create server state for metrics endpoint
        let server_state = ServerState {
            file_watcher_system: self.file_watcher_system.clone(),
        };

        // Create MCP router (main server) using StreamableHTTP transport
        info!("🔧 Creating MCP router with StreamableHTTP transport (rmcp 0.8.1)...");
        let mcp_router = self
            .create_mcp_router(is_production_bind, self.auth_handler_state.clone())
            .await;
        info!("✅ MCP router created (StreamableHTTP)");

        // Create REST API router to add to MCP
        let metrics_collector_1 = self.metrics_collector.clone();
        let metrics_router = Router::new()
            .route("/metrics", get(get_file_watcher_metrics))
            .with_state(Arc::new(server_state))
            .layer(axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let metrics = metrics_collector_1.clone();
                    async move {
                        // Record connection opened
                        metrics.record_connection_opened();

                        // Record API request
                        let start = std::time::Instant::now();
                        let response = next.run(req).await;
                        let duration = start.elapsed().as_millis() as f64;

                        // Record API request metrics
                        let is_success = response.status().is_success();
                        metrics.record_api_request(is_success, duration);

                        // Record connection closed
                        metrics.record_connection_closed();

                        response
                    }
                },
            ));

        // Public routes that don't require authentication (even in production)
        let public_routes = Router::new()
            .route("/health", get(rest_handlers::health_check))
            .route(
                "/prometheus/metrics",
                get(rest_handlers::get_prometheus_metrics),
            )
            .with_state(self.clone());

        let metrics_collector_2 = self.metrics_collector.clone();
        let rest_routes = Router::new()
            // Stats and monitoring (may require auth in production)
            .route("/stats", get(rest_handlers::get_stats))
            .route(
                "/indexing/progress",
                get(rest_handlers::get_indexing_progress),
            )
            // GUI-specific endpoints
            .route("/status", get(rest_handlers::get_status))
            .route("/logs", get(rest_handlers::get_logs))
            .route(
                "/collections/{name}/force-save",
                post(rest_handlers::force_save_collection),
            )
            // The 9 admin-only POST routes that used to live here (workspace
            // add/remove/config, setup apply/browse, config update, admin
            // restart, backups create/restore) are now registered on a
            // dedicated `admin_router` further down with router-level
            // `require_admin_middleware`. Authenticated read-only views of
            // the same surface stay here.
            .route("/workspace/list", get(rest_handlers::list_workspaces))
            .route(
                "/workspace/config",
                get(rest_handlers::get_workspace_config),
            )
            .route("/setup/status", get(setup_handlers::get_setup_status))
            .route(
                "/setup/analyze",
                post(setup_handlers::analyze_project_directory),
            )
            .route("/setup/verify", get(setup_handlers::verify_setup))
            .route(
                "/setup/templates",
                get(setup_handlers::get_configuration_templates),
            )
            .route(
                "/setup/templates/{id}",
                get(setup_handlers::get_configuration_template_by_id),
            )
            .route("/config", get(rest_handlers::get_config))
            .route("/backups", get(rest_handlers::list_backups))
            .route(
                "/backups/directory",
                get(rest_handlers::get_backup_directory),
            )
            // HiveHub user-scoped backup routes
            .route("/hub/backups", get(hub_handlers::backup::list_user_backups))
            .route(
                "/hub/backups",
                post(hub_handlers::backup::create_user_backup),
            )
            .route(
                "/hub/backups/restore",
                post(hub_handlers::backup::restore_user_backup),
            )
            .route(
                "/hub/backups/upload",
                post(hub_handlers::backup::upload_user_backup),
            )
            .route(
                "/hub/backups/{backup_id}",
                get(hub_handlers::backup::get_user_backup),
            )
            .route(
                "/hub/backups/{backup_id}",
                delete(hub_handlers::backup::delete_user_backup),
            )
            .route(
                "/hub/backups/{backup_id}/download",
                get(hub_handlers::backup::download_user_backup),
            )
            // HiveHub usage statistics routes
            .route(
                "/hub/usage/statistics",
                get(hub_handlers::usage::get_usage_statistics),
            )
            .route("/hub/usage/quota", get(hub_handlers::usage::get_quota_info))
            // HiveHub tenant management routes
            // Tenant endpoints stay unwired per the disabled flag in
            // `src/server/hub_handlers/mod.rs` (axum+tonic version clash).
            // .route("/api/hub/tenant/cleanup", ...)
            // .route("/api/hub/tenant/{tenant_id}/stats", ...)
            // .route("/api/hub/tenant/{tenant_id}/migrate", ...)
            // HiveHub API key validation
            .route(
                "/hub/validate-key",
                post(hub_handlers::usage::validate_api_key),
            )
            // Collection management
            .route("/collections", get(rest_handlers::list_collections))
            .route("/collections", post(rest_handlers::create_collection))
            .route("/collections/{name}", get(rest_handlers::get_collection))
            .route(
                "/collections/{name}",
                delete(rest_handlers::delete_collection),
            )
            // Collection cleanup (file watcher bug fix)
            .route(
                "/collections/empty",
                get(rest_handlers::list_empty_collections),
            )
            .route(
                "/collections/cleanup",
                delete(rest_handlers::cleanup_empty_collections),
            )
            // Vector operations - single
            .route("/search", post(rest_handlers::search_vectors))
            .route(
                "/collections/{name}/search",
                post(rest_handlers::search_vectors),
            )
            .route(
                "/collections/{name}/search/text",
                post(rest_handlers::search_vectors_by_text),
            )
            .route(
                "/collections/{name}/search/file",
                post(rest_handlers::search_by_file),
            )
            .route(
                "/collections/{name}/hybrid_search",
                post(rest_handlers::hybrid_search_vectors),
            )
            .route("/insert", post(rest_handlers::insert_text))
            .route("/update", post(rest_handlers::update_vector))
            .route("/delete", post(rest_handlers::delete_vector))
            .route("/embed", post(rest_handlers::embed_text))
            .route("/vector", post(rest_handlers::get_vector))
            .route(
                "/collections/{name}/vectors",
                get(rest_handlers::list_vectors),
            )
            .route(
                "/collections/{name}/vectors/{id}",
                get(rest_handlers::get_vector),
            )
            .route(
                "/collections/{name}/vectors/{id}",
                delete(rest_handlers::delete_vector),
            )
            // Vector operations - batch
            .route("/batch_insert", post(rest_handlers::batch_insert_texts))
            .route("/insert_texts", post(rest_handlers::insert_texts))
            .route("/batch_search", post(rest_handlers::batch_search_vectors))
            .route("/batch_update", post(rest_handlers::batch_update_vectors))
            .route("/batch_delete", post(rest_handlers::batch_delete_vectors))
            // Intelligent search routes
            .route(
                "/intelligent_search",
                post(rest_handlers::intelligent_search),
            )
            .route(
                "/multi_collection_search",
                post(rest_handlers::multi_collection_search),
            )
            .route("/semantic_search", post(rest_handlers::semantic_search))
            .route("/contextual_search", post(rest_handlers::contextual_search))
            // Discovery routes
            .route("/discover", post(rest_handlers::discover))
            .route(
                "/discovery/filter_collections",
                post(rest_handlers::filter_collections),
            )
            .route(
                "/discovery/score_collections",
                post(rest_handlers::score_collections),
            )
            .route(
                "/discovery/expand_queries",
                post(rest_handlers::expand_queries),
            )
            .route(
                "/discovery/broad_discovery",
                post(rest_handlers::broad_discovery),
            )
            .route(
                "/discovery/semantic_focus",
                post(rest_handlers::semantic_focus),
            )
            // Cluster management routes are conditionally merged below when cluster is enabled.
            .route(
                "/discovery/promote_readme",
                post(rest_handlers::promote_readme),
            )
            .route(
                "/discovery/compress_evidence",
                post(rest_handlers::compress_evidence),
            )
            .route(
                "/discovery/build_answer_plan",
                post(rest_handlers::build_answer_plan),
            )
            .route(
                "/discovery/render_llm_prompt",
                post(rest_handlers::render_llm_prompt),
            )
            // File Operations routes
            .route("/file/content", post(rest_handlers::get_file_content))
            .route("/file/list", post(rest_handlers::list_files_in_collection))
            .route("/file/summary", post(rest_handlers::get_file_summary))
            .route("/file/chunks", post(rest_handlers::get_file_chunks_ordered))
            .route("/file/outline", post(rest_handlers::get_project_outline))
            .route("/file/related", post(rest_handlers::get_related_files))
            .route(
                "/file/search_by_type",
                post(rest_handlers::search_by_file_type),
            )
            // File Upload routes
            // Note: Axum has a default 2MB limit for multipart. This is increased via
            // DefaultBodyLimit layer (configured via max_request_size_mb in config.yml).
            .route("/files/upload", post(files::upload::upload_file))
            .route("/files/config", get(files::upload::get_upload_config))
            // Replication routes
            .route(
                "/replication/status",
                get(replication_handlers::get_replication_status),
            )
            .route(
                "/replication/configure",
                post(replication_handlers::configure_replication),
            )
            .route(
                "/replication/stats",
                get(replication_handlers::get_replication_stats),
            )
            .route(
                "/replication/replicas",
                get(replication_handlers::list_replicas),
            )
            // Qdrant-compatible routes (under /qdrant prefix)
            .route(
                "/qdrant/collections",
                get(qdrant::handlers::get_collections),
            )
            .route(
                "/qdrant/collections/{name}",
                get(qdrant::handlers::get_collection),
            )
            .route(
                "/qdrant/collections/{name}",
                put(qdrant::handlers::create_collection),
            )
            .route(
                "/qdrant/collections/{name}",
                delete(qdrant::handlers::delete_collection),
            )
            .route(
                "/qdrant/collections/{name}",
                axum::routing::patch(qdrant::handlers::update_collection),
            )
            .route(
                "/qdrant/collections/{name}/points",
                post(qdrant::vector_handlers::retrieve_points),
            )
            .route(
                "/qdrant/collections/{name}/points",
                put(qdrant::vector_handlers::upsert_points),
            )
            .route(
                "/qdrant/collections/{name}/points/delete",
                post(qdrant::vector_handlers::delete_points),
            )
            .route(
                "/qdrant/collections/aliases",
                post(qdrant::alias_handlers::update_aliases),
            )
            .route(
                "/qdrant/collections/{name}/aliases",
                get(qdrant::alias_handlers::list_collection_aliases),
            )
            .route("/qdrant/aliases", get(qdrant::alias_handlers::list_aliases))
            .route(
                "/qdrant/collections/{name}/points/scroll",
                post(qdrant::vector_handlers::scroll_points),
            )
            .route(
                "/qdrant/collections/{name}/points/count",
                post(qdrant::vector_handlers::count_points),
            )
            .route(
                "/qdrant/collections/{name}/points/search",
                post(qdrant::search_handlers::search_points),
            )
            .route(
                "/qdrant/collections/{name}/points/search/batch",
                post(qdrant::search_handlers::batch_search_points),
            )
            .route(
                "/qdrant/collections/{name}/points/recommend",
                post(qdrant::search_handlers::recommend_points),
            )
            .route(
                "/qdrant/collections/{name}/points/recommend/batch",
                post(qdrant::search_handlers::batch_recommend_points),
            )
            // Query API endpoints (Qdrant 1.7+)
            .route(
                "/qdrant/collections/{name}/points/query",
                post(qdrant::query_handlers::query_points),
            )
            .route(
                "/qdrant/collections/{name}/points/query/batch",
                post(qdrant::query_handlers::batch_query_points),
            )
            .route(
                "/qdrant/collections/{name}/points/query/groups",
                post(qdrant::query_handlers::query_points_groups),
            )
            // Search Groups and Matrix API endpoints
            .route(
                "/qdrant/collections/{name}/points/search/groups",
                post(qdrant::search_handlers::search_points_groups),
            )
            .route(
                "/qdrant/collections/{name}/points/search/matrix/pairs",
                post(qdrant::search_handlers::search_matrix_pairs),
            )
            .route(
                "/qdrant/collections/{name}/points/search/matrix/offsets",
                post(qdrant::search_handlers::search_matrix_offsets),
            )
            // Snapshot API endpoints
            .route(
                "/qdrant/collections/{name}/snapshots",
                get(qdrant::snapshot_handlers::list_collection_snapshots),
            )
            .route(
                "/qdrant/collections/{name}/snapshots",
                post(qdrant::snapshot_handlers::create_collection_snapshot),
            )
            .route(
                "/qdrant/collections/{name}/snapshots/{snapshot_name}",
                delete(qdrant::snapshot_handlers::delete_collection_snapshot),
            )
            .route(
                "/qdrant/collections/{name}/snapshots/recover",
                post(qdrant::snapshot_handlers::recover_collection_snapshot),
            )
            .route(
                "/qdrant/collections/{name}/snapshots/upload",
                post(qdrant::snapshot_handlers::upload_collection_snapshot),
            )
            .route(
                "/qdrant/snapshots",
                get(qdrant::snapshot_handlers::list_all_snapshots),
            )
            .route(
                "/qdrant/snapshots",
                post(qdrant::snapshot_handlers::create_full_snapshot),
            )
            // Sharding API endpoints
            .route(
                "/qdrant/collections/{name}/shards",
                get(qdrant::sharding_handlers::list_shard_keys),
            )
            .route(
                "/qdrant/collections/{name}/shards",
                put(qdrant::sharding_handlers::create_shard_key),
            )
            .route(
                "/qdrant/collections/{name}/shards/delete",
                post(qdrant::sharding_handlers::delete_shard_key),
            )
            // Cluster API endpoints
            .route(
                "/qdrant/cluster",
                get(qdrant::cluster_handlers::get_cluster_status),
            )
            .route(
                "/qdrant/cluster/recover",
                post(qdrant::cluster_handlers::cluster_recover),
            )
            .route(
                "/qdrant/cluster/peer/{peer_id}",
                delete(qdrant::cluster_handlers::remove_peer),
            )
            .route(
                "/qdrant/cluster/metadata/keys",
                get(qdrant::cluster_handlers::list_metadata_keys),
            )
            .route(
                "/qdrant/cluster/metadata/keys/{key}",
                get(qdrant::cluster_handlers::get_metadata_key),
            )
            .route(
                "/qdrant/cluster/metadata/keys/{key}",
                put(qdrant::cluster_handlers::update_metadata_key),
            )
            // Dashboard - serve embedded static files (production build)
            // All dashboard assets are embedded in the binary using rust-embed
            // This allows distributing a single binary without external dependencies
            //
            // Route priority for /dashboard/*:
            // 1. Exact file match (assets/, favicon.ico, etc.) - served with cache headers
            // 2. SPA fallback - any other route returns index.html for React Router
            .route("/dashboard", get(embedded_assets::dashboard_root_handler))
            .route("/dashboard/", get(embedded_assets::dashboard_root_handler))
            .route(
                "/dashboard/{*path}",
                get(embedded_assets::dashboard_handler),
            )
            .layer(axum::middleware::from_fn(
                vectorizer::monitoring::correlation_middleware,
            ))
            .layer(axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let metrics = metrics_collector_2.clone();
                    let method = req.method().clone();
                    let uri = req.uri().clone();
                    async move {
                        // Log all requests, especially PUT to /qdrant/collections/*/points
                        if uri.path().contains("/qdrant/collections")
                            && uri.path().contains("/points")
                        {
                            info!("🔵 [MIDDLEWARE] {} {}", method, uri);
                        }

                        // Record connection opened
                        metrics.record_connection_opened();

                        // Record API request
                        let start = std::time::Instant::now();
                        let response = next.run(req).await;
                        let duration = start.elapsed().as_millis() as f64;

                        // Record API request metrics
                        let is_success = response.status().is_success();
                        metrics.record_api_request(is_success, duration);

                        // Record connection closed
                        metrics.record_connection_closed();

                        response
                    }
                },
            ))
            .with_state(self.clone());

        // Add cluster routes if cluster is enabled
        let rest_routes = if let (Some(cluster_mgr), Some(_client_pool)) = (
            self.cluster_manager.as_ref(),
            self.cluster_client_pool.as_ref(),
        ) {
            let cluster_state = crate::api::cluster::ClusterApiState {
                cluster_manager: cluster_mgr.clone(),
                store: self.store.clone(),
            };
            let cluster_router =
                crate::api::cluster::create_cluster_router().with_state(cluster_state);
            rest_routes.merge(cluster_router)
        } else {
            rest_routes
        };

        // Add graph routes
        let graph_state = crate::api::graph::GraphApiState::new(self.store.clone());
        let graph_router = crate::api::graph::create_graph_router().with_state(graph_state);
        let rest_routes = rest_routes.merge(graph_router);

        // Add GraphQL routes
        let graphql_schema = if let Some(ref auto_save) = self.auto_save_manager {
            crate::api::graphql::create_schema_with_auto_save(
                self.store.clone(),
                self.embedding_manager.clone(),
                self.start_time,
                auto_save.clone(),
            )
        } else {
            crate::api::graphql::create_schema(
                self.store.clone(),
                self.embedding_manager.clone(),
                self.start_time,
            )
        };
        let graphql_state = graphql_handlers::GraphQLState {
            schema: graphql_schema,
        };
        let graphql_router = Router::new()
            .route("/graphql", post(graphql_handlers::graphql_handler))
            .route("/graphql", get(graphql_handlers::graphql_playground))
            .route("/graphiql", get(graphql_handlers::graphql_playground))
            .with_state(graphql_state);
        let rest_routes = rest_routes.merge(graphql_router);
        info!("📊 GraphQL API available at /graphql (playground at /graphiql)");

        // Add auth routes and apply auth middleware if auth is enabled
        let rest_routes = if let Some(auth_state) = self.auth_handler_state.clone() {
            info!("🔐 Adding authentication routes...");

            // Public routes (no auth required) - login, password validation, and health check
            let public_auth_router = Router::new()
                .route("/auth/login", post(auth_handlers::login))
                .route(
                    "/auth/validate-password",
                    post(auth_handlers::validate_password_endpoint),
                )
                .with_state(auth_state.clone());

            // Protected + admin-gated auth routes. Every handler already takes
            // `Extension<AuthState>` and calls `ensure_authenticated` or
            // `ensure_admin` at the top — see `src/server/auth_handlers.rs`.
            // The `auth_middleware` layer applied on `rest_routes` below is
            // what populates the `AuthState` extension for every request.
            let protected_auth_router = Router::new()
                .route("/auth/me", get(auth_handlers::get_me))
                .route("/auth/logout", post(auth_handlers::logout))
                .route("/auth/refresh", post(auth_handlers::refresh_token))
                .route("/auth/keys", post(auth_handlers::create_api_key))
                .route("/auth/keys", get(auth_handlers::list_api_keys))
                .route("/auth/keys/{id}", delete(auth_handlers::revoke_api_key))
                // User management — admin role enforced inside handlers.
                .route("/auth/users", post(auth_handlers::create_user))
                .route("/auth/users", get(auth_handlers::list_users))
                .route("/auth/users/{username}", delete(auth_handlers::delete_user))
                .route(
                    "/auth/users/{username}/password",
                    put(auth_handlers::change_password),
                )
                .with_state(auth_state.clone());

            // Admin gating is enforced inside the individual handlers via the
            // `require_admin_from_headers` helper — see
            // src/server/auth_handlers.rs. A router-level `.layer(...)`
            // approach was attempted and rejected because axum's type
            // inference could not unify the two State types
            // (`AuthHandlerState` for auth handlers, `VectorizerServer` for
            // rest/setup handlers) through a single middleware layer.
            //
            // TASK(phase4_router-layer-admin-middleware): revisit once either
            // (a) all admin handlers share a state type, or (b) axum exposes a
            // `.route_layer()` path that compiles against both handler
            // families.

            info!(
                "🔐 Auth buckets — public: /health, /prometheus/metrics, /auth/login, \
                 /auth/validate-password. Authenticated (any logged-in user): /auth/me, \
                 /auth/logout, /auth/refresh, /auth/keys/*, all data-access routes. \
                 Admin (role=admin enforced inside handler): /auth/users*, \
                 /workspace/add, /workspace/remove, POST /workspace/config, \
                 /setup/apply, /setup/browse, POST /config, /admin/restart, \
                 /backups/create, /backups/restore."
            );

            // Merge auth routes
            rest_routes
                .merge(public_auth_router)
                .merge(protected_auth_router)
        } else {
            rest_routes
        };

        // Apply HiveHub middleware if hub integration is enabled
        // This middleware extracts tenant context from headers for multi-tenant isolation
        let rest_routes = if let Some(ref hub_manager) = self.hub_manager {
            info!("🔐 Applying HiveHub tenant middleware to routes...");

            use axum::middleware::from_fn_with_state;

            use vectorizer::hub::middleware::{HubAuthMiddleware, hub_auth_middleware};

            let hub_auth = hub_manager.auth().clone();
            let hub_quota = hub_manager.quota().clone();
            let hub_config = hub_manager.config().clone();

            let hub_middleware_state = HubAuthMiddleware::new(hub_auth, hub_quota, hub_config);

            rest_routes.layer(from_fn_with_state(
                hub_middleware_state,
                hub_auth_middleware,
            ))
        } else {
            rest_routes
        };

        // Create UMICP state
        let umicp_state = crate::umicp::UmicpState {
            store: self.store.clone(),
            embedding_manager: self.embedding_manager.clone(),
        };

        // Create UMICP routes (needs custom state)
        // Note: Auth is enforced via the require_production_auth helper for /umicp POST
        let umicp_routes = Router::new()
            .route("/umicp", post(crate::umicp::transport::umicp_handler))
            .route("/umicp/health", get(crate::umicp::health_check))
            .route(
                "/umicp/discover",
                get(crate::umicp::transport::umicp_discover_handler),
            )
            .with_state(umicp_state);

        // Admin-only router. The 9 routes below were previously protected
        // by per-handler `_admin: AdminAuth` extractors (and 4 others were
        // documented as admin-only but unprotected in practice). Lifting the
        // gate to the router boundary means a future contributor cannot add
        // an admin route into this group and forget the protection — and
        // closes the 4 documented-but-unprotected drift gaps in one move.
        // When auth is globally disabled, the router is merged without the
        // `require_admin_middleware` layer to preserve the existing
        // single-user-mode behaviour.
        let admin_router: Router<()> = Router::new()
            .route("/workspace/add", post(rest_handlers::add_workspace))
            .route("/workspace/remove", post(rest_handlers::remove_workspace))
            .route(
                "/workspace/config",
                post(rest_handlers::update_workspace_config),
            )
            .route("/setup/apply", post(setup_handlers::apply_setup_config))
            .route("/setup/browse", post(setup_handlers::browse_directory))
            .route("/config", post(rest_handlers::update_config))
            .route("/admin/restart", post(rest_handlers::restart_server))
            .route("/backups/create", post(rest_handlers::create_backup))
            .route("/backups/restore", post(rest_handlers::restore_backup))
            .with_state(self.clone());
        let admin_router = if let Some(auth_state) = self.auth_handler_state.clone() {
            admin_router.layer(axum::middleware::from_fn_with_state(
                auth_state,
                crate::server::auth_handlers::require_admin_middleware,
            ))
        } else {
            admin_router
        };

        // Merge all routes - order matters!
        // 1. Public routes first (health check, prometheus metrics) - no auth required
        // 2. UMICP routes (most specific)
        // 3. MCP routes
        // 4. Admin router (router-level admin gate)
        // 5. REST API routes (including /api/*, dashboard with embedded assets)
        // 6. Metrics routes
        // Note: Dashboard assets are embedded in the binary using rust-embed
        let app = Router::new()
            .merge(public_routes) // Health check and prometheus - always public
            .merge(umicp_routes)
            .merge(mcp_router)
            .merge(admin_router)
            .merge(rest_routes)
            .merge(metrics_router)
            // Apply DefaultBodyLimit to increase multipart upload limit beyond Axum's default 2MB
            // This allows file uploads up to max_request_size_mb (configured in config.yml)
            .layer(axum::extract::DefaultBodyLimit::max(
                self.max_request_size_mb * 1024 * 1024,
            ));

        // In production mode, apply global auth middleware BEFORE CORS
        // This middleware handles both standard auth (JWT/API key) and HiveHub integration
        let hub_mgr = self.hub_manager.clone();
        let app = if is_production_bind && (self.auth_handler_state.is_some() || hub_mgr.is_some())
        {
            let auth_mgr = self
                .auth_handler_state
                .as_ref()
                .map(|state| state.auth_manager.clone());
            let hub_manager = hub_mgr.clone();
            app.layer(axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
                let auth_manager = auth_mgr.clone();
                let hub_manager = hub_manager.clone();
                async move {
                    let path = req.uri().path();

                    // Public routes - no auth required
                    // NOTE: /mcp added to bypass auth for MCP access
                    if path == "/health"
                        || path == "/prometheus/metrics"
                        || path == "/auth/login"
                        || path == "/auth/validate-password"
                        || path == "/umicp/health"
                        || path == "/umicp/discover"
                        || path == "/mcp"
                        || path.starts_with("/dashboard")
                        || path.starts_with("/setup")
                    {
                        return next.run(req).await;
                    }

                    // Check for HiveHub internal service header
                    // When HiveHub integration is enabled, internal service requests bypass auth
                    if hub_manager.is_some() {
                        if req.headers().contains_key("x-hivehub-service") {
                            tracing::debug!("HiveHub internal service request - bypassing auth for {}", path);
                            return next.run(req).await;
                        }
                    }

                    // Standard authentication (if auth is enabled)
                    if let Some(ref auth_manager) = auth_manager {
                        // Extract credentials from headers
                        let (jwt_token, api_key) = extract_auth_credentials(&req);

                        // Debug: Log what we found
                        tracing::debug!("Auth check for {}: jwt={:?}, api_key={:?}", path, jwt_token.is_some(), api_key.is_some());

                        // Try to validate and extract claims
                        let mut user_claims = None;

                        // Try JWT first
                        if let Some(token) = jwt_token {
                            if let Ok(claims) = auth_manager.validate_jwt(&token) {
                                user_claims = Some(claims);
                            }
                        }

                        // Try API key if JWT failed
                        if user_claims.is_none() {
                            if let Some(key) = api_key {
                                if let Ok(claims) = auth_manager.validate_api_key(&key).await {
                                    user_claims = Some(claims);
                                }
                            }
                        }

                        // If no valid credentials, return 401
                        if user_claims.is_none() {
                            return axum::response::Response::builder()
                                .status(axum::http::StatusCode::UNAUTHORIZED)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(axum::body::Body::from(
                                    r#"{"error":"unauthorized","message":"Authentication required. Provide a valid JWT token or API key."}"#
                                ))
                                .unwrap();
                        }

                        // Add AuthState as Extension
                        let auth_state = vectorizer::auth::middleware::AuthState {
                            user_claims: user_claims.unwrap(),
                            authenticated: true,
                        };

                        let mut req = req;
                        req.extensions_mut().insert(auth_state);

                        return next.run(req).await;
                    } else if hub_manager.is_none() {
                        // No auth configured and no hub integration - reject
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::UNAUTHORIZED)
                            .header("Content-Type", "application/json")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from(
                                r#"{"error":"unauthorized","message":"Authentication not configured."}"#
                            ))
                            .unwrap();
                    }

                    next.run(req).await
                }
            }))
            // Apply CORS after auth middleware
            .layer(CorsLayer::permissive())
            // Apply security headers
            .layer(axum::middleware::from_fn(security_headers_middleware))
        } else {
            // Development mode: just apply CORS and security headers
            app.layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(security_headers_middleware))
        };

        // Apply write-redirect middleware if this node is a replica
        // Replicas redirect POST/PUT/DELETE/PATCH to the leader with HTTP 307
        let app = if let Some(ref ha) = self.ha_manager {
            let leader_router = ha.leader_router.clone();
            app.layer(axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
                let lr = leader_router.clone();
                async move {
                    // Skip redirect for read-only endpoints that use POST method.
                    // Search uses POST but is a read operation — serve locally.
                    let path = req.uri().path();
                    if path.starts_with("/health")
                        || path.starts_with("/prometheus")
                        || path.starts_with("/auth")
                        || path.starts_with("/api/v1/cluster")
                        || path.contains("/search")
                        || path.contains("/scroll")
                        || path.contains("/recommend")
                        || path.contains("/count")
                        || path.ends_with("/graphql")
                        || path.ends_with("/graphiql")
                    {
                        return next.run(req).await;
                    }

                    // Only redirect write operations on follower nodes
                    if !lr.is_leader() && Self::is_write_request(req.method()) {
                        if let Some(leader_url) = lr.leader_redirect_url() {
                            let redirect_path = req.uri().path_and_query()
                                .map(|pq| pq.as_str())
                                .unwrap_or("/");
                            let location = format!("{}{}", leader_url, redirect_path);
                            tracing::info!("Redirecting write to leader: {}", location);
                            return axum::response::Response::builder()
                                .status(axum::http::StatusCode::TEMPORARY_REDIRECT)
                                .header("Location", &location)
                                .header("X-Vectorizer-Leader", &leader_url)
                                .header("X-Vectorizer-Role", "follower")
                                .body(axum::body::Body::from(
                                    format!("{{\"redirect\":\"write operations must go to leader\",\"leader_url\":\"{}\"}}", leader_url)
                                ))
                                .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()));
                        }
                    }
                    next.run(req).await
                }
            }))
        } else {
            app
        };

        info!("🌐 Vectorizer Server available at:");
        info!("   📡 MCP StreamableHTTP: http://{}:{}/mcp", host, port);
        info!("   🔌 REST API: http://{}:{}", host, port);
        info!("   🔗 UMICP: http://{}:{}/umicp", host, port);
        info!(
            "   🔍 UMICP Discovery (v0.2.1): http://{}:{}/umicp/discover",
            host, port
        );
        info!("   🎯 Qdrant API: http://{}:{}/qdrant", host, port);
        info!("   📊 GraphQL API: http://{}:{}/graphql", host, port);
        info!(
            "   🎮 GraphQL Playground: http://{}:{}/graphiql",
            host, port
        );
        info!("   📊 Dashboard: http://{}:{}/dashboard/", host, port);
        if self.auth_handler_state.is_some() {
            info!("   🔐 Auth API: http://{}:{}/auth", host, port);
        }
        if self.hub_manager.is_some() {
            info!("   🌐 HiveHub: Cluster mode enabled (internal service access)");
        }

        // Bind and start the server
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
        info!(
            "✅ MCP server (StreamableHTTP) with REST API listening on {}:{}",
            host, port
        );

        // Display first-start guidance if setup is needed
        let collection_count = self.store.list_collections().len();
        setup_handlers::display_first_start_guidance(host, port, collection_count);

        // Create shutdown signal for axum graceful shutdown
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn task to listen for shutdown signals (Ctrl+C and SIGTERM on Unix)
        tokio::spawn(async move {
            // Create futures for different shutdown signals
            let ctrl_c = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
                info!("🛑 Received shutdown signal (Ctrl+C)");
            };

            // On Unix, also listen for SIGTERM (used by Docker, Kubernetes, systemd)
            #[cfg(unix)]
            let terminate = async {
                use tokio::signal::unix::{SignalKind, signal};
                let mut sigterm =
                    signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
                sigterm.recv().await;
                info!("🛑 Received shutdown signal (SIGTERM)");
            };

            // On Windows, SIGTERM is not available, so we only listen for Ctrl+C
            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            // Wait for either signal
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }

            // Send shutdown signal
            let _ = shutdown_tx.send(());
        });

        // Serve the application with graceful shutdown
        let server_handle = axum::serve(listener, app).with_graceful_shutdown(async {
            shutdown_rx.await.ok();
            info!("🛑 Graceful shutdown signal received, stopping HTTP server...");
        });

        // Spawn server task
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_handle.await {
                error!("❌ Server error: {}", e);
            } else {
                info!("✅ HTTP server stopped");
            }
        });

        // Get abort handle before moving server_task (for emergency shutdown)
        let server_task_abort = server_task.abort_handle();

        // Wait for HTTP server to stop (this will block until Ctrl+C is pressed)
        // When shutdown signal is received, the server will stop gracefully
        // No timeout here - server should run indefinitely until Ctrl+C
        match server_task.await {
            Ok(_) => {
                info!("✅ HTTP server stopped gracefully");
            }
            Err(e) => {
                error!("❌ HTTP server task join error: {}", e);
                // Force abort as fallback
                server_task_abort.abort();
            }
        }

        // Now shutdown all background tasks AFTER HTTP server has stopped
        info!("🛑 Stopping all background tasks...");

        // Background collection loading task (non-blocking)
        if let Ok(mut bg_task) = self.background_task.try_lock() {
            if let Some((handle, cancel_tx)) = bg_task.take() {
                let _ = cancel_tx.send(true);
                handle.abort();
                info!("✅ Background task aborted");
            }
        }

        // File watcher cancellation (non-blocking)
        if let Ok(mut cancel) = self.file_watcher_cancel.try_lock() {
            if let Some(cancel_tx) = cancel.take() {
                let _ = cancel_tx.send(true);
            }
        }

        // File watcher task (non-blocking)
        if let Ok(mut fw_task) = self.file_watcher_task.try_lock() {
            if let Some(handle) = fw_task.take() {
                handle.abort();
                info!("✅ File watcher task aborted");
            }
        }

        // File watcher system (non-blocking)
        if let Ok(mut fw_system) = self.file_watcher_system.try_lock() {
            fw_system.take(); // Just drop it
            info!("✅ File watcher system dropped");
        }

        // gRPC server task (non-blocking)
        if let Ok(mut grpc_task) = self.grpc_task.try_lock() {
            if let Some(handle) = grpc_task.take() {
                handle.abort();
                info!("✅ gRPC server task aborted");
            }
        }

        // System collector task (non-blocking)
        if let Ok(mut sys_task) = self.system_collector_task.try_lock() {
            if let Some(handle) = sys_task.take() {
                handle.abort();
                info!("✅ System collector task aborted");
            }
        }

        // Force save all data before shutdown to prevent data loss
        // This ensures any changes made since the last auto-save are persisted
        if let Some(auto_save) = &self.auto_save_manager {
            info!("💾 Forcing final save before shutdown...");
            match auto_save.force_save().await {
                Ok(_) => info!("✅ Final save completed successfully"),
                Err(e) => warn!("⚠️ Final save failed (data may be lost): {}", e),
            }
        }

        // Auto save task (non-blocking) - abort AFTER force_save
        if let Ok(mut auto_task) = self.auto_save_task.try_lock() {
            if let Some(handle) = auto_task.take() {
                handle.abort();
                info!("✅ Auto save task aborted");
            }
        }

        // Auto save manager shutdown (non-blocking, no await)
        if let Some(auto_save) = &self.auto_save_manager {
            auto_save.shutdown();
        }

        info!("✅ Server stopped");
        Ok(())
    }

    /// Create MCP router with StreamableHTTP transport (rmcp 0.8.1).
    ///
    /// The historic production-mode auth guard is preserved verbatim as
    /// a commented block below — it was disabled deliberately to make
    /// MCP freely accessible. Re-enable by swapping the `else` branch
    /// once `.route_layer()` supports the two-state unification the
    /// guard requires.
    // Function-scoped allow: the trailing `Response::builder()...body(...)
    // .unwrap()` is a static literal-body construction that cannot fail.
    #[allow(clippy::unwrap_used)]
    async fn create_mcp_router(
        &self,
        _is_production: bool,
        _auth_state: Option<auth_handlers::AuthHandlerState>,
    ) -> Router {
        use hyper::service::Service;
        use hyper_util::service::TowerToHyperService;
        use rmcp::transport::streamable_http_server::StreamableHttpService;
        use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

        // Create MCP service handler
        let store = self.store.clone();
        let embedding_manager = self.embedding_manager.clone();
        let cluster_manager = self.cluster_manager.clone();

        // Create StreamableHTTP service
        let streamable_service = StreamableHttpService::new(
            move || {
                Ok(VectorizerMcpService {
                    store: store.clone(),
                    embedding_manager: embedding_manager.clone(),
                    cluster_manager: cluster_manager.clone(),
                })
            },
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Convert to axum service and create router
        let hyper_service = TowerToHyperService::new(streamable_service);

        Router::new().route(
            "/mcp",
            axum::routing::any(move |req: axum::extract::Request| {
                let mut service = hyper_service.clone();
                async move {
                    // Forward request to hyper service
                    match service.call(req).await {
                        Ok(response) => {
                            // Convert BoxBody to axum Body
                            let (parts, body) = response.into_parts();
                            axum::response::Response::from_parts(parts, axum::body::Body::new(body))
                        }
                        Err(_) => axum::response::Response::builder()
                            .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                            .body(axum::body::Body::from("Internal server error"))
                            .unwrap(),
                    }
                }
            }),
        )
    }
}

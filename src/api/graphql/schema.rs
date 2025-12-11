//! GraphQL Schema and Resolvers for Vectorizer
//!
//! This module defines the GraphQL schema including Query and Mutation types.

use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Schema};
use tracing::{error, info, warn};

use super::types::*;
use crate::config::FileUploadConfig;
use crate::db::VectorStore;
use crate::db::auto_save::AutoSaveManager;
use crate::db::graph::{Edge, Node, RelationshipType};
use crate::embedding::EmbeddingManager;
use crate::file_loader::chunker::Chunker;
use crate::file_loader::config::LoaderConfig;
use crate::hub::auth::TenantContext;
use crate::hub::quota::QuotaManager;
use crate::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector,
};

/// GraphQL context containing shared state
pub struct GraphQLContext {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
    /// Optional tenant context for multi-tenant mode
    pub tenant_context: Option<TenantContext>,
    /// Optional quota manager for multi-tenant mode
    pub quota_manager: Option<Arc<QuotaManager>>,
    /// Optional auto-save manager for persistence
    pub auto_save_manager: Option<Arc<AutoSaveManager>>,
}

/// The GraphQL schema type
pub type VectorizerSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Create the GraphQL schema with the given context
///
/// Schema includes:
/// - Query depth limit of 10 (prevents deeply nested queries)
/// - Query complexity limit of 1000 (prevents expensive queries)
pub fn create_schema(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    start_time: std::time::Instant,
) -> VectorizerSchema {
    let ctx = GraphQLContext {
        store,
        embedding_manager,
        start_time,
        tenant_context: None,
        quota_manager: None,
        auto_save_manager: None,
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        // Limit query depth to prevent deeply nested queries
        .limit_depth(10)
        // Limit query complexity to prevent expensive queries
        .limit_complexity(1000)
        .finish()
}

/// Create the GraphQL schema with auto-save support
pub fn create_schema_with_auto_save(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    start_time: std::time::Instant,
    auto_save_manager: Arc<AutoSaveManager>,
) -> VectorizerSchema {
    let ctx = GraphQLContext {
        store,
        embedding_manager,
        start_time,
        tenant_context: None,
        quota_manager: None,
        auto_save_manager: Some(auto_save_manager),
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .limit_depth(10)
        .limit_complexity(1000)
        .finish()
}

/// Create the GraphQL schema with multi-tenant support
pub fn create_schema_with_hub(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    start_time: std::time::Instant,
    quota_manager: Arc<QuotaManager>,
) -> VectorizerSchema {
    let ctx = GraphQLContext {
        store,
        embedding_manager,
        start_time,
        tenant_context: None, // Set per-request in handler
        quota_manager: Some(quota_manager),
        auto_save_manager: None,
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .limit_depth(10)
        .limit_complexity(1000)
        .finish()
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Verify collection ownership in multi-tenant mode
fn check_collection_ownership(
    store: &VectorStore,
    collection: &str,
    tenant_ctx: Option<&TenantContext>,
) -> async_graphql::Result<()> {
    if let Some(tenant) = tenant_ctx {
        // Parse tenant_id as UUID
        let tenant_uuid = uuid::Uuid::parse_str(&tenant.tenant_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid tenant ID: {e}")))?;

        if !store.is_collection_owned_by(collection, &tenant_uuid) {
            return Err(async_graphql::Error::new(
                "Collection not found or access denied",
            ));
        }
    }
    Ok(())
}

// =============================================================================
// QUERY ROOT
// =============================================================================

/// Root query object for GraphQL
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get all collections (filtered by tenant in multi-tenant mode)
    async fn collections(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<GqlCollection>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Get tenant context if available
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        let collection_names = if let Some(tenant) = tenant_ctx {
            // Multi-tenant mode: filter by owner
            let tenant_uuid = uuid::Uuid::parse_str(&tenant.tenant_id)
                .map_err(|e| async_graphql::Error::new(format!("Invalid tenant ID: {e}")))?;
            gql_ctx.store.list_collections_for_owner(&tenant_uuid)
        } else {
            // Single-tenant mode: list all
            gql_ctx.store.list_collections()
        };

        let mut collections = Vec::new();
        for name in collection_names {
            if let Ok(meta) = gql_ctx.store.get_collection_metadata(&name) {
                collections.push(meta.into());
            }
        }

        Ok(collections)
    }

    /// Get a specific collection by name (with tenant ownership check)
    async fn collection(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<Option<GqlCollection>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        match gql_ctx.store.get_collection_metadata(&name) {
            Ok(meta) => {
                // In multi-tenant mode, verify ownership
                if let Some(tenant) = tenant_ctx {
                    let tenant_uuid = uuid::Uuid::parse_str(&tenant.tenant_id).map_err(|e| {
                        async_graphql::Error::new(format!("Invalid tenant ID: {e}"))
                    })?;
                    if gql_ctx.store.is_collection_owned_by(&name, &tenant_uuid) {
                        Ok(Some(meta.into()))
                    } else {
                        Ok(None) // Not owned by this tenant
                    }
                } else {
                    Ok(Some(meta.into()))
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Get a vector by ID from a collection (with tenant ownership check)
    async fn vector(
        &self,
        ctx: &Context<'_>,
        collection: String,
        id: String,
    ) -> async_graphql::Result<Option<GqlVector>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        match gql_ctx.store.get_vector(&collection, &id) {
            Ok(v) => Ok(Some(v.into())),
            Err(_) => Ok(None), // Vector not found
        }
    }

    /// List vectors in a collection with pagination (with tenant ownership check)
    async fn vectors(
        &self,
        ctx: &Context<'_>,
        input: ScrollInput,
    ) -> async_graphql::Result<GqlPage<GqlVector>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &input.collection, tenant_ctx)?;

        // Get collection to retrieve vectors
        let collection_ref = gql_ctx
            .store
            .get_collection(&input.collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let all_vectors = collection_ref.get_all_vectors();
        let total_count = all_vectors.len() as i32;

        // Apply cursor-based pagination
        let offset = input
            .cursor
            .as_ref()
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);

        let limit = input.limit.min(1000) as usize;
        let items: Vec<GqlVector> = all_vectors
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|v| v.into())
            .collect();

        let has_next_page = offset + items.len() < total_count as usize;
        let next_cursor = if has_next_page {
            Some((offset + items.len()).to_string())
        } else {
            None
        };

        Ok(GqlPage {
            items,
            total_count,
            has_next_page,
            next_cursor,
        })
    }

    /// Semantic vector search (with tenant ownership check)
    async fn search(
        &self,
        ctx: &Context<'_>,
        input: SearchInput,
    ) -> async_graphql::Result<Vec<GqlSearchResult>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &input.collection, tenant_ctx)?;

        let results = gql_ctx
            .store
            .search(&input.collection, &input.vector, input.limit as usize)
            .map_err(|e| async_graphql::Error::new(format!("Search failed: {e}")))?;

        // Apply score threshold filter if specified
        let filtered: Vec<GqlSearchResult> = results
            .into_iter()
            .filter(|r| input.score_threshold.map(|t| r.score >= t).unwrap_or(true))
            .map(|r| r.into())
            .collect();

        Ok(filtered)
    }

    /// Get server statistics (tenant-scoped in multi-tenant mode)
    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<GqlServerStats> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Get collection names based on tenant context
        let collection_names = if let Some(tenant) = tenant_ctx {
            // Multi-tenant mode: only count this tenant's collections
            let tenant_uuid = uuid::Uuid::parse_str(&tenant.tenant_id)
                .map_err(|e| async_graphql::Error::new(format!("Invalid tenant ID: {e}")))?;
            gql_ctx.store.list_collections_for_owner(&tenant_uuid)
        } else {
            // Single-tenant mode: count all collections
            gql_ctx.store.list_collections()
        };

        let mut total_vectors: i64 = 0;

        for name in &collection_names {
            if let Ok(meta) = gql_ctx.store.get_collection_metadata(name) {
                total_vectors += meta.vector_count as i64;
            }
        }

        let uptime = gql_ctx.start_time.elapsed().as_secs() as i64;
        let memory_usage = memory_stats::memory_stats()
            .map(|s| s.physical_mem as i64)
            .unwrap_or(0);

        Ok(GqlServerStats {
            version: env!("CARGO_PKG_VERSION").to_string(),
            collection_count: collection_names.len() as i32,
            total_vectors,
            uptime_seconds: uptime,
            memory_usage_bytes: memory_usage,
        })
    }

    // =========================================================================
    // GRAPH QUERIES
    // =========================================================================

    /// Get graph statistics for a collection (with tenant ownership check)
    async fn graph_stats(
        &self,
        ctx: &Context<'_>,
        collection: String,
    ) -> async_graphql::Result<GqlGraphStats> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        match collection_ref.get_graph() {
            Some(graph) => Ok(GqlGraphStats {
                node_count: graph.node_count() as i32,
                edge_count: graph.edge_count() as i32,
                enabled: true,
            }),
            None => Ok(GqlGraphStats {
                node_count: 0,
                edge_count: 0,
                enabled: false,
            }),
        }
    }

    /// Get all nodes in a collection's graph (with tenant ownership check)
    async fn graph_nodes(
        &self,
        ctx: &Context<'_>,
        collection: String,
        #[graphql(default = 100)] limit: i32,
        #[graphql(default)] cursor: Option<String>,
    ) -> async_graphql::Result<GqlPage<GqlNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let all_nodes = graph.get_all_nodes();
        let total_count = all_nodes.len() as i32;

        // Apply cursor-based pagination
        let offset = cursor
            .as_ref()
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);

        let limit = limit.min(1000) as usize;
        let items: Vec<GqlNode> = all_nodes
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|n| n.into())
            .collect();

        let has_next_page = offset + items.len() < total_count as usize;
        let next_cursor = if has_next_page {
            Some((offset + items.len()).to_string())
        } else {
            None
        };

        Ok(GqlPage {
            items,
            total_count,
            has_next_page,
            next_cursor,
        })
    }

    /// Get all edges in a collection's graph (with tenant ownership check)
    async fn graph_edges(
        &self,
        ctx: &Context<'_>,
        collection: String,
        #[graphql(default = 100)] limit: i32,
        #[graphql(default)] cursor: Option<String>,
    ) -> async_graphql::Result<GqlPage<GqlEdge>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let all_edges = graph.get_all_edges();
        let total_count = all_edges.len() as i32;

        // Apply cursor-based pagination
        let offset = cursor
            .as_ref()
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);

        let limit = limit.min(1000) as usize;
        let items: Vec<GqlEdge> = all_edges
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|e| e.into())
            .collect();

        let has_next_page = offset + items.len() < total_count as usize;
        let next_cursor = if has_next_page {
            Some((offset + items.len()).to_string())
        } else {
            None
        };

        Ok(GqlPage {
            items,
            total_count,
            has_next_page,
            next_cursor,
        })
    }

    /// Get a specific node by ID (with tenant ownership check)
    async fn graph_node(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
    ) -> async_graphql::Result<Option<GqlNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        Ok(graph.get_node(&node_id).map(|n| n.into()))
    }

    /// Get neighbors of a node (with tenant ownership check)
    async fn graph_neighbors(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
        #[graphql(default)] relationship_type: Option<GqlRelationshipType>,
    ) -> async_graphql::Result<Vec<GqlRelatedNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let rel_type = relationship_type.map(|rt| rt.into());

        let neighbors = graph
            .get_neighbors(&node_id, rel_type)
            .map_err(|e| async_graphql::Error::new(format!("Failed to get neighbors: {e}")))?;

        Ok(neighbors
            .into_iter()
            .map(|(node, edge)| GqlRelatedNode {
                node: node.into(),
                hops: 1,
                weight: edge.weight,
            })
            .collect())
    }

    /// Find nodes related to a source node within N hops (with tenant ownership check)
    async fn graph_related(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
        #[graphql(default = 2)] max_hops: i32,
        #[graphql(default)] relationship_type: Option<GqlRelationshipType>,
    ) -> async_graphql::Result<Vec<GqlRelatedNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let rel_type = relationship_type.map(|rt| rt.into());

        let related = graph
            .find_related(&node_id, max_hops as usize, rel_type)
            .map_err(|e| async_graphql::Error::new(format!("Failed to find related nodes: {e}")))?;

        Ok(related
            .into_iter()
            .map(|(node, hops, weight)| GqlRelatedNode {
                node: node.into(),
                hops: hops as i32,
                weight,
            })
            .collect())
    }

    /// Find shortest path between two nodes (with tenant ownership check)
    async fn graph_path(
        &self,
        ctx: &Context<'_>,
        collection: String,
        source: String,
        target: String,
    ) -> async_graphql::Result<Vec<GqlNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let path = graph
            .find_path(&source, &target)
            .map_err(|e| async_graphql::Error::new(format!("Path not found: {e}")))?;

        Ok(path.into_iter().map(|n| n.into()).collect())
    }

    // =========================================================================
    // WORKSPACE QUERIES
    // =========================================================================

    /// List all registered workspaces
    async fn workspaces(&self, _ctx: &Context<'_>) -> async_graphql::Result<Vec<GqlWorkspace>> {
        let workspace_manager = crate::config::WorkspaceManager::new();
        let workspaces = workspace_manager.list_workspaces();

        Ok(workspaces
            .into_iter()
            .map(|w| GqlWorkspace {
                path: w.path,
                collection_name: w.collection_name,
                indexed: w.last_indexed.is_some(),
            })
            .collect())
    }

    /// Get file upload configuration
    async fn file_upload_config(
        &self,
        _ctx: &Context<'_>,
    ) -> async_graphql::Result<GqlFileUploadConfig> {
        let config = load_file_upload_config();

        Ok(GqlFileUploadConfig {
            max_file_size: config.max_file_size as i32,
            max_file_size_mb: (config.max_file_size / (1024 * 1024)) as i32,
            reject_binary: config.reject_binary,
            default_chunk_size: config.default_chunk_size as i32,
            default_chunk_overlap: config.default_chunk_overlap as i32,
            allowed_extensions: config.allowed_extensions,
        })
    }

    /// Get workspace configuration
    async fn workspace_config(
        &self,
        _ctx: &Context<'_>,
    ) -> async_graphql::Result<GqlWorkspaceConfig> {
        let possible_paths = vec![
            "./workspace.yml",
            "../workspace.yml",
            "../../workspace.yml",
            "./config/workspace.yml",
        ];

        for path in &possible_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(config) = serde_yaml::from_str::<serde_json::Value>(&content) {
                    info!("GraphQL: Loaded workspace config from: {}", path);
                    return Ok(GqlWorkspaceConfig {
                        global_settings: async_graphql::Json(
                            config
                                .get("global_settings")
                                .cloned()
                                .unwrap_or(serde_json::json!({})),
                        ),
                        projects: async_graphql::Json(
                            config
                                .get("projects")
                                .cloned()
                                .unwrap_or(serde_json::json!([])),
                        ),
                    });
                }
            }
        }

        // Return minimal default if no file found
        Ok(GqlWorkspaceConfig {
            global_settings: async_graphql::Json(serde_json::json!({
                "file_watcher": {
                    "enabled": true,
                    "debounce_ms": 1000
                }
            })),
            projects: async_graphql::Json(serde_json::json!([])),
        })
    }
}

// =============================================================================
// MUTATION ROOT
// =============================================================================

/// Root mutation object for GraphQL
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new collection (with tenant scoping and quota check)
    async fn create_collection(
        &self,
        ctx: &Context<'_>,
        input: CreateCollectionInput,
    ) -> async_graphql::Result<GqlCollection> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // In multi-tenant mode, enforce quota check
        if let (Some(tenant), Some(quota_mgr)) = (tenant_ctx, &gql_ctx.quota_manager) {
            // Check collection count quota
            match quota_mgr
                .check_quota(
                    &tenant.tenant_id,
                    crate::hub::quota::QuotaType::CollectionCount,
                    1,
                )
                .await
            {
                Ok(true) => {} // Quota OK
                Ok(false) => {
                    return Err(async_graphql::Error::new(
                        "Collection limit exceeded for your plan",
                    ));
                }
                Err(e) => {
                    error!("GraphQL: Quota check failed: {e}");
                    // Continue anyway if quota check fails
                }
            }
        }

        // Build collection config
        let mut config = CollectionConfig {
            dimension: input.dimension as usize,
            metric: input
                .metric
                .map(|m| m.into())
                .unwrap_or(crate::models::DistanceMetric::Cosine),
            hnsw_config: HnswConfig {
                m: input.hnsw_m.unwrap_or(16) as usize,
                ef_construction: input.hnsw_ef_construction.unwrap_or(200) as usize,
                ..Default::default()
            },
            ..Default::default()
        };

        // Configure sharding if requested
        if let Some(shard_count) = input.shard_count {
            config.sharding = Some(crate::models::ShardingConfig {
                shard_count: shard_count as u32,
                ..Default::default()
            });
        }

        // Configure graph if requested
        let enable_graph = input.enable_graph.unwrap_or(false);
        if enable_graph {
            config.graph = Some(crate::models::GraphConfig::default());
        }

        // In multi-tenant mode, add tenant prefix to collection name
        let collection_name = if let Some(tenant) = tenant_ctx {
            format!("user_{}:{}", tenant.tenant_id, input.name)
        } else {
            input.name.clone()
        };

        // Force CPU if requested or if graph is enabled (graphs not supported on GPU)
        let force_cpu = input.force_cpu.unwrap_or(false) || enable_graph;
        if force_cpu {
            gql_ctx
                .store
                .create_collection_cpu_only(&collection_name, config)
                .map_err(|e| {
                    async_graphql::Error::new(format!("Failed to create collection: {e}"))
                })?;
        } else {
            gql_ctx
                .store
                .create_collection(&collection_name, config)
                .map_err(|e| {
                    async_graphql::Error::new(format!("Failed to create collection: {e}"))
                })?;
        }

        info!("GraphQL: Created collection '{}'", collection_name);

        // Mark changes for auto-save
        if let Some(ref auto_save) = gql_ctx.auto_save_manager {
            auto_save.mark_changed();
        }

        // Return the created collection metadata
        let meta = gql_ctx
            .store
            .get_collection_metadata(&input.name)
            .map_err(|e| async_graphql::Error::new(format!("Failed to get metadata: {e}")))?;

        Ok(meta.into())
    }

    /// Delete a collection (with tenant ownership check)
    async fn delete_collection(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // In multi-tenant mode, verify ownership
        if let Some(tenant) = tenant_ctx {
            let tenant_uuid = uuid::Uuid::parse_str(&tenant.tenant_id)
                .map_err(|e| async_graphql::Error::new(format!("Invalid tenant ID: {e}")))?;
            if !gql_ctx.store.is_collection_owned_by(&name, &tenant_uuid) {
                return Ok(MutationResult::err(
                    "Collection not found or access denied".to_string(),
                ));
            }
        }

        match gql_ctx.store.delete_collection(&name) {
            Ok(_) => {
                info!("GraphQL: Deleted collection '{name}'");
                // Mark changes for auto-save
                if let Some(ref auto_save) = gql_ctx.auto_save_manager {
                    auto_save.mark_changed();
                }
                Ok(MutationResult::ok_with_message(format!(
                    "Collection '{name}' deleted"
                )))
            }
            Err(e) => {
                error!("GraphQL: Failed to delete collection '{name}': {e}");
                Ok(MutationResult::err(format!(
                    "Failed to delete collection: {e}"
                )))
            }
        }
    }

    /// Upsert a single vector (with tenant ownership check and quota validation)
    async fn upsert_vector(
        &self,
        ctx: &Context<'_>,
        collection: String,
        input: UpsertVectorInput,
    ) -> async_graphql::Result<GqlVector> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        // Check quota in multi-tenant mode
        if let (Some(tenant), Some(quota_mgr)) = (tenant_ctx, &gql_ctx.quota_manager) {
            match quota_mgr
                .check_quota(
                    &tenant.tenant_id,
                    crate::hub::quota::QuotaType::VectorCount,
                    1,
                )
                .await
            {
                Ok(true) => {} // Quota OK
                Ok(false) => {
                    return Err(async_graphql::Error::new(
                        "Vector count limit exceeded for your plan",
                    ));
                }
                Err(e) => {
                    error!("GraphQL: Quota check failed: {e}");
                }
            }
        }

        let payload = if let Some(payload_json) = input.payload {
            if let Some(ref key) = input.public_key {
                // Encrypt payload
                let encrypted =
                    crate::security::payload_encryption::encrypt_payload(&payload_json.0, key)
                        .map_err(|e| {
                            async_graphql::Error::new(format!("Failed to encrypt payload: {e}"))
                        })?;
                Some(Payload::from_encrypted(encrypted))
            } else {
                Some(Payload::new(payload_json.0))
            }
        } else {
            None
        };

        let vector = if let Some(p) = payload {
            Vector::with_payload(input.id.clone(), input.data.clone(), p)
        } else {
            Vector::new(input.id.clone(), input.data.clone())
        };

        gql_ctx
            .store
            .insert(&collection, vec![vector.clone()])
            .map_err(|e| async_graphql::Error::new(format!("Failed to upsert vector: {e}")))?;

        // Mark changes for auto-save
        if let Some(ref auto_save) = gql_ctx.auto_save_manager {
            auto_save.mark_changed();
        }

        Ok(vector.into())
    }

    /// Upsert multiple vectors in batch (with tenant ownership check and quota validation)
    async fn upsert_vectors(
        &self,
        ctx: &Context<'_>,
        input: UpsertVectorsInput,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &input.collection, tenant_ctx)?;

        // Check quota in multi-tenant mode
        if let (Some(tenant), Some(quota_mgr)) = (tenant_ctx, &gql_ctx.quota_manager) {
            match quota_mgr
                .check_quota(
                    &tenant.tenant_id,
                    crate::hub::quota::QuotaType::VectorCount,
                    1,
                )
                .await
            {
                Ok(true) => {} // Quota OK
                Ok(false) => {
                    return Err(async_graphql::Error::new(
                        "Vector count limit exceeded for your plan",
                    ));
                }
                Err(e) => {
                    error!("GraphQL: Quota check failed: {e}");
                }
            }
        }

        let request_public_key = input.public_key.clone();
        let vectors: Result<Vec<Vector>, async_graphql::Error> = input
            .vectors
            .into_iter()
            .map(|v_input| {
                let payload = if let Some(payload_json) = v_input.payload {
                    // Use vector-level public_key if present, otherwise request-level
                    let public_key_to_use = v_input.public_key.or(request_public_key.clone());

                    if let Some(ref key) = public_key_to_use {
                        // Encrypt payload
                        let encrypted = crate::security::payload_encryption::encrypt_payload(
                            &payload_json.0,
                            key,
                        )
                        .map_err(|e| {
                            async_graphql::Error::new(format!("Failed to encrypt payload: {e}"))
                        })?;
                        Some(Payload::from_encrypted(encrypted))
                    } else {
                        Some(Payload::new(payload_json.0))
                    }
                } else {
                    None
                };

                Ok(if let Some(p) = payload {
                    Vector::with_payload(v_input.id, v_input.data, p)
                } else {
                    Vector::new(v_input.id, v_input.data)
                })
            })
            .collect();

        let vectors = vectors?;
        let count = vectors.len() as i32;

        gql_ctx
            .store
            .insert(&input.collection, vectors)
            .map_err(|e| async_graphql::Error::new(format!("Failed to upsert vectors: {e}")))?;

        // Mark changes for auto-save
        if let Some(ref auto_save) = gql_ctx.auto_save_manager {
            auto_save.mark_changed();
        }

        info!(
            "GraphQL: Upserted {count} vectors in '{}'",
            input.collection
        );
        Ok(MutationResult::ok_with_count(count))
    }

    /// Delete a vector by ID (with tenant ownership check)
    async fn delete_vector(
        &self,
        ctx: &Context<'_>,
        collection: String,
        id: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        match gql_ctx.store.delete(&collection, &id) {
            Ok(_) => {
                // Mark changes for auto-save
                if let Some(ref auto_save) = gql_ctx.auto_save_manager {
                    auto_save.mark_changed();
                }
                Ok(MutationResult::ok_with_message(format!(
                    "Vector '{id}' deleted"
                )))
            }
            Err(e) => Ok(MutationResult::err(format!("Failed to delete vector: {e}"))),
        }
    }

    /// Update vector payload (with tenant ownership check)
    async fn update_payload(
        &self,
        ctx: &Context<'_>,
        collection: String,
        id: String,
        payload: async_graphql::Json<serde_json::Value>,
        #[graphql(default)] public_key: Option<String>,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        // Get existing vector
        let existing = gql_ctx
            .store
            .get_vector(&collection, &id)
            .map_err(|e| async_graphql::Error::new(format!("Vector not found: {e}")))?;

        // Create payload with optional encryption
        let new_payload = if let Some(ref key) = public_key {
            let encrypted = crate::security::payload_encryption::encrypt_payload(&payload.0, key)
                .map_err(|e| {
                async_graphql::Error::new(format!("Failed to encrypt payload: {e}"))
            })?;
            Payload::from_encrypted(encrypted)
        } else {
            Payload::new(payload.0)
        };

        // Update with new payload
        let updated = Vector::with_payload(existing.id, existing.data, new_payload);

        gql_ctx
            .store
            .insert(&collection, vec![updated])
            .map_err(|e| async_graphql::Error::new(format!("Failed to update payload: {e}")))?;

        Ok(MutationResult::ok_with_message("Payload updated"))
    }

    // =========================================================================
    // GRAPH MUTATIONS
    // =========================================================================

    /// Enable graph for a collection (with tenant ownership check)
    async fn enable_graph(
        &self,
        ctx: &Context<'_>,
        collection: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        match gql_ctx.store.enable_graph_for_collection(&collection) {
            Ok(_) => {
                info!("GraphQL: Enabled graph for collection '{collection}'");
                Ok(MutationResult::ok_with_message(format!(
                    "Graph enabled for collection '{collection}'"
                )))
            }
            Err(e) => {
                error!("GraphQL: Failed to enable graph for '{collection}': {e}");
                Ok(MutationResult::err(format!("Failed to enable graph: {e}")))
            }
        }
    }

    /// Add a node to the graph (with tenant ownership check)
    async fn add_graph_node(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
        #[graphql(default = "document")] node_type: String,
        #[graphql(default)] metadata: Option<async_graphql::Json<serde_json::Value>>,
    ) -> async_graphql::Result<GqlNode> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let mut node = Node::new(node_id.clone(), node_type);

        // Add metadata if provided
        if let Some(meta) = metadata {
            if let Some(obj) = meta.0.as_object() {
                for (key, value) in obj {
                    node.metadata.insert(key.clone(), value.clone());
                }
            }
        }

        graph
            .add_node(node.clone())
            .map_err(|e| async_graphql::Error::new(format!("Failed to add node: {e}")))?;

        info!(
            "GraphQL: Added node '{}' to graph '{}'",
            node_id, collection
        );

        Ok(node.into())
    }

    /// Remove a node and all its edges from the graph (with tenant ownership check)
    async fn remove_graph_node(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        match graph.remove_node(&node_id) {
            Ok(_) => {
                info!(
                    "GraphQL: Removed node '{}' from graph '{}'",
                    node_id, collection
                );
                Ok(MutationResult::ok_with_message(format!(
                    "Node '{node_id}' removed"
                )))
            }
            Err(e) => Ok(MutationResult::err(format!("Failed to remove node: {e}"))),
        }
    }

    /// Create an edge between two nodes (with tenant ownership check)
    async fn create_graph_edge(
        &self,
        ctx: &Context<'_>,
        collection: String,
        input: CreateEdgeInput,
    ) -> async_graphql::Result<GqlEdge> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        let rel_type: RelationshipType = input.relationship_type.into();
        let edge_id = format!("{}:{}:{}", input.source, input.target, rel_type as u8);

        let edge = Edge::new(
            edge_id,
            input.source.clone(),
            input.target.clone(),
            rel_type,
            input.weight,
        );

        graph
            .add_edge(edge.clone())
            .map_err(|e| async_graphql::Error::new(format!("Failed to create edge: {e}")))?;

        info!(
            "GraphQL: Created edge from '{}' to '{}' in graph '{}'",
            input.source, input.target, collection
        );

        Ok(edge.into())
    }

    /// Delete an edge from the graph (with tenant ownership check)
    async fn delete_graph_edge(
        &self,
        ctx: &Context<'_>,
        collection: String,
        edge_id: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Verify ownership
        check_collection_ownership(&gql_ctx.store, &collection, tenant_ctx)?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        match graph.remove_edge(&edge_id) {
            Ok(_) => {
                info!(
                    "GraphQL: Removed edge '{}' from graph '{}'",
                    edge_id, collection
                );
                Ok(MutationResult::ok_with_message(format!(
                    "Edge '{edge_id}' removed"
                )))
            }
            Err(e) => Ok(MutationResult::err(format!("Failed to remove edge: {e}"))),
        }
    }

    // =========================================================================
    // WORKSPACE MUTATIONS
    // =========================================================================

    /// Add a workspace directory for indexing
    async fn add_workspace(
        &self,
        _ctx: &Context<'_>,
        input: AddWorkspaceInput,
    ) -> async_graphql::Result<MutationResult> {
        info!(
            "GraphQL: Adding workspace: {} -> {}",
            input.path, input.collection_name
        );

        let workspace_manager = crate::config::WorkspaceManager::new();
        match workspace_manager.add_workspace(&input.path, &input.collection_name) {
            Ok(workspace) => Ok(MutationResult::ok_with_message(format!(
                "Workspace '{}' added for collection '{}' (id: {})",
                workspace.path, workspace.collection_name, workspace.id
            ))),
            Err(e) => {
                error!("Failed to add workspace: {}", e);
                Ok(MutationResult::err(&e))
            }
        }
    }

    /// Remove a workspace directory
    async fn remove_workspace(
        &self,
        _ctx: &Context<'_>,
        path: String,
    ) -> async_graphql::Result<MutationResult> {
        info!("GraphQL: Removing workspace: {}", path);

        let workspace_manager = crate::config::WorkspaceManager::new();
        match workspace_manager.remove_workspace(&path) {
            Ok(workspace) => Ok(MutationResult::ok_with_message(format!(
                "Workspace '{}' removed (collection: {})",
                workspace.path, workspace.collection_name
            ))),
            Err(e) => {
                error!("Failed to remove workspace: {}", e);
                Ok(MutationResult::err(&e))
            }
        }
    }

    /// Update workspace configuration
    async fn update_workspace_config(
        &self,
        _ctx: &Context<'_>,
        config: async_graphql::Json<serde_json::Value>,
    ) -> async_graphql::Result<MutationResult> {
        // Write to workspace.yml
        match serde_yaml::to_string(&config.0) {
            Ok(yaml_content) => match std::fs::write("./workspace.yml", yaml_content) {
                Ok(_) => {
                    info!("GraphQL: Workspace configuration updated successfully");
                    Ok(MutationResult::ok_with_message(
                        "Workspace configuration updated",
                    ))
                }
                Err(e) => {
                    error!("GraphQL: Failed to write workspace config: {}", e);
                    Ok(MutationResult::err(format!(
                        "Failed to write workspace config: {}",
                        e
                    )))
                }
            },
            Err(e) => {
                error!("GraphQL: Failed to serialize workspace config: {}", e);
                Ok(MutationResult::err(format!(
                    "Failed to serialize workspace config: {}",
                    e
                )))
            }
        }
    }

    // =========================================================================
    // FILE UPLOAD MUTATIONS
    // =========================================================================

    /// Upload a file for indexing (base64-encoded content)
    ///
    /// This mutation accepts a file as base64-encoded content and processes it
    /// into chunks, generates embeddings, and stores them in the specified collection.
    async fn upload_file(
        &self,
        ctx: &Context<'_>,
        input: UploadFileInput,
    ) -> async_graphql::Result<GqlFileUploadResult> {
        let start_time = std::time::Instant::now();
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let tenant_ctx = ctx.data_opt::<TenantContext>();

        // Load file upload config
        let upload_config = load_file_upload_config();

        // Decode base64 content
        let file_bytes = match base64_decode(&input.content_base64) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Ok(GqlFileUploadResult::error_result(
                    input.filename.clone(),
                    input.collection_name.clone(),
                    format!("Failed to decode base64 content: {}", e),
                ));
            }
        };

        // Validate file extension
        let extension = std::path::Path::new(&input.filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !upload_config.allowed_extensions.contains(&extension) {
            return Ok(GqlFileUploadResult::error_result(
                input.filename.clone(),
                input.collection_name.clone(),
                format!("File extension '{}' is not allowed", extension),
            ));
        }

        // Validate file size
        if file_bytes.len() > upload_config.max_file_size {
            return Ok(GqlFileUploadResult::error_result(
                input.filename.clone(),
                input.collection_name.clone(),
                format!(
                    "File size {} exceeds maximum of {} bytes",
                    file_bytes.len(),
                    upload_config.max_file_size
                ),
            ));
        }

        // Check for binary content if rejection is enabled
        if upload_config.reject_binary && is_binary_content(&file_bytes) {
            return Ok(GqlFileUploadResult::error_result(
                input.filename.clone(),
                input.collection_name.clone(),
                "Binary files are not allowed".to_string(),
            ));
        }

        // Convert to string
        let content = String::from_utf8_lossy(&file_bytes).into_owned();
        let file_size = file_bytes.len() as i32;

        // Determine language from extension
        let language = get_language_from_extension(&extension);

        // Apply tenant prefix if in hub mode
        let collection_name = if let Some(tenant) = tenant_ctx {
            format!("user_{}_{}", tenant.tenant_id, input.collection_name)
        } else {
            input.collection_name.clone()
        };

        // Check if collection exists, create if not
        if !gql_ctx.store.has_collection_in_memory(&collection_name) {
            let config = CollectionConfig {
                dimension: 512,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: Default::default(),
                normalization: None,
                storage_type: Some(crate::models::StorageType::Memory),
                sharding: None,
                graph: None,
                encryption: None,
            };

            if let Err(e) = gql_ctx
                .store
                .create_collection_with_quantization(&collection_name, config)
            {
                return Ok(GqlFileUploadResult::error_result(
                    input.filename.clone(),
                    input.collection_name.clone(),
                    format!("Failed to create collection: {}", e),
                ));
            }
            info!("GraphQL: Created new collection: {}", collection_name);
        }

        // Create chunks
        let loader_config = LoaderConfig {
            max_chunk_size: input
                .chunk_size
                .unwrap_or(upload_config.default_chunk_size as i32)
                as usize,
            chunk_overlap: input
                .chunk_overlap
                .unwrap_or(upload_config.default_chunk_overlap as i32)
                as usize,
            include_patterns: vec![],
            exclude_patterns: vec![],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: collection_name.clone(),
            max_file_size: upload_config.max_file_size,
        };

        let chunker = Chunker::new(loader_config);
        let file_path = std::path::PathBuf::from(&input.filename);

        let chunks = match chunker.chunk_text(&content, &file_path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(GqlFileUploadResult::error_result(
                    input.filename.clone(),
                    input.collection_name.clone(),
                    format!("Failed to chunk file: {}", e),
                ));
            }
        };

        let chunks_created = chunks.len() as i32;

        if chunks_created == 0 {
            return Ok(GqlFileUploadResult::success_result(
                input.filename,
                input.collection_name,
                0,
                0,
                file_size,
                language.to_string(),
                start_time.elapsed().as_millis() as i64,
            ));
        }

        // Create embeddings and store vectors
        let mut vectors_created = 0i32;

        for chunk in &chunks {
            let embedding = match gql_ctx.embedding_manager.embed(&chunk.content) {
                Ok(emb) => emb,
                Err(_) => continue,
            };

            if embedding.iter().all(|&x| x == 0.0) {
                continue;
            }

            let mut payload_data = serde_json::json!({
                "content": chunk.content,
                "file_path": chunk.file_path,
                "chunk_index": chunk.chunk_index,
                "language": language,
                "source": "graphql_upload",
                "original_filename": input.filename,
                "file_extension": extension,
            });

            // Merge chunk metadata
            if let Some(obj) = payload_data.as_object_mut() {
                for (k, v) in &chunk.metadata {
                    obj.insert(k.clone(), v.clone());
                }

                // Merge extra metadata if provided
                if let Some(ref extra) = input.metadata {
                    if let Some(extra_obj) = extra.0.as_object() {
                        for (k, v) in extra_obj {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }

            // Create payload with optional encryption
            let mut payload = if let Some(ref key) = input.public_key {
                match crate::security::payload_encryption::encrypt_payload(&payload_data, key) {
                    Ok(encrypted) => Payload::from_encrypted(encrypted),
                    Err(e) => {
                        warn!("GraphQL: Failed to encrypt chunk payload: {}", e);
                        continue;
                    }
                }
            } else {
                Payload { data: payload_data }
            };
            payload.normalize();

            let vector = Vector {
                id: uuid::Uuid::new_v4().to_string(),
                data: embedding,
                sparse: None,
                payload: Some(payload),
            };

            if gql_ctx.store.insert(&collection_name, vec![vector]).is_ok() {
                vectors_created += 1;
            }
        }

        let processing_time_ms = start_time.elapsed().as_millis() as i64;

        info!(
            "GraphQL: File upload completed: {} - {} chunks, {} vectors, {}ms",
            input.filename, chunks_created, vectors_created, processing_time_ms
        );

        Ok(GqlFileUploadResult::success_result(
            input.filename,
            input.collection_name,
            chunks_created,
            vectors_created,
            file_size,
            language.to_string(),
            processing_time_ms,
        ))
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Load file upload configuration from config.yml
fn load_file_upload_config() -> FileUploadConfig {
    std::fs::read_to_string("config.yml")
        .ok()
        .and_then(|content| {
            serde_yaml::from_str::<crate::config::VectorizerConfig>(&content)
                .ok()
                .map(|config| config.file_upload)
        })
        .unwrap_or_default()
}

/// Decode base64 string to bytes
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| e.to_string())
}

/// Check if content appears to be binary
fn is_binary_content(content: &[u8]) -> bool {
    let check_size = content.len().min(8192);
    let sample = &content[..check_size];

    let mut null_count = 0;
    let mut non_printable_count = 0;

    for &byte in sample {
        if byte == 0 {
            null_count += 1;
        } else if byte < 0x09 || (byte > 0x0D && byte < 0x20 && byte != 0x1B) {
            non_printable_count += 1;
        }
    }

    let null_ratio = null_count as f32 / check_size as f32;
    let non_printable_ratio = non_printable_count as f32 / check_size as f32;

    null_ratio > 0.01 || non_printable_ratio > 0.10
}

/// Get language from file extension
fn get_language_from_extension(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "rs" => "rust",
        "py" | "pyw" | "pyi" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "jsx" => "javascriptreact",
        "tsx" => "typescriptreact",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" | "rake" | "gemspec" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "r" => "r",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        "ps1" | "psm1" | "psd1" => "powershell",
        "bat" | "cmd" => "batch",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "ini" | "cfg" | "conf" => "ini",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        "md" | "markdown" => "markdown",
        "rst" => "restructuredtext",
        "txt" | "text" => "plaintext",
        "csv" => "csv",
        "log" => "log",
        _ => "plaintext",
    }
}

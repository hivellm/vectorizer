//! `QueryRoot` and its `async-graphql` resolvers — extracted from the prior
//! monolithic `schema.rs` (phase4_split-graphql-schema). The struct +
//! `#[Object] impl` block are byte-for-byte the same; only the file
//! they live in is new.

use std::sync::Arc;

use async_graphql::{Context, Object};
use tracing::{info, warn};

use super::super::types::*;
use super::{GraphQLContext, check_collection_ownership, load_file_upload_config};
use crate::hub::auth::TenantContext;

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

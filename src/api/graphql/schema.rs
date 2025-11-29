//! GraphQL Schema and Resolvers for Vectorizer
//!
//! This module defines the GraphQL schema including Query and Mutation types.

use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Schema};
use tracing::{error, info};

use super::types::*;
use crate::db::VectorStore;
use crate::db::graph::{Edge, Node, RelationshipType};
use crate::embedding::EmbeddingManager;
use crate::models::{CollectionConfig, HnswConfig, Payload, Vector};

/// GraphQL context containing shared state
pub struct GraphQLContext {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
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
    };

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        // Limit query depth to prevent deeply nested queries
        .limit_depth(10)
        // Limit query complexity to prevent expensive queries
        .limit_complexity(1000)
        .finish()
}

// =============================================================================
// QUERY ROOT
// =============================================================================

/// Root query object for GraphQL
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get all collections
    async fn collections(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<GqlCollection>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let collection_names = gql_ctx.store.list_collections();

        let mut collections = Vec::new();
        for name in collection_names {
            if let Ok(meta) = gql_ctx.store.get_collection_metadata(&name) {
                collections.push(meta.into());
            }
        }

        Ok(collections)
    }

    /// Get a specific collection by name
    async fn collection(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<Option<GqlCollection>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        match gql_ctx.store.get_collection_metadata(&name) {
            Ok(meta) => Ok(Some(meta.into())),
            Err(_) => Ok(None),
        }
    }

    /// Get a vector by ID from a collection
    async fn vector(
        &self,
        ctx: &Context<'_>,
        collection: String,
        id: String,
    ) -> async_graphql::Result<Option<GqlVector>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        match gql_ctx.store.get_vector(&collection, &id) {
            Ok(v) => Ok(Some(v.into())),
            Err(_) => Ok(None), // Vector not found
        }
    }

    /// List vectors in a collection with pagination
    async fn vectors(
        &self,
        ctx: &Context<'_>,
        input: ScrollInput,
    ) -> async_graphql::Result<GqlPage<GqlVector>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Semantic vector search
    async fn search(
        &self,
        ctx: &Context<'_>,
        input: SearchInput,
    ) -> async_graphql::Result<Vec<GqlSearchResult>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Get server statistics
    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<GqlServerStats> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let collection_names = gql_ctx.store.list_collections();
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

    /// Get graph statistics for a collection
    async fn graph_stats(
        &self,
        ctx: &Context<'_>,
        collection: String,
    ) -> async_graphql::Result<GqlGraphStats> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Get all nodes in a collection's graph
    async fn graph_nodes(
        &self,
        ctx: &Context<'_>,
        collection: String,
        #[graphql(default = 100)] limit: i32,
        #[graphql(default)] cursor: Option<String>,
    ) -> async_graphql::Result<GqlPage<GqlNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Get all edges in a collection's graph
    async fn graph_edges(
        &self,
        ctx: &Context<'_>,
        collection: String,
        #[graphql(default = 100)] limit: i32,
        #[graphql(default)] cursor: Option<String>,
    ) -> async_graphql::Result<GqlPage<GqlEdge>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Get a specific node by ID
    async fn graph_node(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
    ) -> async_graphql::Result<Option<GqlNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let collection_ref = gql_ctx
            .store
            .get_collection(&collection)
            .map_err(|e| async_graphql::Error::new(format!("Collection not found: {e}")))?;

        let graph = collection_ref
            .get_graph()
            .ok_or_else(|| async_graphql::Error::new("Graph not enabled for this collection"))?;

        Ok(graph.get_node(&node_id).map(|n| n.into()))
    }

    /// Get neighbors of a node (nodes connected by edges)
    async fn graph_neighbors(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
        #[graphql(default)] relationship_type: Option<GqlRelationshipType>,
    ) -> async_graphql::Result<Vec<GqlRelatedNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Find nodes related to a source node within N hops
    async fn graph_related(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
        #[graphql(default = 2)] max_hops: i32,
        #[graphql(default)] relationship_type: Option<GqlRelationshipType>,
    ) -> async_graphql::Result<Vec<GqlRelatedNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Find shortest path between two nodes
    async fn graph_path(
        &self,
        ctx: &Context<'_>,
        collection: String,
        source: String,
        target: String,
    ) -> async_graphql::Result<Vec<GqlNode>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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
        // TODO: Implement workspace manager integration
        // For now, return empty list
        Ok(Vec::new())
    }

    /// Get workspace configuration
    async fn workspace_config(
        &self,
        _ctx: &Context<'_>,
    ) -> async_graphql::Result<GqlWorkspaceConfig> {
        let possible_paths = vec![
            "./vectorize-workspace.yml",
            "../vectorize-workspace.yml",
            "../../vectorize-workspace.yml",
            "./config/vectorize-workspace.yml",
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
    /// Create a new collection
    async fn create_collection(
        &self,
        ctx: &Context<'_>,
        input: CreateCollectionInput,
    ) -> async_graphql::Result<GqlCollection> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

        // Force CPU if requested or if graph is enabled (graphs not supported on GPU)
        let force_cpu = input.force_cpu.unwrap_or(false) || enable_graph;
        if force_cpu {
            gql_ctx
                .store
                .create_collection_cpu_only(&input.name, config)
                .map_err(|e| {
                    async_graphql::Error::new(format!("Failed to create collection: {e}"))
                })?;
        } else {
            gql_ctx
                .store
                .create_collection(&input.name, config)
                .map_err(|e| {
                    async_graphql::Error::new(format!("Failed to create collection: {e}"))
                })?;
        }

        info!("GraphQL: Created collection '{}'", input.name);

        // Return the created collection metadata
        let meta = gql_ctx
            .store
            .get_collection_metadata(&input.name)
            .map_err(|e| async_graphql::Error::new(format!("Failed to get metadata: {e}")))?;

        Ok(meta.into())
    }

    /// Delete a collection
    async fn delete_collection(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        match gql_ctx.store.delete_collection(&name) {
            Ok(_) => {
                info!("GraphQL: Deleted collection '{name}'");
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

    /// Upsert a single vector
    async fn upsert_vector(
        &self,
        ctx: &Context<'_>,
        collection: String,
        input: UpsertVectorInput,
    ) -> async_graphql::Result<GqlVector> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let payload = input.payload.map(|p| Payload::new(p.0));

        let vector = if let Some(p) = payload {
            Vector::with_payload(input.id.clone(), input.data.clone(), p)
        } else {
            Vector::new(input.id.clone(), input.data.clone())
        };

        gql_ctx
            .store
            .insert(&collection, vec![vector.clone()])
            .map_err(|e| async_graphql::Error::new(format!("Failed to upsert vector: {e}")))?;

        Ok(vector.into())
    }

    /// Upsert multiple vectors in batch
    async fn upsert_vectors(
        &self,
        ctx: &Context<'_>,
        input: UpsertVectorsInput,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let vectors: Vec<Vector> = input
            .vectors
            .into_iter()
            .map(|v_input| {
                let payload = v_input.payload.map(|p| Payload::new(p.0));
                if let Some(p) = payload {
                    Vector::with_payload(v_input.id, v_input.data, p)
                } else {
                    Vector::new(v_input.id, v_input.data)
                }
            })
            .collect();

        let count = vectors.len() as i32;

        gql_ctx
            .store
            .insert(&input.collection, vectors)
            .map_err(|e| async_graphql::Error::new(format!("Failed to upsert vectors: {e}")))?;

        info!(
            "GraphQL: Upserted {count} vectors in '{}'",
            input.collection
        );
        Ok(MutationResult::ok_with_count(count))
    }

    /// Delete a vector by ID
    async fn delete_vector(
        &self,
        ctx: &Context<'_>,
        collection: String,
        id: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        match gql_ctx.store.delete(&collection, &id) {
            Ok(_) => Ok(MutationResult::ok_with_message(format!(
                "Vector '{id}' deleted"
            ))),
            Err(e) => Ok(MutationResult::err(format!("Failed to delete vector: {e}"))),
        }
    }

    /// Update vector payload
    async fn update_payload(
        &self,
        ctx: &Context<'_>,
        collection: String,
        id: String,
        payload: async_graphql::Json<serde_json::Value>,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Get existing vector
        let existing = gql_ctx
            .store
            .get_vector(&collection, &id)
            .map_err(|e| async_graphql::Error::new(format!("Vector not found: {e}")))?;

        // Update with new payload
        let updated = Vector::with_payload(existing.id, existing.data, Payload::new(payload.0));

        gql_ctx
            .store
            .insert(&collection, vec![updated])
            .map_err(|e| async_graphql::Error::new(format!("Failed to update payload: {e}")))?;

        Ok(MutationResult::ok_with_message("Payload updated"))
    }

    // =========================================================================
    // GRAPH MUTATIONS
    // =========================================================================

    /// Enable graph for a collection (creates nodes and discovers edges)
    async fn enable_graph(
        &self,
        ctx: &Context<'_>,
        collection: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Add a node to the graph
    async fn add_graph_node(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
        #[graphql(default = "document")] node_type: String,
        #[graphql(default)] metadata: Option<async_graphql::Json<serde_json::Value>>,
    ) -> async_graphql::Result<GqlNode> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Remove a node and all its edges from the graph
    async fn remove_graph_node(
        &self,
        ctx: &Context<'_>,
        collection: String,
        node_id: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Create an edge between two nodes
    async fn create_graph_edge(
        &self,
        ctx: &Context<'_>,
        collection: String,
        input: CreateEdgeInput,
    ) -> async_graphql::Result<GqlEdge> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

    /// Delete an edge from the graph
    async fn delete_graph_edge(
        &self,
        ctx: &Context<'_>,
        collection: String,
        edge_id: String,
    ) -> async_graphql::Result<MutationResult> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

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

        // TODO: Implement workspace manager integration
        Ok(MutationResult::ok_with_message(format!(
            "Workspace '{}' added for collection '{}'",
            input.path, input.collection_name
        )))
    }

    /// Remove a workspace directory
    async fn remove_workspace(
        &self,
        _ctx: &Context<'_>,
        path: String,
    ) -> async_graphql::Result<MutationResult> {
        info!("GraphQL: Removing workspace: {}", path);

        // TODO: Implement workspace manager integration
        Ok(MutationResult::ok_with_message(format!(
            "Workspace '{}' removed",
            path
        )))
    }

    /// Update workspace configuration
    async fn update_workspace_config(
        &self,
        _ctx: &Context<'_>,
        config: async_graphql::Json<serde_json::Value>,
    ) -> async_graphql::Result<MutationResult> {
        // Write to vectorize-workspace.yml
        match serde_yaml::to_string(&config.0) {
            Ok(yaml_content) => match std::fs::write("./vectorize-workspace.yml", yaml_content) {
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
}

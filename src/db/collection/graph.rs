//! Graph integration — enable graph, accessors, and lazy node/edge
//! population from existing vectors.
//!
//! Kept separate from [`data`] because graph population is a distinct
//! concern that runs lazily (on cache load or explicit enable) rather
//! than on every insert. All relationship-discovery logic lives in
//! [`crate::db::graph_relationship_discovery`]; this module just wires
//! the [`Collection`] into it.
//!
//! [`data`]: super::data

use std::sync::Arc;

use tracing::{debug, info};

use super::Collection;
use crate::error::Result;

impl Collection {
    /// Get graph reference (if enabled)
    pub fn get_graph(&self) -> Option<&Arc<crate::db::graph::Graph>> {
        self.graph.as_ref()
    }

    /// Set graph directly (used for loading from persistence)
    pub fn set_graph(&mut self, graph: Arc<crate::db::graph::Graph>) {
        self.graph = Some(graph);
    }

    /// Populate graph with nodes from existing vectors if graph is empty
    /// This is useful when loading a graph from disk that has no nodes
    pub fn populate_graph_if_empty(&mut self) -> Result<()> {
        use crate::db::graph::Node;

        let graph = match &self.graph {
            Some(g) => g,
            None => return Ok(()), // No graph, nothing to populate
        };

        // Check if graph is empty
        if graph.node_count() > 0 {
            return Ok(()); // Graph already has nodes
        }

        info!(
            "Graph for collection '{}' is empty, populating with nodes from existing vectors",
            self.name
        );

        // Save edges from current graph (if any)
        let old_edges = graph.get_all_edges();
        let edge_count = old_edges.len();

        // Get all vector IDs
        let vector_ids: Vec<String> = {
            let vector_order = self.vector_order.read();
            vector_order.iter().cloned().collect()
        };

        // Create nodes for all existing vectors
        let mut nodes_created = 0;
        for vector_id in &vector_ids {
            if let Ok(vector) = self.get_vector(vector_id) {
                let node = Node::from_vector(&vector.id, vector.payload.as_ref());
                if let Err(e) = graph.add_node(node) {
                    debug!("Failed to add graph node for vector '{}': {}", vector_id, e);
                } else {
                    nodes_created += 1;
                }
            }
        }

        info!(
            "Populated graph for collection '{}' with {} nodes (out of {} vectors)",
            self.name,
            nodes_created,
            vector_ids.len()
        );

        // Preserve edges from original graph (if any)
        if edge_count > 0 {
            for edge in &old_edges {
                let _ = graph.add_edge(edge.clone());
            }
            info!(
                "Preserved {} edges from previous graph for collection '{}'",
                edge_count, self.name
            );
        } else {
            // If no edges exist, discover edges automatically
            // Use default config if graph config doesn't have auto_relationship enabled
            let discovery_config = if let Some(graph_config) = &self.config.graph {
                if !graph_config.auto_relationship.enabled_types.is_empty() {
                    Some(graph_config.auto_relationship.clone())
                } else {
                    // Use default config for auto-discovery
                    Some(crate::models::AutoRelationshipConfig {
                        similarity_threshold: 0.7,
                        max_per_node: 10,
                        enabled_types: vec!["SIMILAR_TO".to_string()],
                    })
                }
            } else {
                // No graph config, use default for auto-discovery
                Some(crate::models::AutoRelationshipConfig {
                    similarity_threshold: 0.7,
                    max_per_node: 10,
                    enabled_types: vec!["SIMILAR_TO".to_string()],
                })
            };

            // Limit to first 100 nodes to avoid blocking
            if let Some(config) = discovery_config {
                let nodes = graph.get_all_nodes();
                let nodes_to_process: Vec<String> =
                    nodes.iter().take(100).map(|n| n.id.clone()).collect();

                if !nodes_to_process.is_empty() {
                    info!(
                        "Auto-discovering edges for collection '{}' (limited to first {} nodes)",
                        self.name,
                        nodes_to_process.len()
                    );

                    let mut edges_created = 0;
                    for node_id in &nodes_to_process {
                        if let Ok(_edges) =
                            crate::db::graph_relationship_discovery::discover_edges_for_node(
                                graph, node_id, self, &config,
                            )
                        {
                            edges_created += _edges;
                        }
                    }

                    info!(
                        "Auto-discovery created {} edges for {} nodes in collection '{}'",
                        edges_created,
                        nodes_to_process.len(),
                        self.name
                    );
                }
            }
        }

        Ok(())
    }

    /// Enable graph for this collection and populate with existing vectors
    /// This creates graph nodes for all existing vectors in the collection
    /// and discovers relationships based on metadata (fast) and optionally similarity (slower)
    pub fn enable_graph(&mut self) -> Result<()> {
        use crate::db::graph::Node;

        // If graph already exists, check if it needs edges discovered
        if let Some(existing_graph) = &self.graph {
            let edge_count = existing_graph.edge_count();
            let node_count = existing_graph.node_count();

            // If graph has nodes but no edges, discover edges automatically
            if edge_count == 0 && node_count > 0 {
                info!(
                    "Graph for collection '{}' already enabled with {} nodes but no edges, discovering edges automatically",
                    self.name, node_count
                );

                // Use default config for auto-discovery
                let discovery_config = if let Some(graph_config) = &self.config.graph {
                    if !graph_config.auto_relationship.enabled_types.is_empty() {
                        graph_config.auto_relationship.clone()
                    } else {
                        crate::models::AutoRelationshipConfig {
                            similarity_threshold: 0.7,
                            max_per_node: 10,
                            enabled_types: vec!["SIMILAR_TO".to_string()],
                        }
                    }
                } else {
                    crate::models::AutoRelationshipConfig {
                        similarity_threshold: 0.7,
                        max_per_node: 10,
                        enabled_types: vec!["SIMILAR_TO".to_string()],
                    }
                };

                // Limit to first 100 nodes to avoid blocking
                let nodes = existing_graph.get_all_nodes();
                let nodes_to_process: Vec<String> =
                    nodes.iter().take(100).map(|n| n.id.clone()).collect();

                let mut edges_created = 0;
                for node_id in &nodes_to_process {
                    if let Ok(_edges) =
                        crate::db::graph_relationship_discovery::discover_edges_for_node(
                            existing_graph,
                            node_id,
                            self,
                            &discovery_config,
                        )
                    {
                        edges_created += _edges;
                    }
                }

                info!(
                    "Auto-discovery created {} edges for {} nodes in collection '{}' (use API endpoint /graph/discover/{} for full discovery)",
                    edges_created,
                    nodes_to_process.len().min(node_count),
                    self.name,
                    self.name
                );
            } else {
                info!(
                    "Graph already enabled for collection '{}' with {} nodes and {} edges",
                    self.name, node_count, edge_count
                );
            }
            return Ok(());
        }

        info!("Enabling graph for collection '{}'", self.name);

        // Create new graph
        let graph = Arc::new(crate::db::graph::Graph::new(self.name.clone()));

        // Set graph field immediately
        self.graph = Some(graph.clone());

        // Create nodes for ALL existing vectors (no limit - nodes are lightweight)
        let vector_ids: Vec<String> = {
            let vector_order = self.vector_order.read();
            vector_order.iter().cloned().collect() // Create nodes for ALL vectors
        };

        if !vector_ids.is_empty() {
            info!(
                "Creating graph nodes for {} existing vectors in collection '{}'",
                vector_ids.len(),
                self.name
            );

            let mut nodes_created = 0;
            for vector_id in &vector_ids {
                if let Ok(vector) = self.get_vector(vector_id) {
                    let node = Node::from_vector(&vector.id, vector.payload.as_ref());
                    if let Err(e) = graph.add_node(node) {
                        debug!("Failed to add graph node for vector '{}': {}", vector_id, e);
                    } else {
                        nodes_created += 1;
                    }
                }
            }

            info!(
                "Created {} graph nodes for collection '{}' (out of {} vectors)",
                nodes_created,
                self.name,
                vector_ids.len()
            );
        }

        info!("Graph enabled for collection '{}'", self.name);

        // Discover edges automatically for existing collections
        // Use default config if graph config doesn't have auto_relationship enabled
        let discovery_config = if let Some(graph_config) = &self.config.graph {
            if !graph_config.auto_relationship.enabled_types.is_empty() {
                Some(graph_config.auto_relationship.clone())
            } else {
                // Use default config for auto-discovery
                Some(crate::models::AutoRelationshipConfig {
                    similarity_threshold: 0.7,
                    max_per_node: 10,
                    enabled_types: vec!["SIMILAR_TO".to_string()],
                })
            }
        } else {
            // No graph config, use default for auto-discovery
            Some(crate::models::AutoRelationshipConfig {
                similarity_threshold: 0.7,
                max_per_node: 10,
                enabled_types: vec!["SIMILAR_TO".to_string()],
            })
        };

        // Limit to first 100 nodes to avoid blocking (full discovery can be done via API)
        if let Some(config) = discovery_config {
            let node_count = graph.node_count();
            if node_count > 0 {
                info!(
                    "Auto-discovering edges for collection '{}' (limited to first 100 nodes to avoid blocking)",
                    self.name
                );

                // Limit to first 100 nodes for auto-discovery
                let nodes_to_process: Vec<String> = {
                    let vector_order = self.vector_order.read();
                    vector_order.iter().cloned().take(100).collect()
                };

                let mut edges_created = 0;
                for node_id in &nodes_to_process {
                    if let Ok(_edges) =
                        crate::db::graph_relationship_discovery::discover_edges_for_node(
                            &graph, node_id, self, &config,
                        )
                    {
                        edges_created += _edges;
                    }
                }

                info!(
                    "Auto-discovery created {} edges for {} nodes in collection '{}' (use API endpoint /graph/discover/{} for full discovery)",
                    edges_created,
                    nodes_to_process.len().min(node_count),
                    self.name,
                    self.name
                );
            }
        }

        Ok(())
    }
}

//! Graph surface: nodes, edges, neighbours, paths, discovery.
//!
//! 10 methods covering the explicit-graph endpoints + the auto-
//! discovery pipeline that infers `SIMILAR_TO` edges from vector
//! distance.

use crate::error::{Result, VectorizerError};
use crate::models::*;

use super::VectorizerClient;

impl VectorizerClient {
    /// List every node in a collection's graph.
    pub async fn list_graph_nodes(&self, collection: &str) -> Result<ListNodesResponse> {
        let url = format!("/graph/nodes/{collection}");
        let response = self.make_request("GET", &url, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list nodes response: {e}"))
        })
    }

    /// Get neighbours of a specific node.
    pub async fn get_graph_neighbors(
        &self,
        collection: &str,
        node_id: &str,
    ) -> Result<GetNeighborsResponse> {
        let url = format!("/graph/nodes/{collection}/{node_id}/neighbors");
        let response = self.make_request("GET", &url, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse neighbors response: {e}"))
        })
    }

    /// Find related nodes within N hops.
    pub async fn find_related_nodes(
        &self,
        collection: &str,
        node_id: &str,
        request: FindRelatedRequest,
    ) -> Result<FindRelatedResponse> {
        let url = format!("/graph/nodes/{collection}/{node_id}/related");
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self.make_request("POST", &url, Some(payload)).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse related nodes response: {e}"))
        })
    }

    /// Find the shortest path between two nodes.
    pub async fn find_graph_path(&self, request: FindPathRequest) -> Result<FindPathResponse> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/graph/path", Some(payload))
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse path response: {e}")))
    }

    /// Create an explicit edge between two nodes.
    pub async fn create_graph_edge(
        &self,
        request: CreateEdgeRequest,
    ) -> Result<CreateEdgeResponse> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/graph/edges", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse create edge response: {e}"))
        })
    }

    /// Delete an edge by id.
    pub async fn delete_graph_edge(&self, edge_id: &str) -> Result<()> {
        let url = format!("/graph/edges/{edge_id}");
        self.make_request("DELETE", &url, None).await?;
        Ok(())
    }

    /// List every edge in a collection.
    pub async fn list_graph_edges(&self, collection: &str) -> Result<ListEdgesResponse> {
        let url = format!("/graph/collections/{collection}/edges");
        let response = self.make_request("GET", &url, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list edges response: {e}"))
        })
    }

    /// Discover SIMILAR_TO edges for an entire collection.
    pub async fn discover_graph_edges(
        &self,
        collection: &str,
        request: DiscoverEdgesRequest,
    ) -> Result<DiscoverEdgesResponse> {
        let url = format!("/graph/discover/{collection}");
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self.make_request("POST", &url, Some(payload)).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discover edges response: {e}"))
        })
    }

    /// Discover SIMILAR_TO edges for one specific node.
    pub async fn discover_graph_edges_for_node(
        &self,
        collection: &str,
        node_id: &str,
        request: DiscoverEdgesRequest,
    ) -> Result<DiscoverEdgesResponse> {
        let url = format!("/graph/discover/{collection}/{node_id}");
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self.make_request("POST", &url, Some(payload)).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discover edges response: {e}"))
        })
    }

    /// Get discovery status for a collection.
    pub async fn get_graph_discovery_status(
        &self,
        collection: &str,
    ) -> Result<DiscoveryStatusResponse> {
        let url = format!("/graph/discover/{collection}/status");
        let response = self.make_request("GET", &url, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discovery status response: {e}"))
        })
    }
}

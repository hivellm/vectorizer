"""Graph operations surface — nodes, edges, path-finding, discovery."""

from __future__ import annotations

import logging
from dataclasses import asdict

import aiohttp

try:
    from ..exceptions import NetworkError, ServerError, ValidationError
    from ..models import (
        CreateEdgeRequest,
        CreateEdgeResponse,
        DiscoverEdgesRequest,
        DiscoverEdgesResponse,
        DiscoveryStatusResponse,
        FindPathRequest,
        FindPathResponse,
        FindRelatedRequest,
        FindRelatedResponse,
        GetNeighborsResponse,
        ListEdgesResponse,
        ListNodesResponse,
    )
    from ..utils.validation import validate_non_empty_string
except ImportError:  # pragma: no cover
    from exceptions import NetworkError, ServerError, ValidationError
    from models import (
        CreateEdgeRequest,
        CreateEdgeResponse,
        DiscoverEdgesRequest,
        DiscoverEdgesResponse,
        DiscoveryStatusResponse,
        FindPathRequest,
        FindPathResponse,
        FindRelatedRequest,
        FindRelatedResponse,
        GetNeighborsResponse,
        ListEdgesResponse,
        ListNodesResponse,
    )
    from utils.validation import validate_non_empty_string

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class GraphClient(_ApiBase):
    """Graph traversal, path-finding, and edge management."""

    async def list_graph_nodes(self, collection: str) -> ListNodesResponse:
        """List all nodes in a collection's graph."""
        try:
            validate_non_empty_string(collection)
            logger.debug(f"Listing graph nodes for collection: {collection}")

            data = await self._transport.get(f"/graph/nodes/{collection}")
            result = ListNodesResponse(**data)
            logger.debug(f"Graph nodes listed: {result.count} nodes found")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error listing graph nodes: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error listing graph nodes: {e}")
            raise NetworkError(f"Failed to list graph nodes: {e}")

    async def get_graph_neighbors(self, collection: str, node_id: str) -> GetNeighborsResponse:
        """Get neighbors of a specific node."""
        try:
            validate_non_empty_string(collection)
            validate_non_empty_string(node_id)
            logger.debug(f"Getting graph neighbors for node {node_id} in collection: {collection}")

            data = await self._transport.get(f"/graph/nodes/{collection}/{node_id}/neighbors")
            result = GetNeighborsResponse(**data)
            logger.debug(f"Graph neighbors retrieved: {len(result.neighbors)} neighbors found")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error getting graph neighbors: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error getting graph neighbors: {e}")
            raise NetworkError(f"Failed to get graph neighbors: {e}")

    async def find_related_nodes(
        self,
        collection: str,
        node_id: str,
        request: FindRelatedRequest
    ) -> FindRelatedResponse:
        """Find related nodes within N hops."""
        try:
            validate_non_empty_string(collection)
            validate_non_empty_string(node_id)
            logger.debug(f"Finding related nodes for node {node_id} in collection: {collection}")

            data = await self._transport.post(f"/graph/nodes/{collection}/{node_id}/related", data=asdict(request))
            result = FindRelatedResponse(**data)
            logger.debug(f"Related nodes found: {len(result.related)} nodes found")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error finding related nodes: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error finding related nodes: {e}")
            raise NetworkError(f"Failed to find related nodes: {e}")

    async def find_graph_path(self, request: FindPathRequest) -> FindPathResponse:
        """Find shortest path between two nodes."""
        try:
            logger.debug(f"Finding graph path from {request.source} to {request.target} in collection: {request.collection}")

            data = await self._transport.post("/graph/path", data=asdict(request))
            result = FindPathResponse(**data)
            logger.debug(f"Graph path found: found={result.found}, path_length={len(result.path)}")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error finding graph path: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error finding graph path: {e}")
            raise NetworkError(f"Failed to find graph path: {e}")

    async def create_graph_edge(self, request: CreateEdgeRequest) -> CreateEdgeResponse:
        """Create an explicit edge between two nodes."""
        try:
            logger.debug(f"Creating graph edge from {request.source} to {request.target} ({request.relationship_type}) in collection: {request.collection}")

            data = await self._transport.post("/graph/edges", data=asdict(request))
            result = CreateEdgeResponse(**data)
            logger.info(f"Graph edge created: edge_id={result.edge_id}, success={result.success}")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error creating graph edge: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error creating graph edge: {e}")
            raise NetworkError(f"Failed to create graph edge: {e}")

    async def delete_graph_edge(self, edge_id: str) -> bool:
        """Delete an edge by ID."""
        try:
            validate_non_empty_string(edge_id)
            logger.debug(f"Deleting graph edge: {edge_id}")

            data = await self._transport.delete(f"/graph/collections/{collection}/edges")
            result = ListEdgesResponse(**data)
            logger.debug(f"Graph edges listed: {result.count} edges found")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error listing graph edges: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error listing graph edges: {e}")
            raise NetworkError(f"Failed to list graph edges: {e}")

    async def discover_graph_edges(
        self,
        collection: str,
        request: DiscoverEdgesRequest
    ) -> DiscoverEdgesResponse:
        """Discover SIMILAR_TO edges for entire collection."""
        try:
            validate_non_empty_string(collection)
            logger.debug(f"Discovering graph edges for collection: {collection}")

            data = await self._transport.post(f"/graph/discover/{collection}", data=asdict(request))
            result = DiscoverEdgesResponse(**data)
            logger.info(f"Graph edges discovered: edges_created={result.edges_created}, success={result.success}")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error discovering graph edges: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error discovering graph edges: {e}")
            raise NetworkError(f"Failed to discover graph edges: {e}")

    async def discover_graph_edges_for_node(
        self,
        collection: str,
        node_id: str,
        request: DiscoverEdgesRequest
    ) -> DiscoverEdgesResponse:
        """Discover SIMILAR_TO edges for a specific node."""
        try:
            validate_non_empty_string(collection)
            validate_non_empty_string(node_id)
            logger.debug(f"Discovering graph edges for node {node_id} in collection: {collection}")

            data = await self._transport.post(f"/graph/discover/{collection}/{node_id}", data=asdict(request))
            result = DiscoverEdgesResponse(**data)
            logger.info(f"Graph edges discovered for node: edges_created={result.edges_created}, success={result.success}")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error discovering graph edges for node: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error discovering graph edges for node: {e}")
            raise NetworkError(f"Failed to discover graph edges: {e}")

    async def get_graph_discovery_status(self, collection: str) -> DiscoveryStatusResponse:
        """Get discovery status for a collection."""
        try:
            validate_non_empty_string(collection)
            logger.debug(f"Getting graph discovery status for collection: {collection}")

            data = await self._transport.get(f"/graph/discover/{collection}/status")
            result = DiscoveryStatusResponse(**data)
            logger.debug(f"Graph discovery status retrieved: progress={result.progress_percentage}%, total_edges={result.total_edges}")
            return result
        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error getting graph discovery status: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error getting graph discovery status: {e}")
            raise NetworkError(f"Failed to get discovery status: {e}")

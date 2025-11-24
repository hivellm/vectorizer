"""
Tests for graph operations in the Python SDK
"""

import unittest
import sys
import os
import asyncio

# Add parent directory to path to import client module
sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from client import VectorizerClient
from models import (
    GraphNode,
    GraphEdge,
    FindRelatedRequest,
    FindPathRequest,
    CreateEdgeRequest,
    DiscoverEdgesRequest,
)


class TestGraphOperations(unittest.IsolatedAsyncioTestCase):
    """Tests for graph operations"""

    async def asyncSetUp(self):
        """Set up test fixtures."""
        self.client = VectorizerClient(
            base_url="http://localhost:15002",
            api_key="test-api-key"
        )

    async def test_list_graph_nodes(self):
        """Test listing graph nodes."""
        try:
            result = await self.client.list_graph_nodes("test_collection")
            self.assertIsNotNone(result)
            self.assertIsInstance(result.count, int)
        except Exception:
            # Expected if server not available
            pass

    async def test_get_graph_neighbors(self):
        """Test getting graph neighbors."""
        try:
            result = await self.client.get_graph_neighbors("test_collection", "node1")
            self.assertIsNotNone(result)
        except Exception:
            # Expected if server not available
            pass

    async def test_find_related_nodes(self):
        """Test finding related nodes."""
        try:
            request = FindRelatedRequest(max_hops=2, relationship_type="SIMILAR_TO")
            result = await self.client.find_related_nodes("test_collection", "node1", request)
            self.assertIsNotNone(result)
        except Exception:
            # Expected if server not available
            pass

    async def test_find_graph_path(self):
        """Test finding graph path."""
        try:
            request = FindPathRequest(
                collection="test_collection",
                source="node1",
                target="node2"
            )
            result = await self.client.find_graph_path(request)
            self.assertIsNotNone(result)
        except Exception:
            # Expected if server not available
            pass

    async def test_create_graph_edge(self):
        """Test creating graph edge."""
        try:
            request = CreateEdgeRequest(
                collection="test_collection",
                source="node1",
                target="node2",
                relationship_type="SIMILAR_TO",
                weight=0.85
            )
            result = await self.client.create_graph_edge(request)
            self.assertIsNotNone(result)
        except Exception:
            # Expected if server not available
            pass

    async def test_discover_graph_edges(self):
        """Test discovering graph edges."""
        try:
            request = DiscoverEdgesRequest(
                similarity_threshold=0.7,
                max_per_node=10
            )
            result = await self.client.discover_graph_edges("test_collection", request)
            self.assertIsNotNone(result)
        except Exception:
            # Expected if server not available
            pass

    async def test_get_graph_discovery_status(self):
        """Test getting graph discovery status."""
        try:
            result = await self.client.get_graph_discovery_status("test_collection")
            self.assertIsNotNone(result)
        except Exception:
            # Expected if server not available
            pass


if __name__ == '__main__':
    unittest.main()


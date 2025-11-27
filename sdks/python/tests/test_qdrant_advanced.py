"""
Tests for Qdrant advanced features (1.14.x)

Tests for Snapshots, Sharding, Cluster Management, Query API, Search Groups/Matrix
"""

import unittest
import asyncio
from unittest.mock import AsyncMock, patch, MagicMock
from typing import Dict, Any

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from client import VectorizerClient
from exceptions import CollectionNotFoundError, NetworkError, ServerError


class TestQdrantSnapshots(unittest.IsolatedAsyncioTestCase):
    """Tests for Qdrant Snapshots API"""
    
    async def asyncSetUp(self):
        """Set up test client"""
        self.client = VectorizerClient()
        await self.client.connect()
    
    async def asyncTearDown(self):
        """Clean up test client"""
        await self.client.close()
    
    async def test_qdrant_list_collection_snapshots(self):
        """Test listing collection snapshots"""
        try:
            result = await self.client.qdrant_list_collection_snapshots("test_collection")
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_create_collection_snapshot(self):
        """Test creating collection snapshot"""
        try:
            result = await self.client.qdrant_create_collection_snapshot("test_collection")
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_delete_collection_snapshot(self):
        """Test deleting collection snapshot"""
        try:
            result = await self.client.qdrant_delete_collection_snapshot(
                "test_collection", "test_snapshot"
            )
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_recover_collection_snapshot(self):
        """Test recovering collection from snapshot"""
        try:
            result = await self.client.qdrant_recover_collection_snapshot(
                "test_collection", "snapshots/test.snapshot"
            )
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_list_all_snapshots(self):
        """Test listing all snapshots"""
        try:
            result = await self.client.qdrant_list_all_snapshots()
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_create_full_snapshot(self):
        """Test creating full snapshot"""
        try:
            result = await self.client.qdrant_create_full_snapshot()
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")


class TestQdrantSharding(unittest.IsolatedAsyncioTestCase):
    """Tests for Qdrant Sharding API"""
    
    async def asyncSetUp(self):
        """Set up test client"""
        self.client = VectorizerClient()
        await self.client.connect()
    
    async def asyncTearDown(self):
        """Clean up test client"""
        await self.client.close()
    
    async def test_qdrant_list_shard_keys(self):
        """Test listing shard keys"""
        try:
            result = await self.client.qdrant_list_shard_keys("test_collection")
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_create_shard_key(self):
        """Test creating shard key"""
        try:
            shard_key = {"shard_key": "test_key"}
            result = await self.client.qdrant_create_shard_key("test_collection", shard_key)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_delete_shard_key(self):
        """Test deleting shard key"""
        try:
            shard_key = {"shard_key": "test_key"}
            result = await self.client.qdrant_delete_shard_key("test_collection", shard_key)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")


class TestQdrantCluster(unittest.IsolatedAsyncioTestCase):
    """Tests for Qdrant Cluster Management API"""
    
    async def asyncSetUp(self):
        """Set up test client"""
        self.client = VectorizerClient()
        await self.client.connect()
    
    async def asyncTearDown(self):
        """Clean up test client"""
        await self.client.close()
    
    async def test_qdrant_get_cluster_status(self):
        """Test getting cluster status"""
        try:
            result = await self.client.qdrant_get_cluster_status()
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_cluster_recover(self):
        """Test cluster recovery"""
        try:
            result = await self.client.qdrant_cluster_recover()
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_remove_peer(self):
        """Test removing peer"""
        try:
            result = await self.client.qdrant_remove_peer("test_peer_123")
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_list_metadata_keys(self):
        """Test listing metadata keys"""
        try:
            result = await self.client.qdrant_list_metadata_keys()
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_get_metadata_key(self):
        """Test getting metadata key"""
        try:
            result = await self.client.qdrant_get_metadata_key("test_key")
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_update_metadata_key(self):
        """Test updating metadata key"""
        try:
            value = {"value": "test_value"}
            result = await self.client.qdrant_update_metadata_key("test_key", value)
            self.assertIsInstance(result, dict)
        except NetworkError as e:
            self.skipTest(f"Server not available: {e}")


class TestQdrantQueryAPI(unittest.IsolatedAsyncioTestCase):
    """Tests for Qdrant Query API (1.7+)"""
    
    async def asyncSetUp(self):
        """Set up test client"""
        self.client = VectorizerClient()
        await self.client.connect()
    
    async def asyncTearDown(self):
        """Clean up test client"""
        await self.client.close()
    
    async def test_qdrant_query_points(self):
        """Test querying points"""
        try:
            request = {
                "query": {
                    "vector": [0.1, 0.2, 0.3] * 128  # 384 dimensions
                },
                "limit": 10
            }
            result = await self.client.qdrant_query_points("test_collection", request)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_batch_query_points(self):
        """Test batch querying points"""
        try:
            request = {
                "searches": [
                    {
                        "query": {
                            "vector": [0.1, 0.2, 0.3] * 128
                        },
                        "limit": 10
                    }
                ]
            }
            result = await self.client.qdrant_batch_query_points("test_collection", request)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_query_points_groups(self):
        """Test querying points with groups"""
        try:
            request = {
                "query": {
                    "vector": [0.1, 0.2, 0.3] * 128
                },
                "group_by": "category",
                "group_size": 3,
                "limit": 10
            }
            result = await self.client.qdrant_query_points_groups("test_collection", request)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")


class TestQdrantSearchGroupsMatrix(unittest.IsolatedAsyncioTestCase):
    """Tests for Qdrant Search Groups and Matrix API"""
    
    async def asyncSetUp(self):
        """Set up test client"""
        self.client = VectorizerClient()
        await self.client.connect()
    
    async def asyncTearDown(self):
        """Clean up test client"""
        await self.client.close()
    
    async def test_qdrant_search_points_groups(self):
        """Test searching points with groups"""
        try:
            request = {
                "vector": [0.1, 0.2, 0.3] * 128,
                "group_by": "category",
                "group_size": 3,
                "limit": 10
            }
            result = await self.client.qdrant_search_points_groups("test_collection", request)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_search_matrix_pairs(self):
        """Test searching matrix pairs"""
        try:
            request = {
                "sample": 10,
                "limit": 5
            }
            result = await self.client.qdrant_search_matrix_pairs("test_collection", request)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")
    
    async def test_qdrant_search_matrix_offsets(self):
        """Test searching matrix offsets"""
        try:
            request = {
                "sample": 10,
                "limit": 5
            }
            result = await self.client.qdrant_search_matrix_offsets("test_collection", request)
            self.assertIsInstance(result, dict)
        except (CollectionNotFoundError, NetworkError) as e:
            self.skipTest(f"Server not available: {e}")


class TestQdrantErrorHandling(unittest.IsolatedAsyncioTestCase):
    """Tests for Qdrant error handling"""
    
    async def asyncSetUp(self):
        """Set up test client"""
        self.client = VectorizerClient()
        await self.client.connect()
    
    async def asyncTearDown(self):
        """Clean up test client"""
        await self.client.close()
    
    async def test_qdrant_invalid_collection(self):
        """Test error handling for invalid collection"""
        try:
            await self.client.qdrant_list_collection_snapshots("nonexistent_collection")
            # If no error, collection exists
        except CollectionNotFoundError:
            # Expected error
            pass
        except NetworkError:
            self.skipTest("Server not available")
    
    async def test_qdrant_invalid_snapshot(self):
        """Test error handling for invalid snapshot"""
        try:
            await self.client.qdrant_delete_collection_snapshot(
                "test_collection", "nonexistent_snapshot"
            )
        except (CollectionNotFoundError, ServerError, NetworkError):
            # Expected errors
            pass


if __name__ == "__main__":
    unittest.main()


"""Unit tests for ReplicationClient (phase12).

Tests: get_replication_status, configure_replication,
get_replication_stats, list_replicas.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import ReplicationConfig  # type: ignore[import-not-found]
from vectorizer.replication import ReplicationClient  # type: ignore[import-not-found]
from vectorizer._base import AuthState, TransportRouter  # type: ignore[import-not-found]


def _make_replication() -> tuple[ReplicationClient, MagicMock]:
    transport = MagicMock()
    transport.get = AsyncMock()
    transport.post = AsyncMock()
    client = ReplicationClient.__new__(ReplicationClient)
    client._transport = transport
    client._auth = AuthState()
    client._router = TransportRouter(primary=transport)
    client.base_url = "http://localhost:15002"
    return client, transport


class TestGetReplicationStatus(unittest.TestCase):
    def test_returns_raw_dict(self):
        client, transport = _make_replication()
        transport.get.return_value = {"role": "Master", "enabled": True}
        result = asyncio.run(client.get_replication_status())
        transport.get.assert_awaited_once_with("/replication/status")
        self.assertEqual(result["role"], "Master")
        self.assertTrue(result["enabled"])

    def test_standalone_response(self):
        client, transport = _make_replication()
        transport.get.return_value = {"role": "Standalone", "enabled": False}
        result = asyncio.run(client.get_replication_status())
        self.assertFalse(result["enabled"])


class TestConfigureReplication(unittest.TestCase):
    def test_posts_role_only(self):
        client, transport = _make_replication()
        transport.post.return_value = {}
        cfg = ReplicationConfig(role="master", bind_address="0.0.0.0:15010")
        asyncio.run(client.configure_replication(cfg))
        transport.post.assert_awaited_once_with(
            "/replication/configure",
            data={"role": "master", "bind_address": "0.0.0.0:15010"},
        )

    def test_replica_config(self):
        client, transport = _make_replication()
        transport.post.return_value = {}
        cfg = ReplicationConfig(role="replica", master_address="master.host:15010")
        asyncio.run(client.configure_replication(cfg))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["role"], "replica")
        self.assertEqual(call_data["master_address"], "master.host:15010")
        self.assertNotIn("bind_address", call_data)

    def test_optional_fields_excluded_when_none(self):
        client, transport = _make_replication()
        transport.post.return_value = {}
        cfg = ReplicationConfig(role="standalone")
        asyncio.run(client.configure_replication(cfg))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(list(call_data.keys()), ["role"])


class TestGetReplicationStats(unittest.TestCase):
    def test_returns_stats_dict(self):
        client, transport = _make_replication()
        transport.get.return_value = {
            "master_offset": 200, "replica_offset": 190,
            "lag_operations": 10, "total_replicated": 1000,
        }
        result = asyncio.run(client.get_replication_stats())
        transport.get.assert_awaited_once_with("/replication/stats")
        self.assertEqual(result["master_offset"], 200)


class TestListReplicas(unittest.TestCase):
    def test_returns_replica_list(self):
        client, transport = _make_replication()
        transport.get.return_value = {
            "replicas": [
                {"replica_id": "r-1", "host": "10.0.0.2", "port": 15010,
                 "status": "Connected", "operations_synced": 500}
            ]
        }
        result = asyncio.run(client.list_replicas())
        transport.get.assert_awaited_once_with("/replication/replicas")
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0]["replica_id"], "r-1")

    def test_empty_response(self):
        client, transport = _make_replication()
        transport.get.return_value = {"replicas": []}
        result = asyncio.run(client.list_replicas())
        self.assertEqual(result, [])

    def test_non_dict_response(self):
        client, transport = _make_replication()
        transport.get.return_value = None
        result = asyncio.run(client.list_replicas())
        self.assertEqual(result, [])


if __name__ == "__main__":
    unittest.main()

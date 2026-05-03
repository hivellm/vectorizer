"""Unit tests for the SDK phase14 schema-evolution + observability surface.

Covers:
- CollectionsClient: rename_collection, reindex_collection,
  snapshot_collection_native, list_collection_snapshots_native,
  restore_collection_snapshot_native
- SearchClient: explain_search
- AdminClient: list_slow_queries, set_slow_query_config

Wire shapes are validated against the canonical server handlers at
``crates/vectorizer-server/src/server/rest_handlers/collections.rs``,
``search.rs``, and ``slow_queries.rs``.

Pattern mirrors tests/test_tier_control.py.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from exceptions import ValidationError  # type: ignore[import-not-found]
from models import (  # type: ignore[import-not-found]
    ExplainResponse,
    ExplainTrace,
    NativeSnapshotInfo,
    ReindexJob,
    ReindexParams,
    SlowQueryConfig,
    SlowQueryEntry,
)
from vectorizer.admin import AdminClient  # type: ignore[import-not-found]
from vectorizer.collections import CollectionsClient  # type: ignore[import-not-found]
from vectorizer.search import SearchClient  # type: ignore[import-not-found]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_collections_client() -> tuple[CollectionsClient, MagicMock]:
    client = CollectionsClient.__new__(CollectionsClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.get = AsyncMock()
    transport.delete = AsyncMock()
    transport.patch = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


def _make_search_client() -> tuple[SearchClient, MagicMock]:
    client = SearchClient.__new__(SearchClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.get = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


def _make_admin_client() -> tuple[AdminClient, MagicMock]:
    client = AdminClient.__new__(AdminClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.get = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


# ---------------------------------------------------------------------------
# Model dataclass round-trips
# ---------------------------------------------------------------------------


class TestReindexParamsDefaults(unittest.TestCase):
    def test_defaults_match_server(self) -> None:
        p = ReindexParams()
        self.assertEqual(p.m, 16)
        self.assertEqual(p.ef_construction, 200)
        self.assertEqual(p.ef_search, 100)


class TestReindexJob(unittest.TestCase):
    def test_from_dict_server_contract(self) -> None:
        raw = {
            "job_id": "reindex-docs-1746000000",
            "collection": "docs",
            "state": "completed",
            "params": {"m": 32, "ef_construction": 400, "ef_search": 200},
            "progress": 1.0,
        }
        job = ReindexJob.from_dict(raw)
        self.assertEqual(job.job_id, "reindex-docs-1746000000")
        self.assertEqual(job.collection, "docs")
        self.assertEqual(job.state, "completed")
        self.assertAlmostEqual(job.progress, 1.0)
        self.assertEqual(job.params["m"], 32)


class TestNativeSnapshotInfo(unittest.TestCase):
    def test_from_dict_server_contract(self) -> None:
        raw = {
            "id": "snap-abc-123",
            "collection": "docs",
            "created_at": "2026-05-02T00:00:00Z",
            "size_bytes": 4096,
            "status": "ok",  # server-only field, ignored by dataclass
        }
        info = NativeSnapshotInfo.from_dict(raw)
        self.assertEqual(info.id, "snap-abc-123")
        self.assertEqual(info.collection, "docs")
        self.assertEqual(info.size_bytes, 4096)
        self.assertEqual(info.created_at, "2026-05-02T00:00:00Z")


class TestExplainTrace(unittest.TestCase):
    def test_from_dict_all_fields(self) -> None:
        raw = {
            "visited_nodes": 120,
            "ef_search": 100,
            "hnsw_search_ms": 1.23,
            "payload_filter_evals": 5,
            "quantization_score_ms": 0.45,
            "total_ms": 2.10,
        }
        trace = ExplainTrace.from_dict(raw)
        self.assertEqual(trace.visited_nodes, 120)
        self.assertEqual(trace.ef_search, 100)
        self.assertAlmostEqual(trace.hnsw_search_ms, 1.23)
        self.assertEqual(trace.payload_filter_evals, 5)
        self.assertAlmostEqual(trace.quantization_score_ms, 0.45)
        self.assertAlmostEqual(trace.total_ms, 2.10)

    def test_from_dict_defaults_on_empty(self) -> None:
        trace = ExplainTrace.from_dict({})
        self.assertEqual(trace.visited_nodes, 0)
        self.assertAlmostEqual(trace.total_ms, 0.0)


class TestExplainResponse(unittest.TestCase):
    def test_from_dict_server_contract(self) -> None:
        raw = {
            "collection": "docs",
            "k": 10,
            "results": [{"id": "vec-1", "score": 0.95, "payload": None}],
            "trace": {
                "visited_nodes": 80,
                "ef_search": 64,
                "hnsw_search_ms": 0.9,
                "payload_filter_evals": 0,
                "quantization_score_ms": 0.1,
                "total_ms": 1.2,
            },
        }
        resp = ExplainResponse.from_dict(raw)
        self.assertEqual(resp.collection, "docs")
        self.assertEqual(resp.k, 10)
        self.assertEqual(len(resp.results), 1)
        self.assertIsInstance(resp.trace, ExplainTrace)
        self.assertEqual(resp.trace.visited_nodes, 80)


class TestSlowQueryEntry(unittest.TestCase):
    def test_from_dict_server_contract(self) -> None:
        raw = {
            "timestamp": "2026-05-02T00:01:00Z",
            "collection": "docs",
            "k": 10,
            "duration_ms": 312.5,
        }
        entry = SlowQueryEntry.from_dict(raw)
        self.assertEqual(entry.collection, "docs")
        self.assertEqual(entry.k, 10)
        self.assertAlmostEqual(entry.duration_ms, 312.5)

    def test_from_dict_defaults_on_empty(self) -> None:
        entry = SlowQueryEntry.from_dict({})
        self.assertEqual(entry.collection, "")
        self.assertAlmostEqual(entry.duration_ms, 0.0)


class TestSlowQueryConfig(unittest.TestCase):
    def test_from_dict_server_contract(self) -> None:
        raw = {"threshold_ms": 200, "capacity": 500, "status": "ok"}
        cfg = SlowQueryConfig.from_dict(raw)
        self.assertEqual(cfg.threshold_ms, 200)
        self.assertEqual(cfg.capacity, 500)

    def test_defaults(self) -> None:
        cfg = SlowQueryConfig()
        self.assertEqual(cfg.threshold_ms, 200)
        self.assertEqual(cfg.capacity, 1000)


# ---------------------------------------------------------------------------
# rename_collection wire shape
# ---------------------------------------------------------------------------


class TestRenameCollectionWire(unittest.TestCase):
    def test_posts_correct_endpoint_and_body(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "old_name": "docs",
            "new_name": "docs_v2",
            "alias_retained": "docs",
            "status": "ok",
        }
        result = asyncio.run(client.rename_collection("docs", "docs_v2"))
        transport.post.assert_awaited_once_with(
            "/collections/docs/rename",
            data={"new_name": "docs_v2"},
        )
        self.assertIsNone(result)

    def test_rejects_empty_new_name(self) -> None:
        client, _ = _make_collections_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.rename_collection("docs", ""))


# ---------------------------------------------------------------------------
# reindex_collection wire shape
# ---------------------------------------------------------------------------


class TestReindexCollectionWire(unittest.TestCase):
    def test_posts_correct_endpoint_with_params(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "job_id": "reindex-docs-1746000001",
            "collection": "docs",
            "state": "completed",
            "params": {"m": 32, "ef_construction": 400, "ef_search": 200},
            "progress": 1.0,
        }
        params = ReindexParams(m=32, ef_construction=400, ef_search=200)
        job = asyncio.run(client.reindex_collection("docs", params))
        transport.post.assert_awaited_once_with(
            "/collections/docs/reindex",
            data={"m": 32, "ef_construction": 400, "ef_search": 200},
        )
        self.assertIsInstance(job, ReindexJob)
        self.assertEqual(job.state, "completed")
        self.assertAlmostEqual(job.progress, 1.0)

    def test_uses_server_defaults_when_params_none(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "job_id": "reindex-c-1",
            "collection": "c",
            "state": "completed",
            "params": {"m": 16, "ef_construction": 200, "ef_search": 100},
            "progress": 1.0,
        }
        asyncio.run(client.reindex_collection("c", None))
        _, kwargs = transport.post.call_args
        sent = kwargs.get("data", {})
        self.assertEqual(sent["m"], 16)
        self.assertEqual(sent["ef_construction"], 200)
        self.assertEqual(sent["ef_search"], 100)


# ---------------------------------------------------------------------------
# snapshot_collection_native wire shape
# ---------------------------------------------------------------------------


class TestSnapshotCollectionNativeWire(unittest.TestCase):
    def test_posts_empty_body_to_correct_endpoint(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "id": "snap-abc-123",
            "collection": "docs",
            "created_at": "2026-05-02T00:00:00Z",
            "size_bytes": 4096,
            "status": "ok",
        }
        info = asyncio.run(client.snapshot_collection_native("docs"))
        transport.post.assert_awaited_once_with(
            "/collections/docs/snapshot",
            data={},
        )
        self.assertIsInstance(info, NativeSnapshotInfo)
        self.assertEqual(info.id, "snap-abc-123")
        self.assertEqual(info.size_bytes, 4096)


# ---------------------------------------------------------------------------
# list_collection_snapshots_native wire shape
# ---------------------------------------------------------------------------


class TestListCollectionSnapshotsNativeWire(unittest.TestCase):
    def test_gets_correct_endpoint_and_parses_list(self) -> None:
        client, transport = _make_collections_client()
        transport.get.return_value = {
            "collection": "docs",
            "snapshots": [
                {
                    "id": "snap-1",
                    "collection": "docs",
                    "created_at": "2026-05-02T00:00:00Z",
                    "size_bytes": 2048,
                },
                {
                    "id": "snap-2",
                    "collection": "docs",
                    "created_at": "2026-05-02T01:00:00Z",
                    "size_bytes": 3000,
                },
            ],
            "total": 2,
        }
        snaps = asyncio.run(client.list_collection_snapshots_native("docs"))
        transport.get.assert_awaited_once_with("/collections/docs/snapshots")
        self.assertEqual(len(snaps), 2)
        self.assertIsInstance(snaps[0], NativeSnapshotInfo)
        self.assertEqual(snaps[0].id, "snap-1")
        self.assertEqual(snaps[1].size_bytes, 3000)

    def test_returns_empty_list_when_snapshots_missing(self) -> None:
        client, transport = _make_collections_client()
        transport.get.return_value = {"collection": "docs", "total": 0}
        snaps = asyncio.run(client.list_collection_snapshots_native("docs"))
        self.assertEqual(snaps, [])


# ---------------------------------------------------------------------------
# restore_collection_snapshot_native wire shape
# ---------------------------------------------------------------------------


class TestRestoreCollectionSnapshotNativeWire(unittest.TestCase):
    def test_posts_empty_body_to_correct_endpoint(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "collection": "docs",
            "snapshot_id": "snap-1",
            "status": "restored",
        }
        result = asyncio.run(client.restore_collection_snapshot_native("docs", "snap-1"))
        transport.post.assert_awaited_once_with(
            "/collections/docs/snapshots/snap-1/restore",
            data={},
        )
        self.assertIsNone(result)

    def test_rejects_empty_snapshot_id(self) -> None:
        client, _ = _make_collections_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.restore_collection_snapshot_native("docs", ""))


# ---------------------------------------------------------------------------
# explain_search wire shape
# ---------------------------------------------------------------------------


class TestExplainSearchWire(unittest.TestCase):
    _server_response = {
        "collection": "docs",
        "k": 10,
        "results": [{"id": "vec-1", "score": 0.95, "payload": None}],
        "trace": {
            "visited_nodes": 120,
            "ef_search": 100,
            "hnsw_search_ms": 1.23,
            "payload_filter_evals": 0,
            "quantization_score_ms": 0.45,
            "total_ms": 2.10,
        },
    }

    def test_posts_vector_and_k_to_correct_endpoint(self) -> None:
        client, transport = _make_search_client()
        transport.post.return_value = self._server_response
        resp = asyncio.run(client.explain_search("docs", [0.1, 0.2, 0.3], k=10))
        transport.post.assert_awaited_once_with(
            "/collections/docs/explain",
            data={"vector": [0.1, 0.2, 0.3], "k": 10},
        )
        self.assertIsInstance(resp, ExplainResponse)
        self.assertEqual(resp.collection, "docs")
        self.assertEqual(resp.k, 10)
        self.assertEqual(len(resp.results), 1)

    def test_omits_k_from_body_when_not_provided(self) -> None:
        client, transport = _make_search_client()
        transport.post.return_value = self._server_response
        asyncio.run(client.explain_search("docs", [0.1, 0.2]))
        _, kwargs = transport.post.call_args
        self.assertNotIn("k", kwargs.get("data", {}))

    def test_trace_fields_decoded(self) -> None:
        client, transport = _make_search_client()
        transport.post.return_value = self._server_response
        resp = asyncio.run(client.explain_search("docs", [0.1, 0.2, 0.3]))
        self.assertEqual(resp.trace.visited_nodes, 120)
        self.assertEqual(resp.trace.ef_search, 100)
        self.assertAlmostEqual(resp.trace.hnsw_search_ms, 1.23)
        self.assertEqual(resp.trace.payload_filter_evals, 0)
        self.assertAlmostEqual(resp.trace.quantization_score_ms, 0.45)
        self.assertAlmostEqual(resp.trace.total_ms, 2.10)

    def test_rejects_empty_vector(self) -> None:
        client, _ = _make_search_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.explain_search("docs", []))


# ---------------------------------------------------------------------------
# list_slow_queries wire shape
# ---------------------------------------------------------------------------


class TestListSlowQueriesWire(unittest.TestCase):
    def test_gets_correct_endpoint_and_parses_entries(self) -> None:
        client, transport = _make_admin_client()
        transport.get.return_value = {
            "entries": [
                {
                    "timestamp": "2026-05-02T00:01:00Z",
                    "collection": "docs",
                    "k": 10,
                    "duration_ms": 312.5,
                },
                {
                    "timestamp": "2026-05-02T00:02:00Z",
                    "collection": "logs",
                    "k": 20,
                    "duration_ms": 800.0,
                },
            ],
            "total": 2,
            "config": {"threshold_ms": 200, "capacity": 1000},
        }
        entries = asyncio.run(client.list_slow_queries())
        transport.get.assert_awaited_once_with("/slow_queries")
        self.assertEqual(len(entries), 2)
        self.assertIsInstance(entries[0], SlowQueryEntry)
        self.assertEqual(entries[0].collection, "docs")
        self.assertAlmostEqual(entries[0].duration_ms, 312.5)
        self.assertEqual(entries[1].k, 20)

    def test_returns_empty_list_when_entries_missing(self) -> None:
        client, transport = _make_admin_client()
        transport.get.return_value = {
            "total": 0,
            "config": {"threshold_ms": 200, "capacity": 1000},
        }
        entries = asyncio.run(client.list_slow_queries())
        self.assertEqual(entries, [])


# ---------------------------------------------------------------------------
# set_slow_query_config wire shape
# ---------------------------------------------------------------------------


class TestSetSlowQueryConfigWire(unittest.TestCase):
    def test_posts_correct_endpoint_and_body(self) -> None:
        client, transport = _make_admin_client()
        transport.post.return_value = {
            "threshold_ms": 150,
            "capacity": 500,
            "status": "ok",
        }
        cfg = asyncio.run(client.set_slow_query_config(threshold_ms=150, capacity=500))
        transport.post.assert_awaited_once_with(
            "/slow_queries/config",
            data={"threshold_ms": 150, "capacity": 500},
        )
        self.assertIsInstance(cfg, SlowQueryConfig)
        self.assertEqual(cfg.threshold_ms, 150)
        self.assertEqual(cfg.capacity, 500)

    def test_rejects_zero_capacity(self) -> None:
        client, _ = _make_admin_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.set_slow_query_config(threshold_ms=200, capacity=0))

    def test_rejects_negative_capacity(self) -> None:
        client, _ = _make_admin_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.set_slow_query_config(threshold_ms=200, capacity=-1))


if __name__ == "__main__":
    unittest.main()

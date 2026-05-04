"""Phase25 §7 — Python SDK runtime metrics + extended Stats / Collection.

Mirrors tests/test_admin_phase12.py for the new phase25 surface:
get_runtime_metrics(), Stats.default_quantization / compression_ratio,
and CollectionInfo.vector_count_history.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    CollectionInfo,
    RouteStats,
    RuntimeMetrics,
    Stats,
    VectorCountSample,
    WalSnapshot,
)
from vectorizer.admin import AdminClient  # type: ignore[import-not-found]


def _make_admin() -> tuple[AdminClient, MagicMock]:
    client = AdminClient.__new__(AdminClient)
    transport = MagicMock()
    transport.get = AsyncMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    client._transport = transport
    return client, transport


class TestGetRuntimeMetrics(unittest.TestCase):
    def test_targets_metrics_runtime_route(self):
        client, transport = _make_admin()
        transport.get.return_value = {}
        asyncio.run(client.get_runtime_metrics())
        transport.get.assert_awaited_once_with("/metrics/runtime")

    def test_decodes_full_snapshot(self):
        client, transport = _make_admin()
        transport.get.return_value = {
            "cpu_percent": 12.4,
            "memory_rss_bytes": 124857600,
            "memory_total_bytes": 17179869184,
            "memory_percent": 0.73,
            "active_connections": 8,
            "uptime_seconds": 3712,
            "qps_window_60s": 142.3,
            "error_rate_5xx_60s": 0.001,
            "throughput_by_route": [
                {"route": "/insert_texts", "qps": 12.0, "p50_ms": 8.2, "p99_ms": 41.0},
            ],
            "wal": {
                "current_seq": 482919,
                "size_bytes": 12582912,
                "last_checkpoint_at": 1714828800,
                "last_checkpoint_seq": 482800,
            },
        }
        m = asyncio.run(client.get_runtime_metrics())
        self.assertIsInstance(m, RuntimeMetrics)
        self.assertAlmostEqual(m.cpu_percent, 12.4)
        self.assertEqual(m.active_connections, 8)
        self.assertEqual(len(m.throughput_by_route), 1)
        self.assertIsInstance(m.throughput_by_route[0], RouteStats)
        self.assertEqual(m.throughput_by_route[0].route, "/insert_texts")
        self.assertAlmostEqual(m.throughput_by_route[0].p99_ms, 41.0)
        self.assertIsInstance(m.wal, WalSnapshot)
        self.assertEqual(m.wal.current_seq, 482919)
        self.assertEqual(m.wal.last_checkpoint_seq, 482800)

    def test_tolerates_partial_payload(self):
        client, transport = _make_admin()
        # Older / standalone server: no routes, no wal block.
        transport.get.return_value = {
            "cpu_percent": 1.0,
            "memory_total_bytes": 8_000_000_000,
        }
        m = asyncio.run(client.get_runtime_metrics())
        self.assertAlmostEqual(m.cpu_percent, 1.0)
        self.assertEqual(m.throughput_by_route, [])
        self.assertEqual(m.wal.current_seq, 0)
        self.assertEqual(m.wal.last_checkpoint_seq, 0)

    def test_tolerates_empty_response(self):
        client, transport = _make_admin()
        transport.get.return_value = {}
        m = asyncio.run(client.get_runtime_metrics())
        self.assertEqual(m.cpu_percent, 0.0)
        self.assertEqual(m.active_connections, 0)
        self.assertEqual(m.throughput_by_route, [])


class TestStatsQuantizationFields(unittest.TestCase):
    def test_decodes_phase25_quantization_fields(self):
        s = Stats.from_dict(
            {
                "collections": 3,
                "total_vectors": 12000,
                "uptime_seconds": 60,
                "version": "3.4.0",
                "default_quantization": "sq-8bit",
                "compression_ratio": 4.0,
            }
        )
        self.assertEqual(s.default_quantization, "sq-8bit")
        self.assertAlmostEqual(s.compression_ratio, 4.0)

    def test_falls_back_for_older_servers(self):
        s = Stats.from_dict(
            {
                "collections": 0,
                "total_vectors": 0,
                "uptime_seconds": 0,
                "version": "3.3.0",
            }
        )
        self.assertEqual(s.default_quantization, "none")
        self.assertAlmostEqual(s.compression_ratio, 1.0)


class TestCollectionInfoVectorCountHistory(unittest.TestCase):
    def test_hydrates_dict_samples_into_dataclasses(self):
        ci = CollectionInfo(
            name="docs",
            dimension=768,
            vector_count=482919,
            vector_count_history=[
                {"at": 1714828740, "count": 482900},
                {"at": 1714828800, "count": 482919},
            ],
        )
        self.assertEqual(len(ci.vector_count_history), 2)
        self.assertIsInstance(ci.vector_count_history[0], VectorCountSample)
        self.assertEqual(ci.vector_count_history[0].count, 482900)
        self.assertEqual(ci.vector_count_history[1].at, 1714828800)

    def test_defaults_to_empty_for_older_servers(self):
        ci = CollectionInfo(name="older", dimension=384, vector_count=0)
        self.assertEqual(ci.vector_count_history, [])


if __name__ == "__main__":
    unittest.main()

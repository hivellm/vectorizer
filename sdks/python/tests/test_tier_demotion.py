"""Unit tests for the SDK tier-demotion surface (issue #265):
``delete_vector``, ``delete_vectors``, ``move_to_collection`` and the
:class:`DeleteReport` / :class:`MoveReport` / :class:`VectorOpResult`
dataclasses.

Wire-level integration tests live alongside the server in
``crates/vectorizer/tests/api/rest/move_vectors_real.rs``.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

# Make the flat-layout SDK importable when this file is run from
# `sdks/python/`.
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import DeleteReport, MoveReport, VectorOpResult  # type: ignore[import-not-found]
from exceptions import ValidationError  # type: ignore[import-not-found]
from vectorizer.vectors import VectorsClient  # type: ignore[import-not-found]


def _make_client_with_mock_transport() -> tuple[VectorsClient, MagicMock]:
    """Build a VectorsClient whose ``_transport`` is a ``MagicMock``
    with async ``post`` / ``delete`` methods.
    """
    client = VectorsClient.__new__(VectorsClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    transport.get = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


class TestVectorOpResultDataclass(unittest.TestCase):
    def test_from_dict_full_row(self):
        row = VectorOpResult.from_dict(
            {"id": "vec-1", "status": "ok", "index": 0}
        )
        self.assertEqual(row.id, "vec-1")
        self.assertEqual(row.status, "ok")
        self.assertEqual(row.index, 0)
        self.assertIsNone(row.error)

    def test_from_dict_null_id(self):
        row = VectorOpResult.from_dict(
            {
                "id": None,
                "status": "missing_in_src",
                "error": "id must be a string",
            }
        )
        self.assertIsNone(row.id)
        self.assertEqual(row.status, "missing_in_src")
        self.assertEqual(row.error, "id must be a string")


class TestDeleteReportDataclass(unittest.TestCase):
    def test_from_dict_decodes_server_contract(self):
        raw = {
            "collection": "cortex.consolidation.fp32",
            "count": 3,
            "deleted": 2,
            "failed": 1,
            "results": [
                {"id": "vec-1", "status": "ok", "index": 0},
                {"id": "vec-2", "status": "ok", "index": 1},
                {"id": "missing", "status": "error", "error": "not found", "index": 2},
            ],
        }
        report = DeleteReport.from_dict(raw)
        self.assertEqual(report.collection, "cortex.consolidation.fp32")
        self.assertEqual(report.count, 3)
        self.assertEqual(report.deleted, 2)
        self.assertEqual(report.failed, 1)
        self.assertEqual(len(report.results), 3)
        self.assertEqual(report.results[2].status, "error")


class TestMoveReportDataclass(unittest.TestCase):
    def test_from_dict_decodes_server_contract(self):
        raw = {
            "src": "cortex.consolidation.fp32",
            "dst": "cortex.consolidation.pq",
            "requested": 3,
            "moved": 1,
            "failed": 2,
            "results": [
                {"id": "vec-1", "status": "ok"},
                {"id": "vec-missing", "status": "missing_in_src", "error": "not found"},
                {
                    "id": "vec-bad-dim",
                    "status": "dst_insert_failed",
                    "error": "dimension mismatch",
                },
            ],
        }
        report = MoveReport.from_dict(raw)
        self.assertEqual(report.src, "cortex.consolidation.fp32")
        self.assertEqual(report.dst, "cortex.consolidation.pq")
        self.assertEqual(report.requested, 3)
        self.assertEqual(report.moved, 1)
        self.assertEqual(report.failed, 2)
        self.assertEqual(
            [r.status for r in report.results],
            ["ok", "missing_in_src", "dst_insert_failed"],
        )


class TestDeleteVectorWire(unittest.TestCase):
    def test_calls_transport_delete_with_path(self):
        client, transport = _make_client_with_mock_transport()
        asyncio.run(client.delete_vector("c", "vec-1"))
        transport.delete.assert_awaited_once_with("/collections/c/vectors/vec-1")

    def test_rejects_empty_id(self):
        client, _ = _make_client_with_mock_transport()
        with self.assertRaises(ValidationError):
            asyncio.run(client.delete_vector("c", ""))


class TestDeleteVectorsWire(unittest.TestCase):
    def test_posts_batch_delete_and_decodes_report(self):
        client, transport = _make_client_with_mock_transport()
        transport.post.return_value = {
            "collection": "c",
            "count": 2,
            "deleted": 2,
            "failed": 0,
            "results": [
                {"id": "vec-1", "status": "ok", "index": 0},
                {"id": "vec-2", "status": "ok", "index": 1},
            ],
        }

        report = asyncio.run(client.delete_vectors("c", ["vec-1", "vec-2"]))

        transport.post.assert_awaited_once_with(
            "/batch_delete",
            data={"collection": "c", "ids": ["vec-1", "vec-2"]},
        )
        self.assertIsInstance(report, DeleteReport)
        self.assertEqual(report.deleted, 2)

    def test_rejects_empty_ids(self):
        client, _ = _make_client_with_mock_transport()
        with self.assertRaises(ValidationError):
            asyncio.run(client.delete_vectors("c", []))


class TestMoveToCollectionWire(unittest.TestCase):
    def test_posts_move_endpoint_and_decodes_report(self):
        client, transport = _make_client_with_mock_transport()
        transport.post.return_value = {
            "src": "src",
            "dst": "dst",
            "requested": 2,
            "moved": 2,
            "failed": 0,
            "results": [
                {"id": "vec-1", "status": "ok"},
                {"id": "vec-2", "status": "ok"},
            ],
        }

        report = asyncio.run(
            client.move_to_collection("src", "dst", ["vec-1", "vec-2"])
        )

        transport.post.assert_awaited_once_with(
            "/collections/src/vectors/move",
            data={"destination": "dst", "ids": ["vec-1", "vec-2"]},
        )
        self.assertIsInstance(report, MoveReport)
        self.assertEqual(report.moved, 2)

    def test_rejects_empty_ids(self):
        client, _ = _make_client_with_mock_transport()
        with self.assertRaises(ValidationError):
            asyncio.run(client.move_to_collection("src", "dst", []))

    def test_rejects_same_src_and_dst(self):
        client, _ = _make_client_with_mock_transport()
        with self.assertRaises(ValidationError):
            asyncio.run(client.move_to_collection("c", "c", ["vec-1"]))


if __name__ == "__main__":
    unittest.main()

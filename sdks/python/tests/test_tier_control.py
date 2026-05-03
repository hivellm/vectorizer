"""Unit tests for the SDK phase13 tier-control surface:
``delete_by_filter``, ``bulk_update_metadata``, ``copy_vectors``,
``set_vector_expiry`` on :class:`VectorsClient`, and
``reencode_collection``, ``set_collection_ttl`` on
:class:`CollectionsClient`.

Wire shapes are validated against the canonical Rust implementation at
``sdks/rust/src/client/vectors.rs`` and
``sdks/rust/src/client/collections.rs``.
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
    BulkUpdateReport,
    CopyReport,
    DeleteByFilterReport,
    ReencodeJob,
    VectorOpResult,
)
from vectorizer.collections import CollectionsClient  # type: ignore[import-not-found]
from vectorizer.vectors import VectorsClient  # type: ignore[import-not-found]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_vectors_client() -> tuple[VectorsClient, MagicMock]:
    """Return a ``VectorsClient`` with a mock transport."""
    client = VectorsClient.__new__(VectorsClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    transport.get = AsyncMock()
    transport.patch = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


def _make_collections_client() -> tuple[CollectionsClient, MagicMock]:
    """Return a ``CollectionsClient`` with a mock transport."""
    client = CollectionsClient.__new__(CollectionsClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    transport.get = AsyncMock()
    transport.patch = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


# ---------------------------------------------------------------------------
# DeleteByFilterReport dataclass
# ---------------------------------------------------------------------------


class TestDeleteByFilterReport(unittest.TestCase):
    def test_from_dict_full_server_contract(self) -> None:
        raw = {
            "scanned": 100,
            "matched": 3,
            "deleted": 2,
            "results": [
                {"id": "v1", "status": "deleted"},
                {"id": "v2", "status": "deleted"},
                {"id": "v3", "status": "error", "error": "not found"},
            ],
        }
        report = DeleteByFilterReport.from_dict(raw)
        self.assertEqual(report.scanned, 100)
        self.assertEqual(report.matched, 3)
        self.assertEqual(report.deleted, 2)
        self.assertEqual(len(report.results), 3)

    def test_from_dict_defaults_to_empty_results(self) -> None:
        report = DeleteByFilterReport.from_dict({"scanned": 5, "matched": 0, "deleted": 0})
        self.assertEqual(report.results, [])


# ---------------------------------------------------------------------------
# BulkUpdateReport dataclass
# ---------------------------------------------------------------------------


class TestBulkUpdateReport(unittest.TestCase):
    def test_from_dict_full_server_contract(self) -> None:
        raw = {
            "scanned": 50,
            "matched": 5,
            "updated": 5,
            "results": [{"id": "v1", "status": "updated"}],
        }
        report = BulkUpdateReport.from_dict(raw)
        self.assertEqual(report.scanned, 50)
        self.assertEqual(report.matched, 5)
        self.assertEqual(report.updated, 5)
        self.assertEqual(len(report.results), 1)

    def test_from_dict_defaults_to_empty_results(self) -> None:
        report = BulkUpdateReport.from_dict({"scanned": 10, "matched": 0, "updated": 0})
        self.assertEqual(report.results, [])


# ---------------------------------------------------------------------------
# CopyReport dataclass
# ---------------------------------------------------------------------------


class TestCopyReport(unittest.TestCase):
    def test_from_dict_full_server_contract(self) -> None:
        raw = {
            "src": "hot",
            "dst": "cold",
            "requested": 3,
            "copied": 2,
            "failed": 1,
            "results": [
                {"id": "v1", "status": "ok"},
                {"id": "v2", "status": "ok"},
                {"id": "v3", "status": "missing_in_src", "error": "not found"},
            ],
        }
        report = CopyReport.from_dict(raw)
        self.assertEqual(report.src, "hot")
        self.assertEqual(report.dst, "cold")
        self.assertEqual(report.requested, 3)
        self.assertEqual(report.copied, 2)
        self.assertEqual(report.failed, 1)
        statuses = [r.status for r in report.results]
        self.assertEqual(statuses, ["ok", "ok", "missing_in_src"])

    def test_results_are_vector_op_result_instances(self) -> None:
        raw = {
            "src": "a",
            "dst": "b",
            "requested": 1,
            "copied": 1,
            "failed": 0,
            "results": [{"id": "v1", "status": "ok"}],
        }
        report = CopyReport.from_dict(raw)
        self.assertIsInstance(report.results[0], VectorOpResult)


# ---------------------------------------------------------------------------
# ReencodeJob dataclass
# ---------------------------------------------------------------------------


class TestReencodeJob(unittest.TestCase):
    def test_from_dict_full_server_contract(self) -> None:
        raw = {
            "job_id": "reencode-mycol-1234567890",
            "collection": "mycol",
            "state": "completed",
            "target_encoding": "sq8",
            "progress": 1.0,
        }
        job = ReencodeJob.from_dict(raw)
        self.assertEqual(job.job_id, "reencode-mycol-1234567890")
        self.assertEqual(job.collection, "mycol")
        self.assertEqual(job.state, "completed")
        self.assertEqual(job.target_encoding, "sq8")
        self.assertAlmostEqual(job.progress, 1.0)

    def test_from_dict_partial_progress(self) -> None:
        raw = {
            "job_id": "reencode-c-1",
            "collection": "c",
            "state": "running",
            "target_encoding": "binary",
            "progress": 0.42,
        }
        job = ReencodeJob.from_dict(raw)
        self.assertEqual(job.state, "running")
        self.assertAlmostEqual(job.progress, 0.42)


# ---------------------------------------------------------------------------
# delete_by_filter wire shape
# ---------------------------------------------------------------------------


class TestDeleteByFilterWire(unittest.TestCase):
    def test_posts_correct_endpoint_with_filter(self) -> None:
        client, transport = _make_vectors_client()
        transport.post.return_value = {
            "scanned": 10,
            "matched": 2,
            "deleted": 2,
            "results": [],
        }
        report = asyncio.run(
            client.delete_by_filter("my_col", {"must": [{"key": "tier", "match": {"value": "hot"}}]})
        )
        transport.post.assert_awaited_once_with(
            "/collections/my_col/vectors/delete_by_filter",
            data={"filter": {"must": [{"key": "tier", "match": {"value": "hot"}}]}},
        )
        self.assertIsInstance(report, DeleteByFilterReport)
        self.assertEqual(report.deleted, 2)

    def test_rejects_empty_filter(self) -> None:
        client, _ = _make_vectors_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.delete_by_filter("c", {}))


# ---------------------------------------------------------------------------
# bulk_update_metadata wire shape
# ---------------------------------------------------------------------------


class TestBulkUpdateMetadataWire(unittest.TestCase):
    def test_posts_correct_endpoint_with_filter_and_patch(self) -> None:
        client, transport = _make_vectors_client()
        transport.post.return_value = {
            "scanned": 20,
            "matched": 5,
            "updated": 5,
            "results": [],
        }
        filt = {"must": [{"key": "env", "match": {"value": "prod"}}]}
        patch = {"tier": "warm", "archived": None}
        report = asyncio.run(client.bulk_update_metadata("col", filt, patch))
        transport.post.assert_awaited_once_with(
            "/collections/col/vectors/bulk_update_metadata",
            data={"filter": filt, "patch": patch},
        )
        self.assertIsInstance(report, BulkUpdateReport)
        self.assertEqual(report.updated, 5)

    def test_rejects_empty_filter(self) -> None:
        client, _ = _make_vectors_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.bulk_update_metadata("c", {}, {"k": "v"}))


# ---------------------------------------------------------------------------
# copy_vectors wire shape
# ---------------------------------------------------------------------------


class TestCopyVectorsWire(unittest.TestCase):
    def test_posts_correct_endpoint_and_decodes_report(self) -> None:
        client, transport = _make_vectors_client()
        transport.post.return_value = {
            "src": "hot",
            "dst": "cold",
            "requested": 2,
            "copied": 2,
            "failed": 0,
            "results": [
                {"id": "v1", "status": "ok"},
                {"id": "v2", "status": "ok"},
            ],
        }
        report = asyncio.run(client.copy_vectors("hot", "cold", ["v1", "v2"]))
        transport.post.assert_awaited_once_with(
            "/collections/hot/vectors/copy",
            data={"destination": "cold", "ids": ["v1", "v2"]},
        )
        self.assertIsInstance(report, CopyReport)
        self.assertEqual(report.copied, 2)
        self.assertEqual(report.src, "hot")
        self.assertEqual(report.dst, "cold")

    def test_rejects_empty_ids(self) -> None:
        client, _ = _make_vectors_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.copy_vectors("a", "b", []))

    def test_missing_in_src_status_decoded(self) -> None:
        client, transport = _make_vectors_client()
        transport.post.return_value = {
            "src": "hot",
            "dst": "cold",
            "requested": 1,
            "copied": 0,
            "failed": 1,
            "results": [{"id": "v1", "status": "missing_in_src", "error": "not found"}],
        }
        report = asyncio.run(client.copy_vectors("hot", "cold", ["v1"]))
        self.assertEqual(report.results[0].status, "missing_in_src")
        self.assertEqual(report.results[0].error, "not found")


# ---------------------------------------------------------------------------
# set_vector_expiry wire shape
# ---------------------------------------------------------------------------


class TestSetVectorExpiryWire(unittest.TestCase):
    def test_patches_correct_endpoint_with_timestamp(self) -> None:
        client, transport = _make_vectors_client()
        transport.patch.return_value = ""  # 204 No Content
        result = asyncio.run(client.set_vector_expiry("col", "vec-1", 1746000000000))
        transport.patch.assert_awaited_once_with(
            "/collections/col/vectors/vec-1/expiry",
            data={"expires_at": 1746000000000},
        )
        self.assertIsNone(result)

    def test_patches_endpoint_with_none_to_clear(self) -> None:
        client, transport = _make_vectors_client()
        transport.patch.return_value = ""  # 204 No Content
        asyncio.run(client.set_vector_expiry("col", "vec-1", None))
        transport.patch.assert_awaited_once_with(
            "/collections/col/vectors/vec-1/expiry",
            data={"expires_at": None},
        )

    def test_rejects_empty_vector_id(self) -> None:
        client, _ = _make_vectors_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.set_vector_expiry("col", "", 123))


# ---------------------------------------------------------------------------
# reencode_collection wire shape
# ---------------------------------------------------------------------------


class TestReencodeCollectionWire(unittest.TestCase):
    def test_posts_correct_endpoint_and_decodes_job(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "job_id": "reencode-col-1746000000",
            "collection": "col",
            "state": "completed",
            "target_encoding": "sq8",
            "progress": 1.0,
        }
        job = asyncio.run(client.reencode_collection("col", "sq8"))
        transport.post.assert_awaited_once_with(
            "/collections/col/reencode",
            data={"target_encoding": "sq8"},
        )
        self.assertIsInstance(job, ReencodeJob)
        self.assertEqual(job.state, "completed")
        self.assertEqual(job.target_encoding, "sq8")
        self.assertAlmostEqual(job.progress, 1.0)

    def test_rejects_empty_target_encoding(self) -> None:
        client, _ = _make_collections_client()
        with self.assertRaises(ValidationError):
            asyncio.run(client.reencode_collection("col", ""))

    def test_fp32_encoding(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = {
            "job_id": "reencode-col-2",
            "collection": "col",
            "state": "completed",
            "target_encoding": "fp32",
            "progress": 1.0,
        }
        job = asyncio.run(client.reencode_collection("col", "fp32"))
        self.assertEqual(job.target_encoding, "fp32")


# ---------------------------------------------------------------------------
# set_collection_ttl wire shape
# ---------------------------------------------------------------------------


class TestSetCollectionTtlWire(unittest.TestCase):
    def test_posts_correct_endpoint_with_ttl(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = ""  # 204 No Content
        result = asyncio.run(client.set_collection_ttl("col", 3600))
        transport.post.assert_awaited_once_with(
            "/collections/col/ttl",
            data={"ttl_secs": 3600},
        )
        self.assertIsNone(result)

    def test_posts_null_to_clear_ttl(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = ""  # 204 No Content
        asyncio.run(client.set_collection_ttl("col", None))
        transport.post.assert_awaited_once_with(
            "/collections/col/ttl",
            data={"ttl_secs": None},
        )

    def test_returns_none(self) -> None:
        client, transport = _make_collections_client()
        transport.post.return_value = ""
        result = asyncio.run(client.set_collection_ttl("col", 7200))
        self.assertIsNone(result)


if __name__ == "__main__":
    unittest.main()

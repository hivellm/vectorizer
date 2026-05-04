"""pytest tests for the typed QdrantFilter helpers (phase23).

Wire shapes are validated against the canonical Rust implementation at
``sdks/rust/src/models/filter.rs`` and the TypeScript implementation at
``sdks/typescript/src/models/filter.ts``.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from exceptions import ValidationError  # type: ignore[import-not-found]
from models import BulkUpdateReport, DeleteByFilterReport  # type: ignore[import-not-found]
from vectorizer.filter import (  # type: ignore[import-not-found]
    FilterCondition,
    FilterMatch,
    FilterRange,
    QdrantFilter,
    filter_eq,
    filter_in,
    filter_nested,
    filter_range,
    must_filter,
    must_not_filter,
    should_filter,
)
from vectorizer.vectors import VectorsClient  # type: ignore[import-not-found]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_vectors_client() -> tuple[VectorsClient, MagicMock]:
    """Return a VectorsClient with a mock transport."""
    client = VectorsClient.__new__(VectorsClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


# ---------------------------------------------------------------------------
# FilterMatch / FilterRange wire shapes
# ---------------------------------------------------------------------------


def test_filter_eq_wire_shape() -> None:
    """filter_eq serialises to {"key": ..., "match": {"value": ...}}."""
    cond = filter_eq("topic", "index")
    result = cond.to_dict()
    assert result == {"key": "topic", "match": {"value": "index"}}


def test_filter_in_wire_shape() -> None:
    """filter_in serialises to {"key": ..., "match": {"any": [...]}}."""
    cond = filter_in("tier", ["hot", "warm"])
    result = cond.to_dict()
    assert result == {"key": "tier", "match": {"any": ["hot", "warm"]}}


def test_filter_range_wire_shape() -> None:
    """filter_range with both bounds serialises {"gte": ..., "lte": ...}."""
    cond = filter_range("score", gte=0.5, lte=0.9)
    result = cond.to_dict()
    assert result == {"key": "score", "range": {"gte": 0.5, "lte": 0.9}}


def test_filter_range_single_bound() -> None:
    """filter_range with only gte omits lte from the output."""
    cond = filter_range("score", gte=0.8)
    result = cond.to_dict()
    assert result == {"key": "score", "range": {"gte": 0.8}}
    assert "lte" not in result["range"]


# ---------------------------------------------------------------------------
# QdrantFilter clause presence
# ---------------------------------------------------------------------------


def test_must_only_omits_others() -> None:
    """must_filter produces {"must": [...]} with no should/must_not keys."""
    f = must_filter(filter_eq("topic", "index"))
    result = f.to_dict()
    assert "must" in result
    assert "should" not in result
    assert "must_not" not in result


def test_should_only_omits_others() -> None:
    """should_filter produces {"should": [...]} with no must/must_not keys."""
    f = should_filter(filter_eq("status", "active"))
    result = f.to_dict()
    assert "should" in result
    assert "must" not in result
    assert "must_not" not in result


def test_must_not_only_omits_others() -> None:
    """must_not_filter produces {"must_not": [...]} with no must/should keys."""
    f = must_not_filter(filter_eq("archived", True))
    result = f.to_dict()
    assert "must_not" in result
    assert "must" not in result
    assert "should" not in result


def test_compound_must_and_must_not() -> None:
    """A filter with must + must_not contains exactly those two keys."""
    f = QdrantFilter(
        must=[filter_eq("tier", "hot")],
        must_not=[filter_eq("archived", True)],
    )
    result = f.to_dict()
    assert "must" in result
    assert "must_not" in result
    assert "should" not in result
    assert len(result["must"]) == 1
    assert len(result["must_not"]) == 1


# ---------------------------------------------------------------------------
# Nested filter round-trip
# ---------------------------------------------------------------------------


def test_nested_filter_round_trip() -> None:
    """A filter wrapped inside filter_nested serialises correctly."""
    inner = must_filter(filter_eq("inner_key", "value"))
    outer = must_filter(filter_nested(inner))
    result = outer.to_dict()

    nested_cond = result["must"][0]
    assert nested_cond["key"] == "__nested__"
    assert "filter" in nested_cond
    assert nested_cond["filter"] == {"must": [{"key": "inner_key", "match": {"value": "value"}}]}


# ---------------------------------------------------------------------------
# is_empty
# ---------------------------------------------------------------------------


def test_is_empty_on_empty_filter() -> None:
    """An all-None QdrantFilter reports is_empty() == True."""
    assert QdrantFilter().is_empty() is True


def test_is_empty_on_populated_filter() -> None:
    """A filter with at least one condition reports is_empty() == False."""
    f = must_filter(filter_eq("topic", "index"))
    assert f.is_empty() is False


def test_is_empty_on_empty_lists() -> None:
    """A filter with explicit empty lists still reports is_empty() == True."""
    f = QdrantFilter(must=[], should=[], must_not=[])
    assert f.is_empty() is True


# ---------------------------------------------------------------------------
# Integration: delete_by_filter accepts QdrantFilter
# ---------------------------------------------------------------------------


def test_delete_by_filter_typed_path() -> None:
    """delete_by_filter sends the typed filter serialised correctly."""
    client, transport = _make_vectors_client()
    transport.post.return_value = {"scanned": 5, "matched": 2, "deleted": 2, "results": []}

    f = must_filter(filter_eq("tier", "hot"))
    report = asyncio.run(client.delete_by_filter("my_col", f))

    assert isinstance(report, DeleteByFilterReport)
    transport.post.assert_called_once_with(
        "/collections/my_col/vectors/delete_by_filter",
        data={"filter": {"must": [{"key": "tier", "match": {"value": "hot"}}]}},
    )


def test_delete_by_filter_typed_empty_raises() -> None:
    """delete_by_filter raises ValidationError for an empty QdrantFilter."""
    client, _ = _make_vectors_client()
    with unittest.TestCase().assertRaises(ValidationError):
        asyncio.run(client.delete_by_filter("c", QdrantFilter()))


def test_delete_by_filter_legacy_dict_path() -> None:
    """delete_by_filter still accepts a plain dict (legacy path)."""
    client, transport = _make_vectors_client()
    transport.post.return_value = {"scanned": 1, "matched": 1, "deleted": 1, "results": []}

    filt = {"must": [{"key": "tier", "match": {"value": "hot"}}]}
    asyncio.run(client.delete_by_filter("col", filt))

    transport.post.assert_called_once_with(
        "/collections/col/vectors/delete_by_filter",
        data={"filter": filt},
    )


# ---------------------------------------------------------------------------
# Integration: bulk_update_metadata accepts QdrantFilter
# ---------------------------------------------------------------------------


def test_bulk_update_metadata_typed_path() -> None:
    """bulk_update_metadata sends the typed filter serialised correctly."""
    client, transport = _make_vectors_client()
    transport.post.return_value = {"scanned": 3, "matched": 1, "updated": 1, "results": []}

    f = must_filter(filter_eq("status", "active"))
    patch = {"tier": "hot"}
    report = asyncio.run(client.bulk_update_metadata("col", f, patch))

    assert isinstance(report, BulkUpdateReport)
    transport.post.assert_called_once_with(
        "/collections/col/vectors/bulk_update_metadata",
        data={
            "filter": {"must": [{"key": "status", "match": {"value": "active"}}]},
            "patch": {"tier": "hot"},
        },
    )


def test_bulk_update_metadata_typed_empty_raises() -> None:
    """bulk_update_metadata raises ValidationError for an empty QdrantFilter."""
    client, _ = _make_vectors_client()
    with unittest.TestCase().assertRaises(ValidationError):
        asyncio.run(client.bulk_update_metadata("c", QdrantFilter(), {}))

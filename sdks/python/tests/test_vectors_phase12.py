"""Unit tests for phase12 VectorsClient additions.

Tests: update_vector, insert_text, list_vectors, get_vector_by_path,
insert_vectors, batch_insert, batch_search, batch_update.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    BatchInsertItem,
    BatchInsertReport,
    BatchSearchQuery,
    BatchUpdateReport,
    RawVectorInsert,
    UpdateVectorRequest,
    Vector,
    VectorPage,
    VectorUpdate,
)
from vectorizer.vectors import VectorsClient  # type: ignore[import-not-found]
from vectorizer._base import AuthState, TransportRouter  # type: ignore[import-not-found]


def _make_vectors() -> tuple[VectorsClient, MagicMock]:
    transport = MagicMock()
    transport.get = AsyncMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    client = VectorsClient.__new__(VectorsClient)
    client._transport = transport
    client._auth = AuthState()
    client._router = TransportRouter(primary=transport)
    client.base_url = "http://localhost:15002"
    return client, transport


class TestUpdateVector(unittest.TestCase):
    def test_posts_update_and_returns_response(self):
        client, transport = _make_vectors()
        transport.post.return_value = {"message": "updated"}
        req = UpdateVectorRequest(id="v-1", metadata={"tag": "hot"})
        result = asyncio.run(client.update_vector("my_col", "v-1", req))
        transport.post.assert_awaited_once_with(
            "/update",
            data={"collection": "my_col", "id": "v-1", "metadata": {"tag": "hot"}},
        )
        self.assertEqual(result, {"message": "updated"})

    def test_metadata_none_excluded(self):
        client, transport = _make_vectors()
        transport.post.return_value = {}
        req = UpdateVectorRequest(id="v-2")
        result = asyncio.run(client.update_vector("col", "v-2", req))
        call_data = transport.post.call_args[1]["data"]
        self.assertNotIn("metadata", call_data)
        self.assertEqual(result, {})


class TestInsertText(unittest.TestCase):
    def test_posts_insert_and_returns_response(self):
        client, transport = _make_vectors()
        response = {
            "message": "ok", "vectors_created": 1,
            "vector_ids": ["uuid-123"], "collection": "col", "chunked": False,
        }
        transport.post.return_value = response
        result = asyncio.run(client.insert_text("col", "client-id", "Hello world"))
        transport.post.assert_awaited_once_with(
            "/insert",
            data={"collection": "col", "id": "client-id", "text": "Hello world"},
        )
        self.assertEqual(result, response)
        self.assertEqual(result["vector_ids"][0], "uuid-123")

    def test_returns_empty_dict_on_non_dict_response(self):
        client, transport = _make_vectors()
        transport.post.return_value = None
        result = asyncio.run(client.insert_text("col", "my-id", "text"))
        self.assertEqual(result, {})

    def test_metadata_included(self):
        client, transport = _make_vectors()
        transport.post.return_value = {"vector_ids": ["u-1"]}
        asyncio.run(client.insert_text("col", "id", "text", metadata={"k": "v"}))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["metadata"], {"k": "v"})


class TestListVectors(unittest.TestCase):
    def test_returns_vector_page(self):
        client, transport = _make_vectors()
        transport.get.return_value = {
            "vectors": [{"id": "v-1", "vector": [0.1, 0.2], "payload": {}}],
            "total": 100, "limit": 10, "offset": 0,
        }
        result = asyncio.run(client.list_vectors("col"))
        transport.get.assert_awaited_once_with(
            "/collections/col/vectors?limit=10&offset=0"
        )
        self.assertIsInstance(result, VectorPage)
        self.assertEqual(result.total, 100)
        self.assertEqual(len(result.vectors), 1)

    def test_page_translates_to_offset(self):
        client, transport = _make_vectors()
        transport.get.return_value = {"vectors": [], "total": 0, "limit": 20, "offset": 40}
        asyncio.run(client.list_vectors("col", page=2, limit=20))
        transport.get.assert_awaited_once_with(
            "/collections/col/vectors?limit=20&offset=40"
        )

    def test_tolerates_non_dict_response(self):
        client, transport = _make_vectors()
        transport.get.return_value = None
        result = asyncio.run(client.list_vectors("col"))
        self.assertEqual(result.total, 0)


class TestGetVectorByPath(unittest.TestCase):
    def test_returns_vector_from_server(self):
        client, transport = _make_vectors()
        transport.get.return_value = {
            "id": "v-1", "vector": [0.1, 0.2, 0.3], "payload": {"source": "test"}
        }
        result = asyncio.run(client.get_vector_by_path("col", "v-1"))
        transport.get.assert_awaited_once_with("/collections/col/vectors/v-1")
        self.assertIsInstance(result, Vector)
        self.assertEqual(result.id, "v-1")
        self.assertEqual(len(result.data), 3)
        self.assertEqual(result.metadata["source"], "test")

    def test_returns_none_on_non_dict_response(self):
        client, transport = _make_vectors()
        transport.get.return_value = None
        result = asyncio.run(client.get_vector_by_path("col", "v-99"))
        self.assertIsNone(result)


class TestInsertVectors(unittest.TestCase):
    def test_posts_insert_vectors(self):
        client, transport = _make_vectors()
        transport.post.return_value = {
            "collection": "col", "inserted": 2, "failed": 0, "count": 2, "results": []
        }
        items = [
            RawVectorInsert(embedding=[0.1, 0.2], id="v-1"),
            RawVectorInsert(embedding=[0.3, 0.4]),
        ]
        result = asyncio.run(client.insert_vectors("col", items))
        transport.post.assert_awaited_once()
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["collection"], "col")
        self.assertEqual(len(call_data["vectors"]), 2)
        self.assertIsInstance(result, BatchInsertReport)
        self.assertEqual(result.successful, 2)

    def test_optional_fields_excluded_when_none(self):
        client, transport = _make_vectors()
        transport.post.return_value = {"collection": "col", "inserted": 1, "failed": 0, "count": 1, "results": []}
        items = [RawVectorInsert(embedding=[0.1])]
        asyncio.run(client.insert_vectors("col", items))
        vec = transport.post.call_args[1]["data"]["vectors"][0]
        self.assertNotIn("id", vec)
        self.assertNotIn("payload", vec)
        self.assertNotIn("metadata", vec)


class TestBatchInsert(unittest.TestCase):
    def test_posts_batch_insert(self):
        client, transport = _make_vectors()
        transport.post.return_value = {
            "collection": "col", "inserted": 2, "failed": 0, "count": 2, "results": []
        }
        items = [
            BatchInsertItem(text="Hello", id="c-1"),
            BatchInsertItem(text="World"),
        ]
        result = asyncio.run(client.batch_insert("col", items))
        transport.post.assert_awaited_once()
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["collection"], "col")
        self.assertEqual(len(call_data["texts"]), 2)
        self.assertIsInstance(result, BatchInsertReport)


class TestBatchSearch(unittest.TestCase):
    def test_posts_and_returns_results_list(self):
        client, transport = _make_vectors()
        transport.post.return_value = {
            "results": [
                {"results": [], "query_time_ms": 1.0},
                {"results": [], "query_time_ms": 2.0},
            ]
        }
        queries = [
            BatchSearchQuery(query="test 1", limit=5),
            BatchSearchQuery(query="test 2"),
        ]
        result = asyncio.run(client.batch_search("col", queries))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["collection"], "col")
        self.assertEqual(len(call_data["queries"]), 2)
        self.assertEqual(len(result), 2)

    def test_none_fields_excluded_from_query(self):
        client, transport = _make_vectors()
        transport.post.return_value = {"results": []}
        queries = [BatchSearchQuery(query="q")]
        asyncio.run(client.batch_search("col", queries))
        sent_query = transport.post.call_args[1]["data"]["queries"][0]
        self.assertNotIn("vector", sent_query)
        self.assertNotIn("limit", sent_query)
        self.assertNotIn("threshold", sent_query)


class TestBatchUpdate(unittest.TestCase):
    def test_posts_and_returns_report(self):
        client, transport = _make_vectors()
        transport.post.return_value = {
            "collection": "col", "count": 2, "updated": 2, "failed": 0, "results": []
        }
        updates = [
            VectorUpdate(id="v-1", payload={"tag": "new"}),
            VectorUpdate(id="v-2", vector=[0.5, 0.6]),
        ]
        result = asyncio.run(client.batch_update("col", updates))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["collection"], "col")
        self.assertEqual(len(call_data["updates"]), 2)
        self.assertIsInstance(result, BatchUpdateReport)
        self.assertEqual(result.successful, 2)

    def test_none_vector_excluded(self):
        client, transport = _make_vectors()
        transport.post.return_value = {"collection": "col", "count": 1, "updated": 1, "failed": 0, "results": []}
        asyncio.run(client.batch_update("col", [VectorUpdate(id="v-1")]))
        sent = transport.post.call_args[1]["data"]["updates"][0]
        self.assertNotIn("vector", sent)
        self.assertNotIn("payload", sent)


if __name__ == "__main__":
    unittest.main()

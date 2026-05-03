"""Unit tests for the phase12 search_by_file addition to SearchClient."""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from vectorizer.search import SearchClient  # type: ignore[import-not-found]
from vectorizer._base import AuthState, TransportRouter  # type: ignore[import-not-found]


def _make_search() -> tuple[SearchClient, MagicMock]:
    transport = MagicMock()
    transport.post = AsyncMock()
    client = SearchClient.__new__(SearchClient)
    client._transport = transport
    client._auth = AuthState()
    client._router = TransportRouter(primary=transport)
    client.base_url = "http://localhost:15002"
    return client, transport


class TestSearchByFile(unittest.TestCase):
    def test_posts_with_file_path_and_default_limit(self):
        client, transport = _make_search()
        transport.post.return_value = {"results": [], "query_time_ms": 0.5}
        result = asyncio.run(client.search_by_file("my_col", "src/main.rs"))
        transport.post.assert_awaited_once_with(
            "/collections/my_col/search/file",
            data={"file_path": "src/main.rs", "limit": 10},
        )
        self.assertIn("results", result)

    def test_custom_limit(self):
        client, transport = _make_search()
        transport.post.return_value = {"results": []}
        asyncio.run(client.search_by_file("col", "README.md", limit=5))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["limit"], 5)

    def test_returns_raw_dict(self):
        client, transport = _make_search()
        transport.post.return_value = {
            "results": [{"id": "v-1", "score": 0.9, "content": "line 1"}]
        }
        result = asyncio.run(client.search_by_file("col", "foo.py"))
        self.assertEqual(len(result["results"]), 1)
        self.assertEqual(result["results"][0]["id"], "v-1")


if __name__ == "__main__":
    unittest.main()

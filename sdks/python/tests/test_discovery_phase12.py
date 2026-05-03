"""Unit tests for DiscoveryClient phase12 methods.

Tests: broad_discovery, semantic_focus, promote_readme,
compress_evidence, build_answer_plan, render_llm_prompt.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    AnswerPlan,
    AnswerPlanRequest,
    BroadDiscoveryRequest,
    BroadDiscoveryResponse,
    CompressEvidenceRequest,
    CompressEvidenceResponse,
    LlmPrompt,
    PromoteReadmeRequest,
    PromoteReadmeResponse,
    RenderPromptRequest,
    SemanticFocusRequest,
    SemanticFocusResponse,
)
from vectorizer.discovery import DiscoveryClient  # type: ignore[import-not-found]
from vectorizer._base import AuthState, TransportRouter  # type: ignore[import-not-found]


def _make_discovery() -> tuple[DiscoveryClient, MagicMock]:
    transport = MagicMock()
    transport.post = AsyncMock()
    client = DiscoveryClient.__new__(DiscoveryClient)
    client._transport = transport
    client._auth = AuthState()
    client._router = TransportRouter(primary=transport)
    client.base_url = "http://localhost:15002"
    return client, transport


class TestBroadDiscovery(unittest.TestCase):
    def test_posts_and_returns_response(self):
        client, transport = _make_discovery()
        transport.post.return_value = {
            "chunks": [{"collection": "docs", "score": 0.9, "content_preview": "x"}],
            "count": 1,
        }
        req = BroadDiscoveryRequest(queries=["HNSW", "embedding"], k=30)
        result = asyncio.run(client.broad_discovery(req))
        transport.post.assert_awaited_once_with(
            "/discovery/broad_discovery",
            data={"queries": ["HNSW", "embedding"], "k": 30},
        )
        self.assertIsInstance(result, BroadDiscoveryResponse)
        self.assertEqual(result.count, 1)
        self.assertEqual(len(result.chunks), 1)

    def test_default_k(self):
        client, transport = _make_discovery()
        transport.post.return_value = {"chunks": [], "count": 0}
        req = BroadDiscoveryRequest(queries=["test"])
        asyncio.run(client.broad_discovery(req))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["k"], 50)

    def test_tolerates_empty_response(self):
        client, transport = _make_discovery()
        transport.post.return_value = {}
        result = asyncio.run(client.broad_discovery(BroadDiscoveryRequest(queries=[])))
        self.assertEqual(result.count, 0)
        self.assertEqual(result.chunks, [])


class TestSemanticFocus(unittest.TestCase):
    def test_posts_and_returns_response(self):
        client, transport = _make_discovery()
        transport.post.return_value = {"chunks": [], "count": 0}
        req = SemanticFocusRequest(collection="code", queries=["async runtime"])
        result = asyncio.run(client.semantic_focus(req))
        transport.post.assert_awaited_once_with(
            "/discovery/semantic_focus",
            data={"collection": "code", "queries": ["async runtime"], "k": 15},
        )
        self.assertIsInstance(result, SemanticFocusResponse)

    def test_custom_k(self):
        client, transport = _make_discovery()
        transport.post.return_value = {"chunks": [], "count": 0}
        req = SemanticFocusRequest(collection="c", queries=["q"], k=5)
        asyncio.run(client.semantic_focus(req))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["k"], 5)


class TestPromoteReadme(unittest.TestCase):
    def test_posts_chunks_and_returns_response(self):
        client, transport = _make_discovery()
        chunks = [{"collection": "docs", "score": 0.8, "content": "README text"}]
        transport.post.return_value = {"promoted_chunks": chunks, "count": 1}
        req = PromoteReadmeRequest(chunks=chunks)
        result = asyncio.run(client.promote_readme(req))
        transport.post.assert_awaited_once_with(
            "/discovery/promote_readme", data={"chunks": chunks}
        )
        self.assertIsInstance(result, PromoteReadmeResponse)
        self.assertEqual(result.count, 1)

    def test_empty_chunks(self):
        client, transport = _make_discovery()
        transport.post.return_value = {"promoted_chunks": [], "count": 0}
        result = asyncio.run(client.promote_readme(PromoteReadmeRequest(chunks=[])))
        self.assertEqual(result.promoted_chunks, [])


class TestCompressEvidence(unittest.TestCase):
    def test_posts_with_optional_params(self):
        client, transport = _make_discovery()
        chunks = [{"collection": "c", "score": 1.0, "content": "x"}]
        transport.post.return_value = {
            "bullets": [{"text": "b", "source_id": "s", "category": "F", "score": 0.9}],
            "count": 1,
        }
        req = CompressEvidenceRequest(chunks=chunks, max_bullets=5, max_per_doc=2)
        result = asyncio.run(client.compress_evidence(req))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["max_bullets"], 5)
        self.assertEqual(call_data["max_per_doc"], 2)
        self.assertIsInstance(result, CompressEvidenceResponse)
        self.assertEqual(result.count, 1)

    def test_omits_none_optionals(self):
        client, transport = _make_discovery()
        transport.post.return_value = {"bullets": [], "count": 0}
        req = CompressEvidenceRequest(chunks=[])
        asyncio.run(client.compress_evidence(req))
        call_data = transport.post.call_args[1]["data"]
        self.assertNotIn("max_bullets", call_data)
        self.assertNotIn("max_per_doc", call_data)


class TestBuildAnswerPlan(unittest.TestCase):
    def test_posts_and_returns_plan(self):
        client, transport = _make_discovery()
        transport.post.return_value = {
            "sections": [{"title": "Intro", "bullets_count": 1, "bullets": []}],
            "total_bullets": 1,
            "sources": ["docs"],
        }
        req = AnswerPlanRequest(bullets=[{"text": "bullet"}])
        result = asyncio.run(client.build_answer_plan(req))
        transport.post.assert_awaited_once_with(
            "/discovery/build_answer_plan",
            data={"bullets": [{"text": "bullet"}]},
        )
        self.assertIsInstance(result, AnswerPlan)
        self.assertEqual(result.total_bullets, 1)
        self.assertEqual(result.sources, ["docs"])


class TestRenderLlmPrompt(unittest.TestCase):
    def test_posts_plan_and_returns_prompt(self):
        client, transport = _make_discovery()
        transport.post.return_value = {
            "prompt": "Answer: ...", "length": 10, "estimated_tokens": 2
        }
        plan = AnswerPlan(sections=[], total_bullets=0, sources=[])
        req = RenderPromptRequest(plan=plan)
        result = asyncio.run(client.render_llm_prompt(req))
        transport.post.assert_awaited_once()
        call_data = transport.post.call_args[1]["data"]
        self.assertIn("plan", call_data)
        self.assertIsInstance(result, LlmPrompt)
        self.assertEqual(result.prompt, "Answer: ...")
        self.assertEqual(result.estimated_tokens, 2)

    def test_none_plan_uses_empty_defaults(self):
        client, transport = _make_discovery()
        transport.post.return_value = {"prompt": "", "length": 0, "estimated_tokens": 0}
        req = RenderPromptRequest(plan=None)
        asyncio.run(client.render_llm_prompt(req))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["plan"]["total_bullets"], 0)
        self.assertEqual(call_data["plan"]["sections"], [])


if __name__ == "__main__":
    unittest.main()

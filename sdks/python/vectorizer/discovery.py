"""Discovery surface: orchestrated multi-stage retrieval.

``discover`` is the headline pipeline (filter → score → expand →
search → bullet-summarise). The other methods expose individual
pipeline stages added in phase12:

* :meth:`broad_discovery` — multi-query broad search across collections
* :meth:`semantic_focus` — focused search within one collection
* :meth:`promote_readme` — README-quality chunk promotion
* :meth:`compress_evidence` — evidence compression into bullets
* :meth:`build_answer_plan` — bullet → section organisation
* :meth:`render_llm_prompt` — plan → final LLM prompt string
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

try:
    from ..models import (
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
except ImportError:  # pragma: no cover
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

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class DiscoveryClient(_ApiBase):
    """Multi-stage discovery pipeline: broad search, focus, compression, and prompt rendering."""

    async def broad_discovery(
        self, request: BroadDiscoveryRequest
    ) -> BroadDiscoveryResponse:
        """Multi-query broad search across all collections.

        Calls ``POST /discovery/broad_discovery`` with ``{queries, k?}``.

        Args:
            request: :class:`BroadDiscoveryRequest` with queries list and optional k.

        Returns:
            :class:`BroadDiscoveryResponse` with chunks and count.
        """
        payload: Dict[str, Any] = {
            "queries": request.queries,
            "k": request.k if request.k is not None else 50,
        }
        data = await self._transport.post("/discovery/broad_discovery", data=payload)
        return BroadDiscoveryResponse.from_dict(data if isinstance(data, dict) else {})

    async def semantic_focus(
        self, request: SemanticFocusRequest
    ) -> SemanticFocusResponse:
        """Focused semantic search within a single collection.

        Calls ``POST /discovery/semantic_focus`` with ``{collection, queries, k?}``.

        Args:
            request: :class:`SemanticFocusRequest` with collection, queries, optional k.

        Returns:
            :class:`SemanticFocusResponse` with chunks and count.
        """
        payload: Dict[str, Any] = {
            "collection": request.collection,
            "queries": request.queries,
            "k": request.k if request.k is not None else 15,
        }
        data = await self._transport.post("/discovery/semantic_focus", data=payload)
        return SemanticFocusResponse.from_dict(data if isinstance(data, dict) else {})

    async def promote_readme(
        self, request: PromoteReadmeRequest
    ) -> PromoteReadmeResponse:
        """Promote README-quality chunks to the top of a result set.

        Calls ``POST /discovery/promote_readme`` with ``{chunks}``.

        Args:
            request: :class:`PromoteReadmeRequest` with chunks list.

        Returns:
            :class:`PromoteReadmeResponse` with promoted_chunks and count.
        """
        payload: Dict[str, Any] = {"chunks": request.chunks}
        data = await self._transport.post("/discovery/promote_readme", data=payload)
        return PromoteReadmeResponse.from_dict(data if isinstance(data, dict) else {})

    async def compress_evidence(
        self, request: CompressEvidenceRequest
    ) -> CompressEvidenceResponse:
        """Compress a chunk set into a concise bullet list.

        Calls ``POST /discovery/compress_evidence`` with
        ``{chunks, max_bullets?, max_per_doc?}``.

        Args:
            request: :class:`CompressEvidenceRequest` with chunks and optional limits.

        Returns:
            :class:`CompressEvidenceResponse` with bullets and count.
        """
        payload: Dict[str, Any] = {"chunks": request.chunks}
        if request.max_bullets is not None:
            payload["max_bullets"] = request.max_bullets
        if request.max_per_doc is not None:
            payload["max_per_doc"] = request.max_per_doc
        data = await self._transport.post("/discovery/compress_evidence", data=payload)
        return CompressEvidenceResponse.from_dict(data if isinstance(data, dict) else {})

    async def build_answer_plan(
        self, request: AnswerPlanRequest
    ) -> AnswerPlan:
        """Organise bullets into a structured answer plan.

        Calls ``POST /discovery/build_answer_plan`` with ``{bullets}``.

        Args:
            request: :class:`AnswerPlanRequest` with bullets list.

        Returns:
            :class:`AnswerPlan` with sections, total_bullets, and sources.
        """
        payload: Dict[str, Any] = {"bullets": request.bullets}
        data = await self._transport.post("/discovery/build_answer_plan", data=payload)
        return AnswerPlan.from_dict(data if isinstance(data, dict) else {})

    async def render_llm_prompt(
        self, request: RenderPromptRequest
    ) -> LlmPrompt:
        """Render an answer plan into a final LLM prompt string.

        Calls ``POST /discovery/render_llm_prompt`` with ``{plan}``.

        Args:
            request: :class:`RenderPromptRequest` with the :class:`AnswerPlan`.

        Returns:
            :class:`LlmPrompt` with prompt, length, and estimated_tokens.
        """
        if request.plan is None:
            plan_payload: Dict[str, Any] = {"sections": [], "total_bullets": 0, "sources": []}
        else:
            plan_payload = {
                "sections": request.plan.sections,
                "total_bullets": request.plan.total_bullets,
                "sources": request.plan.sources,
            }
        payload: Dict[str, Any] = {"plan": plan_payload}
        data = await self._transport.post("/discovery/render_llm_prompt", data=payload)
        return LlmPrompt.from_dict(data if isinstance(data, dict) else {})

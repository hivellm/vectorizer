"""Vector CRUD + batch operations surface.

Covers: :meth:`embed_text`, :meth:`insert_texts`, :meth:`get_vector`,
:meth:`delete_vectors`, the ``batch_*`` family, and the Qdrant-compatible
point operations that operate on individual vectors
(upsert / delete / retrieve / count).
"""

from __future__ import annotations

import logging
from dataclasses import asdict
from typing import Any, Dict, List, Optional, Union

import aiohttp

try:
    from ..exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from ..models import (
        BatchDeleteRequest,
        BatchInsertRequest,
        BatchResponse,
        BatchUpdateRequest,
        Vector,
    )
except ImportError:  # pragma: no cover
    from exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import (
        BatchDeleteRequest,
        BatchInsertRequest,
        BatchResponse,
        BatchUpdateRequest,
        Vector,
    )

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class VectorsClient(_ApiBase):
    """Per-vector operations — ingestion, retrieval, deletion, batching."""

    async def embed_text(self, text: str) -> List[float]:
        """
        Generate embedding for text.

        Args:
            text: Text to embed

        Returns:
            Embedding vector

        Raises:
            ValidationError: If text is invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        if not text or not isinstance(text, str):
            raise ValidationError("Text must be a non-empty string")

        payload = {"text": text}

        return await self._transport.post("/embed", data=payload)
    async def insert_texts(
        self,
        collection: str,
        vectors: List[Vector],
        public_key: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Insert vectors into a collection.

        Args:
            collection: Collection name
            vectors: List of vectors to insert
            public_key: Optional ECC public key for payload encryption (PEM, base64, or hex format)

        Returns:
            Insert operation result

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            ValidationError: If vectors are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        if not vectors:
            raise ValidationError("Vectors list cannot be empty")

        payload: Dict[str, Any] = {
            "collection": collection,
            "vectors": [asdict(vector) for vector in vectors]
        }

        effective_public_key = public_key or next((v.public_key for v in vectors if v.public_key), None)
        if effective_public_key:
            payload["public_key"] = effective_public_key

        return await self._transport.post(f"/collections/{collection}/vectors", data=payload)
    async def get_vector(self, collection: str, vector_id: str) -> Vector:
        """
        Get a specific vector by ID.

        .. warning::
           **Server caveat (observed on ``hivehub/vectorizer:3.0.x``):**
           this endpoint currently returns HTTP 200 with a synthetic
           uniform-vector payload (``[0.1, 0.1, ...]``) even for ids
           that don't exist. Callers that need real miss detection
           should probe via :meth:`list_vectors` or search and not
           trust a successful response as proof of existence until
           the server fix ships.

        Args:
            collection: Collection name
            vector_id: Vector ID

        Returns:
            Vector data

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        return await self._transport.get(f"/collections/{collection}/vectors/{vector_id}")

    async def insert_text_batch(
        self,
        collection: str,
        texts: List[Dict[str, Any]],
    ) -> Dict[str, Any]:
        """Insert a batch of **texts** into a collection. The server
        embeds each entry with the collection's configured provider
        (BM25 default, FastEmbed ONNX when selected in ``config.yml``).

        Wire contract: ``POST /insert_texts`` with
        ``{"collection": ..., "texts": [...]}`` payload. The collection
        is a top-level JSON field, **not** a path segment.

        Per-entry ``id``: the server **reassigns** every inserted
        vector a server-generated UUID. The original client id
        round-trips on the response as ``client_id``. Callers that
        need idempotency by client id should key off the returned
        ``results[].client_id``, not the server-assigned UUID.

        Distinct from :meth:`insert_texts` on this class, which takes
        :class:`Vector` objects with pre-computed embeddings and hits
        the raw-vector write path.

        Args:
            collection: Collection name.
            texts: Entries of shape
                ``{"id": "...", "text": "...", "metadata": {...}}``.

        Returns:
            Raw ``/insert_texts`` response dict with fields
            ``collection``, ``count``, ``inserted``, ``failed``,
            ``results``.
        """
        if not texts:
            raise ValidationError("texts list cannot be empty")
        payload = {"collection": collection, "texts": texts}
        return await self._transport.post("/insert_texts", data=payload)
    async def delete_vectors(self, collection: str, vector_ids: List[str]) -> bool:
        """
        Delete vectors from a collection.

        Args:
            collection: Collection name
            vector_ids: List of vector IDs to delete

        Returns:
            True if deleted successfully

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            ValidationError: If vector IDs are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        if not vector_ids:
            raise ValidationError("Vector IDs list cannot be empty")

        # `POST /batch_delete` is the canonical batch-delete endpoint
        # on v3.0.0+ (phase8_fix-batch-insert-endpoints follow-up / F5).
        # The SDK's Transport abstraction keeps `delete(path)`
        # body-less to match the REST spec; batch deletes go through
        # a dedicated POST endpoint that accepts `{collection, ids}`.
        payload = {"collection": collection, "ids": vector_ids}
        return await self._transport.post("/batch_delete", data=payload)
    # ==================== BATCH OPERATIONS ====================

    async def batch_insert_texts(
        self,
        collection: str,
        request: BatchInsertRequest
    ) -> BatchResponse:
        """
        Batch insert texts into a collection (embeddings generated automatically).

        Args:
            collection: Collection name
            request: Batch insert request

        Returns:
            Batch operation response

        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        """
        logger.debug(f"Batch inserting {len(request.texts)} texts into collection '{collection}'")

        try:
            data = await self._transport.post("/batch_insert", data=asdict(request))
            logger.info(f"Batch insert completed: {data['successful_operations']} successful, {data['failed_operations']} failed")
            return BatchResponse(**data)
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to batch insert texts: {e}")

    async def batch_update_vectors(
        self,
        collection: str,
        request: BatchUpdateRequest
    ) -> BatchResponse:
        """
        Batch update vectors in a collection.

        Args:
            collection: Collection name
            request: Batch update request

        Returns:
            Batch operation response

        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        """
        logger.debug(f"Batch updating {len(request.updates)} vectors in collection '{collection}'")

        try:
            data = await self._transport.post("/batch_update", data=asdict(request))
            logger.info(f"Batch update completed: {data['successful_operations']} successful, {data['failed_operations']} failed")
            return BatchResponse(**data)
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to batch update vectors: {e}")

    async def batch_delete_vectors(
        self,
        collection: str,
        request: BatchDeleteRequest
    ) -> BatchResponse:
        """
        Batch delete vectors from a collection.

        Args:
            collection: Collection name
            request: Batch delete request

        Returns:
            Batch operation response

        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        """
        logger.debug(f"Batch deleting {len(request.vector_ids)} vectors from collection '{collection}'")

        try:
            data = await self._transport.post("/batch_delete", data=asdict(request))
            logger.info(f"Batch delete completed: {data['successful_operations']} successful, {data['failed_operations']} failed")
            return BatchResponse(**data)
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to batch delete vectors: {e}")

    # ==================== QDRANT-COMPATIBLE POINT OPERATIONS ====================

    async def qdrant_upsert_points(
        self,
        collection: str,
        points: List[Dict[str, Any]],
        wait: bool = False,
        public_key: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Upsert points to collection (Qdrant-compatible API).

        Args:
            collection: Collection name
            points: List of Qdrant point structures
            wait: Wait for operation completion
            public_key: Optional ECC public key for payload encryption (PEM, base64, or hex format)

        Returns:
            Qdrant operation result

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            ValidationError: If points are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            payload_data: Dict[str, Any] = {"points": points, "wait": wait}
            if public_key:
                payload_data["public_key"] = public_key

            return await self._transport.put(f"/qdrant/collections/{collection}/points", data=payload_data)
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to upsert points: {e}")

    async def qdrant_delete_points(
        self,
        collection: str,
        point_ids: List[Union[str, int]],
        wait: bool = False,
    ) -> Dict[str, Any]:
        """
        Delete points from collection (Qdrant-compatible API).

        Args:
            collection: Collection name
            point_ids: List of point IDs to delete
            wait: Wait for operation completion

        Returns:
            Qdrant operation result

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            ValidationError: If point IDs are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        return await self._transport.post(f"/qdrant/collections/{collection}/points/delete", data={"points": point_ids, "wait": wait})
    async def qdrant_retrieve_points(
        self,
        collection: str,
        point_ids: List[Union[str, int]],
        with_payload: bool = True,
        with_vector: bool = False,
    ) -> Dict[str, Any]:
        """
        Retrieve points by IDs (Qdrant-compatible API).

        Args:
            collection: Collection name
            point_ids: List of point IDs to retrieve
            with_payload: Include payload in results
            with_vector: Include vector in results

        Returns:
            Qdrant retrieve response

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        # Query-string params are inlined into the URL because the
        # Transport abstraction's `get(path)` does not take a separate
        # `params=` kwarg — see `vectorizer/_base.py::Transport.get`.
        from urllib.parse import urlencode

        query = urlencode(
            {
                "ids": ",".join(str(pid) for pid in point_ids),
                "with_payload": str(with_payload).lower(),
                "with_vector": str(with_vector).lower(),
            }
        )
        return await self._transport.get(
            f"/qdrant/collections/{collection}/points?{query}"
        )

    async def qdrant_count_points(
        self,
        collection: str,
        filter: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """
        Count points in collection (Qdrant-compatible API).

        Args:
            collection: Collection name
            filter: Optional Qdrant filter

        Returns:
            Qdrant count response

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            payload: Dict[str, Any] = {}
            if filter:
                payload["filter"] = filter

            return await self._transport.post(f"/qdrant/collections/{collection}/points/count", data=payload)
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to count points: {e}")

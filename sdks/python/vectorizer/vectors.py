"""Vector CRUD + batch operations surface.

Covers: :meth:`embed_text`, :meth:`insert_texts`, :meth:`get_vector`,
:meth:`delete_vectors`, the ``batch_*`` family, the Qdrant-compatible
point operations, and the phase12 additions:
:meth:`update_vector`, :meth:`insert_text`, :meth:`list_vectors`,
:meth:`get_vector_by_path`, :meth:`insert_vectors`.
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
        BatchInsertItem,
        BatchInsertReport,
        BatchInsertRequest,
        BatchResponse,
        BatchSearchQuery,
        BatchUpdateReport,
        BatchUpdateRequest,
        BulkUpdateReport,
        CopyReport,
        DeleteByFilterReport,
        DeleteReport,
        MoveReport,
        RawVectorInsert,
        UpdateVectorRequest,
        Vector,
        VectorPage,
        VectorUpdate,
    )
except ImportError:  # pragma: no cover
    from exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import (  # type: ignore[import-not-found]
        BatchDeleteRequest,
        BatchInsertItem,
        BatchInsertReport,
        BatchInsertRequest,
        BatchResponse,
        BatchSearchQuery,
        BatchUpdateReport,
        BatchUpdateRequest,
        BulkUpdateReport,
        CopyReport,
        DeleteByFilterReport,
        DeleteReport,
        MoveReport,
        RawVectorInsert,
        UpdateVectorRequest,
        Vector,
        VectorPage,
        VectorUpdate,
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
    async def delete_vector(self, collection: str, vector_id: str) -> None:
        """Delete a single vector by id (issue #265).

        Calls ``DELETE /collections/{collection}/vectors/{vector_id}``.
        Server treats not-found as an error; a successful return means
        the vector was removed.

        Companion to :meth:`delete_vectors` (batch) and
        :meth:`move_to_collection` (cross-collection move).

        Args:
            collection: Collection name.
            vector_id: Vector id to delete.

        Raises:
            ValidationError: If ``vector_id`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not vector_id:
            raise ValidationError("vector_id cannot be empty")
        await self._transport.delete(f"/collections/{collection}/vectors/{vector_id}")

    async def delete_vectors(
        self, collection: str, vector_ids: List[str]
    ) -> DeleteReport:
        """Delete a batch of vectors from a single collection.

        Calls ``POST /batch_delete`` with ``{collection, ids}``. Per-id
        failures (e.g. not-found) populate :attr:`DeleteReport.results`
        without aborting the batch.

        .. note::
           Issue #265: prior 3.2 versions returned ``bool``. The 3.3
           contract surfaces the server's full per-id status array via
           :class:`DeleteReport` so callers can audit which vectors
           failed and why.

        Args:
            collection: Collection name.
            vector_ids: Vector ids to delete (must be non-empty).

        Returns:
            :class:`DeleteReport` with per-id outcomes.

        Raises:
            ValidationError: If ``vector_ids`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not vector_ids:
            raise ValidationError("Vector IDs list cannot be empty")

        # `POST /batch_delete` is the canonical batch-delete endpoint
        # on v3.0.0+ (phase8_fix-batch-insert-endpoints follow-up / F5).
        # The SDK's Transport abstraction keeps `delete(path)`
        # body-less to match the REST spec; batch deletes go through
        # a dedicated POST endpoint that accepts `{collection, ids}`.
        payload = {"collection": collection, "ids": vector_ids}
        data = await self._transport.post("/batch_delete", data=payload)
        return DeleteReport.from_dict(data)

    async def move_to_collection(
        self, src: str, dst: str, ids: List[str]
    ) -> MoveReport:
        """Move vectors between collections without re-embedding (issue #265).

        Calls ``POST /collections/{src}/vectors/move`` with
        ``{destination, ids}``.

        Server invariant: the destination insert lands BEFORE the
        source delete. A mid-batch failure leaves a recoverable
        duplicate (never data loss). Per-id outcomes
        (``ok``, ``missing_in_src``, ``dst_insert_failed``,
        ``src_delete_failed``) populate
        :attr:`MoveReport.results` without aborting the batch.

        Typical use: a tier-demotion pruner that walks a hot
        collection and relocates aged vectors into a warm/cold tier.

        Args:
            src: Source collection name.
            dst: Destination collection name (must differ from ``src``).
            ids: Vector ids to move (must be non-empty).

        Returns:
            :class:`MoveReport` with per-id outcomes.

        Raises:
            ValidationError: If ``ids`` is empty or ``src == dst``.
            CollectionNotFoundError: If a collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not ids:
            raise ValidationError("ids list cannot be empty")
        if src == dst:
            raise ValidationError(
                "destination must differ from source collection"
            )

        payload = {"destination": dst, "ids": ids}
        data = await self._transport.post(
            f"/collections/{src}/vectors/move", data=payload
        )
        return MoveReport.from_dict(data)

    # ==================== PHASE13 TIER-CONTROL VECTOR METHODS ====================

    async def delete_by_filter(
        self,
        collection: str,
        filter: Dict[str, Any],
    ) -> DeleteByFilterReport:
        """Delete every vector in a collection matching a metadata filter (phase13).

        Calls ``POST /collections/{name}/vectors/delete_by_filter`` with
        ``{"filter": <filter>}``. An empty filter is rejected by the server
        with 400 to prevent accidental full-collection wipes.

        Args:
            collection: Collection name.
            filter: Qdrant-style metadata filter dict (must be non-empty).

        Returns:
            :class:`DeleteByFilterReport` with scanned / matched / deleted counts.

        Raises:
            ValidationError: If ``filter`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not filter:
            raise ValidationError("filter must be a non-empty dict")
        payload = {"filter": filter}
        data = await self._transport.post(
            f"/collections/{collection}/vectors/delete_by_filter", data=payload
        )
        return DeleteByFilterReport.from_dict(data)

    async def bulk_update_metadata(
        self,
        collection: str,
        filter: Dict[str, Any],
        patch: Dict[str, Any],
    ) -> BulkUpdateReport:
        """Apply a JSON-merge-patch to every vector matching a filter (phase13).

        Calls ``POST /collections/{name}/vectors/bulk_update_metadata`` with
        ``{"filter": <filter>, "patch": <patch>}``. Patch is applied with RFC
        7396 semantics: keys in ``patch`` overwrite existing payload values;
        ``null`` values remove keys.

        Args:
            collection: Collection name.
            filter: Qdrant-style metadata filter dict.
            patch: Metadata merge-patch to apply to matched vectors.

        Returns:
            :class:`BulkUpdateReport` with scanned / matched / updated counts.

        Raises:
            ValidationError: If ``filter`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not filter:
            raise ValidationError("filter must be a non-empty dict")
        payload = {"filter": filter, "patch": patch}
        data = await self._transport.post(
            f"/collections/{collection}/vectors/bulk_update_metadata", data=payload
        )
        return BulkUpdateReport.from_dict(data)

    async def copy_vectors(
        self,
        src: str,
        dst: str,
        ids: List[str],
    ) -> CopyReport:
        """Copy vectors from ``src`` to ``dst`` without re-embedding (phase13).

        Unlike :meth:`move_to_collection`, the source vectors are NOT deleted.
        Calls ``POST /collections/{src}/vectors/copy`` with
        ``{"destination": dst, "ids": [...]}``.

        Per-id status: ``ok | missing_in_src | dst_insert_failed``.

        Args:
            src: Source collection name.
            dst: Destination collection name.
            ids: Vector ids to copy (must be non-empty).

        Returns:
            :class:`CopyReport` with per-id outcomes.

        Raises:
            ValidationError: If ``ids`` is empty.
            CollectionNotFoundError: If a collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not ids:
            raise ValidationError("ids list cannot be empty")
        payload = {"destination": dst, "ids": ids}
        data = await self._transport.post(
            f"/collections/{src}/vectors/copy", data=payload
        )
        return CopyReport.from_dict(data)

    async def set_vector_expiry(
        self,
        collection: str,
        vector_id: str,
        expires_at: Optional[int],
    ) -> None:
        """Set or clear a per-vector expiry timestamp (phase13).

        Calls ``PATCH /collections/{name}/vectors/{id}/expiry`` with
        ``{"expires_at": <unix_ms>}``. Pass ``None`` to clear an existing
        expiry. The timestamp is stored as ``__expires_at`` inside the vector
        payload and is read by the per-collection TTL reaper.

        Args:
            collection: Collection name.
            vector_id: Vector id to update.
            expires_at: Unix timestamp in milliseconds, or ``None`` to clear.

        Returns:
            ``None`` (server responds with 204 No Content).

        Raises:
            ValidationError: If ``vector_id`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not vector_id:
            raise ValidationError("vector_id cannot be empty")
        payload = {"expires_at": expires_at}
        await self._transport.patch(
            f"/collections/{collection}/vectors/{vector_id}/expiry", data=payload
        )

    # ==================== PHASE12 VECTOR METHODS ====================

    async def update_vector(
        self,
        collection: str,
        vector_id: str,
        request: UpdateVectorRequest,
    ) -> Dict[str, Any]:
        """Update a vector's metadata in-place.

        Calls ``POST /update`` with ``{collection, id, metadata}``.
        Returns the raw server response (server does not echo the full vector).

        Args:
            collection: Collection name.
            vector_id: Vector id to update.
            request: :class:`UpdateVectorRequest` with optional new metadata.

        Returns:
            Server response dict, typically ``{"message": "updated", "id": vector_id}``.
        """
        payload: Dict[str, Any] = {"collection": collection, "id": vector_id}
        if request.metadata is not None:
            payload["metadata"] = request.metadata
        data = await self._transport.post("/update", data=payload)
        return data if isinstance(data, dict) else {}

    async def insert_text(
        self,
        collection: str,
        vector_id: str,
        text: str,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        """Insert a single text document (auto-chunking for long texts).

        Calls ``POST /insert`` with ``{collection, id, text, metadata?}``.
        Returns the raw server response.

        Server response: ``{message, vectors_created, vector_ids, collection, chunked}``.
        Use ``response["vector_ids"][0]`` for the assigned id when chunking is off.

        Args:
            collection: Collection name.
            vector_id: Client-side id hint (server may reassign).
            text: Text content to embed and store.
            metadata: Optional free-form metadata dict.

        Returns:
            Server response dict.
        """
        payload: Dict[str, Any] = {
            "collection": collection,
            "id": vector_id,
            "text": text,
        }
        if metadata is not None:
            payload["metadata"] = metadata
        data = await self._transport.post("/insert", data=payload)
        return data if isinstance(data, dict) else {}

    async def list_vectors(
        self,
        collection: str,
        page: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> VectorPage:
        """List vectors in a collection with pagination.

        Calls ``GET /collections/{name}/vectors?limit=&offset=``.
        ``page`` is translated to ``offset = page * limit`` (server uses offset).

        Args:
            collection: Collection name.
            page: Zero-based page index (default 0).
            limit: Page size (default 10).

        Returns:
            :class:`VectorPage` with vectors, total, limit, and offset.
        """
        limit_val = limit if limit is not None else 10
        offset_val = (page or 0) * limit_val
        endpoint = f"/collections/{collection}/vectors?limit={limit_val}&offset={offset_val}"
        data = await self._transport.get(endpoint)
        return VectorPage.from_dict(data if isinstance(data, dict) else {})

    async def get_vector_by_path(
        self, collection: str, vector_id: str
    ) -> Optional[Vector]:
        """Fetch a single vector by id via the path-based GET endpoint.

        Calls ``GET /collections/{name}/vectors/{id}``.

        .. note::
           The server currently returns a synthetic uniform-vector payload
           even for ids that don't exist — see :meth:`get_vector` caveat.

        Args:
            collection: Collection name.
            vector_id: Vector id to retrieve.

        Returns:
            :class:`Vector` populated from the server response, or ``None`` if the
            response is malformed / missing the vector data.
        """
        data = await self._transport.get(
            f"/collections/{collection}/vectors/{vector_id}"
        )
        if not isinstance(data, dict):
            return None
        raw_vec = data.get("vector", [])
        if not raw_vec:
            return None
        assigned_id = str(data.get("id", vector_id))
        floats: List[float] = [float(x) for x in raw_vec]
        raw_meta = data.get("payload")
        metadata = dict(raw_meta) if isinstance(raw_meta, dict) else None
        return Vector(id=assigned_id, data=floats, metadata=metadata)

    async def insert_vectors(
        self,
        collection: str,
        vectors: List[RawVectorInsert],
    ) -> BatchInsertReport:
        """Bulk-insert pre-computed embeddings.

        Calls ``POST /insert_vectors`` with ``{collection, vectors: [...]}``.
        Skips the server-side embedding pipeline — caller supplies raw floats.

        Args:
            collection: Collection name.
            vectors: List of :class:`RawVectorInsert` entries with embeddings.

        Returns:
            :class:`BatchInsertReport` with inserted / failed counts.
        """
        serialized = []
        for v in vectors:
            obj: Dict[str, Any] = {"embedding": v.embedding}
            if v.id is not None:
                obj["id"] = v.id
            if v.payload is not None:
                obj["payload"] = v.payload
            if v.metadata is not None:
                obj["metadata"] = v.metadata
            serialized.append(obj)
        payload = {"collection": collection, "vectors": serialized}
        data = await self._transport.post("/insert_vectors", data=payload)
        return BatchInsertReport.from_dict(data if isinstance(data, dict) else {})

    async def batch_insert(
        self,
        collection: str,
        items: List[BatchInsertItem],
    ) -> BatchInsertReport:
        """Batch-insert multiple text documents into a collection.

        Calls ``POST /batch_insert`` with ``{collection, texts: [...]}``.
        Returns aggregate insert counts in :class:`BatchInsertReport`.

        Args:
            collection: Collection name.
            items: List of :class:`BatchInsertItem` entries with text and optional metadata.

        Returns:
            :class:`BatchInsertReport` with inserted / failed counts.
        """
        texts = []
        for item in items:
            obj: Dict[str, Any] = {"text": item.text}
            if item.id is not None:
                obj["id"] = item.id
            if item.metadata is not None:
                obj["metadata"] = item.metadata
            texts.append(obj)
        payload = {"collection": collection, "texts": texts}
        data = await self._transport.post("/batch_insert", data=payload)
        return BatchInsertReport.from_dict(data if isinstance(data, dict) else {})

    async def batch_search(
        self,
        collection: str,
        queries: List[BatchSearchQuery],
    ) -> List[Dict[str, Any]]:
        """Run multiple search queries against one collection in a single round-trip.

        Calls ``POST /batch_search`` with ``{collection, queries: [...]}``.
        Returns one response dict per query.

        Args:
            collection: Collection name.
            queries: List of :class:`BatchSearchQuery` entries.

        Returns:
            List of raw search-response dicts (one per query).
        """
        serialized = []
        for q in queries:
            obj: Dict[str, Any] = {}
            if q.query is not None:
                obj["query"] = q.query
            if q.vector is not None:
                obj["vector"] = q.vector
            if q.limit is not None:
                obj["limit"] = q.limit
            if q.threshold is not None:
                obj["threshold"] = q.threshold
            serialized.append(obj)
        payload = {"collection": collection, "queries": serialized}
        data = await self._transport.post("/batch_search", data=payload)
        if isinstance(data, dict):
            return list(data.get("results", []))
        return []

    async def batch_update(
        self,
        collection: str,
        updates: List[VectorUpdate],
    ) -> BatchUpdateReport:
        """Batch-update vector payloads (and optionally dense vectors).

        Calls ``POST /batch_update`` with ``{collection, updates: [...]}``.

        Args:
            collection: Collection name.
            updates: List of :class:`VectorUpdate` entries.

        Returns:
            :class:`BatchUpdateReport` with updated / failed counts.
        """
        serialized = []
        for u in updates:
            obj: Dict[str, Any] = {"id": u.id}
            if u.vector is not None:
                obj["vector"] = u.vector
            if u.payload is not None:
                obj["payload"] = u.payload
            serialized.append(obj)
        payload = {"collection": collection, "updates": serialized}
        data = await self._transport.post("/batch_update", data=payload)
        return BatchUpdateReport.from_dict(data if isinstance(data, dict) else {})

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

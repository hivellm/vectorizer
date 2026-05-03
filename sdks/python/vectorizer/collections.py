"""Collection management surface — create, list, describe, delete."""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

import aiohttp

try:
    from ..exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from ..models import (
        CollectionInfo,
        NativeSnapshotInfo,
        ReadOptions,
        ReencodeJob,
        ReindexJob,
        ReindexParams,
    )
except ImportError:  # pragma: no cover
    from exceptions import (  # type: ignore[import-not-found]
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import (  # type: ignore[import-not-found]
        CollectionInfo,
        NativeSnapshotInfo,
        ReadOptions,
        ReencodeJob,
        ReindexJob,
        ReindexParams,
    )

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class CollectionsClient(_ApiBase):
    """Collections sub-client.

    Usable standalone::

        from vectorizer import RestTransport, CollectionsClient

        transport = RestTransport("http://localhost:15002")
        collections = CollectionsClient(transport)
        info = await collections.list_collections()

    Or composed into the flat :class:`vectorizer.VectorizerClient`
    facade — both routes hit the same code.
    """

    async def list_collections(self, options: Optional[ReadOptions] = None) -> List[CollectionInfo]:
        """
        List all available collections.

        Args:
            options: Optional read options for routing override

        Returns:
            List of collection information

        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            transport = self._get_read_transport(options)
            data = await transport.get("/collections")
            if isinstance(data, dict) and "collections" in data:
                return [CollectionInfo(**collection) for collection in data.get("collections", [])]
            return []
        except (NetworkError, ServerError):
            raise
        except Exception as e:
            raise NetworkError(f"Failed to list collections: {e}")

    async def get_collection_info(
        self, name: str, options: Optional[ReadOptions] = None
    ) -> CollectionInfo:
        """
        Get information about a specific collection.

        Args:
            name: Collection name
            options: Optional read options for routing override

        Returns:
            Collection information

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            transport = self._get_read_transport(options)
            data = await transport.get(f"/collections/{name}")
            return CollectionInfo(**data)
        except ServerError as e:
            if "not found" in str(e).lower() or "404" in str(e):
                raise CollectionNotFoundError(f"Collection '{name}' not found")
            raise
        except (NetworkError, ServerError):
            raise
        except Exception as e:
            raise NetworkError(f"Failed to get collection info: {e}")

    async def create_collection(
        self,
        name: str,
        dimension: int = 512,
        similarity_metric: str = "cosine",
        description: Optional[str] = None,
    ) -> CollectionInfo:
        """
        Create a new collection.

        Args:
            name: Collection name
            dimension: Vector dimension (default: 512)
            similarity_metric: Similarity metric (default: "cosine")
            description: Optional collection description

        Returns:
            Created collection information

        Raises:
            ValidationError: If parameters are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        if dimension <= 0:
            raise ValidationError("Dimension must be positive")

        payload: Dict[str, Any] = {
            "name": name,
            "dimension": dimension,
            "similarity_metric": similarity_metric,
        }

        if description:
            payload["description"] = description

        try:
            transport = self._get_write_transport()
            data = await transport.post("/collections", payload)
            return CollectionInfo(**data)
        except ServerError as e:
            if "400" in str(e) or "invalid" in str(e).lower():
                raise ValidationError(f"Invalid request: {str(e)}")
            raise
        except (NetworkError, ValidationError):
            raise
        except Exception as e:
            raise NetworkError(f"Failed to create collection: {e}")

    async def delete_collection(self, name: str) -> bool:
        """
        Delete a collection.

        Args:
            name: Collection name

        Returns:
            True if deleted successfully

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            transport = self._get_write_transport()
            await transport.delete(f"/collections/{name}")
            return True
        except ServerError as e:
            if "not found" in str(e).lower() or "404" in str(e):
                raise CollectionNotFoundError(f"Collection '{name}' not found")
            raise
        except (NetworkError, ServerError):
            raise
        except Exception as e:
            raise NetworkError(f"Failed to delete collection: {e}")

    async def qdrant_list_collections(self) -> Dict[str, Any]:
        """
        List all collections (Qdrant-compatible API).

        Returns:
            Qdrant collection list response

        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        return await self._transport.get("/qdrant/collections")
    async def qdrant_get_collection(self, name: str) -> Dict[str, Any]:
        """
        Get collection information (Qdrant-compatible API).

        Args:
            name: Collection name

        Returns:
            Qdrant collection info response

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        return await self._transport.get(f"/qdrant/collections/{name}")
    async def qdrant_create_collection(self, name: str, config: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create collection (Qdrant-compatible API).

        Args:
            name: Collection name
            config: Qdrant collection configuration

        Returns:
            Qdrant operation result

        Raises:
            ValidationError: If configuration is invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        return await self._transport.put(f"/qdrant/collections/{name}", data={"config": config})

    # ==================== PHASE13 TIER-CONTROL COLLECTION METHODS ====================

    async def reencode_collection(
        self,
        collection: str,
        target_encoding: str,
    ) -> ReencodeJob:
        """Re-quantize a collection in-place without re-embedding (phase13).

        Calls ``POST /collections/{name}/reencode`` with
        ``{"target_encoding": "<encoding>"}``. Valid encoding values:
        ``"sq8"``, ``"binary"``, ``"fp32"``.

        The server runs the reencode synchronously and returns
        ``{job_id, collection, state, target_encoding, progress}`` on
        completion. ``state`` will be ``"completed"`` on success.

        Args:
            collection: Collection name to re-quantize.
            target_encoding: Target encoding (``"sq8"``, ``"binary"``, or ``"fp32"``).

        Returns:
            :class:`ReencodeJob` with job id, state, and progress.

        Raises:
            ValidationError: If ``target_encoding`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not target_encoding:
            raise ValidationError("target_encoding cannot be empty")
        payload = {"target_encoding": target_encoding}
        data = await self._transport.post(
            f"/collections/{collection}/reencode", data=payload
        )
        return ReencodeJob.from_dict(data if isinstance(data, dict) else {})

    async def set_collection_ttl(
        self,
        collection: str,
        ttl_secs: Optional[int],
    ) -> None:
        """Set or clear a per-collection TTL (phase13).

        Calls ``POST /collections/{name}/ttl`` with ``{"ttl_secs": <secs>}``.
        Pass ``None`` to clear the collection-level TTL. Existing vectors are
        NOT retroactively expired; only subsequent insertions that carry
        ``__expires_at`` in their payload are affected.

        For per-vector expiry use ``set_vector_expiry`` on the vectors surface.

        Args:
            collection: Collection name.
            ttl_secs: TTL in seconds, or ``None`` to clear.

        Returns:
            ``None`` (server responds with 204 No Content).

        Raises:
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        payload: Dict[str, Any] = {"ttl_secs": ttl_secs}
        await self._transport.post(
            f"/collections/{collection}/ttl", data=payload
        )

    # ── Phase-14: schema-evolution methods ────────────────────────────────────

    async def rename_collection(
        self,
        collection: str,
        new_name: str,
    ) -> None:
        """Atomically rename a collection (phase14).

        Calls ``POST /collections/{name}/rename`` with
        ``{"new_name": "<name>"}``.  The server keeps the old name as an
        in-memory alias for one minor version so existing clients keep
        working without reconfiguration.

        Args:
            collection: Current collection name.
            new_name: Desired new name.

        Raises:
            ValidationError: If ``new_name`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not new_name:
            raise ValidationError("new_name cannot be empty")
        await self._transport.post(
            f"/collections/{collection}/rename", data={"new_name": new_name}
        )

    async def reindex_collection(
        self,
        collection: str,
        params: Optional[ReindexParams] = None,
    ) -> ReindexJob:
        """Rebuild the HNSW index with new parameters (phase14).

        Calls ``POST /collections/{name}/reindex`` with
        ``{"m": <u32>, "ef_construction": <u32>, "ef_search": <u32>}``.
        No re-embedding is required — the existing stored vectors are
        used. The server holds the write-lock for the duration.

        Args:
            collection: Collection to re-index.
            params: HNSW parameters; server defaults apply when ``None``.

        Returns:
            :class:`ReindexJob` with ``state == "completed"`` on success.

        Raises:
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        p = params or ReindexParams()
        payload: Dict[str, Any] = {
            "m": p.m,
            "ef_construction": p.ef_construction,
            "ef_search": p.ef_search,
        }
        data = await self._transport.post(
            f"/collections/{collection}/reindex", data=payload
        )
        return ReindexJob.from_dict(data if isinstance(data, dict) else {})

    async def snapshot_collection_native(
        self,
        collection: str,
    ) -> NativeSnapshotInfo:
        """Create a native per-collection snapshot (phase14).

        Calls ``POST /collections/{name}/snapshot`` with an empty body.
        The server writes a gzip-compressed JSON snapshot and returns
        its metadata.

        Args:
            collection: Collection to snapshot.

        Returns:
            :class:`NativeSnapshotInfo` with the snapshot id, collection,
            creation timestamp, and compressed size in bytes.

        Raises:
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        data = await self._transport.post(
            f"/collections/{collection}/snapshot", data={}
        )
        return NativeSnapshotInfo.from_dict(data if isinstance(data, dict) else {})

    async def list_collection_snapshots_native(
        self,
        collection: str,
    ) -> List[NativeSnapshotInfo]:
        """List all native snapshots for a collection (phase14).

        Calls ``GET /collections/{name}/snapshots``.
        Returns snapshots newest-first as reported by the server.

        Args:
            collection: Collection name.

        Returns:
            List of :class:`NativeSnapshotInfo` entries.

        Raises:
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        data = await self._transport.get(f"/collections/{collection}/snapshots")
        if isinstance(data, dict):
            items = data.get("snapshots", [])
        else:
            items = []
        return [NativeSnapshotInfo.from_dict(s) for s in items]

    async def restore_collection_snapshot_native(
        self,
        collection: str,
        snapshot_id: str,
    ) -> None:
        """Restore a collection from a native snapshot (phase14).

        Calls ``POST /collections/{name}/snapshots/{id}/restore`` with
        an empty body. Drops the current in-memory state and replaces it
        with the snapshot data.

        Args:
            collection: Collection name.
            snapshot_id: Snapshot id to restore from.

        Raises:
            ValidationError: If ``snapshot_id`` is empty.
            CollectionNotFoundError: If the collection does not exist.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if not snapshot_id:
            raise ValidationError("snapshot_id cannot be empty")
        await self._transport.post(
            f"/collections/{collection}/snapshots/{snapshot_id}/restore",
            data={},
        )

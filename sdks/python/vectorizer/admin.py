"""Administrative surface.

Groups: health and indexing progress, file content/listing/summary/chunks,
project outline and related files, Qdrant-compatible cluster/snapshot
admin endpoints, file-upload endpoints, and phase12 admin methods:
stats, status, logs, force-save, empty-collection cleanup, config
management, backup management, server restart, and workspace management.
"""

from __future__ import annotations

import json as _json
import logging
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
        AddWorkspaceRequest,
        BackupInfo,
        CleanupReport,
        CreateBackupRequest,
        FileUploadConfig,
        FileUploadResponse,
        LogEntry,
        LogsQuery,
        RestoreBackupRequest,
        ServerStatus,
        SlowQueryConfig,
        SlowQueryEntry,
        Stats,
    )
    from ..utils.validation import validate_non_empty_string
except ImportError:  # pragma: no cover
    from exceptions import (  # type: ignore[import-not-found]
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import (  # type: ignore[import-not-found]
        AddWorkspaceRequest,
        BackupInfo,
        CleanupReport,
        CreateBackupRequest,
        FileUploadConfig,
        FileUploadResponse,
        LogEntry,
        LogsQuery,
        RestoreBackupRequest,
        ServerStatus,
        SlowQueryConfig,
        SlowQueryEntry,
        Stats,
    )
    from utils.validation import validate_non_empty_string  # type: ignore[import-not-found]

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class AdminClient(_ApiBase):
    """Health, file inspection/upload, Qdrant cluster admin."""

    # ---- Health & indexing ----

    async def health_check(self) -> Dict[str, Any]:
        """
        Check service health status.

        Returns:
            Health status information

        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service reports unhealthy status
        """
        try:
            data = await self._transport.get("/health")
            return data
        except (NetworkError, ServerError):
            raise
        except Exception as e:
            raise NetworkError(f"Failed to connect to service: {e}")

    async def get_indexing_progress(self) -> Dict[str, Any]:
        """Get indexing progress information."""
        return await self._transport.get("/indexing/progress")
    # ---- File operations ----

    async def get_file_content(
        self,
        collection: str,
        file_path: str,
        max_size_kb: int = 500
    ) -> Dict[str, Any]:
        """Retrieve complete file content from a collection."""
        payload = {
            "collection": collection,
            "file_path": file_path,
            "max_size_kb": max_size_kb
        }

        return await self._transport.post("/file/content", data=payload)
    async def list_files_in_collection(
        self,
        collection: str,
        filter_by_type: Optional[list] = None,
        min_chunks: Optional[int] = None,
        max_results: int = 100,
        sort_by: str = "name"
    ) -> Dict[str, Any]:
        """List all indexed files in a collection."""
        payload: Dict[str, Any] = {
            "collection": collection,
            "max_results": max_results,
            "sort_by": sort_by
        }
        if filter_by_type:
            payload["filter_by_type"] = filter_by_type
        if min_chunks:
            payload["min_chunks"] = min_chunks

        return await self._transport.post("/file/list", data=payload)
    async def get_file_summary(
        self,
        collection: str,
        file_path: str,
        summary_type: str = "both",
        max_sentences: int = 5
    ) -> Dict[str, Any]:
        """Get extractive or structural summary of an indexed file."""
        payload = {
            "collection": collection,
            "file_path": file_path,
            "summary_type": summary_type,
            "max_sentences": max_sentences
        }

        return await self._transport.post("/file/summary", data=payload)
    async def get_file_chunks_ordered(
        self,
        collection: str,
        file_path: str,
        start_chunk: int = 0,
        limit: int = 10,
        include_context: bool = False
    ) -> Dict[str, Any]:
        """Retrieve chunks in original file order for progressive reading."""
        payload = {
            "collection": collection,
            "file_path": file_path,
            "start_chunk": start_chunk,
            "limit": limit,
            "include_context": include_context
        }

        return await self._transport.post("/file/chunks", data=payload)
    async def get_project_outline(
        self,
        collection: str,
        max_depth: int = 5,
        include_summaries: bool = False,
        highlight_key_files: bool = True
    ) -> Dict[str, Any]:
        """Generate hierarchical project structure overview."""
        payload = {
            "collection": collection,
            "max_depth": max_depth,
            "include_summaries": include_summaries,
            "highlight_key_files": highlight_key_files
        }

        return await self._transport.post("/file/outline", data=payload)
    async def get_related_files(
        self,
        collection: str,
        file_path: str,
        limit: int = 5,
        similarity_threshold: float = 0.6,
        include_reason: bool = True
    ) -> Dict[str, Any]:
        """Find semantically related files using vector similarity."""
        payload = {
            "collection": collection,
            "file_path": file_path,
            "limit": limit,
            "similarity_threshold": similarity_threshold,
            "include_reason": include_reason
        }

        return await self._transport.post("/file/related", data=payload)
    # ---- Qdrant snapshot & cluster admin ----

    async def qdrant_list_collection_snapshots(self, collection: str) -> Dict[str, Any]:
        """List snapshots for a collection (Qdrant-compatible API)."""
        return await self._transport.get(f"/qdrant/collections/{collection}/snapshots")
    async def qdrant_create_collection_snapshot(self, collection: str) -> Dict[str, Any]:
        """Create snapshot for a collection (Qdrant-compatible API)."""
        return await self._transport.post(f"/qdrant/collections/{collection}/snapshots")
    async def qdrant_delete_collection_snapshot(
        self, collection: str, snapshot_name: str
    ) -> Dict[str, Any]:
        """Delete snapshot (Qdrant-compatible API)."""
        return await self._transport.delete(f"/qdrant/collections/{collection}/snapshots/{snapshot_name}")
    async def qdrant_recover_collection_snapshot(
        self, collection: str, location: str
    ) -> Dict[str, Any]:
        """Recover collection from snapshot (Qdrant-compatible API)."""
        payload = {"location": location}
        return await self._transport.post(
            f"/qdrant/collections/{collection}/snapshots/recover",
            data=payload,
        )
    async def qdrant_list_all_snapshots(self) -> Dict[str, Any]:
        """List all snapshots (Qdrant-compatible API)."""
        return await self._transport.get("/qdrant/snapshots")
    async def qdrant_create_full_snapshot(self) -> Dict[str, Any]:
        """Create full snapshot (Qdrant-compatible API)."""
        return await self._transport.post("/qdrant/snapshots")
    async def qdrant_list_shard_keys(self, collection: str) -> Dict[str, Any]:
        """List shard keys for a collection (Qdrant-compatible API)."""
        return await self._transport.get(f"/qdrant/collections/{collection}/shards")
    async def qdrant_create_shard_key(
        self, collection: str, shard_key: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Create shard key (Qdrant-compatible API)."""
        payload = {"shard_key": shard_key}
        return await self._transport.put(
            f"/qdrant/collections/{collection}/shards",
            data=payload,
        )

    async def qdrant_delete_shard_key(
        self, collection: str, shard_key: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Delete shard key (Qdrant-compatible API)."""
        payload = {"shard_key": shard_key}
        return await self._transport.post(
            f"/qdrant/collections/{collection}/shards/delete",
            data=payload,
        )
    async def qdrant_get_cluster_status(self) -> Dict[str, Any]:
        """Get cluster status (Qdrant-compatible API)."""
        return await self._transport.get("/qdrant/cluster")
    async def qdrant_cluster_recover(self) -> Dict[str, Any]:
        """Recover current peer (Qdrant-compatible API)."""
        return await self._transport.post("/qdrant/cluster/recover")
    async def qdrant_remove_peer(self, peer_id: str) -> Dict[str, Any]:
        """Remove peer from cluster (Qdrant-compatible API)."""
        return await self._transport.delete(f"/qdrant/cluster/peer/{peer_id}")
    async def qdrant_list_metadata_keys(self) -> Dict[str, Any]:
        """List metadata keys (Qdrant-compatible API)."""
        return await self._transport.get("/qdrant/cluster/metadata/keys")
    async def qdrant_get_metadata_key(self, key: str) -> Dict[str, Any]:
        """Get metadata key (Qdrant-compatible API)."""
        return await self._transport.get(f"/qdrant/cluster/metadata/keys/{key}")
    async def qdrant_update_metadata_key(
        self, key: str, value: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Update metadata key (Qdrant-compatible API)."""
        payload = {"value": value}
        return await self._transport.put(
            f"/qdrant/cluster/metadata/keys/{key}",
            data=payload,
        )
    # ---- phase12 admin methods ----

    async def get_stats(self) -> Stats:
        """Aggregate collection and vector counts.

        Calls ``GET /stats``.

        Returns:
            :class:`Stats` with collections, total_vectors, uptime_seconds, version.
        """
        data = await self._transport.get("/stats")
        return Stats.from_dict(data if isinstance(data, dict) else {})

    async def get_status(self) -> ServerStatus:
        """Server liveness, version, and uptime.

        Calls ``GET /status``.

        Returns:
            :class:`ServerStatus` with online, version, uptime_seconds, collections_count.
        """
        data = await self._transport.get("/status")
        return ServerStatus.from_dict(data if isinstance(data, dict) else {})

    async def get_logs(self, params: Optional[LogsQuery] = None) -> List[LogEntry]:
        """Tail recent log lines.

        Calls ``GET /logs?lines=N&level=LEVEL``.

        Args:
            params: Optional :class:`LogsQuery` with ``lines`` and ``level`` filters.

        Returns:
            List of :class:`LogEntry` objects.
        """
        qs_parts: List[str] = []
        if params is not None:
            if params.lines is not None:
                qs_parts.append(f"lines={params.lines}")
            if params.level is not None:
                qs_parts.append(f"level={params.level}")
        endpoint = "/logs" if not qs_parts else "/logs?" + "&".join(qs_parts)
        data = await self._transport.get(endpoint)
        if isinstance(data, dict):
            raw_logs = data.get("logs", [])
        else:
            raw_logs = []
        return [LogEntry.from_dict(entry) for entry in raw_logs]

    async def force_save_collection(self, collection: str) -> None:
        """Flush one collection to disk immediately.

        Calls ``POST /collections/{name}/force-save``.

        Args:
            collection: Collection name to flush.
        """
        await self._transport.post(f"/collections/{collection}/force-save")

    async def list_empty_collections(self) -> List[str]:
        """List collections that contain zero vectors.

        Calls ``GET /collections/empty``.

        Returns:
            List of collection names with no vectors.
        """
        data = await self._transport.get("/collections/empty")
        if isinstance(data, list):
            return [str(v) for v in data]
        if isinstance(data, dict):
            return [str(v) for v in data.get("collections", [])]
        return []

    async def cleanup_empty_collections(self) -> CleanupReport:
        """Delete all empty collections in one call.

        Calls ``DELETE /collections/cleanup``.

        Returns:
            :class:`CleanupReport` with success, removed count, and names.
        """
        data = await self._transport.delete("/collections/cleanup")
        return CleanupReport.from_dict(data if isinstance(data, dict) else {})

    async def get_config(self) -> Dict[str, Any]:
        """Read the server's current ``config.yml``.

        Calls ``GET /config``.

        Returns:
            Free-form dict mirroring the server's configuration.
        """
        return await self._transport.get("/config")

    async def update_config(self, patch: Dict[str, Any]) -> Dict[str, Any]:
        """Overwrite the server's ``config.yml`` (admin).

        Calls ``POST /config`` with the full config object.

        Args:
            patch: Configuration dict to write.

        Returns:
            The config as echoed back by the server.
        """
        return await self._transport.post("/config", data=patch)

    async def list_backups(self) -> List[BackupInfo]:
        """List all server-side backup files.

        Calls ``GET /backups``.

        Returns:
            List of :class:`BackupInfo` entries.
        """
        data = await self._transport.get("/backups")
        if isinstance(data, dict):
            raw = data.get("backups", [])
        else:
            raw = []
        return [BackupInfo.from_dict(b) for b in raw]

    async def create_backup(self, request: CreateBackupRequest) -> BackupInfo:
        """Create a new backup (admin).

        Calls ``POST /backups/create`` with ``{name, collections}``.

        Args:
            request: :class:`CreateBackupRequest` with name and optional collection list.

        Returns:
            :class:`BackupInfo` for the newly created backup.
        """
        payload: Dict[str, Any] = {"name": request.name, "collections": request.collections}
        data = await self._transport.post("/backups/create", data=payload)
        return BackupInfo.from_dict(data if isinstance(data, dict) else {})

    async def restore_backup(self, request: RestoreBackupRequest) -> None:
        """Restore a backup from the server's backup directory (admin).

        Calls ``POST /backups/restore`` with ``{backup_id}``.

        Args:
            request: :class:`RestoreBackupRequest` with the backup id.
        """
        await self._transport.post("/backups/restore", data={"backup_id": request.backup_id})

    async def restart_server(self) -> None:
        """Initiate a graceful server restart (admin).

        Calls ``POST /admin/restart``. The server responds before the
        process actually restarts; callers should poll ``/health`` until
        the server is back.
        """
        await self._transport.post("/admin/restart")

    async def list_workspaces(self) -> List[Dict[str, Any]]:
        """List configured workspace directories.

        Calls ``GET /workspace/list``.

        Returns:
            List of workspace config dicts (free-form JSON per workspace entry).
        """
        data = await self._transport.get("/workspace/list")
        if isinstance(data, dict):
            return list(data.get("workspaces", []))
        return []

    async def get_workspace_config(self) -> Dict[str, Any]:
        """Read the workspace configuration file.

        Calls ``GET /workspace/config``.

        Returns:
            Free-form dict mirroring the workspace configuration.
        """
        return await self._transport.get("/workspace/config")

    async def add_workspace(self, request: AddWorkspaceRequest) -> None:
        """Register a new workspace directory (admin).

        Calls ``POST /workspace/add`` with ``{path, collection_name}``.

        Args:
            request: :class:`AddWorkspaceRequest` with path and collection_name.
        """
        await self._transport.post(
            "/workspace/add",
            data={"path": request.path, "collection_name": request.collection_name},
        )

    async def remove_workspace(self, name: str) -> None:
        """Remove a registered workspace directory (admin).

        Calls ``POST /workspace/remove`` with ``{path}``.

        Args:
            name: The workspace path to remove.
        """
        await self._transport.post("/workspace/remove", data={"path": name})

    # ---- File upload operations ----

    async def upload_file(
        self,
        file_path: str,
        collection_name: str,
        chunk_size: Optional[int] = None,
        chunk_overlap: Optional[int] = None,
        metadata: Optional[Dict[str, Any]] = None,
        public_key: Optional[str] = None
    ) -> FileUploadResponse:
        """
        Upload a file for indexing.

        The file will be validated, chunked, and indexed into the specified collection.
        If the collection doesn't exist, it will be created automatically.

        Args:
            file_path: Path to the file to upload
            collection_name: Target collection name
            chunk_size: Chunk size in characters (uses server default if not specified)
            chunk_overlap: Chunk overlap in characters (uses server default if not specified)
            metadata: Additional metadata to attach to all chunks
            public_key: Optional ECC public key for payload encryption (PEM, base64, or hex format)

        Returns:
            FileUploadResponse with upload results

        Raises:
            ValidationError: If file path or collection name is invalid
            NetworkError: If network error occurs
            ServerError: If server returns an error
        """
        import os

        try:
            validate_non_empty_string(file_path)
            validate_non_empty_string(collection_name)

            if not os.path.exists(file_path):
                raise ValidationError(f"File not found: {file_path}")

            if not os.path.isfile(file_path):
                raise ValidationError(f"Path is not a file: {file_path}")

            filename = os.path.basename(file_path)
            logger.debug(f"Uploading file: {filename} to collection: {collection_name}")

            with open(file_path, 'rb') as f:
                file_content = f.read()

            form_data = aiohttp.FormData()
            form_data.add_field(
                'file',
                file_content,
                filename=filename,
                content_type='application/octet-stream'
            )
            form_data.add_field('collection_name', collection_name)

            if chunk_size is not None:
                form_data.add_field('chunk_size', str(chunk_size))

            if chunk_overlap is not None:
                form_data.add_field('chunk_overlap', str(chunk_overlap))

            if metadata is not None:
                form_data.add_field('metadata', _json.dumps(metadata))

            if public_key is not None:
                form_data.add_field('public_key', public_key)

            data = await self._transport.post("/files/config")
            result = FileUploadConfig(**data)
            logger.debug(
                f"Upload config retrieved: max_size={result.max_file_size_mb}MB, "
                f"extensions={len(result.allowed_extensions)}"
            )
            return result
        except aiohttp.ClientError as e:
            logger.error(f"Network error getting upload config: {e}")
            raise NetworkError(f"Failed to get upload config: {e}")

    # ── Phase-14: observability ────────────────────────────────────────────────

    async def list_slow_queries(self) -> List[SlowQueryEntry]:
        """List slow-query ring-buffer entries (phase14).

        Calls ``GET /slow_queries``. Returns entries in the order they were
        recorded (oldest first). Use :meth:`set_slow_query_config` to tune
        the threshold and capacity.

        Returns:
            List of :class:`SlowQueryEntry` recorded since the last restart
            or capacity eviction.

        Raises:
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        data = await self._transport.get("/slow_queries")
        if isinstance(data, dict):
            items = data.get("entries", [])
        else:
            items = []
        return [SlowQueryEntry.from_dict(e) for e in items]

    async def set_slow_query_config(
        self,
        threshold_ms: int,
        capacity: int,
    ) -> SlowQueryConfig:
        """Reconfigure the slow-query ring buffer (phase14).

        Calls ``POST /slow_queries/config`` with
        ``{"threshold_ms": <int>, "capacity": <int>}``.

        Existing entries are retained. If the new capacity is smaller than
        the current entry count the oldest entries are evicted by the server.

        Args:
            threshold_ms: Minimum query duration (ms) to record.
            capacity: Maximum ring-buffer size (entries).

        Returns:
            :class:`SlowQueryConfig` echoed back by the server.

        Raises:
            ValidationError: If ``capacity`` is zero or negative.
            NetworkError: If the transport fails.
            ServerError: If the server returns a non-2xx status.
        """
        if capacity <= 0:
            raise ValidationError("capacity must be at least 1")
        payload: Dict[str, Any] = {
            "threshold_ms": threshold_ms,
            "capacity": capacity,
        }
        data = await self._transport.post("/slow_queries/config", data=payload)
        return SlowQueryConfig.from_dict(data if isinstance(data, dict) else {})

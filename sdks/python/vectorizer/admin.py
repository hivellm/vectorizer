"""Administrative surface.

Groups: health and indexing progress, file content/listing/summary/chunks,
project outline and related files, Qdrant-compatible cluster/snapshot
admin endpoints, and file-upload endpoints.
"""

from __future__ import annotations

import json as _json
import logging
from typing import Any, Dict, Optional, Union

import aiohttp

try:
    from ..exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from ..models import FileUploadConfig, FileUploadResponse
    from ..utils.validation import validate_non_empty_string
except ImportError:  # pragma: no cover
    from exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import FileUploadConfig, FileUploadResponse
    from utils.validation import validate_non_empty_string

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
        try:
            async with self._transport.get(f"{self.base_url}/indexing/progress") as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get indexing progress: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get indexing progress: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/file/content",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to get file content: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get file content: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/file/list",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to list files: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list files: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/file/summary",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get file summary: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get file summary: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/file/chunks",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get file chunks: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get file chunks: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/file/outline",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get project outline: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get project outline: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/file/related",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get related files: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get related files: {e}")

    # ---- Qdrant snapshot & cluster admin ----

    async def qdrant_list_collection_snapshots(self, collection: str) -> Dict[str, Any]:
        """List snapshots for a collection (Qdrant-compatible API)."""
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/collections/{collection}/snapshots"
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to list snapshots: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list snapshots: {e}")

    async def qdrant_create_collection_snapshot(self, collection: str) -> Dict[str, Any]:
        """Create snapshot for a collection (Qdrant-compatible API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/snapshots"
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to create snapshot: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to create snapshot: {e}")

    async def qdrant_delete_collection_snapshot(
        self, collection: str, snapshot_name: str
    ) -> Dict[str, Any]:
        """Delete snapshot (Qdrant-compatible API)."""
        try:
            async with self._transport.delete(
                f"{self.base_url}/qdrant/collections/{collection}/snapshots/{snapshot_name}"
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Snapshot '{snapshot_name}' not found")
                else:
                    raise ServerError(f"Failed to delete snapshot: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to delete snapshot: {e}")

    async def qdrant_recover_collection_snapshot(
        self, collection: str, location: str
    ) -> Dict[str, Any]:
        """Recover collection from snapshot (Qdrant-compatible API)."""
        try:
            payload = {"location": location}
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/snapshots/recover",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to recover snapshot: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to recover snapshot: {e}")

    async def qdrant_list_all_snapshots(self) -> Dict[str, Any]:
        """List all snapshots (Qdrant-compatible API)."""
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/snapshots"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to list snapshots: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list snapshots: {e}")

    async def qdrant_create_full_snapshot(self) -> Dict[str, Any]:
        """Create full snapshot (Qdrant-compatible API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/snapshots"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to create full snapshot: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to create full snapshot: {e}")

    async def qdrant_list_shard_keys(self, collection: str) -> Dict[str, Any]:
        """List shard keys for a collection (Qdrant-compatible API)."""
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/collections/{collection}/shards"
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to list shard keys: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list shard keys: {e}")

    async def qdrant_create_shard_key(
        self, collection: str, shard_key: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Create shard key (Qdrant-compatible API)."""
        try:
            payload = {"shard_key": shard_key}
            async with self._transport.put(
                f"{self.base_url}/qdrant/collections/{collection}/shards",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to create shard key: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to create shard key: {e}")

    async def qdrant_delete_shard_key(
        self, collection: str, shard_key: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Delete shard key (Qdrant-compatible API)."""
        try:
            payload = {"shard_key": shard_key}
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/shards/delete",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to delete shard key: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to delete shard key: {e}")

    async def qdrant_get_cluster_status(self) -> Dict[str, Any]:
        """Get cluster status (Qdrant-compatible API)."""
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/cluster"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get cluster status: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get cluster status: {e}")

    async def qdrant_cluster_recover(self) -> Dict[str, Any]:
        """Recover current peer (Qdrant-compatible API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/cluster/recover"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to recover cluster: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to recover cluster: {e}")

    async def qdrant_remove_peer(self, peer_id: str) -> Dict[str, Any]:
        """Remove peer from cluster (Qdrant-compatible API)."""
        try:
            async with self._transport.delete(
                f"{self.base_url}/qdrant/cluster/peer/{peer_id}"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to remove peer: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to remove peer: {e}")

    async def qdrant_list_metadata_keys(self) -> Dict[str, Any]:
        """List metadata keys (Qdrant-compatible API)."""
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/cluster/metadata/keys"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to list metadata keys: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list metadata keys: {e}")

    async def qdrant_get_metadata_key(self, key: str) -> Dict[str, Any]:
        """Get metadata key (Qdrant-compatible API)."""
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/cluster/metadata/keys/{key}"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get metadata key: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get metadata key: {e}")

    async def qdrant_update_metadata_key(
        self, key: str, value: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Update metadata key (Qdrant-compatible API)."""
        try:
            payload = {"value": value}
            async with self._transport.put(
                f"{self.base_url}/qdrant/cluster/metadata/keys/{key}",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to update metadata key: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to update metadata key: {e}")

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

            async with self._transport.post(
                f"{self.base_url}/files/upload",
                data=form_data
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    result = FileUploadResponse(**data)
                    logger.info(
                        f"File uploaded successfully: {filename} - "
                        f"{result.chunks_created} chunks, {result.vectors_created} vectors"
                    )
                    return result
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(error_data.get("message", "Invalid file"))
                elif response.status == 413:
                    raise ValidationError("File too large")
                else:
                    raise ServerError(f"Failed to upload file: {response.status}")

        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error uploading file: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error uploading file: {e}")
            raise NetworkError(f"Failed to upload file: {e}")

    async def upload_file_content(
        self,
        content: Union[str, bytes],
        filename: str,
        collection_name: str,
        chunk_size: Optional[int] = None,
        chunk_overlap: Optional[int] = None,
        metadata: Optional[Dict[str, Any]] = None,
        public_key: Optional[str] = None
    ) -> FileUploadResponse:
        """
        Upload file content directly for indexing.

        Similar to :meth:`upload_file`, but accepts content directly instead of a path.
        """
        try:
            validate_non_empty_string(filename)
            validate_non_empty_string(collection_name)

            logger.debug(f"Uploading content as: {filename} to collection: {collection_name}")

            if isinstance(content, str):
                file_content = content.encode('utf-8')
            else:
                file_content = content

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

            async with self._transport.post(
                f"{self.base_url}/files/upload",
                data=form_data
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    result = FileUploadResponse(**data)
                    logger.info(
                        f"Content uploaded successfully: {filename} - "
                        f"{result.chunks_created} chunks, {result.vectors_created} vectors"
                    )
                    return result
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(error_data.get("message", "Invalid content"))
                elif response.status == 413:
                    raise ValidationError("Content too large")
                else:
                    raise ServerError(f"Failed to upload content: {response.status}")

        except (ValidationError, ValueError) as e:
            logger.error(f"Validation error uploading content: {e}")
            if isinstance(e, ValueError):
                raise ValidationError(str(e))
            raise
        except aiohttp.ClientError as e:
            logger.error(f"Network error uploading content: {e}")
            raise NetworkError(f"Failed to upload content: {e}")

    async def get_upload_config(self) -> FileUploadConfig:
        """Get file upload configuration from the server."""
        try:
            logger.debug("Getting file upload configuration")

            async with self._transport.get(
                f"{self.base_url}/files/config"
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    result = FileUploadConfig(**data)
                    logger.debug(
                        f"Upload config retrieved: max_size={result.max_file_size_mb}MB, "
                        f"extensions={len(result.allowed_extensions)}"
                    )
                    return result
                else:
                    raise ServerError(f"Failed to get upload config: {response.status}")

        except aiohttp.ClientError as e:
            logger.error(f"Network error getting upload config: {e}")
            raise NetworkError(f"Failed to get upload config: {e}")

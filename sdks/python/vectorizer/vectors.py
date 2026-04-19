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

        try:
            async with self._transport.post(
                f"{self.base_url}/embed",
                json=payload
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return data.get("embedding", [])
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to generate embedding: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to generate embedding: {e}")

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

        try:
            async with self._transport.post(
                f"{self.base_url}/collections/{collection}/vectors",
                json=payload
            ) as response:
                if response.status == 201:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to insert vectors: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to insert vectors: {e}")

    async def get_vector(self, collection: str, vector_id: str) -> Vector:
        """
        Get a specific vector by ID.

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
        try:
            async with self._transport.get(
                f"{self.base_url}/collections/{collection}/vectors/{vector_id}"
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return Vector(**data)
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Vector '{vector_id}' not found in collection '{collection}'")
                else:
                    raise ServerError(f"Failed to get vector: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get vector: {e}")

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

        payload = {
            "collection": collection,
            "vector_ids": vector_ids
        }

        try:
            async with self._transport.delete(
                f"{self.base_url}/collections/{collection}/vectors",
                json=payload
            ) as response:
                if response.status == 200:
                    return True
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to delete vectors: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to delete vectors: {e}")

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
            async with self._transport.post(
                f"{self.base_url}/batch_insert",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Batch insert completed: {data['successful_operations']} successful, {data['failed_operations']} failed")
                    return BatchResponse(**data)
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to batch insert texts: {response.status}")
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
            async with self._transport.post(
                f"{self.base_url}/batch_update",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Batch update completed: {data['successful_operations']} successful, {data['failed_operations']} failed")
                    return BatchResponse(**data)
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to batch update vectors: {response.status}")
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
            async with self._transport.post(
                f"{self.base_url}/batch_delete",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Batch delete completed: {data['successful_operations']} successful, {data['failed_operations']} failed")
                    return BatchResponse(**data)
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to batch delete vectors: {response.status}")
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

            async with self._transport.put(
                f"{self.base_url}/qdrant/collections/{collection}/points",
                json=payload_data
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid points: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to upsert points: {response.status}")
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
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/delete",
                json={"points": point_ids, "wait": wait}
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid point IDs: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to delete points: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to delete points: {e}")

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
        try:
            params = {
                "ids": ",".join(str(pid) for pid in point_ids),
                "with_payload": str(with_payload).lower(),
                "with_vector": str(with_vector).lower()
            }
            async with self._transport.get(
                f"{self.base_url}/qdrant/collections/{collection}/points",
                params=params
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to retrieve points: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to retrieve points: {e}")

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

            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/count",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to count points: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to count points: {e}")

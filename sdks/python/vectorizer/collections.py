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
    from ..models import CollectionInfo, ReadOptions
except ImportError:  # pragma: no cover
    from exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import CollectionInfo, ReadOptions

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
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/collections"
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to list collections: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list collections: {e}")

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
        try:
            async with self._transport.get(
                f"{self.base_url}/qdrant/collections/{name}"
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{name}' not found")
                else:
                    raise ServerError(f"Failed to get collection: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get collection: {e}")

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
        try:
            async with self._transport.put(
                f"{self.base_url}/qdrant/collections/{name}",
                json={"config": config},
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(
                        f"Invalid configuration: {error_data.get('message', 'Unknown error')}"
                    )
                else:
                    raise ServerError(f"Failed to create collection: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to create collection: {e}")

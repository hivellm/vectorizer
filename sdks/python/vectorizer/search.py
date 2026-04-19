"""Search surface — dense/sparse/hybrid search, discovery, Qdrant query API."""

from __future__ import annotations

import logging
from dataclasses import asdict
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
        BatchSearchRequest,
        BatchSearchResponse,
        ContextualSearchRequest,
        ContextualSearchResponse,
        HybridSearchRequest,
        HybridSearchResponse,
        HybridSearchResult,
        IntelligentSearchRequest,
        IntelligentSearchResponse,
        MultiCollectionSearchRequest,
        MultiCollectionSearchResponse,
        SearchResult,
        SemanticSearchRequest,
        SemanticSearchResponse,
    )
except ImportError:  # pragma: no cover
    from exceptions import (
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
    )
    from models import (
        BatchSearchRequest,
        BatchSearchResponse,
        ContextualSearchRequest,
        ContextualSearchResponse,
        HybridSearchRequest,
        HybridSearchResponse,
        HybridSearchResult,
        IntelligentSearchRequest,
        IntelligentSearchResponse,
        MultiCollectionSearchRequest,
        MultiCollectionSearchResponse,
        SearchResult,
        SemanticSearchRequest,
        SemanticSearchResponse,
    )

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class SearchClient(_ApiBase):
    """All search flavors: dense, sparse, hybrid, intelligent, Qdrant-compat."""

    async def search_vectors(
        self,
        collection: str,
        query: str,
        limit: int = 10,
        filter: Optional[Dict[str, Any]] = None
    ) -> List[SearchResult]:
        """
        Search for similar vectors.

        Args:
            collection: Collection name
            query: Search query text
            limit: Maximum number of results
            filter: Optional metadata filter

        Returns:
            List of search results

        Raises:
            CollectionNotFoundError: If collection doesn't exist
            ValidationError: If parameters are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        if not query or not isinstance(query, str):
            raise ValidationError("Query must be a non-empty string")

        if limit <= 0:
            raise ValidationError("Limit must be positive")

        payload: Dict[str, Any] = {
            "collection": collection,
            "query": query,
            "limit": limit
        }

        if filter:
            payload["filter"] = filter

        try:
            async with self._transport.post(
                f"{self.base_url}/collections/{collection}/search",
                json=payload
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return [SearchResult(**result) for result in data.get("results", [])]
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to search vectors: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to search vectors: {e}")

    async def intelligent_search(self, request: IntelligentSearchRequest) -> IntelligentSearchResponse:
        """Advanced intelligent search with multi-query expansion and semantic reranking."""
        try:
            async with self._transport.post(
                f"{self.base_url}/intelligent_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'queries_generated': data.get('queries_generated'),
                        'collections_searched': data.get('collections_searched'),
                        'metadata': data.get('metadata'),
                    }
                    return IntelligentSearchResponse(**{
                        k: v for k, v in filtered_data.items()
                        if v is not None or k in ['results', 'total_results', 'duration_ms']
                    })
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform intelligent search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform intelligent search: {e}")

    async def semantic_search(self, request: SemanticSearchRequest) -> SemanticSearchResponse:
        """Semantic search with advanced reranking and similarity thresholds."""
        try:
            async with self._transport.post(
                f"{self.base_url}/semantic_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'collection': data.get('collection', ''),
                        'metadata': data.get('metadata'),
                    }
                    return SemanticSearchResponse(**{
                        k: v for k, v in filtered_data.items()
                        if v is not None or k in ['results', 'total_results', 'duration_ms', 'collection']
                    })
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform semantic search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform semantic search: {e}")

    async def contextual_search(self, request: ContextualSearchRequest) -> ContextualSearchResponse:
        """Context-aware search with metadata filtering and contextual reranking."""
        try:
            async with self._transport.post(
                f"{self.base_url}/contextual_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'collection': data.get('collection', ''),
                        'context_filters': data.get('context_filters'),
                        'metadata': data.get('metadata'),
                    }
                    return ContextualSearchResponse(**{
                        k: v for k, v in filtered_data.items()
                        if v is not None or k in ['results', 'total_results', 'duration_ms', 'collection']
                    })
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform contextual search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform contextual search: {e}")

    async def multi_collection_search(
        self, request: MultiCollectionSearchRequest
    ) -> MultiCollectionSearchResponse:
        """Multi-collection search with cross-collection reranking and aggregation."""
        try:
            async with self._transport.post(
                f"{self.base_url}/multi_collection_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'collections_searched': data.get('collections_searched', []),
                        'results_per_collection': data.get('results_per_collection'),
                        'metadata': data.get('metadata'),
                    }
                    return MultiCollectionSearchResponse(**{
                        k: v for k, v in filtered_data.items()
                        if v is not None or k in ['results', 'total_results', 'duration_ms', 'collections_searched']
                    })
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform multi-collection search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform multi-collection search: {e}")

    async def hybrid_search(self, request: HybridSearchRequest) -> HybridSearchResponse:
        """Perform hybrid search combining dense and sparse vectors."""
        try:
            payload: Dict[str, Any] = {
                "query": request.query,
                "alpha": request.alpha,
                "algorithm": request.algorithm,
                "dense_k": request.dense_k,
                "sparse_k": request.sparse_k,
                "final_k": request.final_k,
            }
            if request.query_sparse:
                payload["query_sparse"] = {
                    "indices": request.query_sparse.indices,
                    "values": request.query_sparse.values,
                }

            async with self._transport.post(
                f"{self.base_url}/collections/{request.collection}/hybrid_search",
                json=payload
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    results = [
                        HybridSearchResult(
                            id=r["id"],
                            score=r["score"],
                            vector=r.get("vector"),
                            payload=r.get("payload"),
                        )
                        for r in data.get("results", [])
                    ]
                    return HybridSearchResponse(
                        results=results,
                        query=data.get("query", request.query),
                        query_sparse=data.get("query_sparse"),
                        alpha=data.get("alpha", request.alpha),
                        algorithm=data.get("algorithm", request.algorithm),
                        duration_ms=data.get("duration_ms"),
                    )
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{request.collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform hybrid search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform hybrid search: {e}")

    async def batch_search_vectors(
        self, collection: str, request: BatchSearchRequest
    ) -> BatchSearchResponse:
        """Batch search vectors in a collection."""
        logger.debug(f"Batch searching with {len(request.queries)} queries in collection '{collection}'")

        try:
            async with self._transport.post(
                f"{self.base_url}/batch_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Batch search completed: {data['successful_queries']} successful, {data['failed_queries']} failed")
                    return BatchSearchResponse(**data)
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to batch search vectors: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to batch search vectors: {e}")

    # =============================================================================
    # DISCOVERY OPERATIONS
    # =============================================================================

    async def discover(
        self,
        query: str,
        include_collections: Optional[List[str]] = None,
        exclude_collections: Optional[List[str]] = None,
        max_bullets: int = 20,
        broad_k: int = 50,
        focus_k: int = 15
    ) -> Dict[str, Any]:
        """Complete discovery pipeline with intelligent search and prompt generation."""
        payload: Dict[str, Any] = {"query": query}
        if include_collections:
            payload["include_collections"] = include_collections
        if exclude_collections:
            payload["exclude_collections"] = exclude_collections
        payload["max_bullets"] = max_bullets
        payload["broad_k"] = broad_k
        payload["focus_k"] = focus_k

        try:
            async with self._transport.post(
                f"{self.base_url}/discover",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to discover: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to discover: {e}")

    async def filter_collections(
        self,
        query: str,
        include: Optional[List[str]] = None,
        exclude: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        """Pre-filter collections by name patterns."""
        payload: Dict[str, Any] = {"query": query}
        if include:
            payload["include"] = include
        if exclude:
            payload["exclude"] = exclude

        try:
            async with self._transport.post(
                f"{self.base_url}/discovery/filter_collections",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to filter collections: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to filter collections: {e}")

    async def score_collections(
        self,
        query: str,
        name_match_weight: float = 0.4,
        term_boost_weight: float = 0.3,
        signal_boost_weight: float = 0.3
    ) -> Dict[str, Any]:
        """Rank collections by relevance."""
        payload = {
            "query": query,
            "name_match_weight": name_match_weight,
            "term_boost_weight": term_boost_weight,
            "signal_boost_weight": signal_boost_weight
        }

        try:
            async with self._transport.post(
                f"{self.base_url}/discovery/score_collections",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to score collections: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to score collections: {e}")

    async def expand_queries(
        self,
        query: str,
        max_expansions: int = 8,
        include_definition: bool = True,
        include_features: bool = True,
        include_architecture: bool = True
    ) -> Dict[str, Any]:
        """Generate query variations."""
        payload = {
            "query": query,
            "max_expansions": max_expansions,
            "include_definition": include_definition,
            "include_features": include_features,
            "include_architecture": include_architecture
        }

        try:
            async with self._transport.post(
                f"{self.base_url}/discovery/expand_queries",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to expand queries: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to expand queries: {e}")

    async def search_by_file_type(
        self,
        collection: str,
        query: str,
        file_types: List[str],
        limit: int = 10,
        return_full_files: bool = False
    ) -> Dict[str, Any]:
        """Semantic search filtered by file type."""
        payload = {
            "collection": collection,
            "query": query,
            "file_types": file_types,
            "limit": limit,
            "return_full_files": return_full_files
        }

        try:
            async with self._transport.post(
                f"{self.base_url}/file/search_by_type",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to search by file type: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to search by file type: {e}")

    # =============================================================================
    # QDRANT SEARCH + QUERY API
    # =============================================================================

    async def qdrant_search_points(
        self,
        collection: str,
        vector: List[float],
        limit: int = 10,
        filter: Optional[Dict[str, Any]] = None,
        with_payload: bool = True,
        with_vector: bool = False,
    ) -> Dict[str, Any]:
        """Search points in collection (Qdrant-compatible API)."""
        try:
            payload: Dict[str, Any] = {
                "vector": vector,
                "limit": limit,
                "with_payload": with_payload,
                "with_vector": with_vector
            }
            if filter:
                payload["filter"] = filter

            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/search",
                json=payload
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid search request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to search points: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to search points: {e}")

    async def qdrant_query_points(
        self, collection: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Query points (Qdrant 1.7+ Query API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/query",
                json=request
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to query points: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to query points: {e}")

    async def qdrant_batch_query_points(
        self, collection: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Batch query points (Qdrant 1.7+ Query API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/query/batch",
                json=request
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to batch query points: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to batch query points: {e}")

    async def qdrant_query_points_groups(
        self, collection: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Query points with groups (Qdrant 1.7+ Query API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/query/groups",
                json=request
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to query points groups: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to query points groups: {e}")

    async def qdrant_search_points_groups(
        self, collection: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Search points with groups (Qdrant Search Groups API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/search/groups",
                json=request
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to search points groups: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to search points groups: {e}")

    async def qdrant_search_matrix_pairs(
        self, collection: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Search matrix pairs (Qdrant Search Matrix API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/search/matrix/pairs",
                json=request
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to search matrix pairs: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to search matrix pairs: {e}")

    async def qdrant_search_matrix_offsets(
        self, collection: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Search matrix offsets (Qdrant Search Matrix API)."""
        try:
            async with self._transport.post(
                f"{self.base_url}/qdrant/collections/{collection}/points/search/matrix/offsets",
                json=request
            ) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{collection}' not found")
                else:
                    raise ServerError(f"Failed to search matrix offsets: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to search matrix offsets: {e}")

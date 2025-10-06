"""
Hive Vectorizer Client

Main client class for interacting with the Hive Vectorizer service.
Provides high-level methods for vector operations, semantic search,
and collection management.
"""

import asyncio
import json
import logging
from typing import List, Dict, Any, Optional, Union
from urllib.parse import urljoin
import aiohttp
from dataclasses import asdict

from exceptions import (
    VectorizerError,
    AuthenticationError,
    CollectionNotFoundError,
    ValidationError,
    NetworkError,
    ServerError
)
from models import (
    Vector,
    Collection,
    SearchResult,
    EmbeddingRequest,
    SearchRequest,
    CollectionInfo,
    BatchInsertRequest,
    BatchSearchRequest,
    BatchUpdateRequest,
    BatchDeleteRequest,
    BatchResponse,
    BatchSearchResponse,
    BatchTextRequest,
    BatchSearchQuery,
    BatchVectorUpdate,
    BatchConfig,
    # Summarization models
    SummarizeTextRequest,
    SummarizeTextResponse,
    SummarizeContextRequest,
    SummarizeContextResponse,
    GetSummaryResponse,
    SummaryInfo,
    ListSummariesResponse,
    # Intelligent search models
    IntelligentSearchRequest,
    IntelligentSearchResponse,
    SemanticSearchRequest,
    SemanticSearchResponse,
    ContextualSearchRequest,
    ContextualSearchResponse,
    MultiCollectionSearchRequest,
    MultiCollectionSearchResponse,
    IntelligentSearchResult,
)

logger = logging.getLogger(__name__)


class VectorizerClient:
    """
    Main client for interacting with the Hive Vectorizer service.
    
    This client provides both HTTP/REST and WebSocket interfaces for
    communicating with the vectorizer service.
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:15002",
        ws_url: str = "ws://localhost:15002/ws",
        api_key: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3
    ):
        """
        Initialize the Vectorizer client.
        
        Args:
            base_url: Base URL for HTTP API
            ws_url: WebSocket URL for real-time communication
            api_key: API key for authentication
            timeout: Request timeout in seconds
            max_retries: Maximum number of retry attempts
        """
        self.base_url = base_url.rstrip('/')
        self.ws_url = ws_url
        self.api_key = api_key
        self.timeout = timeout
        self.max_retries = max_retries
        self._session: Optional[aiohttp.ClientSession] = None
        self._ws_connection: Optional[websockets.WebSocketServerProtocol] = None
        
    async def __aenter__(self):
        """Async context manager entry."""
        await self.connect()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
        
    async def connect(self):
        """Initialize HTTP session."""
        if self._session is None or self._session.closed:
            headers = {}
            if self.api_key:
                headers["Authorization"] = f"Bearer {self.api_key}"
                
            timeout = aiohttp.ClientTimeout(total=self.timeout)
            self._session = aiohttp.ClientSession(
                headers=headers,
                timeout=timeout
            )
            
    async def close(self):
        """Close HTTP session and WebSocket connection."""
        if self._session and not self._session.closed:
            await self._session.close()
            
        if self._ws_connection:
            await self._ws_connection.close()
            
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
            async with self._session.get(f"{self.base_url}/health") as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Health check failed: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to connect to service: {e}")
            
    # Collection Management
    
    async def list_collections(self) -> List[CollectionInfo]:
        """
        List all available collections.
        
        Returns:
            List of collection information
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.get(f"{self.base_url}/collections") as response:
                if response.status == 200:
                    data = await response.json()
                    return [CollectionInfo(**collection) for collection in data.get("collections", [])]
                else:
                    raise ServerError(f"Failed to list collections: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list collections: {e}")
            
    async def get_collection_info(self, name: str) -> CollectionInfo:
        """
        Get information about a specific collection.
        
        Args:
            name: Collection name
            
        Returns:
            Collection information
            
        Raises:
            CollectionNotFoundError: If collection doesn't exist
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.get(f"{self.base_url}/collections/{name}") as response:
                if response.status == 200:
                    data = await response.json()
                    return CollectionInfo(**data)
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{name}' not found")
                else:
                    raise ServerError(f"Failed to get collection info: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get collection info: {e}")
            
    async def create_collection(
        self,
        name: str,
        dimension: int = 512,
        similarity_metric: str = "cosine",
        description: Optional[str] = None
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
            
        payload = {
            "name": name,
            "dimension": dimension,
            "similarity_metric": similarity_metric
        }
        
        if description:
            payload["description"] = description
            
        try:
            async with self._session.post(
                f"{self.base_url}/collections",
                json=payload
            ) as response:
                if response.status == 201:
                    data = await response.json()
                    return CollectionInfo(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to create collection: {response.status}")
        except aiohttp.ClientError as e:
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
            async with self._session.delete(f"{self.base_url}/collections/{name}") as response:
                if response.status == 200:
                    return True
                elif response.status == 404:
                    raise CollectionNotFoundError(f"Collection '{name}' not found")
                else:
                    raise ServerError(f"Failed to delete collection: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to delete collection: {e}")
            
    # Vector Operations
    
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
            async with self._session.post(
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
        vectors: List[Vector]
    ) -> Dict[str, Any]:
        """
        Insert vectors into a collection.
        
        Args:
            collection: Collection name
            vectors: List of vectors to insert
            
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
            
        payload = {
            "collection": collection,
            "vectors": [asdict(vector) for vector in vectors]
        }
        
        try:
            async with self._session.post(
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
            
        payload = {
            "collection": collection,
            "query": query,
            "limit": limit
        }
        
        if filter:
            payload["filter"] = filter
            
        try:
            async with self._session.post(
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
    
    # ===== INTELLIGENT SEARCH OPERATIONS =====
    
    async def intelligent_search(self, request: IntelligentSearchRequest) -> IntelligentSearchResponse:
        """
        Advanced intelligent search with multi-query expansion and semantic reranking.
        
        Args:
            request: Intelligent search request
            
        Returns:
            Intelligent search response
            
        Raises:
            ValidationError: If parameters are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.post(
                f"{self.base_url}/intelligent_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return IntelligentSearchResponse(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform intelligent search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform intelligent search: {e}")
    
    async def semantic_search(self, request: SemanticSearchRequest) -> SemanticSearchResponse:
        """
        Semantic search with advanced reranking and similarity thresholds.
        
        Args:
            request: Semantic search request
            
        Returns:
            Semantic search response
            
        Raises:
            ValidationError: If parameters are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.post(
                f"{self.base_url}/semantic_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return SemanticSearchResponse(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform semantic search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform semantic search: {e}")
    
    async def contextual_search(self, request: ContextualSearchRequest) -> ContextualSearchResponse:
        """
        Context-aware search with metadata filtering and contextual reranking.
        
        Args:
            request: Contextual search request
            
        Returns:
            Contextual search response
            
        Raises:
            ValidationError: If parameters are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.post(
                f"{self.base_url}/contextual_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return ContextualSearchResponse(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform contextual search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform contextual search: {e}")
    
    async def multi_collection_search(self, request: MultiCollectionSearchRequest) -> MultiCollectionSearchResponse:
        """
        Multi-collection search with cross-collection reranking and aggregation.
        
        Args:
            request: Multi-collection search request
            
        Returns:
            Multi-collection search response
            
        Raises:
            ValidationError: If parameters are invalid
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.post(
                f"{self.base_url}/multi_collection_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    return MultiCollectionSearchResponse(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to perform multi-collection search: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to perform multi-collection search: {e}")
            
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
            async with self._session.get(
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
            async with self._session.delete(
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
            async with self._session.post(
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

    async def batch_search_vectors(
        self, 
        collection: str, 
        request: BatchSearchRequest
    ) -> BatchSearchResponse:
        """
        Batch search vectors in a collection.
        
        Args:
            collection: Collection name
            request: Batch search request
            
        Returns:
            Batch search response
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        """
        logger.debug(f"Batch searching with {len(request.queries)} queries in collection '{collection}'")
        
        try:
            async with self._session.post(
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
            async with self._session.post(
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
            async with self._session.post(
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

    # =============================================================================
    # SUMMARIZATION METHODS
    # =============================================================================

    # NOTE: Summarization endpoints are not available in the current server version
    # The following methods are commented out until summarization is re-implemented

    """
    async def summarize_text(
        self, 
        request: SummarizeTextRequest
    ) -> SummarizeTextResponse:
        \"\"\"
        Summarize text using various methods.
        
        Args:
            request: Summarization request
            
        Returns:
            Summarization response
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        \"\"\"
        logger.debug(f"Summarizing text using method '{request.method}'")
        
        try:
            async with self._session.post(
                f"{self.base_url}/summarize/text",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Text summarized successfully: {data['summary_length']} chars from {data['original_length']} chars")
                    return SummarizeTextResponse(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to summarize text: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to summarize text: {e}")

    async def summarize_context(
        self, 
        request: SummarizeContextRequest
    ) -> SummarizeContextResponse:
        \"\"\"
        Summarize context using various methods.
        
        Args:
            request: Context summarization request
            
        Returns:
            Context summarization response
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        \"\"\"
        logger.debug(f"Summarizing context using method '{request.method}'")
        
        try:
            async with self._session.post(
                f"{self.base_url}/summarize/context",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Context summarized successfully: {data['summary_length']} chars from {data['original_length']} chars")
                    return SummarizeContextResponse(**data)
                elif response.status == 400:
                    error_data = await response.json()
                    raise ValidationError(f"Invalid request: {error_data.get('message', 'Unknown error')}")
                else:
                    raise ServerError(f"Failed to summarize context: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to summarize context: {e}")

    async def get_summary(
        self, 
        summary_id: str
    ) -> GetSummaryResponse:
        \"\"\"
        Get a specific summary by ID.
        
        Args:
            summary_id: Summary ID
            
        Returns:
            Summary response
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If summary not found
        \"\"\"
        logger.debug(f"Getting summary '{summary_id}'")
        
        try:
            async with self._session.get(
                f"{self.base_url}/summaries/{summary_id}"
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Summary retrieved successfully: {data['summary_length']} chars")
                    return GetSummaryResponse(**data)
                elif response.status == 404:
                    raise ValidationError(f"Summary '{summary_id}' not found")
                else:
                    raise ServerError(f"Failed to get summary: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get summary: {e}")

    async def list_summaries(
        self, 
        method: Optional[str] = None,
        language: Optional[str] = None,
        limit: Optional[int] = None,
        offset: Optional[int] = None
    ) -> ListSummariesResponse:
        \"\"\"
        List summaries with optional filtering.
        
        Args:
            method: Filter by summarization method
            language: Filter by language
            limit: Maximum number of summaries to return
            offset: Offset for pagination
            
        Returns:
            List of summaries response
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        \"\"\"
        logger.debug(f"Listing summaries with filters: method={method}, language={language}, limit={limit}, offset={offset}")
        
        params = {}
        if method:
            params['method'] = method
        if language:
            params['language'] = language
        if limit:
            params['limit'] = limit
        if offset:
            params['offset'] = offset
        
        try:
            async with self._session.get(
                f"{self.base_url}/summaries",
                params=params
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Retrieved {len(data['summaries'])} summaries (total: {data['total_count']})")
                    return ListSummariesResponse(**data)
                else:
                    raise ServerError(f"Failed to list summaries: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to list summaries: {e}")
    """

    async def get_indexing_progress(self) -> Dict[str, Any]:
        """
        Get indexing progress information.
        
        Returns:
            Indexing progress data
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
        """
        try:
            async with self._session.get(f"{self.base_url}/indexing/progress") as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get indexing progress: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get indexing progress: {e}")

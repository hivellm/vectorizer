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
from utils.transport import TransportFactory, TransportProtocol, parse_connection_string
from utils.http_client import HTTPClient

logger = logging.getLogger(__name__)


class VectorizerClient:
    """
    Main client for interacting with the Hive Vectorizer service.
    
    This client supports multiple transport protocols:
    - HTTP/HTTPS (default)
    - UMICP (Universal Messaging and Inter-process Communication Protocol)
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:15002",
        ws_url: str = "ws://localhost:15002/ws",
        api_key: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3,
        connection_string: Optional[str] = None,
        protocol: Optional[str] = None,
        umicp: Optional[Dict[str, Any]] = None
    ):
        """
        Initialize the Vectorizer client.
        
        Args:
            base_url: Base URL for HTTP API
            ws_url: WebSocket URL for real-time communication
            api_key: API key for authentication
            timeout: Request timeout in seconds
            max_retries: Maximum number of retry attempts
            connection_string: Connection string (supports http://, https://, umicp://)
            protocol: Protocol to use ('http' or 'umicp')
            umicp: UMICP-specific configuration dict
        """
        self.base_url = base_url.rstrip('/')
        self.ws_url = ws_url
        self.api_key = api_key
        self.timeout = timeout
        self.max_retries = max_retries
        self._session: Optional[aiohttp.ClientSession] = None
        self._ws_connection: Optional[websockets.WebSocketServerProtocol] = None
        
        # Determine protocol and create transport
        if connection_string:
            # Use connection string
            proto, config = parse_connection_string(connection_string, api_key)
            config['timeout'] = timeout
            config['max_retries'] = max_retries
            self._transport = TransportFactory.create(proto, config)
            self._protocol = proto
            logger.info(f"VectorizerClient initialized from connection string (protocol: {proto})")
        elif protocol:
            # Use explicit protocol
            proto_enum = TransportProtocol(protocol.lower())
            
            if proto_enum == TransportProtocol.HTTP:
                config = {
                    "base_url": base_url,
                    "api_key": api_key,
                    "timeout": timeout,
                    "max_retries": max_retries
                }
                self._transport = TransportFactory.create(proto_enum, config)
                self._protocol = proto_enum
            elif proto_enum == TransportProtocol.UMICP:
                if not umicp:
                    raise ValueError("UMICP configuration is required when using UMICP protocol")
                
                config = {
                    "host": umicp.get("host", "localhost"),
                    "port": umicp.get("port", 15003),
                    "api_key": api_key,
                    "timeout": timeout
                }
                self._transport = TransportFactory.create(proto_enum, config)
                self._protocol = proto_enum
                logger.info(f"VectorizerClient initialized with UMICP (host: {config['host']}, port: {config['port']})")
        else:
            # Use default HTTP transport
            config = {
                "base_url": base_url,
                "api_key": api_key,
                "timeout": timeout,
                "max_retries": max_retries
            }
            self._transport = HTTPClient(**config)
            self._protocol = TransportProtocol.HTTP
            logger.info(f"VectorizerClient initialized with HTTP (base_url: {base_url})")
    
    def get_protocol(self) -> str:
        """Get the current transport protocol being used."""
        return self._protocol.value if hasattr(self._protocol, 'value') else str(self._protocol)
        
    async def __aenter__(self):
        """Async context manager entry."""
        await self.connect()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
        
    async def connect(self):
        """Initialize transport session."""
        if self._protocol == TransportProtocol.HTTP:
            # For HTTP, ensure session is created
            if self._session is None or self._session.closed:
                headers = {}
                if self.api_key:
                    headers["Authorization"] = f"Bearer {self.api_key}"
                    
                timeout = aiohttp.ClientTimeout(total=self.timeout)
                self._session = aiohttp.ClientSession(
                    headers=headers,
                    timeout=timeout
                )
        elif self._protocol == TransportProtocol.UMICP:
            # For UMICP, connect via transport
            if hasattr(self._transport, 'connect'):
                await self._transport.connect()
            
    async def close(self):
        """Close transport session and WebSocket connection."""
        # Close HTTP session if exists
        if self._session and not self._session.closed:
            await self._session.close()
        
        # Close transport
        if hasattr(self._transport, 'close'):
            await self._transport.close()
        elif hasattr(self._transport, 'disconnect'):
            await self._transport.disconnect()
            
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
            async with self._transport.get(f"{self.base_url}/health") as response:
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
            async with self._transport.get(f"{self.base_url}/collections") as response:
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
            async with self._transport.get(f"{self.base_url}/collections/{name}") as response:
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
            async with self._transport.post(
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
            async with self._transport.delete(f"{self.base_url}/collections/{name}") as response:
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
            async with self._transport.post(
                f"{self.base_url}/intelligent_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    # Filter only known fields
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'queries_generated': data.get('queries_generated'),
                        'collections_searched': data.get('collections_searched'),
                        'metadata': data.get('metadata'),
                    }
                    return IntelligentSearchResponse(**{k: v for k, v in filtered_data.items() if v is not None or k in ['results', 'total_results', 'duration_ms']})
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
            async with self._transport.post(
                f"{self.base_url}/semantic_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    # Filter only known fields
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'collection': data.get('collection', ''),
                        'metadata': data.get('metadata'),
                    }
                    return SemanticSearchResponse(**{k: v for k, v in filtered_data.items() if v is not None or k in ['results', 'total_results', 'duration_ms', 'collection']})
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
            async with self._transport.post(
                f"{self.base_url}/contextual_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    # Filter only known fields
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'collection': data.get('collection', ''),
                        'context_filters': data.get('context_filters'),
                        'metadata': data.get('metadata'),
                    }
                    return ContextualSearchResponse(**{k: v for k, v in filtered_data.items() if v is not None or k in ['results', 'total_results', 'duration_ms', 'collection']})
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
            async with self._transport.post(
                f"{self.base_url}/multi_collection_search",
                json=asdict(request)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    # Filter only known fields
                    filtered_data = {
                        'results': data.get('results', []),
                        'total_results': data.get('total_results', 0),
                        'duration_ms': data.get('duration_ms', 0),
                        'collections_searched': data.get('collections_searched', []),
                        'results_per_collection': data.get('results_per_collection'),
                        'metadata': data.get('metadata'),
                    }
                    return MultiCollectionSearchResponse(**{k: v for k, v in filtered_data.items() if v is not None or k in ['results', 'total_results', 'duration_ms', 'collections_searched']})
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
            async with self._transport.post(
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
            async with self._transport.post(
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
            async with self._transport.get(
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
            async with self._transport.get(
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
            async with self._transport.get(f"{self.base_url}/indexing/progress") as response:
                if response.status == 200:
                    return await response.json()
                else:
                    raise ServerError(f"Failed to get indexing progress: {response.status}")
        except aiohttp.ClientError as e:
            raise NetworkError(f"Failed to get indexing progress: {e}")
    
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
        """
        Complete discovery pipeline with intelligent search and prompt generation.
        
        Args:
            query: User question or search query
            include_collections: Collections to include (glob patterns)
            exclude_collections: Collections to exclude
            max_bullets: Maximum evidence bullets
            broad_k: Broad search results
            focus_k: Focus search results per collection
            
        Returns:
            Discovery response with LLM-ready prompt
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        """
        payload = {"query": query}
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
        """
        Pre-filter collections by name patterns.
        
        Args:
            query: Search query for filtering
            include: Include patterns
            exclude: Exclude patterns
            
        Returns:
            Filtered collections
        """
        payload = {"query": query}
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
        """
        Rank collections by relevance.
        
        Args:
            query: Search query for scoring
            name_match_weight: Weight for name matching
            term_boost_weight: Weight for term boost
            signal_boost_weight: Weight for signals
            
        Returns:
            Scored collections
        """
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
        """
        Generate query variations.
        
        Args:
            query: Original query to expand
            max_expansions: Maximum expansions
            include_definition: Include definition queries
            include_features: Include features queries
            include_architecture: Include architecture queries
            
        Returns:
            Expanded queries
        """
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
    
    # =============================================================================
    # FILE OPERATIONS
    # =============================================================================
    
    async def get_file_content(
        self,
        collection: str,
        file_path: str,
        max_size_kb: int = 500
    ) -> Dict[str, Any]:
        """
        Retrieve complete file content from a collection.
        
        Args:
            collection: Collection name
            file_path: Relative file path within collection
            max_size_kb: Maximum file size in KB
            
        Returns:
            File content and metadata
            
        Raises:
            NetworkError: If unable to connect to service
            ServerError: If service returns error
            ValidationError: If request is invalid
        """
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
        filter_by_type: Optional[List[str]] = None,
        min_chunks: Optional[int] = None,
        max_results: int = 100,
        sort_by: str = "name"
    ) -> Dict[str, Any]:
        """
        List all indexed files in a collection.
        
        Args:
            collection: Collection name
            filter_by_type: Filter by file types
            min_chunks: Minimum number of chunks
            max_results: Maximum number of results
            sort_by: Sort order (name, size, chunks, recent)
            
        Returns:
            List of files with metadata
        """
        payload = {
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
        """
        Get extractive or structural summary of an indexed file.
        
        Args:
            collection: Collection name
            file_path: Relative file path within collection
            summary_type: Type of summary (extractive, structural, both)
            max_sentences: Maximum sentences for extractive summary
            
        Returns:
            File summary
        """
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
        """
        Retrieve chunks in original file order for progressive reading.
        
        Args:
            collection: Collection name
            file_path: Relative file path within collection
            start_chunk: Starting chunk index
            limit: Number of chunks to retrieve
            include_context: Include prev/next chunk hints
            
        Returns:
            File chunks
        """
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
        """
        Generate hierarchical project structure overview.
        
        Args:
            collection: Collection name
            max_depth: Maximum directory depth
            include_summaries: Include file summaries in outline
            highlight_key_files: Highlight important files like README
            
        Returns:
            Project outline
        """
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
        """
        Find semantically related files using vector similarity.
        
        Args:
            collection: Collection name
            file_path: Reference file path
            limit: Maximum number of related files
            similarity_threshold: Minimum similarity score 0.0-1.0
            include_reason: Include explanation of why files are related
            
        Returns:
            Related files
        """
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
    
    async def search_by_file_type(
        self,
        collection: str,
        query: str,
        file_types: List[str],
        limit: int = 10,
        return_full_files: bool = False
    ) -> Dict[str, Any]:
        """
        Semantic search filtered by file type.
        
        Args:
            collection: Collection name
            query: Search query
            file_types: File extensions to search
            limit: Maximum results
            return_full_files: Return complete file content
            
        Returns:
            Search results
        """
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
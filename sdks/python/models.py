"""
Data models for the Hive Vectorizer SDK.

This module contains all the data models used for representing
vectors, collections, search results, and other entities.
"""

import math
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Union, Literal
from datetime import datetime
from enum import Enum


# ===== CLIENT-SIDE REPLICATION CONFIGURATION =====

class ReadPreference(str, Enum):
    """
    Read preference for routing read operations.
    Similar to MongoDB's read preferences.
    """
    MASTER = "master"
    REPLICA = "replica"
    NEAREST = "nearest"


@dataclass
class HostConfig:
    """
    Host configuration for master/replica topology.
    """
    master: str
    """Master node URL (receives all write operations)"""

    replicas: List[str] = field(default_factory=list)
    """Replica node URLs (receive read operations based on read_preference)"""


@dataclass
class ReadOptions:
    """
    Options that can be passed to read operations for per-operation override.
    """
    read_preference: Optional[ReadPreference] = None
    """Override the default read preference for this operation"""


@dataclass
class Vector:
    """Represents a vector with metadata."""

    id: str
    data: List[float]
    metadata: Optional[Dict[str, Any]] = None
    public_key: Optional[str] = None
    """Optional ECC public key for payload encryption (PEM, base64, or hex format)"""

    def __post_init__(self):
        """Validate vector data after initialization."""
        if not self.id:
            raise ValueError("Vector ID cannot be empty")
        if not self.data:
            raise ValueError("Vector data cannot be empty")

        # Check for valid numbers and reject NaN/Infinity
        for x in self.data:
            if not isinstance(x, (int, float)):
                raise ValueError("Vector data must contain only numbers")
            if math.isnan(x):
                raise ValueError("Vector data must not contain NaN values")
            if math.isinf(x):
                raise ValueError("Vector data must not contain Infinity values")


@dataclass
class Collection:
    """Represents a collection of vectors."""
    
    name: str
    dimension: int
    similarity_metric: str = "cosine"
    description: Optional[str] = None
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None
    
    def __post_init__(self):
        """Validate collection data after initialization."""
        if not self.name:
            raise ValueError("Collection name cannot be empty")
        if self.dimension <= 0:
            raise ValueError("Dimension must be positive")
        if self.similarity_metric not in ["cosine", "euclidean", "dot_product"]:
            raise ValueError("Invalid similarity metric")


@dataclass
class CollectionInfo:
    """Information about a collection."""

    name: str
    dimension: int
    vector_count: int
    similarity_metric: Optional[str] = None
    metric: Optional[str] = None  # Alternative field name from API
    status: Optional[str] = None
    document_count: Optional[int] = None
    error_message: Optional[str] = None
    last_updated: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    embedding_provider: Optional[str] = None
    indexing_status: Optional[dict] = None
    normalization: Optional[dict] = None
    quantization: Optional[dict] = None
    size: Optional[dict] = None

    def __post_init__(self):
        """Validate collection info after initialization."""
        if not self.name:
            raise ValueError("Collection name cannot be empty")
        if self.dimension <= 0:
            raise ValueError("Dimension must be positive")
        if self.vector_count < 0:
            raise ValueError("Vector count cannot be negative")
        # Normalize similarity_metric from metric field if not set
        if not self.similarity_metric and self.metric:
            self.similarity_metric = self.metric.lower()


@dataclass
class SearchResult:
    """Represents a search result."""
    
    id: str
    score: float
    content: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Validate search result data after initialization."""
        if not self.id:
            raise ValueError("SearchResult ID cannot be empty")
        if not isinstance(self.score, (int, float)):
            raise ValueError("Score must be a number")


@dataclass
class EmbeddingRequest:
    """Request for generating embeddings."""
    
    text: str
    model: Optional[str] = None
    normalize: bool = True
    
    def __post_init__(self):
        """Validate embedding request after initialization."""
        if not self.text or not isinstance(self.text, str):
            raise ValueError("Text must be a non-empty string")


@dataclass
class SearchRequest:
    """Request for searching vectors."""
    
    collection: str
    query: str
    limit: int = 10
    filter: Optional[Dict[str, Any]] = None
    include_metadata: bool = True
    
    def __post_init__(self):
        """Validate search request after initialization."""
        if not self.collection:
            raise ValueError("Collection name cannot be empty")
        if not self.query:
            raise ValueError("Query cannot be empty")
        if self.limit <= 0:
            raise ValueError("Limit must be positive")


@dataclass
class BatchOperation:
    """Represents a batch operation."""
    
    operation: str  # "insert", "delete", "update"
    vectors: Optional[List[Vector]] = None
    vector_ids: Optional[List[str]] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Validate batch operation after initialization."""
        if self.operation not in ["insert", "delete", "update"]:
            raise ValueError("Invalid operation type")
        
        if self.operation == "insert" and not self.vectors:
            raise ValueError("Insert operation requires vectors")
        
        if self.operation in ["delete", "update"] and not self.vector_ids:
            raise ValueError("Delete/Update operation requires vector IDs")


@dataclass
class IndexingProgress:
    """Represents indexing progress information."""
    
    is_indexing: bool
    overall_status: str
    collections: List[str] = field(default_factory=list)
    progress_percentage: Optional[float] = None
    estimated_completion: Optional[str] = None
    
    def __post_init__(self):
        """Validate indexing progress after initialization."""
        if self.progress_percentage is not None:
            if not 0 <= self.progress_percentage <= 100:
                raise ValueError("Progress percentage must be between 0 and 100")


@dataclass
class HealthStatus:
    """Represents service health status."""
    
    status: str
    service: str
    version: str
    timestamp: str
    error_message: Optional[str] = None
    
    def __post_init__(self):
        """Validate health status after initialization."""
        if self.status not in ["healthy", "unhealthy", "degraded"]:
            raise ValueError("Invalid health status")


@dataclass
class ClientConfig:
    """Configuration for the Vectorizer client."""
    
    base_url: str = "http://localhost:15001"
    ws_url: str = "ws://localhost:15001/ws"
    api_key: Optional[str] = None
    timeout: int = 30
    max_retries: int = 3
    retry_delay: float = 1.0
    verify_ssl: bool = True
    
    def __post_init__(self):
        """Validate client configuration after initialization."""
        if self.timeout <= 0:
            raise ValueError("Timeout must be positive")
        if self.max_retries < 0:
            raise ValueError("Max retries cannot be negative")
        if self.retry_delay < 0:
            raise ValueError("Retry delay cannot be negative")


# ==================== BATCH OPERATION MODELS ====================

@dataclass
class BatchTextRequest:
    """Text request for batch operations."""
    
    id: str
    text: str
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Validate batch text request after initialization."""
        if not self.id:
            raise ValueError("Text ID cannot be empty")
        if not self.text:
            raise ValueError("Text content cannot be empty")


@dataclass
class BatchConfig:
    """Configuration for batch operations."""
    
    max_batch_size: Optional[int] = None
    parallel_workers: Optional[int] = None
    atomic: Optional[bool] = None


@dataclass
class BatchInsertRequest:
    """Request for batch text insertion."""
    
    texts: List[BatchTextRequest]
    config: Optional[BatchConfig] = None
    
    def __post_init__(self):
        """Validate batch insert request after initialization."""
        if not self.texts:
            raise ValueError("Texts list cannot be empty")


@dataclass
class BatchResponse:
    """Response for batch operations."""
    
    success: bool
    collection: str
    operation: str
    total_operations: int
    successful_operations: int
    failed_operations: int
    duration_ms: int
    errors: List[str] = field(default_factory=list)


@dataclass
class BatchSearchQuery:
    """Search query for batch operations."""
    
    query: str
    limit: Optional[int] = None
    score_threshold: Optional[float] = None
    
    def __post_init__(self):
        """Validate batch search query after initialization."""
        if not self.query:
            raise ValueError("Query cannot be empty")


@dataclass
class BatchSearchRequest:
    """Request for batch search operations."""
    
    queries: List[BatchSearchQuery]
    config: Optional[BatchConfig] = None
    
    def __post_init__(self):
        """Validate batch search request after initialization."""
        if not self.queries:
            raise ValueError("Queries list cannot be empty")


@dataclass
class BatchSearchResponse:
    """Response for batch search operations."""
    
    success: bool
    collection: str
    total_queries: int
    successful_queries: int
    failed_queries: int
    duration_ms: int
    results: List[List[SearchResult]] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)


@dataclass
class BatchVectorUpdate:
    """Vector update for batch operations."""
    
    id: str
    data: Optional[List[float]] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Validate batch vector update after initialization."""
        if not self.id:
            raise ValueError("Vector ID cannot be empty")


@dataclass
class BatchUpdateRequest:
    """Request for batch vector updates."""
    
    updates: List[BatchVectorUpdate]
    config: Optional[BatchConfig] = None
    
    def __post_init__(self):
        """Validate batch update request after initialization."""
        if not self.updates:
            raise ValueError("Updates list cannot be empty")


@dataclass
class BatchDeleteRequest:
    """Request for batch vector deletion."""
    
    vector_ids: List[str]
    config: Optional[BatchConfig] = None
    
    def __post_init__(self):
        """Validate batch delete request after initialization."""
        if not self.vector_ids:
            raise ValueError("Vector IDs list cannot be empty")


# =============================================================================
# SUMMARIZATION MODELS
# =============================================================================

@dataclass
class SummarizeTextRequest:
    """Request to summarize text."""
    
    text: str
    method: str = "extractive"
    max_length: Optional[int] = None
    compression_ratio: Optional[float] = None
    language: Optional[str] = None
    metadata: Optional[Dict[str, str]] = None
    
    def __post_init__(self):
        """Validate summarization request after initialization."""
        if not self.text:
            raise ValueError("Text cannot be empty")
        if self.method not in ["extractive", "keyword", "sentence", "abstractive"]:
            raise ValueError("Invalid summarization method")
        if self.compression_ratio is not None and not (0.1 <= self.compression_ratio <= 0.9):
            raise ValueError("Compression ratio must be between 0.1 and 0.9")


@dataclass
class SummarizeTextResponse:
    """Response for text summarization."""
    
    summary_id: str
    original_text: str
    summary: str
    method: str
    original_length: int
    summary_length: int
    compression_ratio: float
    language: str
    status: str
    message: str
    metadata: Dict[str, str]


@dataclass
class SummarizeContextRequest:
    """Request to summarize context."""
    
    context: str
    method: str = "extractive"
    max_length: Optional[int] = None
    compression_ratio: Optional[float] = None
    language: Optional[str] = None
    metadata: Optional[Dict[str, str]] = None
    
    def __post_init__(self):
        """Validate context summarization request after initialization."""
        if not self.context:
            raise ValueError("Context cannot be empty")
        if self.method not in ["extractive", "keyword", "sentence", "abstractive"]:
            raise ValueError("Invalid summarization method")
        if self.compression_ratio is not None and not (0.1 <= self.compression_ratio <= 0.9):
            raise ValueError("Compression ratio must be between 0.1 and 0.9")


@dataclass
class SummarizeContextResponse:
    """Response for context summarization."""
    
    summary_id: str
    original_context: str
    summary: str
    method: str
    original_length: int
    summary_length: int
    compression_ratio: float
    language: str
    status: str
    message: str
    metadata: Dict[str, str]


@dataclass
class GetSummaryResponse:
    """Response for getting a summary."""
    
    summary_id: str
    original_text: str
    summary: str
    method: str
    original_length: int
    summary_length: int
    compression_ratio: float
    language: str
    created_at: str
    metadata: Dict[str, str]
    status: str


@dataclass
class SummaryInfo:
    """Summary information for listing."""
    
    summary_id: str
    method: str
    language: str
    original_length: int
    summary_length: int
    compression_ratio: float
    created_at: str
    metadata: Dict[str, str]


@dataclass
class ListSummariesResponse:
    """Response for listing summaries."""
    
    summaries: List[SummaryInfo]
    total_count: int
    status: str


# ===== INTELLIGENT SEARCH MODELS =====

@dataclass
class IntelligentSearchRequest:
    """Request for intelligent search."""
    
    query: str
    collections: Optional[List[str]] = None
    max_results: int = 10
    domain_expansion: bool = True
    technical_focus: bool = True
    mmr_enabled: bool = True
    mmr_lambda: float = 0.7
    
    def __post_init__(self):
        if not self.query or not isinstance(self.query, str):
            raise ValueError("Query must be a non-empty string")
        if self.max_results <= 0:
            raise ValueError("Max results must be positive")
        if not (0.0 <= self.mmr_lambda <= 1.0):
            raise ValueError("MMR lambda must be between 0.0 and 1.0")


@dataclass
class SemanticSearchRequest:
    """Request for semantic search."""
    
    query: str
    collection: str
    max_results: int = 10
    semantic_reranking: bool = True
    cross_encoder_reranking: bool = False
    similarity_threshold: float = 0.5
    
    def __post_init__(self):
        if not self.query or not isinstance(self.query, str):
            raise ValueError("Query must be a non-empty string")
        if not self.collection or not isinstance(self.collection, str):
            raise ValueError("Collection must be a non-empty string")
        if self.max_results <= 0:
            raise ValueError("Max results must be positive")
        if not (0.0 <= self.similarity_threshold <= 1.0):
            raise ValueError("Similarity threshold must be between 0.0 and 1.0")


@dataclass
class ContextualSearchRequest:
    """Request for contextual search."""
    
    query: str
    collection: str
    context_filters: Optional[Dict[str, Any]] = None
    max_results: int = 10
    context_reranking: bool = True
    context_weight: float = 0.3
    
    def __post_init__(self):
        if not self.query or not isinstance(self.query, str):
            raise ValueError("Query must be a non-empty string")
        if not self.collection or not isinstance(self.collection, str):
            raise ValueError("Collection must be a non-empty string")
        if self.max_results <= 0:
            raise ValueError("Max results must be positive")
        if not (0.0 <= self.context_weight <= 1.0):
            raise ValueError("Context weight must be between 0.0 and 1.0")


@dataclass
class MultiCollectionSearchRequest:
    """Request for multi-collection search."""
    
    query: str
    collections: List[str]
    max_per_collection: int = 5
    max_total_results: int = 20
    cross_collection_reranking: bool = True
    
    def __post_init__(self):
        if not self.query or not isinstance(self.query, str):
            raise ValueError("Query must be a non-empty string")
        if not self.collections or not isinstance(self.collections, list):
            raise ValueError("Collections must be a non-empty list")
        if self.max_per_collection <= 0:
            raise ValueError("Max per collection must be positive")
        if self.max_total_results <= 0:
            raise ValueError("Max total results must be positive")


@dataclass
class IntelligentSearchResult:
    """Result from intelligent search."""
    
    id: str
    score: float
    content: str
    metadata: Optional[Dict[str, Any]] = None
    collection: Optional[str] = None
    query_used: Optional[str] = None


@dataclass
class IntelligentSearchResponse:
    """Response from intelligent search."""
    
    results: List[IntelligentSearchResult]
    total_results: int
    duration_ms: int = 0
    queries_generated: Optional[List[str]] = None
    collections_searched: Optional[List[str]] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Allow extra fields from server."""
        pass


@dataclass
class SemanticSearchResponse:
    """Response from semantic search."""
    
    results: List[IntelligentSearchResult]
    total_results: int
    duration_ms: int = 0
    collection: str = ""
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Allow extra fields from server."""
        pass


@dataclass
class ContextualSearchResponse:
    """Response from contextual search."""
    
    results: List[IntelligentSearchResult]
    total_results: int
    duration_ms: int = 0
    collection: str = ""
    context_filters: Optional[Dict[str, Any]] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Allow extra fields from server."""
        pass


@dataclass
class MultiCollectionSearchResponse:
    """Response from multi-collection search."""
    
    results: List[IntelligentSearchResult]
    total_results: int
    duration_ms: int = 0
    collections_searched: List[str] = None
    results_per_collection: Optional[Dict[str, int]] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Allow extra fields from server."""
        if self.collections_searched is None:
            self.collections_searched = []


# ==================== HYBRID SEARCH MODELS ====================

@dataclass
class SparseVector:
    """Represents a sparse vector with indices and values."""
    
    indices: List[int]
    values: List[float]
    
    def __post_init__(self):
        """Validate sparse vector after initialization."""
        if len(self.indices) != len(self.values):
            raise ValueError("Indices and values must have the same length")
        if len(self.indices) == 0:
            raise ValueError("Sparse vector cannot be empty")
        for idx in self.indices:
            if idx < 0:
                raise ValueError("Indices must be non-negative")
        for val in self.values:
            if not isinstance(val, (int, float)):
                raise ValueError("Values must be numbers")
            if math.isnan(val) or math.isinf(val):
                raise ValueError("Values must not contain NaN or Infinity")


@dataclass
class HybridSearchRequest:
    """Request for hybrid search (dense + sparse vectors)."""
    
    collection: str
    query: str
    query_sparse: Optional[SparseVector] = None
    alpha: float = 0.7
    algorithm: str = "rrf"  # "rrf", "weighted", "alpha"
    dense_k: int = 20
    sparse_k: int = 20
    final_k: int = 10
    
    def __post_init__(self):
        """Validate hybrid search request after initialization."""
        if not self.collection:
            raise ValueError("Collection name cannot be empty")
        if not self.query:
            raise ValueError("Query cannot be empty")
        if not 0.0 <= self.alpha <= 1.0:
            raise ValueError("Alpha must be between 0.0 and 1.0")
        if self.algorithm not in ["rrf", "weighted", "alpha"]:
            raise ValueError("Algorithm must be 'rrf', 'weighted', or 'alpha'")
        if self.dense_k <= 0 or self.sparse_k <= 0 or self.final_k <= 0:
            raise ValueError("K values must be positive")
        if self.final_k > self.dense_k + self.sparse_k:
            raise ValueError("final_k cannot be greater than dense_k + sparse_k")


@dataclass
class HybridSearchResult:
    """Result from hybrid search."""
    
    id: str
    score: float
    vector: Optional[List[float]] = None
    payload: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Validate hybrid search result after initialization."""
        if not self.id:
            raise ValueError("Result ID cannot be empty")
        if not isinstance(self.score, (int, float)):
            raise ValueError("Score must be a number")


@dataclass
class HybridSearchResponse:
    """Response for hybrid search operations."""
    
    results: List[HybridSearchResult]
    query: str
    alpha: float
    algorithm: str
    query_sparse: Optional[Dict[str, List]] = None
    duration_ms: Optional[int] = None


# ===== REPLICATION MODELS =====

@dataclass
class ReplicaStatus:
    """Status of a replica node."""
    
    status: str  # "Connected", "Syncing", "Lagging", "Disconnected"
    
    def __post_init__(self):
        """Validate replica status."""
        valid_statuses = ["Connected", "Syncing", "Lagging", "Disconnected"]
        if self.status not in valid_statuses:
            raise ValueError(f"Invalid replica status. Must be one of: {valid_statuses}")


@dataclass
class ReplicaInfo:
    """Information about a replica node."""
    
    replica_id: str
    host: str
    port: int
    status: str  # ReplicaStatus enum value
    last_heartbeat: datetime
    operations_synced: int
    # Legacy fields (backwards compatible)
    offset: Optional[int] = None
    lag: Optional[int] = None
    
    def __post_init__(self):
        """Validate replica info."""
        if not self.replica_id:
            raise ValueError("Replica ID cannot be empty")
        if not self.host:
            raise ValueError("Host cannot be empty")
        if self.port <= 0 or self.port > 65535:
            raise ValueError("Port must be between 1 and 65535")
        if self.operations_synced < 0:
            raise ValueError("Operations synced cannot be negative")


@dataclass
class ReplicationStats:
    """Statistics for replication status."""
    
    # New fields (v1.2.0+)
    role: Optional[str] = None  # "Master" or "Replica"
    bytes_sent: Optional[int] = None
    bytes_received: Optional[int] = None
    last_sync: Optional[datetime] = None
    operations_pending: Optional[int] = None
    snapshot_size: Optional[int] = None
    connected_replicas: Optional[int] = None  # Only for Master
    
    # Legacy fields (backwards compatible)
    master_offset: int = 0
    replica_offset: int = 0
    lag_operations: int = 0
    total_replicated: int = 0
    
    def __post_init__(self):
        """Validate replication stats."""
        if self.role is not None and self.role not in ["Master", "Replica"]:
            raise ValueError("Role must be 'Master' or 'Replica'")
        if self.bytes_sent is not None and self.bytes_sent < 0:
            raise ValueError("Bytes sent cannot be negative")
        if self.bytes_received is not None and self.bytes_received < 0:
            raise ValueError("Bytes received cannot be negative")
        if self.operations_pending is not None and self.operations_pending < 0:
            raise ValueError("Operations pending cannot be negative")
        if self.snapshot_size is not None and self.snapshot_size < 0:
            raise ValueError("Snapshot size cannot be negative")
        if self.connected_replicas is not None and self.connected_replicas < 0:
            raise ValueError("Connected replicas cannot be negative")


@dataclass
class ReplicationStatusResponse:
    """Response for replication status endpoint."""
    
    status: str
    stats: ReplicationStats
    message: Optional[str] = None
    
    def __post_init__(self):
        """Validate replication status response."""
        if not self.status:
            raise ValueError("Status cannot be empty")


@dataclass
class ReplicaListResponse:
    """Response for listing replicas."""
    
    replicas: List[ReplicaInfo]
    count: int
    message: str
    
    def __post_init__(self):
        """Validate replica list response."""
        if self.count < 0:
            raise ValueError("Count cannot be negative")
        if self.count != len(self.replicas):
            raise ValueError("Count must match number of replicas")


# ========== Graph Models ==========

@dataclass
class GraphNode:
    """Graph node representing a document/file."""
    
    id: str
    node_type: str
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class GraphEdge:
    """Graph edge representing a relationship between nodes."""
    
    id: str
    source: str
    target: str
    relationship_type: str
    weight: float
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: str = ""


@dataclass
class NeighborInfo:
    """Neighbor information."""
    
    node: GraphNode
    edge: GraphEdge


@dataclass
class RelatedNodeInfo:
    """Related node information."""
    
    node: GraphNode
    distance: int
    weight: float


@dataclass
class FindRelatedRequest:
    """Request to find related nodes."""
    
    max_hops: Optional[int] = None
    relationship_type: Optional[str] = None
    
    def __post_init__(self):
        """Validate find related request data after initialization."""
        if self.max_hops is not None:
            if not isinstance(self.max_hops, int) or self.max_hops < 1:
                raise ValueError("max_hops must be a positive integer")
        
        if self.relationship_type is not None:
            if not isinstance(self.relationship_type, str) or not self.relationship_type.strip():
                raise ValueError("relationship_type must be a non-empty string")


@dataclass
class FindRelatedResponse:
    """Response for finding related nodes."""
    
    related: List[RelatedNodeInfo]


@dataclass
class FindPathRequest:
    """Request to find path between nodes."""
    
    collection: str
    source: str
    target: str
    
    def __post_init__(self):
        """Validate find path request data after initialization."""
        if not isinstance(self.collection, str) or not self.collection.strip():
            raise ValueError("collection must be a non-empty string")
        
        if not isinstance(self.source, str) or not self.source.strip():
            raise ValueError("source must be a non-empty string")
        
        if not isinstance(self.target, str) or not self.target.strip():
            raise ValueError("target must be a non-empty string")


@dataclass
class FindPathResponse:
    """Response for finding path."""
    
    path: List[GraphNode]
    found: bool


@dataclass
class CreateEdgeRequest:
    """Request to create an edge."""
    
    collection: str
    source: str
    target: str
    relationship_type: str
    weight: Optional[float] = None
    
    def __post_init__(self):
        """Validate create edge request data after initialization."""
        if not isinstance(self.collection, str) or not self.collection.strip():
            raise ValueError("collection must be a non-empty string")
        
        if not isinstance(self.source, str) or not self.source.strip():
            raise ValueError("source must be a non-empty string")
        
        if not isinstance(self.target, str) or not self.target.strip():
            raise ValueError("target must be a non-empty string")
        
        if not isinstance(self.relationship_type, str) or not self.relationship_type.strip():
            raise ValueError("relationship_type must be a non-empty string")
        
        if self.weight is not None:
            if not isinstance(self.weight, (int, float)):
                raise ValueError("weight must be a number")
            if self.weight < 0.0 or self.weight > 1.0:
                raise ValueError("weight must be between 0.0 and 1.0")


@dataclass
class CreateEdgeResponse:
    """Response for creating an edge."""
    
    edge_id: str
    success: bool
    message: str


@dataclass
class ListNodesResponse:
    """Response for listing nodes."""
    
    nodes: List[GraphNode]
    count: int


@dataclass
class GetNeighborsResponse:
    """Response for getting neighbors."""
    
    neighbors: List[NeighborInfo]


@dataclass
class ListEdgesResponse:
    """Response for listing edges."""
    
    edges: List[GraphEdge]
    count: int


@dataclass
class DiscoverEdgesRequest:
    """Request to discover edges."""
    
    similarity_threshold: Optional[float] = None
    max_per_node: Optional[int] = None
    
    def __post_init__(self):
        """Validate discover edges request data after initialization."""
        if self.similarity_threshold is not None:
            if not isinstance(self.similarity_threshold, (int, float)):
                raise ValueError("similarity_threshold must be a number")
            if self.similarity_threshold < 0.0 or self.similarity_threshold > 1.0:
                raise ValueError("similarity_threshold must be between 0.0 and 1.0")
        
        if self.max_per_node is not None:
            if not isinstance(self.max_per_node, int) or self.max_per_node < 1:
                raise ValueError("max_per_node must be a positive integer")


@dataclass
class DiscoverEdgesResponse:
    """Response for discovering edges."""
    
    success: bool
    edges_created: int
    message: str


@dataclass
class DiscoveryStatusResponse:
    """Response for discovery status."""

    total_nodes: int
    nodes_with_edges: int
    total_edges: int
    progress_percentage: float


# ========== File Upload Models ==========

@dataclass
class FileUploadRequest:
    """Request to upload a file for indexing."""

    collection_name: str
    """Target collection name"""

    chunk_size: Optional[int] = None
    """Chunk size in characters (uses server default if not specified)"""

    chunk_overlap: Optional[int] = None
    """Chunk overlap in characters (uses server default if not specified)"""

    metadata: Optional[Dict[str, Any]] = None
    """Additional metadata to attach to all chunks"""

    public_key: Optional[str] = None
    """Optional ECC public key for payload encryption (PEM, base64, or hex format)"""

    def __post_init__(self):
        """Validate file upload request data."""
        if not self.collection_name or not self.collection_name.strip():
            raise ValueError("collection_name cannot be empty")

        if self.chunk_size is not None:
            if not isinstance(self.chunk_size, int) or self.chunk_size < 1:
                raise ValueError("chunk_size must be a positive integer")

        if self.chunk_overlap is not None:
            if not isinstance(self.chunk_overlap, int) or self.chunk_overlap < 0:
                raise ValueError("chunk_overlap must be a non-negative integer")


@dataclass
class FileUploadResponse:
    """Response from file upload operation."""

    success: bool
    """Whether the upload was successful"""

    filename: str
    """Original filename"""

    collection_name: str
    """Target collection"""

    chunks_created: int
    """Number of chunks created from the file"""

    vectors_created: int
    """Number of vectors created and stored"""

    file_size: int
    """File size in bytes"""

    language: str
    """Detected language/file type"""

    processing_time_ms: int
    """Processing time in milliseconds"""

    def __post_init__(self):
        """Validate file upload response data."""
        if self.chunks_created < 0:
            raise ValueError("chunks_created cannot be negative")
        if self.vectors_created < 0:
            raise ValueError("vectors_created cannot be negative")
        if self.file_size < 0:
            raise ValueError("file_size cannot be negative")
        if self.processing_time_ms < 0:
            raise ValueError("processing_time_ms cannot be negative")


@dataclass
class FileUploadConfig:
    """Configuration for file uploads."""

    max_file_size: int
    """Maximum file size in bytes"""

    max_file_size_mb: int
    """Maximum file size in megabytes"""

    allowed_extensions: List[str]
    """List of allowed file extensions"""

    reject_binary: bool
    """Whether binary files are rejected"""

    default_chunk_size: int
    """Default chunk size in characters"""

    default_chunk_overlap: int
    """Default chunk overlap in characters"""

    def __post_init__(self):
        """Validate file upload config data."""
        if self.max_file_size < 0:
            raise ValueError("max_file_size cannot be negative")
        if self.max_file_size_mb < 0:
            raise ValueError("max_file_size_mb cannot be negative")
        if self.default_chunk_size < 1:
            raise ValueError("default_chunk_size must be at least 1")
        if self.default_chunk_overlap < 0:
            raise ValueError("default_chunk_overlap cannot be negative")
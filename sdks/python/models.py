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
    # Phase25 §6 — per-collection vector-count ring buffer surfaced by
    # GET /collections/{name}. Empty list on older servers or for
    # collections that have never been read.
    vector_count_history: List[Any] = field(default_factory=list)

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
        # Hydrate vector_count_history into VectorCountSample instances
        # when the server sent dicts (kwargs unpacking from JSON).
        if self.vector_count_history:
            self.vector_count_history = [
                VectorCountSample.from_dict(s) if isinstance(s, dict) else s
                for s in self.vector_count_history
            ]


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
    
    base_url: str = "http://localhost:15002"
    ws_url: str = "ws://localhost:15002/ws"
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


# ===== TIER-DEMOTION REPORTS (issue #265) =====


@dataclass
class VectorOpResult:
    """Per-vector outcome for ``delete_vectors`` and
    ``move_to_collection`` calls.

    Attributes:
        id: Vector id this row refers to. ``None`` when the request
            payload contained a non-string entry that the server
            rejected upfront.
        status: One of ``"ok"``, ``"missing_in_src"``,
            ``"dst_insert_failed"``, ``"src_delete_failed"``,
            ``"error"`` (delete only).
        error: Server-side error message; populated when
            ``status != "ok"``.
        index: Index of this entry in the request's ``ids`` array
            (delete only).
    """

    status: str
    id: Optional[str] = None
    error: Optional[str] = None
    index: Optional[int] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "VectorOpResult":
        return cls(
            status=str(data["status"]),
            id=data.get("id"),
            error=data.get("error"),
            index=data.get("index"),
        )


@dataclass
class DeleteReport:
    """Aggregate outcome of a ``delete_vectors`` call against
    ``POST /batch_delete``.

    Server contract: ``{collection, count, deleted, failed, results}``.
    Per-id failures populate ``results`` without aborting the batch.
    """

    collection: str
    count: int
    deleted: int
    failed: int
    results: List[VectorOpResult]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "DeleteReport":
        return cls(
            collection=str(data["collection"]),
            count=int(data.get("count", 0)),
            deleted=int(data.get("deleted", 0)),
            failed=int(data.get("failed", 0)),
            results=[VectorOpResult.from_dict(r) for r in data.get("results", [])],
        )


@dataclass
class MoveReport:
    """Aggregate outcome of a ``move_to_collection`` call against
    ``POST /collections/{src}/vectors/move`` (issue #265).

    Server invariant: vectors are inserted into ``dst`` BEFORE being
    deleted from ``src``. A mid-batch failure leaves a recoverable
    duplicate, never data loss. Per-id failures populate ``results``
    without aborting the batch — operators chasing tier-demotion
    sweeps want partial progress, not abort-on-first-error.
    """

    src: str
    dst: str
    requested: int
    moved: int
    failed: int
    results: List[VectorOpResult]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "MoveReport":
        return cls(
            src=str(data["src"]),
            dst=str(data["dst"]),
            requested=int(data.get("requested", 0)),
            moved=int(data.get("moved", 0)),
            failed=int(data.get("failed", 0)),
            results=[VectorOpResult.from_dict(r) for r in data.get("results", [])],
        )


# ===== TIER-CONTROL REPORTS (phase13) =====


@dataclass
class DeleteByFilterReport:
    """Aggregate outcome of a ``delete_by_filter`` call against
    ``POST /collections/{name}/vectors/delete_by_filter``.

    Server contract: ``{scanned, matched, deleted, results}``.
    An empty filter is rejected with 400 to prevent accidental full-collection wipes.
    """

    scanned: int
    matched: int
    deleted: int
    results: List[Any] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "DeleteByFilterReport":
        return cls(
            scanned=int(data.get("scanned", 0)),
            matched=int(data.get("matched", 0)),
            deleted=int(data.get("deleted", 0)),
            results=list(data.get("results", [])),
        )


@dataclass
class BulkUpdateReport:
    """Aggregate outcome of a ``bulk_update_metadata`` call against
    ``POST /collections/{name}/vectors/bulk_update_metadata``.

    Server contract: ``{scanned, matched, updated, results}``.
    Patch is applied with RFC 7396 semantics: ``null`` values remove keys.
    """

    scanned: int
    matched: int
    updated: int
    results: List[Any] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BulkUpdateReport":
        return cls(
            scanned=int(data.get("scanned", 0)),
            matched=int(data.get("matched", 0)),
            updated=int(data.get("updated", 0)),
            results=list(data.get("results", [])),
        )


@dataclass
class CopyReport:
    """Aggregate outcome of a ``copy_vectors`` call against
    ``POST /collections/{src}/vectors/copy``.

    Server contract: ``{src, dst, requested, copied, failed, results}``.
    Per-id status: ``ok | missing_in_src | dst_insert_failed``.
    Unlike ``move_to_collection``, the source vectors are NOT deleted.
    """

    src: str
    dst: str
    requested: int
    copied: int
    failed: int
    results: List[VectorOpResult] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "CopyReport":
        return cls(
            src=str(data["src"]),
            dst=str(data["dst"]),
            requested=int(data.get("requested", 0)),
            copied=int(data.get("copied", 0)),
            failed=int(data.get("failed", 0)),
            results=[VectorOpResult.from_dict(r) for r in data.get("results", [])],
        )


@dataclass
class ReencodeJob:
    """Job descriptor returned by ``reencode_collection`` against
    ``POST /collections/{name}/reencode``.

    Server contract: ``{job_id, collection, state, target_encoding, progress}``.
    ``state`` will be ``"completed"`` on success. ``progress`` is in ``[0.0, 1.0]``.
    Valid ``target_encoding`` values: ``"sq8"``, ``"binary"``, ``"fp32"``.
    """

    job_id: str
    collection: str
    state: str
    target_encoding: str
    progress: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ReencodeJob":
        return cls(
            job_id=str(data.get("job_id", "")),
            collection=str(data.get("collection", "")),
            state=str(data.get("state", "")),
            target_encoding=str(data.get("target_encoding", "")),
            progress=float(data.get("progress", 0.0)),
        )


# ===== ADMIN / OBSERVABILITY TYPES (phase12) =====


@dataclass
class Stats:
    """Server statistics returned by ``GET /stats``.

    Fields: ``collections``, ``total_vectors``, ``uptime_seconds``, ``version``.

    Phase25 §5 additions ``default_quantization`` and
    ``compression_ratio`` default to ``("none", 1.0)`` on older servers
    that do not emit them.
    """

    collections: int = 0
    total_vectors: int = 0
    uptime_seconds: int = 0
    version: str = ""
    default_quantization: str = "none"
    compression_ratio: float = 1.0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Stats":
        """Deserialize from a server response dict."""
        return cls(
            collections=int(data.get("collections", 0)),
            total_vectors=int(data.get("total_vectors", 0)),
            uptime_seconds=int(data.get("uptime_seconds", 0)),
            version=str(data.get("version", "")),
            default_quantization=str(data.get("default_quantization", "none")),
            compression_ratio=float(data.get("compression_ratio", 1.0)),
        )


# ===== PHASE25 §7 RUNTIME METRICS =====


@dataclass
class RouteStats:
    """Per-route latency / throughput inside :class:`RuntimeMetrics`."""

    route: str = ""
    qps: float = 0.0
    p50_ms: float = 0.0
    p99_ms: float = 0.0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "RouteStats":
        return cls(
            route=str(data.get("route", "")),
            qps=float(data.get("qps", 0.0)),
            p50_ms=float(data.get("p50_ms", 0.0)),
            p99_ms=float(data.get("p99_ms", 0.0)),
        )


@dataclass
class WalSnapshot:
    """WAL state surfaced inside :class:`RuntimeMetrics`."""

    current_seq: int = 0
    size_bytes: int = 0
    last_checkpoint_at: int = 0
    last_checkpoint_seq: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "WalSnapshot":
        return cls(
            current_seq=int(data.get("current_seq", 0)),
            size_bytes=int(data.get("size_bytes", 0)),
            last_checkpoint_at=int(data.get("last_checkpoint_at", 0)),
            last_checkpoint_seq=int(data.get("last_checkpoint_seq", 0)),
        )


@dataclass
class RuntimeMetrics:
    """Runtime metrics snapshot returned by ``GET /metrics/runtime`` (phase25).

    Every field defaults so the SDK tolerates older servers that do not
    emit the route or partial payloads.
    """

    cpu_percent: float = 0.0
    memory_rss_bytes: int = 0
    memory_total_bytes: int = 0
    memory_percent: float = 0.0
    active_connections: int = 0
    uptime_seconds: int = 0
    qps_window_60s: float = 0.0
    error_rate_5xx_60s: float = 0.0
    throughput_by_route: List[RouteStats] = field(default_factory=list)
    wal: WalSnapshot = field(default_factory=WalSnapshot)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "RuntimeMetrics":
        routes_raw = data.get("throughput_by_route") or []
        routes = [
            RouteStats.from_dict(r) if isinstance(r, dict) else RouteStats()
            for r in routes_raw
        ]
        wal_raw = data.get("wal") or {}
        return cls(
            cpu_percent=float(data.get("cpu_percent", 0.0)),
            memory_rss_bytes=int(data.get("memory_rss_bytes", 0)),
            memory_total_bytes=int(data.get("memory_total_bytes", 0)),
            memory_percent=float(data.get("memory_percent", 0.0)),
            active_connections=int(data.get("active_connections", 0)),
            uptime_seconds=int(data.get("uptime_seconds", 0)),
            qps_window_60s=float(data.get("qps_window_60s", 0.0)),
            error_rate_5xx_60s=float(data.get("error_rate_5xx_60s", 0.0)),
            throughput_by_route=routes,
            wal=WalSnapshot.from_dict(wal_raw if isinstance(wal_raw, dict) else {}),
        )


@dataclass
class VectorCountSample:
    """One sample in the per-collection vector-count history (phase25 §6)."""

    at: int = 0
    count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "VectorCountSample":
        return cls(
            at=int(data.get("at", 0)),
            count=int(data.get("count", 0)),
        )


@dataclass
class ServerStatus:
    """Server liveness / version / uptime returned by ``GET /status``."""

    online: bool = False
    version: str = ""
    uptime_seconds: int = 0
    collections_count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ServerStatus":
        return cls(
            online=bool(data.get("online", False)),
            version=str(data.get("version", "")),
            uptime_seconds=int(data.get("uptime_seconds", 0)),
            collections_count=int(data.get("collections_count", 0)),
        )


@dataclass
class LogsQuery:
    """Query parameters for ``GET /logs``."""

    lines: Optional[int] = None
    level: Optional[str] = None


@dataclass
class LogEntry:
    """One log entry returned by ``GET /logs``."""

    timestamp: str = ""
    level: str = ""
    message: str = ""
    source: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "LogEntry":
        return cls(
            timestamp=str(data.get("timestamp", "")),
            level=str(data.get("level", "")),
            message=str(data.get("message", "")),
            source=str(data.get("source", "")),
        )


@dataclass
class CleanupReport:
    """Report returned by ``cleanup_empty_collections`` (``DELETE /collections/cleanup``)."""

    success: bool = False
    removed: int = 0
    collections: List[str] = field(default_factory=list)
    message: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "CleanupReport":
        return cls(
            success=bool(data.get("success", False)),
            removed=int(data.get("removed", 0)),
            collections=list(data.get("collections", [])),
            message=data.get("message"),
        )


@dataclass
class BackupInfo:
    """Metadata for one server-side backup returned by ``GET /backups``."""

    id: str = ""
    name: str = ""
    date: str = ""
    size: int = 0
    collections: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BackupInfo":
        return cls(
            id=str(data.get("id", "")),
            name=str(data.get("name", "")),
            date=str(data.get("date", "")),
            size=int(data.get("size", 0)),
            collections=list(data.get("collections", [])),
        )


@dataclass
class CreateBackupRequest:
    """Request body for ``create_backup`` (``POST /backups/create``)."""

    name: str = ""
    collections: List[str] = field(default_factory=list)


@dataclass
class RestoreBackupRequest:
    """Request body for ``restore_backup`` (``POST /backups/restore``)."""

    backup_id: str = ""


@dataclass
class AddWorkspaceRequest:
    """Request body for ``add_workspace`` (``POST /workspace/add``)."""

    path: str = ""
    collection_name: str = ""


# ===== AUTH TYPES (phase12) =====


@dataclass
class User:
    """User record returned by auth endpoints.

    Server shape: ``{user_id, username, roles}``.
    """

    user_id: str = ""
    username: str = ""
    roles: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "User":
        return cls(
            user_id=str(data.get("user_id", "")),
            username=str(data.get("username", "")),
            roles=list(data.get("roles", [])),
        )


@dataclass
class JwtToken:
    """JWT token returned by ``POST /auth/refresh``.

    Server shape: ``{access_token, token_type, expires_in}``.
    """

    access_token: str = ""
    token_type: str = "Bearer"
    expires_in: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "JwtToken":
        return cls(
            access_token=str(data.get("access_token", "")),
            token_type=str(data.get("token_type", "Bearer")),
            expires_in=int(data.get("expires_in", 0)),
        )


@dataclass
class PasswordPolicyReport:
    """Password policy report returned by ``POST /auth/validate-password``."""

    valid: bool = False
    errors: List[str] = field(default_factory=list)
    strength: int = 0
    strength_label: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "PasswordPolicyReport":
        return cls(
            valid=bool(data.get("valid", False)),
            errors=list(data.get("errors", [])),
            strength=int(data.get("strength", 0)),
            strength_label=str(data.get("strength_label", "")),
        )


@dataclass
class CreateApiKeyRequest:
    """Request body for ``create_api_key`` (``POST /auth/keys``)."""

    name: str = ""
    permissions: List[str] = field(default_factory=list)
    expires_in: Optional[int] = None


@dataclass
class ApiKey:
    """API key returned by ``POST /auth/keys``.

    The ``api_key`` field is only present at creation time.
    List responses omit it for security.
    """

    id: str = ""
    name: str = ""
    permissions: List[str] = field(default_factory=list)
    api_key: Optional[str] = None
    created_at: int = 0
    last_used: Optional[int] = None
    expires_at: Optional[int] = None
    active: bool = False
    warning: Optional[str] = None
    usage_count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ApiKey":
        return cls(
            id=str(data.get("id", "")),
            name=str(data.get("name", "")),
            permissions=list(data.get("permissions", [])),
            api_key=data.get("api_key"),
            created_at=int(data.get("created_at", 0)),
            last_used=data.get("last_used"),
            expires_at=data.get("expires_at"),
            active=bool(data.get("active", False)),
            warning=data.get("warning"),
            usage_count=int(data.get("usage_count", 0)),
        )


@dataclass
class ApiKeyScope:
    """Per-collection scope attached to an API key."""

    collection: str = ""
    permissions: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ApiKeyScope":
        return cls(
            collection=str(data.get("collection", "")),
            permissions=list(data.get("permissions", [])),
        )

    def to_dict(self) -> Dict[str, Any]:
        return {"collection": self.collection, "permissions": self.permissions}


@dataclass
class UpdateApiKeyPermissionsRequest:
    """Request body for ``PUT /auth/keys/{id}/permissions``."""

    permissions: List[str] = field(default_factory=list)
    scopes: Optional[List[ApiKeyScope]] = None

    def to_dict(self) -> Dict[str, Any]:
        body: Dict[str, Any] = {"permissions": list(self.permissions)}
        if self.scopes is not None:
            body["scopes"] = [s.to_dict() for s in self.scopes]
        return body


@dataclass
class ApiKeyView:
    """Flattened key view returned by the permission-update + usage endpoints."""

    id: str = ""
    name: str = ""
    user_id: str = ""
    permissions: List[str] = field(default_factory=list)
    scopes: List[ApiKeyScope] = field(default_factory=list)
    created_at: int = 0
    last_used: Optional[int] = None
    expires_at: Optional[int] = None
    active: bool = False
    usage_count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ApiKeyView":
        return cls(
            id=str(data.get("id", "")),
            name=str(data.get("name", "")),
            user_id=str(data.get("user_id", "")),
            permissions=list(data.get("permissions", [])),
            scopes=[ApiKeyScope.from_dict(s) for s in data.get("scopes", [])],
            created_at=int(data.get("created_at", 0)),
            last_used=data.get("last_used"),
            expires_at=data.get("expires_at"),
            active=bool(data.get("active", False)),
            usage_count=int(data.get("usage_count", 0)),
        )


@dataclass
class ApiKeyUsageBucket:
    """One day's usage bucket from ``GET /auth/keys/{id}/usage``."""

    date: str = ""
    count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ApiKeyUsageBucket":
        return cls(date=str(data.get("date", "")), count=int(data.get("count", 0)))


@dataclass
class ApiKeyUsageReport:
    """Response body for ``GET /auth/keys/{id}/usage``.

    Buckets are oldest-first; days with zero validations are still
    present so callers can render a continuous sparkline without
    gap-fill logic.
    """

    key: ApiKeyView = field(default_factory=ApiKeyView)
    buckets: List[ApiKeyUsageBucket] = field(default_factory=list)
    window_total: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ApiKeyUsageReport":
        return cls(
            key=ApiKeyView.from_dict(data.get("key", {})),
            buckets=[ApiKeyUsageBucket.from_dict(b) for b in data.get("buckets", [])],
            window_total=int(data.get("window_total", 0)),
        )


@dataclass
class CreateUserRequest:
    """Request body for ``create_user`` (``POST /auth/users``)."""

    username: str = ""
    password: str = ""
    roles: List[str] = field(default_factory=list)


# ===== REPLICATION SDK TYPES (phase12) =====


@dataclass
class ReplicationStatus:
    """Replication status returned by ``GET /replication/status``.

    Server: ``{role, enabled, stats?, replicas?}``.
    """

    role: str = "Standalone"
    enabled: bool = False
    stats: Optional["ReplicationStats"] = None
    replicas: Optional[List["ReplicaInfo"]] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ReplicationStatus":
        from models import ReplicaInfo, ReplicationStats  # type: ignore[import-not-found]
        stats_data = data.get("stats")
        replicas_data = data.get("replicas")
        return cls(
            role=str(data.get("role", "Standalone")),
            enabled=bool(data.get("enabled", False)),
            stats=ReplicationStats(**stats_data) if stats_data else None,
            replicas=[ReplicaInfo(**r) for r in replicas_data] if replicas_data else None,
        )


@dataclass
class ReplicationConfig:
    """Request body for ``configure_replication`` (``POST /replication/configure``)."""

    role: str = "standalone"
    bind_address: Optional[str] = None
    master_address: Optional[str] = None
    heartbeat_interval: Optional[int] = None
    log_size: Optional[int] = None


# ===== VECTOR OPERATIONS — NEW METHODS (phase12) =====


@dataclass
class VectorPage:
    """Paginated vector listing returned by ``GET /collections/{name}/vectors``."""

    vectors: List[Any] = field(default_factory=list)
    total: int = 0
    limit: int = 10
    offset: int = 0
    message: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "VectorPage":
        return cls(
            vectors=list(data.get("vectors", [])),
            total=int(data.get("total", 0)),
            limit=int(data.get("limit", 10)),
            offset=int(data.get("offset", 0)),
            message=data.get("message"),
        )


@dataclass
class UpdateVectorRequest:
    """Request body for ``update_vector`` (``POST /update``)."""

    id: str = ""
    metadata: Optional[Any] = None


@dataclass
class BatchInsertItem:
    """One item in a ``batch_insert_texts`` call (``POST /batch_insert``)."""

    text: str = ""
    id: Optional[str] = None
    metadata: Optional[Any] = None


@dataclass
class BatchInsertReport:
    """Aggregate outcome of a ``batch_insert_texts`` or ``insert_vectors`` call.

    Server response shape: ``{collection, inserted, failed, count, results}``.
    """

    collection: str = ""
    successful: int = 0
    failed: int = 0
    total: int = 0
    results: List[Any] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BatchInsertReport":
        return cls(
            collection=str(data.get("collection", "")),
            successful=int(data.get("inserted", data.get("successful", 0))),
            failed=int(data.get("failed", 0)),
            total=int(data.get("count", data.get("total", 0))),
            results=list(data.get("results", [])),
        )


@dataclass
class VectorUpdate:
    """One entry in a ``batch_update_vectors`` call (``POST /batch_update``)."""

    id: str = ""
    vector: Optional[List[float]] = None
    payload: Optional[Any] = None


@dataclass
class BatchUpdateReport:
    """Aggregate outcome of a ``batch_update_vectors`` call.

    Server response: ``{collection, count, updated, failed, results}``.
    """

    collection: str = ""
    total: int = 0
    successful: int = 0
    failed: int = 0
    results: List[Any] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BatchUpdateReport":
        return cls(
            collection=str(data.get("collection", "")),
            total=int(data.get("count", data.get("total", 0))),
            successful=int(data.get("updated", data.get("successful", 0))),
            failed=int(data.get("failed", 0)),
            results=list(data.get("results", [])),
        )


@dataclass
class RawVectorInsert:
    """One vector entry in an ``insert_vectors`` call (``POST /insert_vectors``)."""

    embedding: List[float] = field(default_factory=list)
    id: Optional[str] = None
    payload: Optional[Any] = None
    metadata: Optional[Dict[str, str]] = None


@dataclass
class BatchSearchQuery:
    """One query in a ``batch_search`` call (``POST /batch_search``)."""

    query: Optional[str] = None
    vector: Optional[List[float]] = None
    limit: Optional[int] = None
    threshold: Optional[float] = None


@dataclass
class SearchByFileRequest:
    """Request for ``search_by_file`` (``POST /collections/{name}/search/file``)."""

    file_path: str = ""
    limit: Optional[int] = None


# ===== DISCOVERY PIPELINE TYPES (phase12) =====


@dataclass
class BroadDiscoveryRequest:
    """Request for ``broad_discovery`` (``POST /discovery/broad_discovery``)."""

    queries: List[str] = field(default_factory=list)
    k: Optional[int] = None


@dataclass
class BroadDiscoveryResponse:
    """Response from ``broad_discovery``.

    Server: ``{chunks: [{collection, score, content_preview}], count}``.
    """

    chunks: List[Any] = field(default_factory=list)
    count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BroadDiscoveryResponse":
        return cls(
            chunks=list(data.get("chunks", [])),
            count=int(data.get("count", 0)),
        )


@dataclass
class SemanticFocusRequest:
    """Request for ``semantic_focus`` (``POST /discovery/semantic_focus``)."""

    collection: str = ""
    queries: List[str] = field(default_factory=list)
    k: Optional[int] = None


@dataclass
class SemanticFocusResponse:
    """Response from ``semantic_focus``.

    Server: ``{chunks: [...], count}``.
    """

    chunks: List[Any] = field(default_factory=list)
    count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SemanticFocusResponse":
        return cls(
            chunks=list(data.get("chunks", [])),
            count=int(data.get("count", 0)),
        )


@dataclass
class PromoteReadmeRequest:
    """Request for ``promote_readme`` (``POST /discovery/promote_readme``)."""

    chunks: List[Any] = field(default_factory=list)


@dataclass
class PromoteReadmeResponse:
    """Response from ``promote_readme``.

    Server: ``{promoted_chunks: [...], count}``.
    """

    promoted_chunks: List[Any] = field(default_factory=list)
    count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "PromoteReadmeResponse":
        return cls(
            promoted_chunks=list(data.get("promoted_chunks", [])),
            count=int(data.get("count", 0)),
        )


@dataclass
class CompressEvidenceRequest:
    """Request for ``compress_evidence`` (``POST /discovery/compress_evidence``)."""

    chunks: List[Any] = field(default_factory=list)
    max_bullets: Optional[int] = None
    max_per_doc: Optional[int] = None


@dataclass
class CompressEvidenceResponse:
    """Response from ``compress_evidence``.

    Server: ``{bullets: [{text, source_id, category, score}], count}``.
    """

    bullets: List[Any] = field(default_factory=list)
    count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "CompressEvidenceResponse":
        return cls(
            bullets=list(data.get("bullets", [])),
            count=int(data.get("count", 0)),
        )


@dataclass
class AnswerPlan:
    """Response from ``build_answer_plan``.

    Server: ``{sections: [...], total_bullets, sources}``.
    """

    sections: List[Any] = field(default_factory=list)
    total_bullets: int = 0
    sources: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AnswerPlan":
        return cls(
            sections=list(data.get("sections", [])),
            total_bullets=int(data.get("total_bullets", 0)),
            sources=list(data.get("sources", [])),
        )


@dataclass
class AnswerPlanRequest:
    """Request for ``build_answer_plan`` (``POST /discovery/build_answer_plan``)."""

    bullets: List[Any] = field(default_factory=list)


@dataclass
class RenderPromptRequest:
    """Request for ``render_llm_prompt`` (``POST /discovery/render_llm_prompt``)."""

    plan: Optional["AnswerPlan"] = None


@dataclass
class LlmPrompt:
    """Response from ``render_llm_prompt``.

    Server: ``{prompt, length, estimated_tokens}``.
    """

    prompt: str = ""
    length: int = 0
    estimated_tokens: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "LlmPrompt":
        return cls(
            prompt=str(data.get("prompt", "")),
            length=int(data.get("length", 0)),
            estimated_tokens=int(data.get("estimated_tokens", 0)),
        )


# ===== HUB TYPES (phase12) =====


@dataclass
class UserBackup:
    """A user-scoped backup entry returned by ``GET /hub/backups``."""

    id: str = ""
    user_id: str = ""
    name: str = ""
    description: Optional[str] = None
    collections: List[str] = field(default_factory=list)
    created_at: str = ""
    size: int = 0
    status: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "UserBackup":
        return cls(
            id=str(data.get("id", "")),
            user_id=str(data.get("user_id", "")),
            name=str(data.get("name", "")),
            description=data.get("description"),
            collections=list(data.get("collections", [])),
            created_at=str(data.get("created_at", "")),
            size=int(data.get("size", 0)),
            status=str(data.get("status", "")),
        )


@dataclass
class CreateUserBackupRequest:
    """Request for ``create_user_backup`` (``POST /hub/backups``)."""

    user_id: str = ""
    name: str = ""
    description: Optional[str] = None
    collections: Optional[List[str]] = None


@dataclass
class RestoreUserBackupRequest:
    """Request for ``restore_user_backup`` (``POST /hub/backups/restore``)."""

    user_id: str = ""
    backup_id: str = ""
    overwrite: bool = False


@dataclass
class UploadUserBackupRequest:
    """Parameters for ``upload_user_backup`` (``POST /hub/backups/upload``)."""

    user_id: str = ""
    name: Optional[str] = None
    data: bytes = field(default_factory=bytes)


@dataclass
class UsageStatistics:
    """Usage statistics returned by ``GET /hub/usage/statistics``."""

    success: bool = False
    message: str = ""
    stats: Optional[Any] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "UsageStatistics":
        return cls(
            success=bool(data.get("success", False)),
            message=str(data.get("message", "")),
            stats=data.get("stats"),
        )


@dataclass
class QuotaInfo:
    """Quota information returned by ``GET /hub/usage/quota``."""

    success: bool = False
    message: str = ""
    quota: Optional[Any] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "QuotaInfo":
        return cls(
            success=bool(data.get("success", False)),
            message=str(data.get("message", "")),
            quota=data.get("quota"),
        )


@dataclass
class HubApiKeyValidation:
    """Validation result returned by ``POST /hub/validate-key``."""

    valid: bool = False
    tenant_id: str = ""
    tenant_name: str = ""
    permissions: List[str] = field(default_factory=list)
    validated_at: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "HubApiKeyValidation":
        return cls(
            valid=bool(data.get("valid", False)),
            tenant_id=str(data.get("tenant_id", "")),
            tenant_name=str(data.get("tenant_name", "")),
            permissions=list(data.get("permissions", [])),
            validated_at=str(data.get("validated_at", "")),
        )


# ===== SCHEMA-EVOLUTION + OBSERVABILITY TYPES (phase14) =====


@dataclass
class ReindexParams:
    """Parameters for ``reindex_collection``
    (``POST /collections/{name}/reindex``).

    All fields carry the server's defaults: ``m=16``,
    ``ef_construction=200``, ``ef_search=100``.
    """

    m: int = 16
    """HNSW ``M`` parameter (number of bi-directional links)."""

    ef_construction: int = 200
    """HNSW ``ef_construction`` (candidate list size during build)."""

    ef_search: int = 100
    """HNSW ``ef_search`` (candidate list size at query time)."""


@dataclass
class ReindexJob:
    """Job descriptor returned by ``reindex_collection``
    (``POST /collections/{name}/reindex``).

    Server contract: ``{job_id, collection, state, params, progress}``.
    ``state`` will be ``"completed"`` on success.
    """

    job_id: str
    collection: str
    state: str
    params: Dict[str, Any]
    progress: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ReindexJob":
        """Deserialize from a server response dict."""
        return cls(
            job_id=str(data.get("job_id", "")),
            collection=str(data.get("collection", "")),
            state=str(data.get("state", "")),
            params=dict(data.get("params", {})),
            progress=float(data.get("progress", 0.0)),
        )


@dataclass
class NativeSnapshotInfo:
    """Native snapshot metadata returned by ``snapshot_collection_native``
    and each entry in ``list_collection_snapshots_native``.

    Server contract: ``{id, collection, created_at, size_bytes}``.
    """

    id: str
    collection: str
    created_at: str
    size_bytes: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "NativeSnapshotInfo":
        """Deserialize from a server response dict."""
        return cls(
            id=str(data.get("id", "")),
            collection=str(data.get("collection", "")),
            created_at=str(data.get("created_at", "")),
            size_bytes=int(data.get("size_bytes", 0)),
        )


@dataclass
class ExplainTrace:
    """Execution trace attached to an :class:`ExplainResponse`.

    Server contract:
    ``{visited_nodes, ef_search, hnsw_search_ms, payload_filter_evals,
    quantization_score_ms, total_ms}``.
    """

    visited_nodes: int = 0
    """Number of HNSW graph nodes visited during the search."""

    ef_search: int = 0
    """Effective ``ef_search`` value used."""

    hnsw_search_ms: float = 0.0
    """Wall-clock time spent inside HNSW traversal (milliseconds)."""

    payload_filter_evals: int = 0
    """Number of payload-filter predicate evaluations."""

    quantization_score_ms: float = 0.0
    """Wall-clock time spent on quantized distance scoring (milliseconds)."""

    total_ms: float = 0.0
    """Total wall-clock time for the explain call (milliseconds)."""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ExplainTrace":
        """Deserialize from a server trace dict."""
        return cls(
            visited_nodes=int(data.get("visited_nodes", 0)),
            ef_search=int(data.get("ef_search", 0)),
            hnsw_search_ms=float(data.get("hnsw_search_ms", 0.0)),
            payload_filter_evals=int(data.get("payload_filter_evals", 0)),
            quantization_score_ms=float(data.get("quantization_score_ms", 0.0)),
            total_ms=float(data.get("total_ms", 0.0)),
        )


@dataclass
class ExplainResponse:
    """Response from ``explain_search``
    (``POST /collections/{name}/explain``).

    Server contract: ``{collection, k, results, trace}``.
    """

    collection: str
    k: int
    results: List[Dict[str, Any]]
    trace: ExplainTrace

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ExplainResponse":
        """Deserialize from a server response dict."""
        return cls(
            collection=str(data.get("collection", "")),
            k=int(data.get("k", 0)),
            results=list(data.get("results", [])),
            trace=ExplainTrace.from_dict(data.get("trace", {})),
        )


@dataclass
class SlowQueryEntry:
    """One entry in the slow-query ring buffer returned by
    ``GET /slow_queries``.

    Server contract: ``{timestamp, collection, k, duration_ms}``.
    """

    timestamp: str = ""
    """ISO-8601 / RFC-3339 timestamp when the slow query was recorded."""

    collection: str = ""
    """Collection the query ran against."""

    k: int = 0
    """Number of neighbours requested."""

    duration_ms: float = 0.0
    """Observed query duration in milliseconds."""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SlowQueryEntry":
        """Deserialize from a server entry dict."""
        return cls(
            timestamp=str(data.get("timestamp", "")),
            collection=str(data.get("collection", "")),
            k=int(data.get("k", 0)),
            duration_ms=float(data.get("duration_ms", 0.0)),
        )


@dataclass
class SlowQueryConfig:
    """Slow-query ring-buffer configuration used as both request body and
    response for ``POST /slow_queries/config``.

    Server contract: ``{threshold_ms, capacity}``.
    """

    threshold_ms: int = 200
    """Minimum duration (ms) for a query to be recorded."""

    capacity: int = 1000
    """Maximum number of entries retained in the ring buffer."""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SlowQueryConfig":
        """Deserialize from a server response dict."""
        return cls(
            threshold_ms=int(data.get("threshold_ms", 200)),
            capacity=int(data.get("capacity", 1000)),
        )


# ===== CLUSTER + AUTH ADMIN TYPES (phase15) =====


@dataclass
class FailoverReport:
    """Report returned by ``POST /cluster/failover``.

    Server contract:
    ``{promoted_replica_id, master_offset_at_promotion,
    replica_offset_at_promotion, residual_lag_operations}``.
    """

    promoted_replica_id: str
    master_offset_at_promotion: int
    replica_offset_at_promotion: int
    residual_lag_operations: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "FailoverReport":
        """Deserialize from a server response dict."""
        return cls(
            promoted_replica_id=str(data.get("promoted_replica_id", "")),
            master_offset_at_promotion=int(data.get("master_offset_at_promotion", 0)),
            replica_offset_at_promotion=int(data.get("replica_offset_at_promotion", 0)),
            residual_lag_operations=int(data.get("residual_lag_operations", 0)),
        )


@dataclass
class ResyncJob:
    """Report returned by ``POST /cluster/replicas/{id}/resync``.

    Server contract: ``{replica_id, snapshot_offset, full_snapshot}``.
    """

    replica_id: str
    snapshot_offset: int
    full_snapshot: bool

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ResyncJob":
        """Deserialize from a server response dict."""
        return cls(
            replica_id=str(data.get("replica_id", "")),
            snapshot_offset=int(data.get("snapshot_offset", 0)),
            full_snapshot=bool(data.get("full_snapshot", False)),
        )


@dataclass
class PeerInfo:
    """Information about a newly added cluster peer.

    Returned by ``POST /cluster/peers``.
    Server contract: ``{node_id, address, role}``.
    """

    node_id: str
    address: str
    role: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "PeerInfo":
        """Deserialize from a server response dict."""
        return cls(
            node_id=str(data.get("node_id", "")),
            address=str(data.get("address", "")),
            role=str(data.get("role", "")),
        )


@dataclass
class AddPeerRequest:
    """Request body for ``POST /cluster/peers``."""

    address: str
    role: str = "member"


@dataclass
class RebalanceJob:
    """Job descriptor returned by ``POST /cluster/rebalance`` and
    ``GET /cluster/rebalance/status``.

    Server contract:
    ``{job_id, status, shards_to_move, shards_moved, last_checkpoint_node?, message}``.
    """

    job_id: str
    status: str
    shards_to_move: int
    shards_moved: int
    message: str
    last_checkpoint_node: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "RebalanceJob":
        """Deserialize from a server response dict."""
        return cls(
            job_id=str(data.get("job_id", "")),
            status=str(data.get("status", "")),
            shards_to_move=int(data.get("shards_to_move", 0)),
            shards_moved=int(data.get("shards_moved", 0)),
            message=str(data.get("message", "")),
            last_checkpoint_node=data.get("last_checkpoint_node"),
        )


@dataclass
class RotatedKey:
    """Response from ``POST /auth/keys/{id}/rotate``.

    Server contract: ``{old_key_id, new_key_id, new_token, grace_until}``.
    """

    old_key_id: str
    new_key_id: str
    new_token: str
    grace_until: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "RotatedKey":
        """Deserialize from a server response dict."""
        return cls(
            old_key_id=str(data.get("old_key_id", "")),
            new_key_id=str(data.get("new_key_id", "")),
            new_token=str(data.get("new_token", "")),
            grace_until=int(data.get("grace_until", 0)),
        )


@dataclass
class TokenScope:
    """Per-collection permission scope in :class:`CreateScopedApiKeyRequest`."""

    collection: str
    permissions: List[str] = field(default_factory=list)


@dataclass
class CreateScopedApiKeyRequest:
    """Request body for ``POST /auth/keys`` — extended with optional scopes."""

    name: str
    permissions: List[str] = field(default_factory=list)
    expires_in: Optional[int] = None
    scopes: List[TokenScope] = field(default_factory=list)


@dataclass
class TokenIntrospection:
    """RFC 7662 token introspection response from ``POST /auth/introspect``.

    Server contract: ``{active, scope?, sub?, exp?, username?}``.
    """

    active: bool = False
    scope: Optional[str] = None
    sub: Optional[str] = None
    exp: Optional[int] = None
    username: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "TokenIntrospection":
        """Deserialize from a server response dict."""
        return cls(
            active=bool(data.get("active", False)),
            scope=data.get("scope"),
            sub=data.get("sub"),
            exp=data.get("exp"),
            username=data.get("username"),
        )


@dataclass
class AuditEntry:
    """One entry in the admin audit log returned by ``GET /auth/audit``.

    Server contract: ``{actor, action, target, at, correlation_id?}``.
    """

    actor: str
    action: str
    target: str
    at: str
    correlation_id: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AuditEntry":
        """Deserialize from a server response dict."""
        return cls(
            actor=str(data.get("actor", "")),
            action=str(data.get("action", "")),
            target=str(data.get("target", "")),
            at=str(data.get("at", "")),
            correlation_id=data.get("correlation_id"),
        )


@dataclass
class AuditQuery:
    """Query parameters for ``GET /auth/audit``."""

    actor: Optional[str] = None
    action: Optional[str] = None
    since: Optional[str] = None
    until: Optional[str] = None
    limit: Optional[int] = None
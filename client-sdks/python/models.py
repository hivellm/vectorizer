"""
Data models for the Hive Vectorizer SDK.

This module contains all the data models used for representing
vectors, collections, search results, and other entities.
"""

from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Union
from datetime import datetime


@dataclass
class Vector:
    """Represents a vector with metadata."""
    
    id: str
    data: List[float]
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Validate vector data after initialization."""
        if not self.id:
            raise ValueError("Vector ID cannot be empty")
        if not self.data:
            raise ValueError("Vector data cannot be empty")
        if not all(isinstance(x, (int, float)) for x in self.data):
            raise ValueError("Vector data must contain only numbers")


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
    similarity_metric: str
    status: str
    vector_count: int
    document_count: Optional[int] = None
    error_message: Optional[str] = None
    last_updated: Optional[str] = None
    
    def __post_init__(self):
        """Validate collection info after initialization."""
        if not self.name:
            raise ValueError("Collection name cannot be empty")
        if self.dimension <= 0:
            raise ValueError("Dimension must be positive")
        if self.vector_count < 0:
            raise ValueError("Vector count cannot be negative")


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
            raise ValueError("Search result ID cannot be empty")
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

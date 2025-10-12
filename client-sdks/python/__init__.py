"""
Hive Vectorizer Python SDK

A Python client library for the Hive Vectorizer service, providing
high-level interfaces for vector operations, semantic search, and
collection management.

Author: HiveLLM Team
Version: 0.3.4
License: MIT
"""

from .client import VectorizerClient
from .exceptions import (
    VectorizerError,
    AuthenticationError,
    CollectionNotFoundError,
    ValidationError,
    NetworkError,
    ServerError
)
from .models import (
    Vector,
    Collection,
    SearchResult,
    EmbeddingRequest,
    SearchRequest,
    CollectionInfo
)

__version__ = "0.4.0"
__author__ = "HiveLLM Team"
__email__ = "team@hivellm.org"

__all__ = [
    "VectorizerClient",
    "VectorizerError",
    "AuthenticationError", 
    "CollectionNotFoundError",
    "ValidationError",
    "NetworkError",
    "ServerError",
    "Vector",
    "Collection",
    "SearchResult",
    "EmbeddingRequest",
    "SearchRequest",
    "CollectionInfo"
]

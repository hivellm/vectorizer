"""Vectorizer components for Langflow.

This package provides LangChain-compatible components for integrating
Vectorizer vector database into Langflow workflows.
"""

from vectorizer_langflow.vectorstore import VectorizerVectorStore
from vectorizer_langflow.retriever import VectorizerRetriever
from vectorizer_langflow.loader import VectorizerLoader

__version__ = "1.6.0"

__all__ = [
    "VectorizerVectorStore",
    "VectorizerRetriever",
    "VectorizerLoader",
]

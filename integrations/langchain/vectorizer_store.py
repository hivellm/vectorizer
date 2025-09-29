"""
LangChain Integration for Vectorizer

This module provides a LangChain VectorStore implementation that uses Vectorizer
as the backend for vector storage and similarity search.
"""

from typing import List, Dict, Any, Optional, Tuple
import json
import requests
import asyncio
import aiohttp
from dataclasses import dataclass
from abc import ABC, abstractmethod

try:
    from langchain.vectorstores.base import VectorStore
    from langchain.schema import Document
    from langchain.embeddings.base import Embeddings
except ImportError:
    # Fallback for older LangChain versions
    try:
        from langchain.vectorstores import VectorStore
        from langchain.docstore.document import Document
        from langchain.embeddings import Embeddings
    except ImportError:
        raise ImportError(
            "LangChain is required for this integration. "
            "Install with: pip install langchain"
        )


@dataclass
class VectorizerConfig:
    """Configuration for Vectorizer connection"""
    host: str = "localhost"
    port: int = 15001
    api_key: Optional[str] = None
    timeout: int = 30
    collection_name: str = "langchain_documents"
    auto_create_collection: bool = True
    batch_size: int = 100
    similarity_threshold: float = 0.7


class VectorizerClient:
    """Client for communicating with Vectorizer API"""
    
    def __init__(self, config: VectorizerConfig):
        self.config = config
        self.base_url = f"http://{config.host}:{config.port}/api/v1"
        self.session = requests.Session()
        
        if config.api_key:
            self.session.headers.update({"Authorization": f"Bearer {config.api_key}"})
    
    def _make_request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """Make HTTP request to Vectorizer API"""
        url = f"{self.base_url}{endpoint}"
        
        try:
            response = self.session.request(
                method, url, timeout=self.config.timeout, **kwargs
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            raise VectorizerError(f"API request failed: {e}")
    
    async def _make_async_request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """Make async HTTP request to Vectorizer API"""
        url = f"{self.base_url}{endpoint}"
        
        async with aiohttp.ClientSession() as session:
            try:
                async with session.request(method, url, **kwargs) as response:
                    response.raise_for_status()
                    return await response.json()
            except aiohttp.ClientError as e:
                raise VectorizerError(f"Async API request failed: {e}")
    
    def health_check(self) -> bool:
        """Check if Vectorizer is healthy"""
        try:
            self._make_request("GET", "/health")
            return True
        except:
            return False
    
    def list_collections(self) -> List[str]:
        """List all collections"""
        response = self._make_request("GET", "/collections")
        return [col["name"] for col in response.get("collections", [])]
    
    def create_collection(self, name: str, dimension: int = 384, metric: str = "cosine") -> bool:
        """Create a new collection"""
        data = {
            "name": name,
            "dimension": dimension,
            "metric": metric,
            "embedding_model": "bm25"
        }
        
        try:
            self._make_request("POST", "/collections", json=data)
            return True
        except VectorizerError:
            return False
    
    def delete_collection(self, name: str) -> bool:
        """Delete a collection"""
        try:
            self._make_request("DELETE", f"/collections/{name}")
            return True
        except VectorizerError:
            return False
    
    def add_texts(self, texts: List[str], metadatas: Optional[List[Dict[str, Any]]] = None) -> List[str]:
        """Add texts to the collection"""
        if metadatas is None:
            metadatas = [{} for _ in texts]
        
        data = {
            "texts": texts,
            "metadatas": metadatas
        }
        
        response = self._make_request(
            "POST", 
            f"/collections/{self.config.collection_name}/vectors/batch",
            json=data
        )
        
        return response.get("vector_ids", [])
    
    def similarity_search(
        self, 
        query: str, 
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """Perform similarity search"""
        data = {
            "query": query,
            "top_k": k,
            "filter": filter or {}
        }
        
        response = self._make_request(
            "POST",
            f"/collections/{self.config.collection_name}/search",
            json=data
        )
        
        return response.get("results", [])
    
    def similarity_search_with_score(
        self, 
        query: str, 
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None
    ) -> List[Tuple[Dict[str, Any], float]]:
        """Perform similarity search with scores"""
        results = self.similarity_search(query, k, filter)
        return [(result, result.get("score", 0.0)) for result in results]
    
    def delete_vectors(self, ids: List[str]) -> bool:
        """Delete vectors by IDs"""
        data = {"vector_ids": ids}
        
        try:
            self._make_request(
                "DELETE",
                f"/collections/{self.config.collection_name}/vectors/batch",
                json=data
            )
            return True
        except VectorizerError:
            return False


class VectorizerError(Exception):
    """Custom exception for Vectorizer operations"""
    pass


class VectorizerStore(VectorStore):
    """
    LangChain VectorStore implementation using Vectorizer as backend.
    
    This class implements the LangChain VectorStore interface and provides
    seamless integration with Vectorizer for document storage and retrieval.
    """
    
    def __init__(
        self,
        config: VectorizerConfig,
        embedding: Optional[Embeddings] = None
    ):
        """Initialize VectorizerStore"""
        self.config = config
        self.client = VectorizerClient(config)
        self.embedding = embedding
        
        # Ensure collection exists
        if config.auto_create_collection:
            self._ensure_collection_exists()
    
    def _ensure_collection_exists(self):
        """Ensure the collection exists, create if it doesn't"""
        collections = self.client.list_collections()
        if self.config.collection_name not in collections:
            self.client.create_collection(
                self.config.collection_name,
                dimension=384,  # Default for BM25
                metric="cosine"
            )
    
    def add_texts(
        self,
        texts: List[str],
        metadatas: Optional[List[Dict[str, Any]]] = None,
        **kwargs
    ) -> List[str]:
        """
        Add texts to the vector store.
        
        Args:
            texts: List of texts to add
            metadatas: Optional list of metadata dictionaries
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of vector IDs
        """
        return self.client.add_texts(texts, metadatas)
    
    def similarity_search(
        self,
        query: str,
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None,
        **kwargs
    ) -> List[Document]:
        """
        Perform similarity search.
        
        Args:
            query: Query text
            k: Number of results to return
            filter: Optional metadata filter
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of Document objects
        """
        results = self.client.similarity_search(query, k, filter)
        
        documents = []
        for result in results:
            doc = Document(
                page_content=result.get("content", ""),
                metadata=result.get("metadata", {})
            )
            documents.append(doc)
        
        return documents
    
    def similarity_search_with_score(
        self,
        query: str,
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None,
        **kwargs
    ) -> List[Tuple[Document, float]]:
        """
        Perform similarity search with scores.
        
        Args:
            query: Query text
            k: Number of results to return
            filter: Optional metadata filter
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of (Document, score) tuples
        """
        results = self.client.similarity_search_with_score(query, k, filter)
        
        documents_with_scores = []
        for result, score in results:
            doc = Document(
                page_content=result.get("content", ""),
                metadata=result.get("metadata", {})
            )
            documents_with_scores.append((doc, score))
        
        return documents_with_scores
    
    def delete(self, ids: List[str], **kwargs) -> bool:
        """
        Delete vectors by IDs.
        
        Args:
            ids: List of vector IDs to delete
            **kwargs: Additional arguments (ignored)
            
        Returns:
            True if successful
        """
        return self.client.delete_vectors(ids)
    
    @classmethod
    def from_texts(
        cls,
        texts: List[str],
        embedding: Optional[Embeddings] = None,
        metadatas: Optional[List[Dict[str, Any]]] = None,
        config: Optional[VectorizerConfig] = None,
        **kwargs
    ) -> "VectorizerStore":
        """
        Create VectorizerStore from texts.
        
        Args:
            texts: List of texts
            embedding: Optional embedding model
            metadatas: Optional metadata
            config: Optional VectorizerConfig
            **kwargs: Additional arguments
            
        Returns:
            VectorizerStore instance
        """
        if config is None:
            config = VectorizerConfig()
        
        store = cls(config, embedding)
        store.add_texts(texts, metadatas)
        return store
    
    @classmethod
    def from_documents(
        cls,
        documents: List[Document],
        embedding: Optional[Embeddings] = None,
        config: Optional[VectorizerConfig] = None,
        **kwargs
    ) -> "VectorizerStore":
        """
        Create VectorizerStore from documents.
        
        Args:
            documents: List of Document objects
            embedding: Optional embedding model
            config: Optional VectorizerConfig
            **kwargs: Additional arguments
            
        Returns:
            VectorizerStore instance
        """
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        
        return cls.from_texts(texts, embedding, metadatas, config, **kwargs)


# Convenience functions for easy usage
def create_vectorizer_store(
    host: str = "localhost",
    port: int = 15001,
    collection_name: str = "langchain_documents",
    api_key: Optional[str] = None,
    **kwargs
) -> VectorizerStore:
    """
    Create a VectorizerStore with default configuration.
    
    Args:
        host: Vectorizer host
        port: Vectorizer port
        collection_name: Collection name
        api_key: Optional API key
        **kwargs: Additional configuration options
        
    Returns:
        VectorizerStore instance
    """
    config = VectorizerConfig(
        host=host,
        port=port,
        collection_name=collection_name,
        api_key=api_key,
        **kwargs
    )
    
    return VectorizerStore(config)


# Example usage
if __name__ == "__main__":
    # Example usage
    config = VectorizerConfig(
        host="localhost",
        port=15001,
        collection_name="test_documents"
    )
    
    # Create store
    store = VectorizerStore(config)
    
    # Add documents
    texts = [
        "This is the first document",
        "This is the second document",
        "This is the third document"
    ]
    
    metadatas = [
        {"source": "doc1.txt", "page": 1},
        {"source": "doc2.txt", "page": 1},
        {"source": "doc3.txt", "page": 1}
    ]
    
    vector_ids = store.add_texts(texts, metadatas)
    print(f"Added {len(vector_ids)} documents")
    
    # Search
    results = store.similarity_search("first document", k=2)
    print(f"Found {len(results)} results")
    
    for doc in results:
        print(f"Content: {doc.page_content}")
        print(f"Metadata: {doc.metadata}")
        print("---")

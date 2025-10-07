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
    port: int = 15002
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
        self.base_url = f"http://{config.host}:{config.port}"
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
    
    def create_collection(self, name: str, dimension: int = 512, metric: str = "cosine") -> bool:
        """Create a new collection"""
        data = {
            "name": name,
            "dimension": dimension,
            "metric": metric
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
        
        vector_ids = []
        for text, metadata in zip(texts, metadatas):
            data = {
                "collection": self.config.collection_name,
                "text": text,
                "metadata": metadata
            }
            
            response = self._make_request("POST", "/insert", json=data)
            vector_ids.append(response.get("vector_id", ""))
        
        return vector_ids
    
    def similarity_search(
        self, 
        query: str, 
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """Perform similarity search"""
        data = {
            "query": query,
            "limit": k,
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
                f"/collections/{self.config.collection_name}/vectors",
                json=data
            )
            return True
        except VectorizerError:
            return False
    
    # ===== INTELLIGENT SEARCH METHODS =====
    
    def intelligent_search(
        self,
        query: str,
        collections: Optional[List[str]] = None,
        max_results: int = 10,
        domain_expansion: bool = True,
        technical_focus: bool = True,
        mmr_enabled: bool = True,
        mmr_lambda: float = 0.7
    ) -> List[Dict[str, Any]]:
        """Perform intelligent search with multi-query expansion"""
        data = {
            "query": query,
            "max_results": max_results,
            "domain_expansion": domain_expansion,
            "technical_focus": technical_focus,
            "mmr_enabled": mmr_enabled,
            "mmr_lambda": mmr_lambda
        }
        
        if collections:
            data["collections"] = collections
        
        response = self._make_request("POST", "/intelligent_search", json=data)
        return response.get("results", [])
    
    def semantic_search(
        self,
        query: str,
        collection: str,
        max_results: int = 10,
        semantic_reranking: bool = True,
        cross_encoder_reranking: bool = False,
        similarity_threshold: float = 0.5
    ) -> List[Dict[str, Any]]:
        """Perform semantic search with advanced reranking"""
        data = {
            "query": query,
            "collection": collection,
            "max_results": max_results,
            "semantic_reranking": semantic_reranking,
            "cross_encoder_reranking": cross_encoder_reranking,
            "similarity_threshold": similarity_threshold
        }
        
        response = self._make_request("POST", "/semantic_search", json=data)
        return response.get("results", [])
    
    def contextual_search(
        self,
        query: str,
        collection: str,
        context_filters: Optional[Dict[str, Any]] = None,
        max_results: int = 10,
        context_reranking: bool = True,
        context_weight: float = 0.3
    ) -> List[Dict[str, Any]]:
        """Perform context-aware search with metadata filtering"""
        data = {
            "query": query,
            "collection": collection,
            "max_results": max_results,
            "context_reranking": context_reranking,
            "context_weight": context_weight
        }
        
        if context_filters:
            data["context_filters"] = context_filters
        
        response = self._make_request("POST", "/contextual_search", json=data)
        return response.get("results", [])
    
    def multi_collection_search(
        self,
        query: str,
        collections: List[str],
        max_per_collection: int = 5,
        max_total_results: int = 20,
        cross_collection_reranking: bool = True
    ) -> List[Dict[str, Any]]:
        """Perform multi-collection search with cross-collection reranking"""
        data = {
            "query": query,
            "collections": collections,
            "max_per_collection": max_per_collection,
            "max_total_results": max_total_results,
            "cross_collection_reranking": cross_collection_reranking
        }
        
        response = self._make_request("POST", "/multi_collection_search", json=data)
        return response.get("results", [])


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
                dimension=512,  # Default for v0.3.0
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
    
    # ===== INTELLIGENT SEARCH METHODS =====
    
    def intelligent_search(
        self,
        query: str,
        collections: Optional[List[str]] = None,
        max_results: int = 10,
        domain_expansion: bool = True,
        technical_focus: bool = True,
        mmr_enabled: bool = True,
        mmr_lambda: float = 0.7,
        **kwargs
    ) -> List[Document]:
        """
        Perform intelligent search with multi-query expansion.
        
        Args:
            query: Query text
            collections: Optional list of collections to search
            max_results: Maximum number of results
            domain_expansion: Enable domain expansion
            technical_focus: Enable technical focus
            mmr_enabled: Enable MMR diversification
            mmr_lambda: MMR balance parameter (0.0-1.0)
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of Document objects
        """
        results = self.client.intelligent_search(
            query, collections, max_results, domain_expansion,
            technical_focus, mmr_enabled, mmr_lambda
        )
        
        documents = []
        for result in results:
            doc = Document(
                page_content=result.get("content", ""),
                metadata=result.get("metadata", {})
            )
            documents.append(doc)
        
        return documents
    
    def semantic_search(
        self,
        query: str,
        collection: str,
        max_results: int = 10,
        semantic_reranking: bool = True,
        cross_encoder_reranking: bool = False,
        similarity_threshold: float = 0.5,
        **kwargs
    ) -> List[Document]:
        """
        Perform semantic search with advanced reranking.
        
        Args:
            query: Query text
            collection: Collection to search
            max_results: Maximum number of results
            semantic_reranking: Enable semantic reranking
            cross_encoder_reranking: Enable cross-encoder reranking
            similarity_threshold: Minimum similarity threshold
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of Document objects
        """
        results = self.client.semantic_search(
            query, collection, max_results, semantic_reranking,
            cross_encoder_reranking, similarity_threshold
        )
        
        documents = []
        for result in results:
            doc = Document(
                page_content=result.get("content", ""),
                metadata=result.get("metadata", {})
            )
            documents.append(doc)
        
        return documents
    
    def contextual_search(
        self,
        query: str,
        collection: str,
        context_filters: Optional[Dict[str, Any]] = None,
        max_results: int = 10,
        context_reranking: bool = True,
        context_weight: float = 0.3,
        **kwargs
    ) -> List[Document]:
        """
        Perform context-aware search with metadata filtering.
        
        Args:
            query: Query text
            collection: Collection to search
            context_filters: Metadata-based context filters
            max_results: Maximum number of results
            context_reranking: Enable context-aware reranking
            context_weight: Weight of context factors (0.0-1.0)
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of Document objects
        """
        results = self.client.contextual_search(
            query, collection, context_filters, max_results,
            context_reranking, context_weight
        )
        
        documents = []
        for result in results:
            doc = Document(
                page_content=result.get("content", ""),
                metadata=result.get("metadata", {})
            )
            documents.append(doc)
        
        return documents
    
    def multi_collection_search(
        self,
        query: str,
        collections: List[str],
        max_per_collection: int = 5,
        max_total_results: int = 20,
        cross_collection_reranking: bool = True,
        **kwargs
    ) -> List[Document]:
        """
        Perform multi-collection search with cross-collection reranking.
        
        Args:
            query: Query text
            collections: Collections to search
            max_per_collection: Maximum results per collection
            max_total_results: Maximum total results
            cross_collection_reranking: Enable cross-collection reranking
            **kwargs: Additional arguments (ignored)
            
        Returns:
            List of Document objects
        """
        results = self.client.multi_collection_search(
            query, collections, max_per_collection,
            max_total_results, cross_collection_reranking
        )
        
        documents = []
        for result in results:
            doc = Document(
                page_content=result.get("content", ""),
                metadata=result.get("metadata", {})
            )
            documents.append(doc)
        
        return documents
    
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
    port: int = 15002,
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
        port=15002,
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
    
    # Traditional search
    results = store.similarity_search("first document", k=2)
    print(f"Found {len(results)} results")
    
    for doc in results:
        print(f"Content: {doc.page_content}")
        print(f"Metadata: {doc.metadata}")
        print("---")
    
    # Intelligent search with multi-query expansion
    intelligent_results = store.intelligent_search(
        query="machine learning algorithms",
        collections=["test_documents"],
        max_results=5,
        domain_expansion=True,
        technical_focus=True,
        mmr_enabled=True,
        mmr_lambda=0.7
    )
    print(f"Intelligent search found {len(intelligent_results)} results")
    
    # Semantic search with reranking
    semantic_results = store.semantic_search(
        query="neural networks",
        collection="test_documents",
        max_results=5,
        semantic_reranking=True,
        similarity_threshold=0.6
    )
    print(f"Semantic search found {len(semantic_results)} results")
    
    # Contextual search with metadata filtering
    contextual_results = store.contextual_search(
        query="deep learning",
        collection="test_documents",
        context_filters={"source": "doc1.txt"},
        max_results=5,
        context_weight=0.4
    )
    print(f"Contextual search found {len(contextual_results)} results")
    
    # Multi-collection search
    multi_results = store.multi_collection_search(
        query="artificial intelligence",
        collections=["test_documents"],
        max_per_collection=3,
        max_total_results=10,
        cross_collection_reranking=True
    )
    print(f"Multi-collection search found {len(multi_results)} results")

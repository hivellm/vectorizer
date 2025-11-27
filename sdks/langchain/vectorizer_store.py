"""
LangChain Integration for Vectorizer

This module provides a LangChain VectorStore implementation that uses Vectorizer
as the backend for vector storage and similarity search.

Updated to use modern Vectorizer Python SDK with async support.
"""

from typing import List, Dict, Any, Optional, Tuple
import json
import asyncio
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

# Try to import VectorizerClient from Python SDK
try:
    import sys
    import os
    # Add parent directory to path to import from Python SDK
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))
    from client import VectorizerClient as SDKClient
    SDK_AVAILABLE = True
except ImportError:
    # Fallback to basic implementation if SDK not available
    SDK_AVAILABLE = False
    try:
        import requests
        import aiohttp
    except ImportError:
        requests = None
        aiohttp = None


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
    use_sdk: bool = True  # Use modern SDK if available


class VectorizerClient:
    """Client for communicating with Vectorizer API"""
    
    def __init__(self, config: VectorizerConfig):
        self.config = config
        self.base_url = f"http://{config.host}:{config.port}"
        
        # Use modern SDK if available and enabled
        if SDK_AVAILABLE and config.use_sdk:
            self._client = SDKClient(
                base_url=self.base_url,
                api_key=config.api_key,
                timeout=config.timeout
            )
            self._use_sdk = True
        else:
            # Fallback to basic requests
            if requests is None:
                raise ImportError(
                    "Vectorizer Python SDK not available. "
                    "Install with: pip install vectorizer-sdk or ensure requests/aiohttp are installed"
                )
            self.session = requests.Session()
            if config.api_key:
                self.session.headers.update({"Authorization": f"Bearer {config.api_key}"})
            self._use_sdk = False
    
    def _make_request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """Make HTTP request to Vectorizer API"""
        if self._use_sdk:
            # Use SDK methods when available
            raise NotImplementedError("Use async methods with SDK client")
        
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
        if self._use_sdk:
            # Use SDK async methods
            raise NotImplementedError("Use SDK client methods directly")
        
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
    
    # ===== FILE OPERATIONS METHODS (v0.3.4+) =====
    
    def get_file_content(
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
            max_size_kb: Maximum file size in KB (default: 500, max: 5000)
            
        Returns:
            Dictionary with file content, metadata, and chunking info
        """
        data = {
            "collection": collection,
            "file_path": file_path,
            "max_size_kb": max_size_kb
        }
        
        response = self._make_request("POST", "/get_file_content", json=data)
        return response
    
    def list_files_in_collection(
        self,
        collection: str,
        filter_by_type: Optional[List[str]] = None,
        min_chunks: Optional[int] = None,
        sort_by: str = "name",
        max_results: int = 100
    ) -> Dict[str, Any]:
        """
        List all indexed files in a collection with metadata.
        
        Args:
            collection: Collection name
            filter_by_type: Filter by file types (e.g., ['rs', 'md', 'py'])
            min_chunks: Minimum number of chunks (filters out small files)
            sort_by: Sort order ('name', 'size', 'chunks', 'recent')
            max_results: Maximum number of results (default: 100)
            
        Returns:
            Dictionary with files list and statistics
        """
        data = {
            "collection": collection,
            "sort_by": sort_by,
            "max_results": max_results
        }
        
        if filter_by_type:
            data["filter_by_type"] = filter_by_type
        if min_chunks:
            data["min_chunks"] = min_chunks
        
        response = self._make_request("POST", "/list_files_in_collection", json=data)
        return response
    
    def get_file_summary(
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
            summary_type: Type of summary ('extractive', 'structural', 'both')
            max_sentences: Maximum sentences for extractive summary (default: 5)
            
        Returns:
            Dictionary with summaries and metadata
        """
        data = {
            "collection": collection,
            "file_path": file_path,
            "summary_type": summary_type,
            "max_sentences": max_sentences
        }
        
        response = self._make_request("POST", "/get_file_summary", json=data)
        return response
    
    def get_project_outline(
        self,
        collection: str,
        max_depth: int = 5,
        highlight_key_files: bool = True,
        include_summaries: bool = False
    ) -> Dict[str, Any]:
        """
        Generate hierarchical project structure overview.
        
        Args:
            collection: Collection name
            max_depth: Maximum directory depth (default: 5)
            highlight_key_files: Highlight important files like README
            include_summaries: Include file summaries in outline
            
        Returns:
            Dictionary with project structure tree and statistics
        """
        data = {
            "collection": collection,
            "max_depth": max_depth,
            "highlight_key_files": highlight_key_files,
            "include_summaries": include_summaries
        }
        
        response = self._make_request("POST", "/get_project_outline", json=data)
        return response
    
    def get_related_files(
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
            limit: Maximum number of related files (default: 5)
            similarity_threshold: Minimum similarity score 0.0-1.0
            include_reason: Include explanation of why files are related
            
        Returns:
            Dictionary with related files and similarity scores
        """
        data = {
            "collection": collection,
            "file_path": file_path,
            "limit": limit,
            "similarity_threshold": similarity_threshold,
            "include_reason": include_reason
        }
        
        response = self._make_request("POST", "/get_related_files", json=data)
        return response
    
    def search_by_file_type(
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
            file_types: File extensions to search (e.g., ['yaml', 'toml', 'json'])
            limit: Maximum results (default: 10)
            return_full_files: Return complete file content
            
        Returns:
            Dictionary with search results filtered by file type
        """
        data = {
            "collection": collection,
            "query": query,
            "file_types": file_types,
            "limit": limit,
            "return_full_files": return_full_files
        }
        
        response = self._make_request("POST", "/search_by_file_type", json=data)
        return response
    
    # ===== DISCOVERY SYSTEM METHODS (v0.3.4+) =====
    
    def discover(
        self,
        query: str,
        include_collections: Optional[List[str]] = None,
        exclude_collections: Optional[List[str]] = None,
        broad_k: int = 50,
        focus_k: int = 15,
        max_bullets: int = 20
    ) -> Dict[str, Any]:
        """
        Complete discovery pipeline with filtering, scoring, expansion, search, ranking, 
        compression, and prompt generation.
        
        Args:
            query: User question or search query
            include_collections: Collections to include (glob patterns like 'vectorizer*')
            exclude_collections: Collections to exclude
            broad_k: Broad search results
            focus_k: Focus search results per collection
            max_bullets: Maximum evidence bullets
            
        Returns:
            Dictionary with answer_prompt, bullets, chunks, and metrics
        """
        data = {
            "query": query,
            "broad_k": broad_k,
            "focus_k": focus_k,
            "max_bullets": max_bullets
        }
        
        if include_collections:
            data["include_collections"] = include_collections
        if exclude_collections:
            data["exclude_collections"] = exclude_collections
        
        response = self._make_request("POST", "/discover", json=data)
        return response
    
    def filter_collections(
        self,
        query: str,
        include: Optional[List[str]] = None,
        exclude: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        """
        Pre-filter collections by name patterns with stopword removal from query.
        
        Args:
            query: Search query for filtering
            include: Include patterns (e.g., ['vectorizer*', '*-docs'])
            exclude: Exclude patterns (e.g., ['*-test'])
            
        Returns:
            Dictionary with filtered collections list
        """
        data = {"query": query}
        
        if include:
            data["include"] = include
        if exclude:
            data["exclude"] = exclude
        
        response = self._make_request("POST", "/filter_collections", json=data)
        return response
    
    def score_collections(
        self,
        query: str,
        name_match_weight: float = 0.4,
        term_boost_weight: float = 0.3,
        signal_boost_weight: float = 0.3
    ) -> Dict[str, Any]:
        """
        Rank collections by relevance using name match, term boost, and signal boost.
        
        Args:
            query: Search query for scoring
            name_match_weight: Weight for name matching
            term_boost_weight: Weight for term boost
            signal_boost_weight: Weight for signals (size, recency, tags)
            
        Returns:
            Dictionary with scored collections list
        """
        data = {
            "query": query,
            "name_match_weight": name_match_weight,
            "term_boost_weight": term_boost_weight,
            "signal_boost_weight": signal_boost_weight
        }
        
        response = self._make_request("POST", "/score_collections", json=data)
        return response
    
    def expand_queries(
        self,
        query: str,
        max_expansions: int = 8,
        include_definition: bool = True,
        include_features: bool = True,
        include_architecture: bool = True
    ) -> Dict[str, Any]:
        """
        Generate query variations (definition, features, architecture, API, performance, use cases).
        
        Args:
            query: Original query to expand
            max_expansions: Maximum expansions
            include_definition: Include definition queries
            include_features: Include features queries
            include_architecture: Include architecture queries
            
        Returns:
            Dictionary with expanded queries list
        """
        data = {
            "query": query,
            "max_expansions": max_expansions,
            "include_definition": include_definition,
            "include_features": include_features,
            "include_architecture": include_architecture
        }
        
        response = self._make_request("POST", "/expand_queries", json=data)
        return response
    
    def broad_discovery(
        self,
        queries: List[str],
        k: int = 50
    ) -> Dict[str, Any]:
        """
        Multi-query broad search with MMR diversification and deduplication.
        
        Args:
            queries: Array of search queries
            k: Maximum results
            
        Returns:
            Dictionary with broad search chunks
        """
        data = {
            "queries": queries,
            "k": k
        }
        
        response = self._make_request("POST", "/broad_discovery", json=data)
        return response
    
    def semantic_focus(
        self,
        collection: str,
        queries: List[str],
        k: int = 15
    ) -> Dict[str, Any]:
        """
        Deep semantic search in specific collection with reranking and context window.
        
        Args:
            collection: Target collection name
            queries: Array of search queries
            k: Maximum results
            
        Returns:
            Dictionary with focused search chunks
        """
        data = {
            "collection": collection,
            "queries": queries,
            "k": k
        }
        
        response = self._make_request("POST", "/semantic_focus", json=data)
        return response
    
    def compress_evidence(
        self,
        chunks: List[Dict[str, Any]],
        max_bullets: int = 20,
        max_per_doc: int = 3
    ) -> Dict[str, Any]:
        """
        Extract key sentences (8-30 words) with citations from chunks.
        
        Args:
            chunks: Array of scored chunks to compress
            max_bullets: Max bullets to extract
            max_per_doc: Max bullets per document
            
        Returns:
            Dictionary with compressed evidence bullets
        """
        data = {
            "chunks": chunks,
            "max_bullets": max_bullets,
            "max_per_doc": max_per_doc
        }
        
        response = self._make_request("POST", "/compress_evidence", json=data)
        return response
    
    def build_answer_plan(
        self,
        bullets: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """
        Organize bullets into structured sections (Definition, Features, Architecture, 
        Performance, Integrations, Use Cases).
        
        Args:
            bullets: Array of evidence bullets
            
        Returns:
            Dictionary with structured answer plan
        """
        data = {"bullets": bullets}
        
        response = self._make_request("POST", "/build_answer_plan", json=data)
        return response
    
    def render_llm_prompt(
        self,
        plan: Dict[str, Any]
    ) -> Dict[str, Any]:
        """
        Generate compact, structured prompt for LLM with instructions, evidence, and citations.
        
        Args:
            plan: Answer plan with sections and bullets
            
        Returns:
            Dictionary with rendered LLM prompt
        """
        data = {"plan": plan}
        
        response = self._make_request("POST", "/render_llm_prompt", json=data)
        return response


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

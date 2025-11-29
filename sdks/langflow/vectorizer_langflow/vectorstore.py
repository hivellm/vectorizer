"""Vectorizer VectorStore implementation for LangChain."""

from typing import Any, Dict, Iterable, List, Optional, Tuple
import requests
from langchain.schema import Document
from langchain.schema.embeddings import Embeddings
from langchain.schema.vectorstore import VectorStore


class VectorizerVectorStore(VectorStore):
    """Vectorizer vector database implementation for LangChain.

    This class provides a LangChain-compatible interface to Vectorizer,
    allowing seamless integration into LangChain workflows and Langflow.

    Example:
        >>> from langchain.embeddings import OpenAIEmbeddings
        >>> vectorstore = VectorizerVectorStore(
        ...     host="http://localhost:15002",
        ...     collection_name="my-collection",
        ...     embedding=OpenAIEmbeddings()
        ... )
        >>> vectorstore.add_texts(["Hello world", "Goodbye world"])
        >>> results = vectorstore.similarity_search("Hello", k=1)
    """

    def __init__(
        self,
        host: str = "http://localhost:15002",
        collection_name: str = "default",
        embedding: Optional[Embeddings] = None,
        dimension: Optional[int] = None,
        metric: str = "cosine",
        api_key: Optional[str] = None,
    ):
        """Initialize Vectorizer vector store.

        Args:
            host: Vectorizer instance URL
            collection_name: Name of the collection to use
            embedding: LangChain embedding model
            dimension: Vector dimension (auto-detected if not provided)
            metric: Distance metric (cosine, euclidean, dot)
            api_key: Optional API key for authentication
        """
        self.host = host.rstrip("/")
        self.collection_name = collection_name
        self.embedding = embedding
        self.dimension = dimension
        self.metric = metric
        self.api_key = api_key

        self.headers = {"Content-Type": "application/json"}
        if api_key:
            self.headers["X-API-Key"] = api_key

        # Ensure collection exists
        self._ensure_collection()

    def _ensure_collection(self) -> None:
        """Create collection if it doesn't exist."""
        # Check if collection exists
        response = requests.get(
            f"{self.host}/collections/{self.collection_name}",
            headers=self.headers
        )

        if response.status_code == 404:
            # Create collection
            dimension = self.dimension or 384  # Default dimension
            requests.post(
                f"{self.host}/collections",
                json={
                    "name": self.collection_name,
                    "dimension": dimension,
                    "metric": self.metric
                },
                headers=self.headers
            )

    def add_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[List[Dict[str, Any]]] = None,
        ids: Optional[List[str]] = None,
        **kwargs: Any,
    ) -> List[str]:
        """Add texts to the vector store."""
        if not self.embedding:
            raise ValueError("Embedding model is required for add_texts")

        texts_list = list(texts)
        embeddings = self.embedding.embed_documents(texts_list)

        if not ids:
            import uuid
            ids = [str(uuid.uuid4()) for _ in texts_list]

        if not metadatas:
            metadatas = [{} for _ in texts_list]

        # Add text to metadata
        for i, text in enumerate(texts_list):
            metadatas[i]["text"] = text

        # Batch insert vectors
        vectors = [
            {"id": id_, "vector": emb, "payload": meta}
            for id_, emb, meta in zip(ids, embeddings, metadatas)
        ]

        requests.post(
            f"{self.host}/collections/{self.collection_name}/batch_insert",
            json={"vectors": vectors},
            headers=self.headers
        )

        return ids

    def similarity_search(
        self,
        query: str,
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None,
        **kwargs: Any,
    ) -> List[Document]:
        """Search for similar documents."""
        if not self.embedding:
            raise ValueError("Embedding model is required for similarity_search")

        query_vector = self.embedding.embed_query(query)

        response = requests.post(
            f"{self.host}/collections/{self.collection_name}/search",
            json={
                "vector": query_vector,
                "limit": k,
                "filter": filter or {}
            },
            headers=self.headers
        )

        results = response.json()
        documents = [
            Document(
                page_content=r.get("payload", {}).get("text", ""),
                metadata={k: v for k, v in r.get("payload", {}).items() if k != "text"}
            )
            for r in results
        ]

        return documents

    def similarity_search_with_score(
        self,
        query: str,
        k: int = 4,
        filter: Optional[Dict[str, Any]] = None,
        **kwargs: Any,
    ) -> List[Tuple[Document, float]]:
        """Search for similar documents with scores."""
        if not self.embedding:
            raise ValueError("Embedding model is required")

        query_vector = self.embedding.embed_query(query)

        response = requests.post(
            f"{self.host}/collections/{self.collection_name}/search",
            json={
                "vector": query_vector,
                "limit": k,
                "filter": filter or {}
            },
            headers=self.headers
        )

        results = response.json()
        documents = [
            (
                Document(
                    page_content=r.get("payload", {}).get("text", ""),
                    metadata={k: v for k, v in r.get("payload", {}).items() if k != "text"}
                ),
                r.get("score", 0.0)
            )
            for r in results
        ]

        return documents

    @classmethod
    def from_texts(
        cls,
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[Dict]] = None,
        **kwargs: Any,
    ) -> "VectorizerVectorStore":
        """Create vector store from texts."""
        vectorstore = cls(embedding=embedding, **kwargs)
        vectorstore.add_texts(texts, metadatas)
        return vectorstore

    @classmethod
    def from_documents(
        cls,
        documents: List[Document],
        embedding: Embeddings,
        **kwargs: Any,
    ) -> "VectorizerVectorStore":
        """Create vector store from documents."""
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        return cls.from_texts(texts, embedding, metadatas, **kwargs)

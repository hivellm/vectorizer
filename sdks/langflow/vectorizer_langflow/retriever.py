"""Vectorizer Retriever for LangChain RAG pipelines."""

from typing import Any, Dict, List, Optional
from langchain.schema import BaseRetriever, Document
from langchain.schema.embeddings import Embeddings
from langchain.callbacks.manager import CallbackManagerForRetrieverRun
import requests


class VectorizerRetriever(BaseRetriever):
    """Retriever implementation for Vectorizer."""

    host: str = "http://localhost:15002"
    collection_name: str
    embedding: Embeddings
    search_kwargs: Dict[str, Any] = {}
    api_key: Optional[str] = None

    def _get_relevant_documents(
        self,
        query: str,
        *,
        run_manager: CallbackManagerForRetrieverRun,
    ) -> List[Document]:
        """Retrieve documents relevant to the query."""
        query_vector = self.embedding.embed_query(query)

        k = self.search_kwargs.get("k", 4)
        score_threshold = self.search_kwargs.get("score_threshold", 0.0)
        filter_dict = self.search_kwargs.get("filter", {})

        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["X-API-Key"] = self.api_key

        response = requests.post(
            f"{self.host}/collections/{self.collection_name}/search",
            json={
                "vector": query_vector,
                "limit": k,
                "score_threshold": score_threshold,
                "filter": filter_dict
            },
            headers=headers
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

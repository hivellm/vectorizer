"""Vectorizer Loader for loading existing vectors as LangChain documents."""

from typing import Iterator, List, Optional
from langchain.schema import Document
from langchain.document_loaders.base import BaseLoader
import requests


class VectorizerLoader(BaseLoader):
    """Load vectors from Vectorizer collection as LangChain documents."""

    def __init__(
        self,
        host: str = "http://localhost:15002",
        collection_name: str = "default",
        limit: Optional[int] = None,
        offset: int = 0,
        api_key: Optional[str] = None,
    ):
        """Initialize the loader.

        Args:
            host: Vectorizer instance URL
            collection_name: Collection to load from
            limit: Maximum number of vectors to load
            offset: Pagination offset
            api_key: Optional API key
        """
        self.host = host.rstrip("/")
        self.collection_name = collection_name
        self.limit = limit
        self.offset = offset
        self.api_key = api_key

        self.headers = {"Content-Type": "application/json"}
        if api_key:
            self.headers["X-API-Key"] = api_key

    def load(self) -> List[Document]:
        """Load documents from collection."""
        params = {"offset": self.offset}
        if self.limit:
            params["limit"] = self.limit

        response = requests.get(
            f"{self.host}/collections/{self.collection_name}/vectors",
            params=params,
            headers=self.headers
        )

        vectors = response.json()
        documents = [
            Document(
                page_content=v.get("payload", {}).get("text", ""),
                metadata={
                    "id": v.get("id"),
                    **{k: val for k, val in v.get("payload", {}).items() if k != "text"}
                }
            )
            for v in vectors
        ]

        return documents

    def lazy_load(self) -> Iterator[Document]:
        """Lazy load documents one at a time."""
        for doc in self.load():
            yield doc

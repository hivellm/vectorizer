"""Raw-vector similarity search.

Returns `(dataset_id, score)` pairs, which is what the framework intersects
with the ground truth. See the id-contract note in `upload.py`.
"""

from typing import List, Tuple

from dataset_reader.base_reader import Query
from engine.base_client.search import BaseSearcher
from engine.clients.vectorizer.config import (
    COLLECTION_NAME,
    MAX_SEARCH_LIMIT,
    get_auth_headers,
    get_base_url,
)
from engine.clients.vectorizer.transport import JsonHttpClient


class VectorizerSearcher(BaseSearcher):
    client: JsonHttpClient = None
    search_params = {}

    @classmethod
    def init_client(cls, host, distance, connection_params, search_params):
        cls.client = JsonHttpClient(
            get_base_url(host, connection_params), headers=get_auth_headers()
        )
        cls.search_params = search_params

    @classmethod
    def search_one(cls, query: Query, top) -> List[Tuple[int, float]]:
        if top > MAX_SEARCH_LIMIT:
            # The server clamps `limit` to MAX_SEARCH_LIMIT without saying so.
            # Left alone, a top-1000 dataset would be scored against 100
            # results and report ~10% recall — a truncation misread as a
            # search-quality finding. Refuse instead.
            raise ValueError(
                f"dataset asks for top-{top} but the server clamps search "
                f"limit to {MAX_SEARCH_LIMIT} (MAX_SEARCH_LIMIT in "
                "rest_handlers/search.rs). Comparing at this top would score "
                "a truncation as poor recall. Either benchmark at a smaller "
                "top or raise the server cap deliberately."
            )

        response = cls.client.request(
            "POST",
            f"/collections/{COLLECTION_NAME}/search",
            {"vector": list(query.vector), "limit": top},
        )

        results = (response or {}).get("results") or []
        return [(int(hit["id"]), float(hit["score"])) for hit in results]

    @classmethod
    def delete_client(cls):
        if cls.client is not None:
            cls.client.close()
            cls.client = None

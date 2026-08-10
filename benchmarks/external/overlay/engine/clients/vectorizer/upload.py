"""Bulk-upload pre-computed embeddings.

The framework scores a run with

    precision = len(returned_ids & query.expected_result[:top]) / top

so the ids we hand back at search time must be the dataset's integers. That
makes the id round-trip — not the latency — the thing most likely to be
silently wrong here, because getting it wrong yields 0.00% precision at full
speed, which reads like a fast engine with bad recall rather than a broken
client. Vectorizer's vector ids are strings, so upload stringifies the
dataset id and search parses it back; both halves live next to this comment
for a reason.
"""

from typing import List

from dataset_reader.base_reader import Record
from engine.base_client.upload import BaseUploader
from engine.clients.vectorizer.config import (
    COLLECTION_NAME,
    get_auth_headers,
    get_base_url,
)
from engine.clients.vectorizer.transport import JsonHttpClient


class VectorizerUploader(BaseUploader):
    client: JsonHttpClient = None
    upload_params = {}

    @classmethod
    def init_client(cls, host, distance, connection_params, upload_params):
        cls.client = JsonHttpClient(
            get_base_url(host, connection_params), headers=get_auth_headers()
        )
        cls.upload_params = upload_params

    @classmethod
    def upload_batch(cls, batch: List[Record]):
        vectors = [
            {
                # str() here, int() in search.py — the two must stay a pair.
                "id": str(record.id),
                "embedding": record.vector,
                "payload": record.metadata or {},
            }
            for record in batch
        ]

        response = cls.client.request(
            "POST",
            "/insert_vectors",
            {"collection": COLLECTION_NAME, "vectors": vectors},
        )

        # `/insert_vectors` answers 200 with a per-row breakdown, so a partial
        # failure is a successful HTTP call. Uploading fewer vectors than the
        # dataset holds quietly lowers recall for every query afterwards, and
        # the run would report it as a search-quality result.
        failed = (response or {}).get("failed")
        if failed:
            raise RuntimeError(
                f"{failed} of {len(vectors)} vectors were rejected by the server; "
                f"first rows: {(response or {}).get('results', [])[:3]}"
            )

    @classmethod
    def post_upload(cls, _distance):
        return {}

    @classmethod
    def delete_client(cls):
        if cls.client is not None:
            cls.client.close()
            cls.client = None

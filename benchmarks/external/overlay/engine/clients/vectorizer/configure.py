"""Create and drop the benchmark collection."""

from benchmark.dataset import Dataset
from engine.base_client.configure import BaseConfigurator
from engine.base_client.distances import Distance
from engine.clients.vectorizer.config import (
    COLLECTION_NAME,
    get_auth_headers,
    get_base_url,
)
from engine.clients.vectorizer.transport import JsonHttpClient, JsonHttpError


class VectorizerConfigurator(BaseConfigurator):
    SPARSE_VECTOR_SUPPORT = False

    DISTANCE_MAPPING = {
        Distance.L2: "euclidean",
        Distance.COSINE: "cosine",
        Distance.DOT: "dot_product",
    }

    def __init__(self, host, collection_params: dict, connection_params: dict):
        super().__init__(host, collection_params, connection_params)
        self.client = JsonHttpClient(
            get_base_url(host, connection_params), headers=get_auth_headers()
        )

    def clean(self):
        try:
            self.client.request("DELETE", f"/collections/{COLLECTION_NAME}")
        except JsonHttpError as exc:
            # Absent is the desired post-condition, so a 404 is success. Any
            # other status is a real problem and must not be swallowed —
            # silently continuing here would benchmark against a collection
            # left over from the previous run.
            if exc.status != 404:
                raise

    def recreate(self, dataset: Dataset, collection_params):
        distance = self.DISTANCE_MAPPING.get(dataset.config.distance)
        if distance is None:
            raise IncompatibleDistance(dataset.config.distance)

        # Created through the Qdrant-compatible endpoint, not the native
        # `POST /collections`, and the reason is a real limitation rather than
        # a preference:
        #
        # Native `create_collection` always resolves an embedding provider
        # (defaulting to `bm25`) and rejects any dimension that differs from
        # that provider's — 512 for BM25. Every ANN benchmark dataset is some
        # other width (glove 100, 384, 768, 1536), so the native endpoint
        # refuses all of them with `provider_dimension_mismatch`, even though
        # `/insert_vectors` exists precisely for callers bringing their own
        # embeddings and needs no provider at all.
        #
        # The Qdrant-compatible route goes straight to `store.create_collection`
        # with no provider resolution, so it accepts the dataset's width.
        #
        # This is setup, not measurement: upload and search below both run on
        # the native API, which is what the benchmark reports on.
        qdrant_distance = {
            "cosine": "Cosine",
            "euclidean": "Euclid",
            "dot_product": "Dot",
        }[distance]

        self.client.request(
            "PUT",
            f"/qdrant/collections/{COLLECTION_NAME}",
            {
                "vectors": {
                    "size": dataset.config.vector_size,
                    "distance": qdrant_distance,
                },
                **collection_params,
            },
        )

    def delete_client(self):
        self.client.close()


class IncompatibleDistance(ValueError):
    def __init__(self, distance):
        super().__init__(
            f"Vectorizer has no mapping for distance {distance!r}. "
            "Add it to VectorizerConfigurator.DISTANCE_MAPPING only if the "
            "server genuinely implements it — a wrong mapping produces a "
            "plausible recall number for the wrong metric."
        )

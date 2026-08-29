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

        # `embedding_provider: "none"` is what makes the native endpoint
        # usable here. Every ANN benchmark dataset arrives pre-vectorized at
        # some width the server has no provider for (glove 100, 384, 768,
        # 1536); an ordinary collection resolves a provider and rejects any
        # dimension that disagrees with it, so it refuses all of them. A
        # provider-less collection takes the caller's own vectors through
        # `/insert_vectors`, which is the path `upload.py` uses.
        self.client.request(
            "POST",
            "/collections",
            {
                "name": COLLECTION_NAME,
                "dimension": dataset.config.vector_size,
                "metric": distance,
                "embedding_provider": "none",
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

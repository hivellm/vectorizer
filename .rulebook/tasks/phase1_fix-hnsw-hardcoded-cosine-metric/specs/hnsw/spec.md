# HNSW metric correctness

## MODIFIED Requirements

### Requirement: HNSW ranks by the collection's configured distance metric
The `OptimizedHnswIndex` SHALL rank neighbors using the distance function that
corresponds to the collection's configured `DistanceMetric`, and MUST NOT
hardcode cosine distance. The reported similarity/score MUST be derived from
that same metric.

#### Scenario: Euclidean collection ranks by L2, not cosine
Given a collection created with `metric: Euclidean`
And two candidate vectors A and B where A is closer to the query by L2 distance
but B is closer by cosine similarity
When the collection is searched with that query
Then A MUST rank above B.

#### Scenario: Dot collection ranks by inner product
Given a collection created with `metric: Dot`
When searched with a query vector
Then results MUST be ordered by inner product (largest inner product first),
not by cosine similarity.

#### Scenario: Cosine collection is unchanged
Given a collection created with `metric: Cosine`
When searched with a query vector
Then the ranking MUST match the previous cosine behavior (regression-safe).

#### Scenario: Metric survives reindex
Given a collection with a non-cosine metric
When `reindex_with_params` rebuilds the index
Then the rebuilt index MUST continue to rank by the original metric.

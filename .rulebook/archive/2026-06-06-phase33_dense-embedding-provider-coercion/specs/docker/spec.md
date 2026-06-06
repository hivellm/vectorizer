# Spec: Docker image bundles a dense provider

## ADDED Requirements

### Requirement: Official image ships a working dense provider out of the box

The official `hivehub/vectorizer` Docker image MUST be built with
the `fastembed` feature enabled and MUST pre-fetch the bundled
dense model so the first boot does not require an external
download.

The bundled model selection (e.g. `all-MiniLM-L6-v2` at 384-dim)
MUST be recorded in `docs/embedding/providers.md` and pinned by
checksum in the `Dockerfile`.

The image MUST register the dense provider as the default at
startup so `POST /collections` and `POST /embed` without explicit
`embedding_provider` / `model` use the dense path.

`BM25` MUST remain registered as the sparse provider so hybrid
retrieval (`dense + bm25`) stays available.

#### Scenario: Fresh container can run semantic search

Given a fresh `hivehub/vectorizer:3.4.0` container started with
   no extra configuration
When the client creates a collection without specifying
   `embedding_provider`
Then the collection's `embedding_provider` is the bundled dense
   provider (not `bm25`)
And the collection's `dimension` matches the bundled provider

#### Scenario: Hybrid search still works

Given a fresh `hivehub/vectorizer:3.4.0` container
When the client creates two collections — one dense, one with
   `embedding_provider: "bm25"`
Then both creates succeed
And dense vector search on the first returns semantically-similar
   paraphrase results
And BM25 search on the second returns lexical-match results

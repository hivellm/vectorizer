# Spec: embedding-provider contract

## ADDED Requirements

### Requirement: `embedding_provider` on create-collection is honoured or explicitly rejected

The system MUST resolve the `embedding_provider` field on
`POST /collections` against its provider registry. Silent
fall-through to a different provider (e.g. BM25) is FORBIDDEN.

If the requested provider is registered and ready, the collection
MUST be created with that provider and the `GET /collections/{name}`
response MUST report the same `embedding_provider` value.

If the requested provider is not registered, the server MUST
respond `400 Bad Request` with body:

```json
{
  "error": "unsupported_provider",
  "requested": "<value>",
  "available": ["<provider-1>", "<provider-2>", ...]
}
```

If the caller supplies a `dimension` that differs from the
provider's native dimension, the server MUST respond `400 Bad
Request` with `{ "error": "dimension_mismatch", "provider": "...",
"provider_dimension": N, "requested_dimension": M }`.

#### Scenario: Requested provider is registered

Given the server is started with `fastembed` registered at 384-dim
When the client posts `POST /collections {name: "c1", dimension:
   384, embedding_provider: "fastembed"}`
Then the server responds `201 Created`
And `GET /collections/c1` returns `embedding_provider: "fastembed",
   dimension: 384`

#### Scenario: Requested provider is not registered

Given the server is started with only `bm25` registered
When the client posts `POST /collections {name: "c2", dimension:
   768, embedding_provider: "fastembed"}`
Then the server responds `400 Bad Request`
And the body is `{ "error": "unsupported_provider", "requested":
   "fastembed", "available": ["bm25"] }`
And `GET /collections/c2` returns 404 (collection was not created)

#### Scenario: Dimension mismatch is rejected

Given the server is started with `fastembed` registered at 384-dim
When the client posts `POST /collections {name: "c3", dimension:
   768, embedding_provider: "fastembed"}`
Then the server responds `400 Bad Request`
And the body is `{ "error": "dimension_mismatch", "provider":
   "fastembed", "provider_dimension": 384, "requested_dimension":
   768 }`

### Requirement: `/embed` honours `model` parameter

The `POST /embed` endpoint MUST resolve the `model` field against
the provider registry. Silent fall-through to a different model is
FORBIDDEN.

When `model` is omitted, the default provider MUST be used and the
default MUST be deterministic across restarts (config-driven or
first-registered, decided in design.md).

#### Scenario: Requested model is registered

Given the server is started with `fastembed` (384-dim) registered
When the client posts `POST /embed {text: "hello", model:
   "fastembed"}`
Then the server responds `200 OK`
And the response body has `dimension: 384`
And the response body has `model: "fastembed"`

#### Scenario: Requested model is not registered

Given the server is started with only `bm25` registered
When the client posts `POST /embed {text: "hello", model:
   "nomic-embed-text-v1.5"}`
Then the server responds `400 Bad Request`
And the body is `{ "error": "unsupported_model", "requested":
   "nomic-embed-text-v1.5", "available": ["bm25"] }`

### Requirement: Provider discovery

The system MUST expose a discovery endpoint that lists every
registered embedding provider with its native dimension and a flag
indicating which one is the default. The endpoint MAY be
`GET /providers` or an extension of `GET /stats` (the design.md
decides which).

#### Scenario: Caller discovers available providers

Given the server is started with `fastembed` (384-dim, default)
   and `bm25` (512-dim) registered
When the client calls the discovery endpoint
Then the response lists both providers with their dimensions
And `fastembed` is flagged `default: true`

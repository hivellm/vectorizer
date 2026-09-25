---
title: Embedding Providers Guide
module: guides
id: embeddings
order: 2
description: Guide to embedding providers in Vectorizer
tags: [embeddings, vectors, fastembed, bert, minilm, bm25, tfidf]
---

# Embedding Providers Guide

Complete guide to embedding providers available in Vectorizer for converting text to vectors.

## Overview

Vectorizer supports multiple embedding providers for different use cases:

| Provider | Type | Dimensions | Status | Recommended For |
|----------|------|------------|--------|-----------------|
| FastEmbed | Dense | 384-1024 | **Production Ready** | All production use cases |
| BM25 | Sparse | Configurable | Production Ready | Keyword matching, hybrid search |
| TF-IDF | Sparse | Configurable | Production Ready | Simple keyword matching |
| SVD | Dense | Configurable | Production Ready | Dimensionality reduction |
| BERT | Dense | 768 | Experimental | Testing only |
| MiniLM | Dense | 384 | Experimental | Testing only |

## Discovering what is registered

Starting with v3.4.0 ([phase33](../../../CHANGELOG.md), issue
[#306](https://github.com/hivellm/vectorizer/issues/306)), every
running server advertises its registered embedding providers via
`GET /stats`:

```bash
curl -s http://localhost:15002/stats | jq '.providers, .default_provider'
```

```json
[
  { "name": "bm25",                            "dimension": 512, "default": true  },
  { "name": "fastembed:multilingual-e5-small", "dimension": 384, "default": false }
]
"bm25"
```

Provider names are exactly what the server registered: `bm25`, the
`embedding.model` value (e.g. `fastembed:all-MiniLM-L6-v2`) and every
`embedding.additional_models` entry (see
[Enabling FastEmbed](#enabling-fastembed)). If the provider you intend
to post to `POST /collections` is not in this list, it is not in the
config or the deployment was built without the matching Cargo feature
(`fastembed`). Posting an unknown provider returns
`400 unsupported_provider { requested, available }` — the 3.3.0
silent-coercion-to-bm25 behaviour is gone.

## Contract: `POST /collections` and `POST /embed`

### Honouring `embedding_provider`

`POST /collections` honours the `embedding_provider` field. The
field is optional and defaults to the server's configured default
when omitted. The server stores the resolved provider on
`CollectionConfig.embedding_provider` so reload is lossless.

**Success — provider is registered:**

```bash
curl -X POST http://localhost:15002/collections \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "denseprobe",
    "dimension": 384,
    "embedding_provider": "fastembed:multilingual-e5-small"
  }'
# 201 Created
# GET /collections/denseprobe → { dimension: 384, embedding_provider: "fastembed:multilingual-e5-small", ... }
```

`dimension` is also optional (3.8.2+): when omitted, it defaults to the
resolved provider's native width (e.g. 384 for `fastembed:multilingual-e5-small`)
instead of always defaulting to 512.

**Error — provider is not registered:**

```json
{
  "error_type": "unsupported_provider",
  "message": "Unsupported embedding provider 'fastembed:multilingual-e5-small'; available: bm25",
  "details": {
    "requested": "fastembed:multilingual-e5-small",
    "available": ["bm25"]
  },
  "status_code": 400
}
```

**Error — caller-supplied `dimension` conflicts with provider's
native dimension:**

```json
{
  "error_type": "provider_dimension_mismatch",
  "message": "Provider 'fastembed:multilingual-e5-small' has dimension 384, request asked for 768",
  "details": {
    "provider": "fastembed:multilingual-e5-small",
    "provider_dimension": 384,
    "requested_dimension": 768
  },
  "status_code": 400
}
```

### Honouring `model` on `POST /embed`

`POST /embed` honours the `model` field — no more silent routing
through the default provider. When `model` is omitted, the default
provider is used; the response echoes the resolved `model` so
callers can confirm.

```bash
curl -X POST http://localhost:15002/embed \
  -H 'Content-Type: application/json' \
  -d '{ "text": "hello", "model": "fastembed:multilingual-e5-small" }'
# 200 OK
# { "embedding": [...], "text": "hello", "dimension": 384, "model": "fastembed:multilingual-e5-small" }
```

```bash
curl -X POST http://localhost:15002/embed \
  -H 'Content-Type: application/json' \
  -d '{ "text": "hello", "model": "nomic-embed-text-v1.5" }'
# 400 Bad Request
# {
#   "error_type": "unsupported_model",
#   "message": "Unsupported embedding model 'nomic-embed-text-v1.5'; available: bm25",
#   "details": { "requested": "...", "available": ["bm25"] },
#   "status_code": 400
# }
```

### Migration from 3.3.0

| 3.3.0 behaviour | 3.4.0 behaviour |
|---|---|
| `POST /collections { embedding_provider: "fastembed" }` returns `201`, reads back as `bm25/512` | Returns `400 unsupported_provider` when the build lacks fastembed; succeeds with `fastembed/384` when it has it |
| `POST /embed { model: "bge-small" }` returns 512-dim BM25 vector regardless of `model` | Returns the requested model's vector OR `400 unsupported_model` |
| No way to discover what providers exist | `GET /stats` lists `providers[]` + `default_provider` |
| `CollectionConfig` has no `embedding_provider` field | New `embedding_provider: String` field — old `.vecdb` files default to `"bm25"` |

## Production Embedding: FastEmbed

**FastEmbed is the recommended embedding provider for production use.**

### Enabling FastEmbed

The `fastembed` Cargo feature is on by default for source builds
(`cargo build --release`); the published Docker image is BM25-only
unless built as the fastembed variant (see the Docker Hub README).

Models are configured in the top-level `embedding:` section of
`config.yml`:

```yaml
embedding:
  # Server default provider: "bm25" (default) or "fastembed:<model-id>".
  # Collections created without `embedding_provider` use it.
  model: "bm25"
  # Extra providers registered next to the default (3.8+), so individual
  # collections can opt into them. Each model is loaded once at boot and
  # downloaded to <data_dir>/fastembed on first use, or to
  # $VECTORIZER_FASTEMBED_CACHE_DIR when that env var is set — the
  # published `-fastembed` image sets it to bundle a model outside `/data`.
  additional_models:
    - "fastembed:multilingual-e5-small"
```

Every provider is registered under its full `fastembed:<model-id>`
name, and `bm25` is always registered as well. An unknown id or prefix
fails boot instead of silently falling back to BM25.

### Supported Models

| Model id (`fastembed:<id>`) | Dimensions | Languages | Use Case |
|-------|------------|-----------|----------|
| `all-MiniLM-L6-v2` | 384 | English | General purpose, fast |
| `all-MiniLM-L12-v2` | 384 | English | General purpose, balanced |
| `all-mpnet-base-v2` | 768 | English | General purpose, higher quality |
| `bge-small-en-v1.5` | 384 | English | English text, high quality |
| `bge-base-en-v1.5` | 768 | English | English text, higher quality |
| `bge-large-en-v1.5` | 1024 | English | English text, highest quality |
| `multilingual-e5-small` | 384 | 100+ (incl. Portuguese) | Multilingual semantic search |
| `multilingual-e5-base` | 768 | 100+ | Multilingual, higher quality |
| `multilingual-e5-large` | 1024 | 100+ | Multilingual, highest quality |
| `paraphrase-multilingual-MiniLM-L12-v2` | 384 | 50+ | Multilingual paraphrase / similarity |

The MiniLM, BGE and `paraphrase-multilingual-MiniLM-L12-v2` ids also
exist with a `-q` suffix (quantized ONNX, same dimension). The enum
names fastembed uses (e.g. `MultilingualE5Small`) are accepted as
aliases and normalized to the short id.

### Per-collection models

Since 3.8 a text insert or text search embeds with the
**collection's** `embedding_provider` when that provider is registered
on the server and produces vectors of the collection's dimension.
Otherwise it falls back to the server default, which is what every
text operation used before 3.8 — e.g. a collection whose provider is
not configured on this server, or a legacy collection whose provider
was back-filled as `bm25` although its vectors have another width.
A collection created without `embedding_provider` stores the default
provider's name, so it keeps embedding exactly as before. Raw-vector
collections (`embedding_provider: "none"`) still reject text
operations.

This lets one server keep its BM25 (512-dim) collections and host
multilingual ones side by side:

```bash
curl -X POST http://localhost:15002/collections \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "suporte_pt",
    "dimension": 384,
    "embedding_provider": "fastembed:multilingual-e5-small"
  }'

curl -X POST http://localhost:15002/insert \
  -H 'Content-Type: application/json' \
  -d '{ "collection": "suporte_pt", "text": "Como cancelo meu pedido?" }'

curl -X POST http://localhost:15002/collections/suporte_pt/search/text \
  -H 'Content-Type: application/json' \
  -d '{ "query": "desistir da compra", "limit": 5 }'
```

`dimension` must equal the model's dimension (384 for
`multilingual-e5-small`); a mismatch returns
`400 provider_dimension_mismatch`.

### Query and passage prefixes (multilingual E5)

The `multilingual-e5-*` models were trained with every input prefixed
by `"query: "` or `"passage: "`, and fastembed does not add them.
Vectorizer does, for those three models only: text being inserted is
embedded as `"passage: <text>"`, and the query of a text search as
`"query: <text>"`. Text that already starts with either prefix is left
as is, so clients that follow the model card are not double-prefixed.
`POST /embed` embeds its input as a passage. Other models embed text
unchanged.

### Changing a collection's model

Vectors from different models live in different spaces, and a
collection's `embedding_provider` is fixed at creation. To move a
collection to another model (e.g. from `bm25` to
`fastembed:multilingual-e5-small`), create a new collection with the
new provider and dimension and re-insert the source texts; switching
the server's `embedding.model` does not re-embed stored vectors.

### Usage via API

```bash
# Embed text with a specific registered provider
curl -X POST "http://localhost:15002/embed" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Your text to embed",
    "model": "fastembed:multilingual-e5-small"
  }'
```

## Sparse Embedding: BM25

BM25 (Best Matching 25) provides sparse embeddings optimized for keyword matching.

### Features

- Vocabulary-based sparse vectors
- TF-IDF weighting with document length normalization
- Ideal for exact keyword matching
- Complements dense embeddings in hybrid search

### Configuration

```yaml
embedding:
  bm25:
    dimension: 30000
    k1: 1.5
    b: 0.75
```

### Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `dimension` | 30000 | Vocabulary size |
| `k1` | 1.5 | Term frequency saturation |
| `b` | 0.75 | Document length normalization |

### Usage

```rust
use vectorizer::embedding::Bm25Embedding;

let bm25 = Bm25Embedding::new(30000);
bm25.fit(&documents)?;
let sparse_vector = bm25.embed("search query")?;
```

## Sparse Embedding: TF-IDF

TF-IDF (Term Frequency-Inverse Document Frequency) provides simple sparse embeddings.

### Features

- Vocabulary-based sparse vectors
- Simple TF-IDF weighting
- Lower memory than BM25
- Good for simple keyword matching

### Configuration

```yaml
embedding:
  tfidf:
    dimension: 10000
```

## Dense Embedding: SVD

SVD (Singular Value Decomposition) provides dimensionality reduction for TF-IDF embeddings.

### Features

- Reduces TF-IDF dimensions to dense vectors
- Captures latent semantic relationships
- Configurable output dimensions

### Configuration

```yaml
embedding:
  svd:
    input_dimension: 10000
    output_dimension: 256
```

## Experimental Providers

> **Warning**: The following providers use placeholder implementations and are NOT suitable for production use. Use FastEmbed for production deployments.

### BERT Embedding (Experimental)

BERT embedding is available as an experimental provider for testing purposes.

**Current Status**: Uses hash-based simulation as placeholder. Real BERT inference is not implemented.

```rust
use vectorizer::embedding::BertEmbedding;

// Creates a placeholder BERT provider
let bert = BertEmbedding::new(768);
bert.load_model()?; // Uses hash-based placeholder

// Embeddings are NOT semantically meaningful
let embedding = bert.embed("text")?;
```

**Limitations**:
- Does not use actual BERT model inference
- Produces hash-based embeddings (not semantically meaningful)
- Included only for API compatibility testing

### MiniLM Embedding (Experimental)

MiniLM embedding is available as an experimental provider for testing purposes.

**Current Status**: Uses hash-based simulation as placeholder. Real MiniLM inference is not implemented.

```rust
use vectorizer::embedding::MiniLmEmbedding;

// Creates a placeholder MiniLM provider
let minilm = MiniLmEmbedding::new(384);
minilm.load_model()?; // Uses hash-based placeholder

// Embeddings are NOT semantically meaningful
let embedding = minilm.embed("text")?;
```

**Limitations**:
- Does not use actual MiniLM model inference
- Produces hash-based embeddings (not semantically meaningful)
- Included only for API compatibility testing

### Real Model Implementation (Feature-Gated)

**NEW in v2.0.0**: BERT and MiniLM now support real model inference via the `real-models` feature flag!

#### Using Real Models

Build with the `real-models` feature to enable actual BERT/MiniLM inference:

```bash
cargo build --release --features real-models
```

```rust
use vectorizer::embedding::BertEmbedding;

// Load real BERT model from HuggingFace
let mut bert = BertEmbedding::new(768);
bert.load_model_with_id("bert-base-uncased", false)?; // false = CPU, true = GPU

// Real semantic embeddings!
let embedding = bert.embed("This is a test sentence")?;
```

```rust
use vectorizer::embedding::MiniLmEmbedding;

// Load real MiniLM model from HuggingFace
let mut minilm = MiniLmEmbedding::new(384);
minilm.load_model_with_id("sentence-transformers/all-MiniLM-L6-v2", false)?;

// Real semantic embeddings with mean pooling!
let embedding = minilm.embed("This is a test sentence")?;
```

**Features:**
- ✅ Real model loading from HuggingFace Hub
- ✅ Automatic model download and caching
- ✅ CPU and GPU (CUDA) support
- ✅ BERT: [CLS] token embedding extraction (768 dimensions)
- ✅ MiniLM: Mean pooling with attention mask (384 dimensions)
- ✅ SafeTensors and PyTorch weights support
- ✅ Fallback to placeholders when feature not enabled

#### Implementation Details

**BERT Implementation:**
- Uses Candle framework for inference
- Extracts [CLS] token embedding
- Default model: `bert-base-uncased` (768 dimensions)
- Supports any BERT-compatible model from HuggingFace

**MiniLM Implementation:**
- Uses Candle framework for inference
- Mean pooling over all token embeddings
- Attention mask weighting for quality
- Default model: `sentence-transformers/all-MiniLM-L6-v2` (384 dimensions)

### Placeholder Mode (Default)

Without the `real-models` feature, BERT and MiniLM use hash-based placeholders:

**Why Placeholders?**

1. **Dependency Size**: Full ML inference (candle, ort, onnxruntime) adds significant binary size (~100MB+ per model)
2. **FastEmbed Alternative**: The `fastembed` feature provides production-ready MiniLM and other models with optimized inference
3. **API Compatibility**: Allows testing embedding provider switching without full ML dependencies
4. **Lightweight Testing**: Useful for development/testing where semantic quality isn't critical

**Important Notes:**
- **NOT Semantic**: Placeholder embeddings are NOT semantically meaningful
- **Deterministic**: Hash-based embeddings are deterministic (same text = same embedding)
- **Testing Only**: Use only for API compatibility testing, not for real semantic search

### Recommendations

**For Production:**
1. **Best Choice**: Use `fastembed` feature (optimized, lightweight, production-ready)
2. **Alternative**: Use `real-models` feature for BERT/MiniLM if needed
3. **Cloud Option**: Use OpenAI embeddings API

**For Development/Testing:**
- Use placeholder mode (default) for fast iteration without model downloads

## Hybrid Search

Combine dense and sparse embeddings for best results:

```yaml
search:
  hybrid:
    enabled: true
    dense_weight: 0.7    # FastEmbed weight
    sparse_weight: 0.3   # BM25 weight
    fusion: "rrf"        # Reciprocal Rank Fusion
```

See [Discovery Guide](../api/DISCOVERY.md) for hybrid search details.

## Embedding Manager

The `EmbeddingManager` provides a unified interface for all providers:

```rust
use vectorizer::embedding::EmbeddingManager;

let mut manager = EmbeddingManager::new();

// Register providers (the first one registered becomes the default)
manager.register_provider("bm25".to_string(), Box::new(bm25_provider));
manager.register_provider("fastembed:multilingual-e5-small".to_string(), fastembed_provider);

// Documents vs queries: `embed*` embeds text being indexed, `embed_query*`
// embeds a search query (E5 models get their "passage: " / "query: " prefix)
let doc = manager.embed_with_provider("fastembed:multilingual-e5-small", "document text")?;
let query = manager.embed_query_with_provider("fastembed:multilingual-e5-small", "query text")?;
let sparse = manager.embed_with_provider("bm25", "query text")?;
```

## Performance Tips

### 1. Batch Embedding

Always embed in batches for better performance:

```rust
let texts: Vec<&str> = documents.iter().map(|d| d.as_str()).collect();
let embeddings = provider.embed_batch(&texts)?;
```

### 2. Model Selection

Choose models based on your needs:

| Priority | Model | Why |
|----------|-------|-----|
| Speed | `all-MiniLM-L6-v2` | Fastest, good quality |
| Quality | `bge-base-en-v1.5` | Best English quality |
| Multilingual | `multilingual-e5-small` | 100+ languages incl. Portuguese, 384 dims |

## Related Documentation

- [Discovery Guide](../api/DISCOVERY.md) - Hybrid search and retrieval
- [Quantization Guide](./QUANTIZATION.md) - Vector compression
- [Sparse Vectors Guide](./SPARSE_VECTORS.md) - Sparse vector details
- [API Reference](../api/API_REFERENCE.md) - Embedding API endpoints

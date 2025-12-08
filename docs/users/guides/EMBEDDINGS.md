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

## Production Embedding: FastEmbed

**FastEmbed is the recommended embedding provider for production use.**

### Enabling FastEmbed

Build with the `fastembed` feature (enabled by default):

```bash
cargo build --release --features fastembed
```

### Supported Models

FastEmbed supports multiple pre-trained models:

| Model | Dimensions | Use Case |
|-------|------------|----------|
| `all-MiniLM-L6-v2` | 384 | General purpose, fast |
| `all-MiniLM-L12-v2` | 384 | General purpose, balanced |
| `bge-small-en-v1.5` | 384 | English text, high quality |
| `bge-base-en-v1.5` | 768 | English text, highest quality |
| `multilingual-e5-small` | 384 | Multilingual support |

### Configuration

```yaml
embedding:
  provider: "fastembed"
  model: "all-MiniLM-L6-v2"
  cache_embeddings: true
  batch_size: 32
```

### Usage via API

```bash
# Generate embeddings
curl -X POST "http://localhost:15002/api/v1/embed" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Your text to embed",
    "provider": "fastembed",
    "model": "all-MiniLM-L6-v2"
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

let manager = EmbeddingManager::new();

// Add providers
manager.add_provider("fastembed", fastembed_provider)?;
manager.add_provider("bm25", bm25_provider)?;

// Generate embeddings
let dense = manager.embed("fastembed", "query text")?;
let sparse = manager.embed("bm25", "query text")?;
```

## Performance Tips

### 1. Batch Embedding

Always embed in batches for better performance:

```rust
let texts: Vec<&str> = documents.iter().map(|d| d.as_str()).collect();
let embeddings = provider.embed_batch(&texts)?;
```

### 2. Caching

Enable embedding cache to avoid re-computing:

```yaml
embedding:
  cache_embeddings: true
  cache_size: 10000
```

### 3. Model Selection

Choose models based on your needs:

| Priority | Model | Why |
|----------|-------|-----|
| Speed | `all-MiniLM-L6-v2` | Fastest, good quality |
| Quality | `bge-base-en-v1.5` | Best English quality |
| Multilingual | `multilingual-e5-small` | Multiple languages |

## Related Documentation

- [Discovery Guide](../api/DISCOVERY.md) - Hybrid search and retrieval
- [Quantization Guide](./QUANTIZATION.md) - Vector compression
- [Sparse Vectors Guide](./SPARSE_VECTORS.md) - Sparse vector details
- [API Reference](../api/API_REFERENCE.md) - Embedding API endpoints

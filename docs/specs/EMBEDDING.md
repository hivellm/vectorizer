# Embedding System

**Version**: 0.7.0  
**Status**: ✅ Production Ready  
**Last Updated**: 2025-09-25

---

## Overview

Vectorizer implements a comprehensive embedding system supporting multiple state-of-the-art retrieval methods, from traditional sparse embeddings to modern dense representations.

---

## Embedding Methods

### Sparse Embeddings

**TF-IDF (Term Frequency-Inverse Document Frequency)**
- Traditional baseline for text vectorization
- Variable dimensionality based on vocabulary
- Efficient for exact keyword matching
- Memory-efficient for large vocabularies

**BM25 (Best Matching 25)**
- Advanced sparse retrieval with probabilistic ranking
- Parameters: k1=1.5, b=0.75
- Document length normalization
- Superior relevance ranking vs TF-IDF

### Reduced Embeddings

**SVD-Reduced (TF-IDF + SVD)**
- Dimensionality reduction of TF-IDF vectors
- Supported dimensions: 300D and 768D
- Orthogonal transformation preserving semantics
- Memory-efficient dense representations

### Dense Embeddings

**BERT (768D)**
- Contextual embeddings capturing semantic meaning
- 768-dimensional vectors (BERT-base compatible)
- Placeholder for real BERT integration
- Deterministic hashing fallback

**MiniLM (384D)**
- Efficient sentence embeddings
- 384-dimensional vectors (MiniLM-L6-v2 compatible)
- Fast inference with good semantic quality
- Placeholder for real model integration

**FastEmbed (ONNX, production dense models)** — see
[FastEmbed Model Matrix](#fastembed-model-matrix).

---

## FastEmbed Model Matrix

Compiled in by the `fastembed` Cargo feature (on by default for source
builds). Configured in the top-level `embedding:` section of
`config.yml`:

| Key | Meaning |
|-----|---------|
| `embedding.model` | Server default provider: `bm25` (default) or `fastembed:<id>`. Used by collections created without `embedding_provider`. |
| `embedding.additional_models` | List of extra `fastembed:<id>` providers registered next to the default, each loaded once at boot. `bm25` and the default are ignored if listed. |

`bm25` (512 dims) is always registered. Every fastembed provider is
registered as `fastembed:<id>` with `<id>` exactly as written in the
config. An unknown id or prefix fails boot. Weights are cached under
`<data_dir>/fastembed` (Hugging Face cache layout; `HF_HOME` overrides
it) and downloaded on first use.

| `<id>` | Hugging Face repo loaded by fastembed | Dims | E5 prefixes |
|--------|---------------------------------------|------|-------------|
| `all-MiniLM-L6-v2` | `Qdrant/all-MiniLM-L6-v2-onnx` | 384 | no |
| `all-MiniLM-L6-v2-q` | `Xenova/all-MiniLM-L6-v2` | 384 | no |
| `all-MiniLM-L12-v2` | `Xenova/all-MiniLM-L12-v2` | 384 | no |
| `all-MiniLM-L12-v2-q` | `Xenova/all-MiniLM-L12-v2` | 384 | no |
| `all-mpnet-base-v2` | `Xenova/all-mpnet-base-v2` | 768 | no |
| `bge-small-en-v1.5` | `Xenova/bge-small-en-v1.5` | 384 | no |
| `bge-base-en-v1.5` / `-q` | `Xenova/bge-base-en-v1.5` / `Qdrant/bge-base-en-v1.5-onnx-Q` | 768 | no |
| `bge-large-en-v1.5` / `-q` | `Xenova/bge-large-en-v1.5` / `Qdrant/bge-large-en-v1.5-onnx-Q` | 1024 | no |
| `multilingual-e5-small` | `intfloat/multilingual-e5-small` | 384 | yes |
| `multilingual-e5-base` | `intfloat/multilingual-e5-base` | 768 | yes |
| `multilingual-e5-large` | `Qdrant/multilingual-e5-large-onnx` | 1024 | yes |
| `paraphrase-multilingual-MiniLM-L12-v2` | `Xenova/paraphrase-multilingual-MiniLM-L12-v2` | 384 | no |
| `paraphrase-multilingual-MiniLM-L12-v2-q` | `Qdrant/paraphrase-multilingual-MiniLM-L12-v2-onnx-Q` | 384 | no |

The fastembed enum names (`MultilingualE5Small`, `BGESmallENV15`, …)
are accepted as aliases of the ids above.

### Per-collection provider resolution

Text inserts and text searches (REST, RPC, MCP, GraphQL upload,
discovery, intelligent search) embed with the collection's
`embedding_provider` when it is registered and its dimension equals the
collection's dimension; otherwise with the server default (the pre-3.8
behavior for every collection). Collections with the raw-vector
sentinel `embedding_provider: "none"` reject text operations before any
provider is resolved. `POST /collections` accepts any registered
provider name and checks `dimension` against that provider.

### Query vs. passage embedding

`EmbeddingProvider` distinguishes documents (`embed`, `embed_batch`)
from search queries (`embed_query`, `embed_query_batch`; the default
implementation is the document path). Insert/index paths use the
document methods, search paths the query methods. For the
`multilingual-e5-*` models the fastembed provider prepends
`"passage: "` to documents and `"query: "` to queries, as the models
were trained; text already starting with either prefix is not
re-prefixed. All other models embed text unchanged.

### Changing a collection's model

Stored vectors are never re-embedded. Moving a collection to another
model means creating a new collection with the new provider and
dimension and re-inserting the source texts.

---

## Hybrid Search Pipeline

Two-stage retrieval combining sparse and dense methods:

**Stage 1 - Sparse Retrieval**:
- Efficient candidate selection (BM25/TF-IDF)
- Top-k retrieval (default k=50)
- Fast ranking based on term matching

**Stage 2 - Dense Re-ranking**:
- Semantic re-ranking of candidates
- Improved relevance using contextual meaning
- Configurable pipeline combinations

**Supported Combinations**:
- BM25 + BERT Re-ranking
- BM25 + MiniLM Re-ranking
- TF-IDF+SVD + BERT Re-ranking

---

## Persistence & Robustness

### .vectorizer Directory Structure

```
project/
├── .vectorizer/
│   ├── cache.bin                    # Document processing cache
│   ├── tokenizer.bm25.json          # BM25 vocabulary & statistics
│   ├── tokenizer.tfidf.json         # TF-IDF vocabulary & weights
│   ├── tokenizer.bow.json           # BagOfWords vocabulary
│   └── tokenizer.charngram.json     # CharNGram N-gram mappings
```

### Tokenizer Persistence

**BM25 Tokenizer**:
```json
{
  "vocabulary": {"term1": 0, "term2": 1},
  "doc_freq": {"term1": 5, "term2": 3},
  "avg_doc_length": 42.3,
  "total_docs": 150,
  "k1": 1.5,
  "b": 0.75
}
```

### Deterministic Fallbacks

All providers guarantee non-zero, normalized vectors:

**Feature Hashing (OOV Handling)**:
```rust
fn hash_term(term: &str) -> usize {
    xxhash(term.as_bytes()) % DIMENSION
}
```

**Hash-based Fallback**:
```rust
fn fallback_hash_embedding(&self, text: &str) -> Vec<f32> {
    let hash = xxhash(text.as_bytes());
    let mut vector = vec![0.0; DIMENSION];
    
    for i in 0..DIMENSION {
        vector[i] = ((hash >> (i % 32)) & 1) as f32 * 0.1;
    }
    
    normalize_l2(&mut vector);
    vector
}
```

---

## Evaluation Framework

### Information Retrieval Metrics

- **MRR (Mean Reciprocal Rank)**: Average reciprocal rank of first relevant document
- **MAP (Mean Average Precision)**: Average precision across all relevant documents
- **Precision@K**: Fraction of relevant documents in top-K
- **Recall@K**: Fraction of relevant documents retrieved in top-K

### Benchmark Suite

| Method | Type | Dimensions | Use Case |
|--------|------|------------|----------|
| TF-IDF | Sparse | Variable | Baseline, exact matching |
| BM25 | Sparse | Variable | Relevance ranking |
| TF-IDF+SVD(300D) | Reduced | 300D | Memory-efficient |
| TF-IDF+SVD(768D) | Reduced | 768D | BERT-compatible |
| BERT | Dense | 768D | Semantic understanding |
| MiniLM | Dense | 384D | Fast semantic search |
| BM25+BERT | Hybrid | 768D | Best relevance |
| BM25+MiniLM | Hybrid | 384D | Balanced performance |

---

## Quality Guarantees

✅ **100% Non-zero**: All embeddings return valid vectors  
✅ **Consistent Dimensions**: Always 512D, L2-normalized  
✅ **Deterministic**: Same input → same output  
✅ **Persistent**: Survives server restarts  
✅ **Robust**: Handles OOV terms gracefully

---

**Version**: 0.7.0  
**Status**: ✅ Production Ready  
**Maintained by**: HiveLLM Team


# Qdrant Advanced Features Proposal

## Why

Qdrant provides advanced features like sparse vectors, hybrid search, quantization, payload indexing, and geo-location filtering that are essential for complete compatibility.

## What Changes

- **ADDED**: Sparse vector support and indexing
- **ADDED**: Hybrid search (dense + sparse vectors)
- **ADDED**: Advanced quantization (scalar, product, binary)
- **ADDED**: Payload indexing and optimization
- **ADDED**: Geo-location filtering and indexing
- **ADDED**: Advanced storage options (on-disk, mmap)

## Impact

- Affected specs: `vector-database`, `search`, `collections`
- Affected code: `src/search/`, `src/db/`, `src/quantization/`
- Breaking changes: None (additive only)
- Migration: Existing Vectorizer functionality remains unchanged

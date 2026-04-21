# Qdrant Search & Query Proposal

## Why

Qdrant provides advanced search capabilities including filtering, scoring functions, and query optimization that need to be implemented for full compatibility.

## What Changes

- **ADDED**: Qdrant search API with filtering support
- **ADDED**: Qdrant scoring functions (cosine, dot product, euclidean)
- **ADDED**: Qdrant query planning and optimization
- **ADDED**: Qdrant scroll, recommend, and count APIs

## Impact

- Affected specs: `search`, `vector-database`
- Affected code: `src/search/`, `src/db/`
- Breaking changes: None (additive only)
- Migration: Existing Vectorizer search remains unchanged

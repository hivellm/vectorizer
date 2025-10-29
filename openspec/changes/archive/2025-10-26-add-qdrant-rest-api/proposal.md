# Qdrant REST API Compatibility Proposal

## Why

Vectorizer needs to implement Qdrant REST API compatibility to enable seamless migration from Qdrant-based applications. This is the foundation for all other Qdrant compatibility features.

## What Changes

- **ADDED**: Complete Qdrant REST API v1.14.x compatibility
- **ADDED**: Qdrant request/response models and validation
- **ADDED**: Qdrant error response format compatibility
- **ADDED**: Qdrant HTTP status codes and headers
- **ADDED**: Qdrant API routing and middleware

## Impact

- Affected specs: `api-rest`, `vector-database`
- Affected code: `src/server/`, `src/models/`, `src/api/`
- Breaking changes: None (additive only)
- Migration: Existing Vectorizer API remains unchanged

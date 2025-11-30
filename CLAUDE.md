# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vectorizer is a high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications. It provides sub-3ms search times with HNSW indexing, supports multiple embedding models, and offers full Qdrant API compatibility.

## Critical Requirements

### Rust Edition 2024 - MANDATORY
- **Cargo.toml edition**: Must be `"2024"` - NEVER change to 2021 or earlier
- The codebase uses Edition 2024 features for advanced async patterns

### REST-First Architecture
When adding features, implement in this order:
1. Core engine first (business logic in `src/db/`, `src/embedding/`)
2. REST endpoints second (`src/api/`)
3. MCP tools third (`src/server/`)

**NEVER implement features only in MCP** - REST and MCP must have identical functionality.

## Build and Development Commands

```bash
# Build
cargo build --release

# Build with GPU acceleration (macOS Metal)
cargo build --release --features hive-gpu

# Build with all features
cargo build --release --features full

# Run server (starts REST on :15002 + MCP)
./target/release/vectorizer
cargo run

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run tests in a specific module
cargo test module_name::

# Run tests with output
cargo test -- --nocapture

# Format and lint
cargo fmt
cargo clippy

# Stop server
pkill vectorizer  # or Ctrl+C
```

## Architecture

```
Client → REST/MCP → Core Engine → Vector Store
```

### Key Source Directories

- `src/api/` - HTTP endpoints (Axum-based REST API)
- `src/db/` - Core database operations (VectorStore, Collection, HNSW indexing)
- `src/models/` - Data models (Vector, CollectionConfig, SearchResult)
- `src/embedding/` - Embedding providers (BM25, BERT, MiniLM, TF-IDF)
- `src/server/` - MCP server implementation
- `src/grpc/` - gRPC service (Qdrant-compatible)
- `src/quantization/` - Vector quantization (Scalar, Product Quantization)
- `src/persistence/` - Storage (MMap, .vecdb format)
- `src/replication/` - Master-replica replication (BETA)
- `src/cluster/` - Distributed sharding (BETA)
- `src/auth/` - JWT + API Key authentication with RBAC
- `src/cache/` - Query caching
- `src/discovery/` - File discovery and indexing pipeline

### Test Organization

- `tests/` - Integration tests organized by feature
- `tests/api/rest/` - REST API integration tests
- `tests/api/mcp/` - MCP integration tests
- `tests/grpc/` - gRPC tests
- `tests/integration/` - Feature integration tests (sharding, clustering, etc.)
- `tests/replication/` - Replication tests
- Unit tests are colocated in `src/` modules

## Access Points (Default)

- **REST API / Dashboard**: http://localhost:15002
- **MCP Server**: ws://localhost:15002/mcp
- **GraphQL**: http://localhost:15002/graphql
- **Health Check**: http://localhost:15002/health

## Key Patterns

### Error Handling
Use `thiserror` for custom error types with `VectorizerError` as the main error type. Propagate errors with `?` operator.

### Concurrency
- `Arc<RwLock<T>>` for shared state
- `DashMap` for concurrent key-value operations
- `parking_lot` locks (not std::sync)

### Serialization
All API types derive `Serialize, Deserialize` from serde. Use `#[serde(rename_all = "lowercase")]` for enums.

### Async
Use `tokio` runtime. Never use blocking operations (`std::thread::sleep`) in async code - use `tokio::time::sleep`.

## Feature Flags

- `default = ["hive-gpu", "fastembed"]`
- `hive-gpu` - GPU acceleration (Metal on macOS)
- `fastembed` - Fast embedding models with ONNX
- `full` - All features including real-models, ONNX, Arrow, Parquet, Transmutation
- `s2s-tests` - Server-to-server tests (requires explicit enable)

## Configuration Files

- `config.yml` - Runtime configuration
- `workspace.yml` - Workspace/project definitions
- `config.example.yml` - Configuration reference

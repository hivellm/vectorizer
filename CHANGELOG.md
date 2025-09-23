# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-09-23

### Added
- Initial implementation of Vectorizer project
- Core vector database engine with thread-safe operations
- HNSW index integration for similarity search
- Basic CRUD operations (Create, Read, Update, Delete)
- Binary persistence with bincode
- Compression support with LZ4
- Collection management system
- Unit tests for all core components
- CI/CD pipeline with GitHub Actions
- Rust edition 2024 support (nightly)

### Technical Details
- Implemented `VectorStore` with DashMap for concurrent access
- Integrated `hnsw_rs` v0.3 for HNSW indexing
- Added support for multiple distance metrics (Cosine, Euclidean, Dot Product)
- Implemented basic persistence layer with save/load functionality
- Created modular architecture with separate modules for db, models, persistence
- Added comprehensive error handling with custom error types

### Project Structure
- Set up Rust project with Cargo workspace
- Organized code into logical modules
- Created documentation structure in `docs/` directory
- Added examples and benchmarks directories (to be populated)

### Dependencies
- tokio 1.40 - Async runtime
- axum 0.7 - Web framework (prepared for Phase 2)
- hnsw_rs 0.3 - HNSW index implementation
- dashmap 6.1 - Concurrent HashMap
- bincode 1.3 - Binary serialization
- lz4_flex 0.11 - Compression
- chrono 0.4 - Date/time handling
- serde 1.0 - Serialization framework

### Notes
- This is the Phase 1 (Foundation) implementation
- REST API and authentication will be added in Phase 2
- Client SDKs (Python, TypeScript) planned for Phase 4

[0.1.0]: https://github.com/hivellm/vectorizer/releases/tag/v0.1.0

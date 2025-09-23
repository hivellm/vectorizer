# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2025-09-23

### Fixed
- **Critical**: Fixed persistence search inconsistency - removed vector ordering that broke HNSW index consistency
- **Major**: Added comprehensive test demonstrating real embedding usage instead of manual vectors
- **Major**: Ensured search results remain consistent after save/load cycles
- **Tests**: Added `test_vector_database_with_real_embeddings` for end-to-end embedding validation

### Added
- **Documentation**: GPT_REVIEWS_ANALYSIS.md documenting GPT-5 and GPT-4 review findings and fixes
- **Tests**: Real embedding integration test with TF-IDF semantic search validation
- **Quality**: Persistence accuracy verification test

### Technical Details
- Removed alphabetical sorting in persistence to preserve HNSW insertion order
- Implemented embedding-first testing pattern for integration tests
- Added semantic search accuracy validation across persistence cycles
- Documented review process and implementation of recommendations

## [0.1.1] - 2025-09-23

### Added
- **Major**: Complete text embedding system with multiple providers
- **Major**: TF-IDF embedding provider for semantic search
- **Major**: Bag-of-Words embedding provider for classification
- **Major**: Character N-gram embedding provider for multilingual support
- **Major**: Embedding manager system for provider orchestration
- **Major**: Comprehensive semantic search capabilities
- **Major**: Real-world use cases (FAQ search, document clustering)
- **Documentation**: Reorganized documentation structure with phase-based folders

### Fixed
- **Critical**: Fixed persistence layer - `save()` method now correctly saves all vectors instead of placeholder
- **Critical**: Corrected distance metrics calculations for proper similarity search
- **Major**: Improved HNSW update operations with rebuild tracking
- **Major**: Added vector normalization for cosine similarity metric
- **Tests**: Fixed test assertions for normalized vectors

### Documentation
- **Reorganized**: Moved all technical docs to `/docs` folder with subfolders
- **Phase 1**: Architecture, technical implementation, configuration, performance, QA
- **Reviews**: Implementation reviews, embedding documentation, project status
- **Future**: API specs, dashboard, integrations, checklists, task tracking
- **Updated**: README.md and ROADMAP.md with current status
- **Added**: PROJECT_STATUS_SUMMARY.md overview

### Testing
- **Expanded**: Test coverage from 13 to 30+ tests
- **Added**: Integration tests for embedding workflows
- **Added**: Semantic search validation tests
- **Added**: Concurrency and persistence tests
- **Added**: Real-world use case demonstrations

### Technical Details
- Implemented proper vector iteration in persistence save method
- Added automatic vector normalization for cosine similarity
- Fixed distance-to-similarity conversions in HNSW search
- Added index rebuild tracking and statistics
- Created specialized tests for normalized vector persistence
- Implemented trait-based embedding provider system
- Added comprehensive embedding validation and error handling

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

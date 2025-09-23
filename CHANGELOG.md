# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Performance Optimizations - 2025-09-24)

#### Ultra-fast Tokenization
- Native Rust tokenizer integration with HuggingFace `tokenizers` crate
- Batch tokenization with truncation/padding support (32-128 tokens)
- In-memory token caching using xxHash for deduplication
- Reusable tokenizer instances with Arc/OnceCell pattern
- Expected throughput: ~50-150k tokens/sec on CPU

#### ONNX Runtime Integration  
- High-performance inference engine for production deployments
- CPU optimization with MKL/OpenMP backends
- INT8 quantization support (2-4x speedup with minimal quality loss)
- Batch inference for 32-128 documents
- Support for MiniLM, E5, MPNet model variants

#### Intelligent Parallelism
- Separate thread pools for embedding and indexing operations
- BLAS thread limiting (OMP_NUM_THREADS=1) to prevent oversubscription
- Bounded channel executors for backpressure management
- Configurable parallelism levels via config file

#### Persistent Embedding Cache
- Zero-copy loading with memory-mapped files
- Content-based hashing for incremental builds
- Sharded cache architecture for parallel access
- Binary format with optional compression
- Optional Arrow/Parquet support for analytics

#### Optimized HNSW Index
- Batch insertion with configurable sizes (100-1000 vectors)
- Pre-allocated memory for known dataset sizes
- Parallel graph construction support
- Adaptive ef_search based on index size
- Real-time memory usage statistics

#### Real Transformer Models (Candle)
- MiniLM Multilingual (384D) - Fast multilingual embeddings
- DistilUSE Multilingual (512D) - Balanced performance
- MPNet Multilingual Base (768D) - Higher accuracy
- E5 Models (384D/768D) - Optimized for retrieval
- GTE Multilingual Base (768D) - Alternative high-quality
- LaBSE (768D) - Language-agnostic embeddings

#### ONNX Production Models
- sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 (384D)
- intfloat/multilingual-e5-base (768D) 
- Alibaba-NLP/gte-multilingual-base (768D)
- Custom ONNX model support

#### Performance Benchmarks
Expected throughput on CPU (8c/16t):
- Tokenization: ~50-150k tokens/sec
- MiniLM-384D embedding: 2-6k docs/sec (short), 300-800 docs/sec (chunked)
- Optimized HNSW indexing: 10-50k vectors/sec
- TF-IDF/BM25: 50-200k docs/sec

### Changed
- Refactored feature flags: `real-models`, `onnx-models`, `candle-models`
- Updated benchmark suite to use optimized components
- Enhanced config.example.yml with performance tuning options

## [0.4.0] - 2025-09-23

### Added
- **SVD Dimensionality Reduction**: Implemented TF-IDF + SVD for reduced dimensional embeddings (300D/768D)
- **Dense Embeddings**: Added BERT and MiniLM embedding support with placeholder implementations
- **Hybrid Search Pipeline**: Implemented BM25/TF-IDF → dense re-ranking architecture
- **Extended Benchmark Suite**: Comprehensive comparison across TF-IDF, BM25, SVD, BERT, MiniLM, and hybrid methods
- **Advanced Evaluation**: Enhanced metrics with MRR@10, Precision@10, Recall@10 calculations

### Enhanced
- **Embedding Framework**: Modular architecture supporting sparse and dense embedding methods
- **Search Quality**: Hybrid retrieval combining efficiency of sparse methods with accuracy of dense embeddings
- **Benchmarking**: Automated evaluation pipeline comparing multiple embedding approaches

### Technical Details
- SVD implementation with simplified orthogonal transformation matrix generation
- Hybrid retriever supporting BM25+BERT and BM25+MiniLM combinations
- Comprehensive benchmark evaluating 8 different embedding approaches
- Modular evaluation framework for easy extension with new methods

## [0.2.1] - 2025-09-23

### Fixed
- **Critical**: Fixed flaky test `test_index_operations_comprehensive` in CI
- **HNSW**: Improved search recall for small indices by using adaptive `ef_search` parameter
- **Testing**: Enhanced HNSW search reliability for indices with < 10 vectors

### Technical Details
- Implemented adaptive `ef_search` calculation based on index size
- For small indices (< 10 vectors): `ef_search = max(vector_count * 2, k * 3)`
- For larger indices: `ef_search = max(k * 2, 64)`
- This ensures better recall in approximate nearest neighbor search for small datasets

## [0.2.0] - 2025-09-23

### Added (Phase 2: REST API Implementation)
- **Major**: Complete REST API implementation with Axum web framework
- **API**: Health check endpoint (`GET /health`)
- **API**: Collection management endpoints (create, list, get, delete)
- **API**: Vector operations endpoints (insert, get, delete)
- **API**: Vector search endpoint with configurable parameters
- **API**: Comprehensive error handling with structured error responses
- **API**: CORS support for cross-origin requests
- **API**: Request/response serialization with proper JSON schemas
- **Documentation**: Complete API documentation in `docs/API.md`
- **Examples**: API usage example in `examples/api_usage.rs`
- **Server**: HTTP server with graceful shutdown and logging
- **Testing**: Basic API endpoint tests

### Technical Details
- Implemented Axum-based HTTP server with Tower middleware
- Added structured API types for request/response serialization
- Created modular handler system for different endpoint categories
- Integrated with existing VectorStore for seamless database operations
- Added comprehensive error handling with HTTP status codes
- Implemented CORS and request tracing middleware

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

### Added (Gemini 2.5 Pro Final Review)
- **QA**: Performed final QA review, confirming stability of 56/57 tests.
- **Analysis**: Identified and documented one flaky test (`test_faq_search_system`) and its root cause.
- **Documentation**: Created `GEMINI_REVIEW_ANALYSIS.md` with findings and recommendations for test stabilization.
- **Fix**: Implemented deterministic vocabulary building in all embedding models to resolve test flakiness.

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

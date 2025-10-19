# Project Context

## Purpose

Vectorizer is a high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications. The project aims to provide:

- **Sub-millisecond search performance** with HNSW indexing
- **GPU acceleration** via Metal (macOS) with cross-platform CPU fallback
- **Multi-format document processing** (PDF, DOCX, XLSX, PPTX, HTML, XML, images)
- **Intelligent semantic search** with query expansion and diversification
- **Production-grade reliability** with 402 passing tests and comprehensive coverage
- **Developer-friendly APIs** via REST, MCP, and UMICP protocols

## Tech Stack

### Core Runtime & Web Framework
- **Rust 2024 Edition** (v1.9+) - Systems programming language
- **Tokio** (1.47) - Async runtime with full features
- **Axum** (0.8) - Web framework with WebSocket support
- **Tower/Tower-HTTP** (0.5/0.6) - Service abstractions with CORS and tracing

### Vector Database & Search
- **HNSW-RS** (0.3) - Hierarchical Navigable Small World indexing
- **Tantivy** (0.25) - BM25 full-text search engine
- **RRF** (0.1) - Reciprocal Rank Fusion for result merging
- **FastEmbed** (5.2) - ONNX-based embeddings and cross-encoder reranking

### GPU Acceleration
- **hive-gpu** (0.1.6) - External GPU acceleration crate
  - Metal support for macOS (Apple Silicon)
  - CUDA support (optional)
  - WGPU support (optional)
  - Graceful CPU fallback on all platforms

### Data Processing
- **Transmutation** (0.1.2) - Document conversion engine (PDF, Office, HTML)
- **Arrow/Parquet** (56) - Columnar data formats
- **UMICP-Core** (0.2.1) - Protocol integration with HTTP2/WebSocket

### Performance & Storage
- **Rayon** (1.10) + **Crossbeam** (0.8) - Parallel processing
- **DashMap** (6.1) + **Parking-Lot** (0.12) - Concurrent data structures
- **Zstd** (0.13) + **LZ4-Flex** (0.11) - Compression
- **Memmap2** (0.9) - Memory-mapped files
- **XXHash** (0.8) - Fast hashing

### Security & Authentication
- **JWT** (jsonwebtoken 9.3) - Token-based authentication
- **UUID** (1.18) - Unique identifiers
- **Blake3** (1.5) - Cryptographic hashing

### Developer Tools
- **Clap** (4.5) - CLI argument parsing
- **Tracing/Tracing-Subscriber** (0.1/0.3) - Structured logging
- **RMCP** (0.8.1) - MCP server SDK
- **Criterion** (0.5) - Benchmarking framework
- **Proptest** (1.4) - Property-based testing

## Project Conventions

### Code Style

**Formatting:**
- Use `rustfmt` with default settings (`rustfmt.toml` in repo)
- Use `clippy` with Qdrant-standard lints (see Cargo.toml `[lints.clippy]` section)
- 100 character line limit preferred

**Naming Conventions:**
- `snake_case` for functions, variables, modules
- `PascalCase` for types, structs, enums, traits
- `SCREAMING_SNAKE_CASE` for constants
- Prefix boolean functions with `is_`, `has_`, `should_`

**Error Handling:**
- Use `anyhow::Result` for application-level errors
- Use `thiserror::Error` for library-level custom errors
- Always provide context with `.context()` or `.with_context()`
- Never use `.unwrap()` in production code paths

**Documentation:**
- Public APIs MUST have doc comments
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in doc comments when helpful

### Architecture Patterns

**Layered Architecture:**
```
┌─────────────────────────────────────┐
│  Protocols (REST, MCP, UMICP)       │
├─────────────────────────────────────┤
│  Service Layer (Search, Discovery)  │
├─────────────────────────────────────┤
│  Storage Layer (Persistence, Cache) │
├─────────────────────────────────────┤
│  Vector Layer (HNSW, Embeddings)    │
└─────────────────────────────────────┘
```

**Key Patterns:**
- **Repository Pattern**: Storage abstraction via traits (`src/persistence/`)
- **Strategy Pattern**: Multiple search strategies (`intelligent`, `semantic`, `contextual`)
- **Facade Pattern**: Unified MCP tools (7 tools consolidating 40+ operations)
- **Observer Pattern**: File watcher with event debouncing
- **Cache-Aside**: Multi-tier caching (LFU hot, mmap warm, Zstd cold)

**Concurrency:**
- Use `Arc<DashMap>` for concurrent collections
- Use `parking_lot::RwLock` over `std::sync::RwLock`
- Use `tokio::spawn` for CPU-bound tasks in thread pool
- Use `rayon::par_iter` for data-parallel operations

**Feature Flags:**
- `default = ["hive-gpu", "fastembed"]` - Standard features
- `hive-gpu-metal` - Metal GPU acceleration (macOS)
- `hive-gpu-cuda` - CUDA GPU acceleration
- `transmutation` - Document conversion
- `full` - All optional features enabled

### Testing Strategy

**Test Organization:**
- Unit tests: In same file with `#[cfg(test)]` module
- Integration tests: `tests/` directory
- Benchmarks: `benchmark/scripts/` (currently disabled, needs refactoring)

**Test Coverage Requirements:**
- **402 tests** currently passing at 100% pass rate
- All public APIs must have tests
- Critical paths must have integration tests
- Property-based tests for parsers and normalizers

**Test Categories:**
- Vector operations (CRUD, search, indexing)
- Storage system (compaction, snapshots, migration)
- Transmutation (19 tests, document conversion)
- UMICP protocol (3 tests, discovery endpoints)
- MCP operations (32/33 manually validated)

**Performance Testing:**
- Search latency < 3ms on CPU, < 1ms on Metal GPU
- Startup must be non-blocking
- Storage reduction targets: 30-50% with normalization

### Git Workflow

**Branch Strategy:**
- `main` - Production-ready code
- Feature branches: `feature/add-<feature-name>`
- Fix branches: `fix/<issue-description>`
- Refactor branches: `refactor/<area>`

**Commit Conventions:**
- Use conventional commits format
- Types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `chore`
- Scope: Component affected (e.g., `feat(search): add MMR diversification`)
- Breaking changes: Add `BREAKING CHANGE:` in commit body

**SSH Certificate Workflow:**
- Never use `git push` directly in automation
- Always provide push commands for user to execute manually
- User has SSH certificate requiring password input

## Domain Context

### Vector Database Fundamentals

**Vector Similarity Search:**
- Embeddings represent text/documents as high-dimensional vectors
- Distance metrics: Cosine similarity (default), Euclidean, Dot Product
- HNSW (Hierarchical Navigable Small World) provides approximate nearest neighbor search

**Embedding Models:**
- TF-IDF: Term frequency-inverse document frequency (baseline)
- BM25: Probabilistic relevance-based scoring
- BERT: Transformer-based contextual embeddings
- MiniLM: Lightweight transformer model (384 dimensions)
- Custom models via ONNX Runtime

**Search Strategies:**
- **Basic**: Simple vector similarity ranking
- **Intelligent**: Multi-query generation with domain expansion and MMR diversification
- **Semantic**: Advanced reranking with similarity thresholds
- **Contextual**: Metadata filtering with context-aware ranking
- **Multi-Collection**: Cross-collection search with deduplication
- **Batch**: Multiple queries in parallel

### Document Processing

**Transmutation Integration:**
- Converts PDF, DOCX, XLSX, PPTX, HTML, XML to Markdown
- 98x faster than alternative tools (Docling)
- Preserves metadata (page numbers, structure)
- Optional feature flag: `transmutation`

**Text Normalization:**
- Content-aware normalization (conservative/moderate/aggressive)
- Line ending normalization (CRLF → LF)
- Whitespace trimming and collapsing
- 30-50% storage reduction without quality loss

### Storage Architecture

**Compact Format (.vecdb):**
- Unified Zstd-compressed archive
- 20-30% space savings vs individual files
- Atomic updates (corruption-safe)
- Snapshot support with retention policies

**Multi-Tier Cache:**
- Hot cache: LFU (Least Frequently Used) in-memory
- Warm store: Memory-mapped files
- Cold storage: Zstd compressed archives

## Important Constraints

### Platform Support
- **Primary**: Linux x86_64 (Ubuntu 24.04)
- **Secondary**: macOS (Apple Silicon with Metal GPU)
- **Tertiary**: Windows (CPU fallback)
- **Deployment**: Docker containers recommended

### Performance Targets
- Search latency: < 3ms (CPU), < 1ms (Metal GPU)
- Startup time: Non-blocking initialization
- Memory: Efficient memory usage with multi-tier caching
- Storage: 20-30% compression ratio minimum

### Rust Version
- **ALWAYS use Rust 1.9+ with edition 2024**
- Never revert to edition 2021
- Keep dependencies updated for security patches

### Build Requirements
- **WSL**: Always use `wsl -d Ubuntu-24.04 -- bash -l -c` for commands
- **Terminal**: Use PowerShell or WSL (never Cursor terminal)
- **CUDA**: Requires specific LIB environment variable configuration
- **Static Linking**: OpenSSL vendored for musl targets

## External Dependencies

### Required Services
- **None** - Vectorizer is self-contained

### Optional Integrations
- **Transmutation Server**: Document conversion service (if not using feature flag)
- **GPU Drivers**: Metal (macOS), CUDA (NVIDIA), or CPU fallback
- **ONNX Runtime**: For FastEmbed models (bundled with feature)

### Client SDKs
- Python: `pip install vectorizer-client`
- TypeScript: `npm install @hivellm/vectorizer-client-ts`
- JavaScript: `npm install @hivellm/vectorizer-client-js`
- Rust: `cargo add vectorizer-rust-sdk`

### Protocols Supported
- **REST API**: HTTP/JSON on port 15002
- **MCP (Model Context Protocol)**: StreamableHTTP transport
- **UMICP v0.2.1**: Native JSON types with tool discovery endpoint
- **WebSocket**: Real-time updates (via Axum)

### Monitoring & Observability
- Structured logging via `tracing`
- Health check endpoint: `/health`
- System metrics: CPU, memory, disk usage
- Desktop GUI: Electron-based management dashboard

## Development Workflow Notes

### Before Starting Work
1. Read relevant specs in `openspec/specs/[capability]/spec.md`
2. Check `openspec/changes/` for conflicting proposals
3. Run `openspec list` and `openspec list --specs`
4. Verify tests pass: `cargo test --all-features`

### After Completing Work
- Run tests and ensure 100% pass rate
- Update existing documentation (avoid creating new .md files)
- Commit changes with conventional commit messages
- Provide git commands for user to push (never auto-push)

### Documentation Organization
- **README.md**: Main project overview
- **CHANGELOG.md**: Version history
- **docs/specs/**: Technical specifications
- **openspec/**: OpenSpec change proposals and specs
- Avoid creating unnecessary documentation files

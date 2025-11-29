# Test Suite Organization

This directory contains all tests organized by functional category for better maintainability and clarity.

## Structure

```
tests/
├── core/              # Core functionality tests
│   ├── simd.rs        # SIMD operations
│   ├── quantization.rs # Quantization
│   ├── storage.rs     # Storage (MMap, etc)
│   └── wal.rs         # Write-Ahead Log
│
├── api/               # API tests
│   ├── rest/          # REST API tests
│   ├── grpc/          # gRPC API tests (organized)
│   └── mcp/           # MCP API tests
│
├── integration/       # Integration tests
│   ├── hybrid_search.rs
│   ├── sparse_vector.rs
│   ├── payload_index.rs
│   ├── query_cache.rs
│   └── binary_quantization.rs
│
├── replication/       # Replication tests
│   ├── integration.rs
│   ├── failover.rs
│   ├── handlers.rs
│   └── qdrant.rs      # Qdrant compatibility
│
├── performance/       # Performance & stress tests
│   ├── concurrent.rs
│   └── multi_collection.rs
│
├── gpu/               # GPU acceleration tests
│   ├── hive_gpu.rs
│   └── metal.rs
│
├── workflow/          # End-to-end workflows
│   └── api_workflow.rs
│
├── infrastructure/    # Infrastructure tests
│   ├── docker.rs
│   └── logging.rs
│
└── helpers/           # Shared test utilities
    └── mod.rs
```

## Test Categories

### Core (`core/`)
- **simd.rs**: SIMD-accelerated operations
- **quantization.rs**: Vector quantization (SQ, PQ, Binary)
- **storage.rs**: Storage backends (Memory, MMap)
- **wal.rs**: Write-Ahead Log functionality

### API (`api/`)
- **rest/**: REST API integration tests
- **grpc/**: gRPC API tests (see `grpc/README.md`)
- **mcp/**: Model Context Protocol tests

### Integration (`integration/`)
- **hybrid_search.rs**: Dense + sparse hybrid search
- **sparse_vector.rs**: Sparse vector operations
- **payload_index.rs**: Payload indexing and filtering
- **query_cache.rs**: Query result caching
- **binary_quantization.rs**: Binary quantization integration

### Replication (`replication/`)
- **integration.rs**: Basic replication
- **failover.rs**: Failover scenarios
- **handlers.rs**: Replication handlers
- **qdrant.rs**: Qdrant API compatibility and migration

### Performance (`performance/`)
- **concurrent.rs**: Concurrent operations
- **multi_collection.rs**: Multiple collections operations

### GPU (`gpu/`)
- **hive_gpu.rs**: HiveGPU integration
- **metal.rs**: Metal GPU validation

### Workflow (`workflow/`)
- **api_workflow.rs**: End-to-end API workflows

### Infrastructure (`infrastructure/`)
- **docker.rs**: Docker and virtual paths
- **logging.rs**: Logging levels and configuration

## Running Tests

```bash
# Run all tests
cargo test

# Run specific category
cargo test --test core
cargo test --test api
cargo test --test integration

# Run with output
cargo test -- --nocapture

# Run specific test file
cargo test --test core::simd
```

## Migration Status

- ✅ gRPC tests organized (see `grpc/README.md`)
- ✅ Core tests structure created
- ✅ API tests structure created (REST, MCP)
- ✅ Integration tests structure created
- ✅ Replication tests structure created
- ✅ Performance tests structure created
- ✅ GPU tests structure created
- ✅ Workflow tests structure created
- ✅ Infrastructure tests structure created

**Note**: All test files have been moved to their respective directories. The structure is fully organized and functional.

## Remaining Files in Root

Some test files remain in the root `tests/` directory:
- `grpc_*.rs` - gRPC tests (legacy, see `grpc/` for organized version)
- `grpc_tests.rs` - Organized gRPC test entry point
- `all_tests.rs` - All tests entry point
- `test_new_features.rs` - New features tests (to be categorized)
- `grpc_s2s.rs` - Server-to-server tests (requires `s2s-tests` feature)

These will be gradually migrated or kept as entry points.


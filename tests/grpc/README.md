# gRPC API Tests - Organized by Theme

This directory contains gRPC API tests organized by functional theme for better maintainability and clarity.

## Structure

```
tests/grpc/
├── mod.rs              # Module declarations
├── helpers.rs          # Shared test utilities
├── collections.rs      # Collection management tests
├── vectors.rs          # Vector CRUD operations
├── search.rs           # Search operations (TODO)
├── configurations.rs   # Configuration variations (TODO)
├── edge_cases.rs       # Error handling & edge cases (TODO)
├── performance.rs      # Stress tests (TODO)
└── s2s.rs              # Server-to-server tests (feature: s2s-tests)
```

## Test Categories

### Collections (`collections.rs`)
- List collections
- Create collection
- Get collection info
- Delete collection
- Multiple collections

### Vectors (`vectors.rs`)
- Insert vector
- Get vector
- Update vector
- Delete vector
- Streaming bulk insert
- Payload handling

### Search (`search.rs`) - TODO
- Basic search
- Batch search
- Hybrid search
- Search with filters
- Search with threshold

### Configurations (`configurations.rs`) - TODO
- Different distance metrics
- Different storage types
- Quantization configurations
- HNSW parameter variations

### Edge Cases (`edge_cases.rs`) - TODO
- Error handling
- Empty collections
- Non-existent resources
- Invalid inputs

### Performance (`performance.rs`) - TODO
- Concurrent operations
- Large payloads
- Stress tests
- Batch operations

### Server-to-Server (`s2s.rs`)
- Tests against real running server
- Requires `s2s-tests` feature flag
- See `tests/README_GRPC_S2S.md` for details

## Running Tests

```bash
# Run all organized gRPC tests
cargo test --test grpc_tests

# Run specific category
cargo test --test grpc_tests collections
cargo test --test grpc_tests vectors

# Run with output
cargo test --test grpc_tests -- --nocapture

# Run S2S tests (requires feature)
cargo test --features s2s-tests --test grpc_s2s
```

## Migration Status

- ✅ Collections tests migrated
- ✅ Vectors tests migrated
- ⏳ Search tests (to be migrated from grpc_comprehensive.rs)
- ⏳ Configuration tests (to be migrated from grpc_advanced.rs)
- ⏳ Edge case tests (to be migrated from grpc_integration.rs)
- ⏳ Performance tests (to be migrated from grpc_advanced.rs)

## Legacy Test Files

The following files still exist and will be gradually migrated:
- `tests/grpc_integration.rs` - Basic integration tests
- `tests/grpc_comprehensive.rs` - Comprehensive tests
- `tests/grpc_advanced.rs` - Advanced tests

These will be deprecated once all tests are migrated to the new structure.


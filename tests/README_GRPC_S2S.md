# gRPC Server-to-Server (S2S) Tests

These tests connect to a **REAL running Vectorizer server** instance via gRPC.

## Prerequisites

1. Start the Vectorizer server:
   ```bash
   ./target/release/vectorizer
   # or
   cargo run --release
   ```

2. The server should be running with gRPC enabled (default port: 15003)

## Configuration

Set environment variables to configure the server address:

```bash
# Default: http://127.0.0.1:15003
export VECTORIZER_GRPC_HOST=127.0.0.1
export VECTORIZER_GRPC_PORT=15003
```

## Running Tests

```bash
# Run all S2S tests
cargo test --test grpc_s2s

# Run with output
cargo test --test grpc_s2s -- --nocapture

# Run specific test
cargo test --test grpc_s2s test_real_server_health_check -- --nocapture
```

## Test Coverage

The S2S tests cover:

1. **Health Check** - Verify server is healthy
2. **Get Stats** - Server statistics
3. **List Collections** - List all collections
4. **Create Collection** - Create new collection
5. **Insert and Get Vector** - Vector CRUD operations
6. **Search** - Vector similarity search
7. **Streaming Bulk Insert** - Bulk operations via streaming
8. **Batch Search** - Multiple queries in one request
9. **Update and Delete** - Vector updates and deletions
10. **Get Collection Info** - Collection metadata

## Notes

- Tests use unique collection names (timestamp-based) to avoid conflicts
- Tests are designed to be non-destructive (create their own collections)
- All tests have timeouts to prevent hanging
- Tests print progress information for debugging


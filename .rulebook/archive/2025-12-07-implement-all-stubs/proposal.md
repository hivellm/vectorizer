# Proposal: implement-all-stubs

## Why

The codebase contains ~177 instances of stub implementations, TODOs, and incomplete functionality across multiple modules. These stubs range from critical production blockers to low-priority enhancements. Completing these implementations will:

1. **Eliminate Production Blockers**: Fix critical issues preventing production deployment
2. **Improve Feature Completeness**: Complete partially implemented features
3. **Enhance Reliability**: Remove placeholder code that may cause unexpected behavior
4. **Better Test Coverage**: Fix broken tests and enable disabled test suites
5. **Code Quality**: Clean up technical debt and improve maintainability

**Current State**:
- 3 critical stubs blocking production (TLS, collection persistence, tenant migration)
- 5 high-priority stubs affecting core functionality
- 13 medium-priority stubs affecting advanced features
- 9 low-priority stubs for optional enhancements
- Multiple test suites disabled or broken

**Problem Scenarios**:
- Production deployments cannot use TLS/HTTPS
- Collections created via API may be lost on restart
- Advanced features like hybrid search don't work
- Some gRPC operations return Unimplemented errors
- Test coverage is incomplete due to broken tests

## What Changes

### Phase 1: Critical Stubs (Production Blockers)

1. **TLS/SSL Support** (`src/security/tls.rs`)
   - Implement `create_server_config()` to load certificates
   - Configure cipher suites and ALPN protocols
   - Add mTLS support for client certificate validation
   - Enable HTTPS/TLS encryption for production

2. **Collection Persistence** (already has separate task)
   - Ensure API-created collections persist immediately
   - Fix auto-save integration
   - Verify collections load on restart

3. **Tenant Migration** (`src/server/hub_tenant_handlers.rs`)
   - Implement `migrate_tenant_data()` handler
   - Add tenant data export/import functionality
   - Support tenant ownership transfer
   - Enable tenant merging capabilities

### Phase 2: High Priority Stubs

4. **Workspace Manager Integration**
   - Complete workspace manager integration in REST handlers
   - Complete workspace manager integration in GraphQL schema
   - Ensure all workspace operations work correctly

5. **BERT and MiniLM Embeddings** (`src/embedding/mod.rs`)
   - Option A: Implement real BERT/MiniLM model loading (requires ML dependencies)
   - Option B: Remove if not needed, document as not supported
   - Replace hash-based placeholders with real implementations or removal

6. **Hybrid Search** (`src/discovery/hybrid.rs`)
   - Implement dense search with HNSW
   - Implement sparse search with BM25/tantivy
   - Implement Reciprocal Rank Fusion (RRF) for result merging
   - Complete `HybridSearcher::search()` implementation

7. **Transmutation Integration** (`src/transmutation_integration/mod.rs`)
   - Update to match actual transmutation API
   - Implement real page count extraction
   - Implement real content extraction from conversion results
   - Remove placeholder implementations

8. **gRPC Unimplemented Methods**
   - Implement 5 unimplemented gRPC methods
   - Ensure all gRPC operations are functional
   - Add proper error handling

### Phase 3: Medium Priority Stubs

9. **Sharded Collection Features**
   - Implement batch insert for distributed collections
   - Implement hybrid search for sharded/distributed collections
   - Add document count tracking
   - Implement requantization for sharded collections

10. **Qdrant Filter Operations**
    - Implement filter-based deletion
    - Implement filter-based payload operations (update, overwrite, delete, clear)
    - Ensure all Qdrant filter operations work correctly

11. **Rate Limiting**
    - Implement per-API-key rate limiting
    - Extract API key from requests
    - Apply rate limits per key

12. **Quantization Cache Tracking**
    - Implement cache hit ratio tracking
    - Add cache performance metrics
    - Monitor cache effectiveness

13. **HiveHub Features**
    - Implement API request tracking
    - Add HiveHub Cloud logging endpoint integration
    - Complete monitoring features

14. **Test Fixes**
    - Fix file watcher pattern matching tests
    - Fix discovery module tests
    - Fix intelligent search tests
    - Re-enable all disabled tests

### Phase 4: Low Priority Stubs

15. **Optional Enhancements**
    - Graceful restart implementation
    - Collection mapping YAML configuration
    - Discovery compress/filter integrations
    - File watcher batch processing re-enable
    - GPU collection multi-tenant support
    - Distributed collection shard router improvements
    - Cluster remote operations completion
    - gRPC improvements (quantization config, uptime tracking, score extraction)
    - Qdrant lookup feature
    - Summarization method improvements

## Impact

- **Affected code**: ~30 files across multiple modules
- **Breaking change**: NO - Only completes existing functionality
- **User benefit**: 
  - Production-ready TLS support
  - Reliable collection persistence
  - Complete feature set
  - Better test coverage
  - Improved code quality

## Implementation Strategy

1. **Prioritize by Impact**: Start with critical production blockers
2. **Incremental Approach**: Complete one stub category at a time
3. **Test-Driven**: Add tests before/while implementing
4. **Documentation**: Update docs as stubs are completed
5. **Validation**: Verify each implementation works correctly

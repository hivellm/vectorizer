## Phase 1: Critical Stubs (Production Blockers)

### 1. TLS/SSL Support
- [x] 1.1 Implement certificate loading from files in `create_server_config()`
- [ ] 1.2 Configure cipher suites for rustls
- [ ] 1.3 Implement ALPN protocol configuration
- [x] 1.4 Add mTLS support (client certificate validation)
- [ ] 1.5 Test TLS connection establishment
- [ ] 1.6 Test HTTPS endpoint access
- [ ] 1.7 Update documentation for TLS configuration

### 2. Collection Persistence
- [x] 2.1 See task: `fix-collection-persistence-on-restart`
- [x] 2.2 Mark as complete when that task is done

### 3. Tenant Migration
- [ ] 3.1 Implement tenant data export functionality
- [ ] 3.2 Implement tenant data import functionality
- [ ] 3.3 Implement tenant ownership transfer
- [ ] 3.4 Implement tenant merging
- [ ] 3.5 Add validation and error handling
- [ ] 3.6 Add tests for tenant migration
- [ ] 3.7 Update API documentation

## Phase 2: High Priority Stubs

### 4. Workspace Manager Integration
- [x] 4.1 Complete REST handler workspace integration
  - [x] 4.1.1 Fix `add_workspace` handler
  - [x] 4.1.2 Fix `remove_workspace` handler
  - [ ] 4.1.3 Fix `update_workspace_config` handler
- [x] 4.2 Complete GraphQL workspace integration
  - [x] 4.2.1 Fix `add_workspace` mutation
  - [x] 4.2.2 Fix `remove_workspace` mutation
  - [ ] 4.2.3 Fix `update_workspace_config` mutation
- [ ] 4.3 Test all workspace operations
- [ ] 4.4 Update documentation

### 5. BERT and MiniLM Embeddings
- [ ] 5.1 Decide: Implement real models or remove
- [ ] 5.2 If implementing:
  - [ ] 5.2.1 Add ML dependencies (candle, ort, etc.)
  - [ ] 5.2.2 Implement `BertEmbedding::load_model()` with real model loading
  - [ ] 5.2.3 Implement `MiniLmEmbedding::load_model()` with real model loading
  - [ ] 5.2.4 Replace `simple_hash_embedding()` with real inference
  - [ ] 5.2.5 Add model download/caching logic
- [ ] 5.3 If removing:
  - [ ] 5.3.1 Remove BERT/MiniLM embedding providers
  - [ ] 5.3.2 Update documentation to note they're not supported
  - [ ] 5.3.3 Remove related tests
- [ ] 5.4 Update embedding documentation

### 6. Hybrid Search
- [x] 6.1 Implement dense search with HNSW in `HybridSearcher::search()`
- [x] 6.2 Implement sparse search with BM25/tantivy
- [x] 6.3 Implement Reciprocal Rank Fusion (RRF) algorithm
- [x] 6.4 Merge dense and sparse results using RRF
- [x] 6.5 Add configuration for alpha parameter (dense/sparse weight)
- [x] 6.6 Add tests for hybrid search
- [ ] 6.7 Update discovery documentation

### 7. Transmutation Integration
- [x] 7.1 Research actual transmutation API from crates.io
- [ ] 7.2 Update `convert_to_markdown()` to use real API
- [ ] 7.3 Implement real page count extraction
- [ ] 7.4 Implement real content extraction from `ConversionResult`
- [ ] 7.5 Remove placeholder implementations
- [x] 7.6 Test with real documents (PDF, DOCX, etc.)
- [ ] 7.7 Update documentation

### 8. gRPC Unimplemented Methods
- [ ] 8.1 Identify all unimplemented gRPC methods
  - [ ] 8.1.1 `src/grpc/qdrant/qdrant.rs` - 3 methods (lines 3340, 8000, 8759)
  - [ ] 8.1.2 `src/grpc/vectorizer.rs` - 1 method (line 1697)
  - [ ] 8.1.3 `src/grpc/vectorizer.cluster.rs` - 1 method (line 1468)
- [ ] 8.2 Implement each method or document why it's not needed
- [ ] 8.3 Add proper error handling
- [ ] 8.4 Add tests for each method
- [ ] 8.5 Update gRPC documentation

## Phase 3: Medium Priority Stubs

### 9. Sharded Collection Features
- [ ] 9.1 Implement batch insert for distributed collections
- [ ] 9.2 Implement hybrid search for sharded collections
- [ ] 9.3 Implement hybrid search for distributed collections
- [ ] 9.4 Add document count tracking for sharded collections
- [ ] 9.5 Implement requantization for sharded collections
- [ ] 9.6 Add tests for each feature
- [ ] 9.7 Update sharding documentation

### 10. Qdrant Filter Operations
- [ ] 10.1 Implement filter-based deletion
- [ ] 10.2 Implement filter-based payload update
- [ ] 10.3 Implement filter-based payload overwrite
- [ ] 10.4 Implement filter-based payload deletion
- [ ] 10.5 Implement filter-based payload clear
- [ ] 10.6 Add filter parsing and validation
- [ ] 10.7 Add tests for each operation
- [ ] 10.8 Update Qdrant compatibility documentation

### 11. Rate Limiting
- [ ] 11.1 Extract API key from requests
- [ ] 11.2 Implement per-API-key rate limiting
- [ ] 11.3 Add rate limit tracking per key
- [ ] 11.4 Add configuration for per-key limits
- [ ] 11.5 Add tests for per-key rate limiting
- [ ] 11.6 Update security documentation

### 12. Quantization Cache Tracking
- [ ] 12.1 Implement cache hit ratio tracking
- [ ] 12.2 Implement cache hit tracking in HNSW integration
- [ ] 12.3 Add cache statistics collection
- [ ] 12.4 Expose cache metrics via monitoring
- [ ] 12.5 Add tests for cache tracking
- [ ] 12.6 Update quantization documentation

### 13. HiveHub Features
- [ ] 13.1 Implement API request tracking
- [ ] 13.2 Implement HiveHub Cloud logging endpoint
- [ ] 13.3 Add request tracking to usage metrics
- [ ] 13.4 Integrate logging with HiveHub API
- [ ] 13.5 Add tests for tracking and logging
- [ ] 13.6 Update HiveHub documentation

### 14. Test Fixes
- [ ] 14.1 Fix file watcher pattern matching tests
  - [ ] 14.1.1 Implement pattern matching methods or fix tests
  - [ ] 14.1.2 Re-enable skipped tests
- [ ] 14.2 Fix discovery module tests
  - [ ] 14.2.1 Update tests to use new Discovery::new signature
  - [ ] 14.2.2 Re-enable all discovery tests
- [ ] 14.3 Fix intelligent search tests
  - [ ] 14.3.1 Fix MCPToolHandler tests
  - [ ] 14.3.2 Fix MCPServerIntegration tests
  - [ ] 14.3.3 Re-enable all commented tests
- [ ] 14.4 Verify all tests pass
- [ ] 14.5 Update test coverage documentation

## Phase 4: Low Priority Stubs (Optional)

### 15. Graceful Restart
- [x] 15.1 Implement graceful restart handler
- [ ] 15.2 Add shutdown signal handling
- [ ] 15.3 Ensure in-flight requests complete
- [ ] 15.4 Test graceful restart

### 16. Collection Mapping Configuration
- [ ] 16.1 Add YAML configuration for collection mapping
- [ ] 16.2 Parse collection mapping from config
- [ ] 16.3 Apply mapping on file watcher startup
- [ ] 16.4 Update configuration documentation

### 17. Discovery Integrations
- [ ] 17.1 Integrate keyword_extraction for compress
- [ ] 17.2 Integrate tantivy for BM25 filtering
- [ ] 17.3 Test integrations
- [ ] 17.4 Update discovery documentation

### 18. File Watcher Batch Processing
- [ ] 18.1 Re-enable batch processing
- [ ] 18.2 Test batch processing stability
- [ ] 18.3 Monitor for issues
- [ ] 18.4 Update file watcher documentation

### 19. GPU Collection Multi-Tenant
- [ ] 19.1 Add owner_id support to HiveGpuCollection
- [ ] 19.2 Test multi-tenant GPU collections
- [ ] 19.3 Update GPU collection documentation

### 20. Distributed Collection Improvements
- [ ] 20.1 Implement shard router method for all shards
- [ ] 20.2 Complete cluster remote operations
  - [ ] 20.2.1 Complete remote collection creation
  - [ ] 20.2.2 Add document count
  - [ ] 20.2.3 Complete remote collection deletion
- [ ] 20.3 Test distributed operations
- [ ] 20.4 Update cluster documentation

### 21. gRPC Improvements
- [ ] 21.1 Implement quantization config conversion
- [ ] 21.2 Implement uptime tracking
- [ ] 21.3 Implement actual dense/sparse score extraction
- [ ] 21.4 Test gRPC improvements
- [ ] 21.5 Update gRPC documentation

### 22. Qdrant Lookup
- [ ] 22.1 Implement with_lookup feature
- [ ] 22.2 Test lookup functionality
- [ ] 22.3 Update Qdrant documentation

### 23. Summarization Methods
- [ ] 23.1 Complete abstractive summarization
- [ ] 23.2 Test summarization methods
- [ ] 23.3 Update summarization documentation

### 24. Placeholder Embeddings
- [ ] 24.1 Review placeholder embeddings
- [ ] 24.2 Decide: Implement real models or document limitations
- [ ] 24.3 Update embedding documentation

## Phase 5: Documentation and Cleanup

- [ ] 25.1 Update CHANGELOG.md with all completed stubs
- [ ] 25.2 Update STUBS_ANALYSIS.md to mark completed items
- [ ] 25.3 Review and update all affected documentation
- [ ] 25.4 Remove any remaining TODO comments for completed items
- [ ] 25.5 Verify no new stubs were introduced during implementation

## 1. Analysis and Planning
- [ ] 1.1 Analyze current memory usage patterns in cluster mode
- [ ] 1.2 Document all code paths that check cluster mode
- [ ] 1.3 Identify all caches that need memory limits
- [ ] 1.4 Review existing MMap implementation completeness

## 2. Add Cluster Configuration
- [ ] 2.1 Add max_cache_memory_bytes to ClusterConfig
- [ ] 2.2 Add enforce_mmap_storage flag to ClusterConfig
- [ ] 2.3 Add disable_file_watcher flag to ClusterConfig
- [ ] 2.4 Update config.example.yml with cluster memory settings
- [ ] 2.5 Set sensible defaults (1GB cache limit)

## 3. Create Cluster Config Validator
- [ ] 3.1 Create src/cluster/validator.rs
- [ ] 3.2 Implement validate_cluster_config() function
- [ ] 3.3 Check MMap is enforced (reject Memory storage)
- [ ] 3.4 Check cache limit is set and valid (≤ 1GB)
- [ ] 3.5 Check file watcher is disabled
- [ ] 3.6 Generate clear error messages for violations

## 4. Enforce MMap in Cluster Mode
- [ ] 4.1 Modify Collection::new() to check cluster mode
- [ ] 4.2 Force StorageType::Mmap when cluster.enabled = true
- [ ] 4.3 Reject Memory storage in VectorStore::create_collection()
- [ ] 4.4 Add error type ClusterConfigViolation
- [ ] 4.5 Update collection creation API to return error

## 5. Implement Global Cache Limit
- [ ] 5.1 Create CacheMemoryManager in src/cache/memory_manager.rs
- [ ] 5.2 Track total cache memory usage across all caches
- [ ] 5.3 Implement forced eviction when limit reached
- [ ] 5.4 Apply limit to AdvancedCache
- [ ] 5.5 Apply limit to HNSW index cache
- [ ] 5.6 Apply limit to metadata cache
- [ ] 5.7 Add cache memory usage metrics

## 6. Disable File Watcher in Cluster
- [ ] 6.1 Check cluster.enabled in FileWatcher::start()
- [ ] 6.2 Return error if trying to start in cluster mode
- [ ] 6.3 Add clear error message about cluster incompatibility
- [ ] 6.4 Update FileWatcher documentation
- [ ] 6.5 Prevent automatic start on server startup in cluster

## 7. Startup Validation
- [ ] 7.1 Add validate_cluster_requirements() to server startup
- [ ] 7.2 Call validator before starting server
- [ ] 7.3 Fail fast with clear error messages
- [ ] 7.4 Log all cluster configuration settings on startup
- [ ] 7.5 Add --skip-cluster-validation flag for debugging (unsafe)

## 8. Memory Monitoring
- [ ] 8.1 Add vectorizer_cluster_memory_usage_bytes metric
- [ ] 8.2 Add vectorizer_cluster_cache_usage_bytes metric
- [ ] 8.3 Add vectorizer_cluster_cache_evictions_total metric
- [ ] 8.4 Add per-tenant memory tracking (if multi-tenant enabled)
- [ ] 8.5 Update Prometheus metrics endpoint
- [ ] 8.6 Add Grafana dashboard template

## 9. Migration Support
- [ ] 9.1 Create config migration script (Memory → MMap)
- [ ] 9.2 Add --migrate-to-cluster CLI command
- [ ] 9.3 Implement collection storage type conversion
- [ ] 9.4 Add dry-run mode for migration
- [ ] 9.5 Create migration guide documentation

## 10. Testing - Unit Tests
- [ ] 10.1 Test ClusterConfigValidator with valid configs
- [ ] 10.2 Test ClusterConfigValidator with invalid configs
- [ ] 10.3 Test Memory storage rejection in cluster mode
- [ ] 10.4 Test cache limit enforcement
- [ ] 10.5 Test file watcher prevention in cluster mode
- [ ] 10.6 Test CacheMemoryManager eviction logic

## 11. Testing - Integration Tests
- [ ] 11.1 Test cluster startup with valid configuration
- [ ] 11.2 Test cluster startup fails with Memory storage
- [ ] 11.3 Test cluster startup fails with file watcher enabled
- [ ] 11.4 Test cache memory limit under load
- [ ] 11.5 Test collection creation in cluster mode
- [ ] 11.6 Test storage type validation

## 12. Testing - Load Tests
- [ ] 12.1 Test memory usage with 10 concurrent users
- [ ] 12.2 Verify cache stays under 1GB limit
- [ ] 12.3 Test eviction performance under pressure
- [ ] 12.4 Measure MMap vs Memory performance difference
- [ ] 12.5 Test cluster stability over 24 hours

## 13. Documentation
- [ ] 13.1 Create docs/specs/CLUSTER.md
- [ ] 13.2 Update docs/specs/MMAP_IMPLEMENTATION.md
- [ ] 13.3 Update docs/specs/FILE_WATCHER.md
- [ ] 13.4 Create docs/users/CLUSTER_SETUP.md
- [ ] 13.5 Update README.md with cluster requirements
- [ ] 13.6 Update CHANGELOG.md

## 14. Configuration Examples
- [ ] 14.1 Add cluster mode example to config.example.yml
- [ ] 14.2 Add standalone mode example
- [ ] 14.3 Document all cluster-specific settings
- [ ] 14.4 Add troubleshooting section
- [ ] 14.5 Create production deployment template

## 15. Verification and Cleanup
- [ ] 15.1 Run full test suite
- [ ] 15.2 Verify linter passes
- [ ] 15.3 Check test coverage (95%+)
- [ ] 15.4 Manual testing in cluster mode
- [ ] 15.5 Performance benchmarking
- [ ] 15.6 Security audit of cluster features

# Proposal: HiveHub Cluster Mode - Multi-Tenant Architecture

## Why

The current Vectorizer implementation is designed for single-user or single-organization deployments. To support SaaS deployment through HiveHub, we need to implement a multi-tenant cluster mode that:

1. **Enables Shared Infrastructure**: Multiple users can share the same Vectorizer instance while maintaining complete data isolation
2. **Quota Management**: Integrates with HiveHub's quota system to enforce storage limits and rate limiting per user
3. **Secure Authentication**: Implements API key-based authentication with granular permission control
4. **Cost Efficiency**: Reduces infrastructure costs by enabling resource sharing among multiple tenants
5. **Commercial Viability**: Enables HiveHub to offer Vectorizer as a managed service with usage-based billing

### Business Value

- **For HiveHub**: Enables monetization through managed Vectorizer-as-a-Service
- **For Users**: Access to production-ready vector database without infrastructure management
- **For Ecosystem**: Standardized multi-tenant vector database pattern for AI applications

### Technical Challenges

1. **Data Isolation**: All collections, vectors, and metadata must be strictly isolated per tenant
2. **Performance**: Multi-tenancy should not degrade performance for individual tenants
3. **Security**: Zero data leakage between tenants, even in case of implementation bugs
4. **Backward Compatibility**: Must not break existing single-tenant deployments

## What Changes

### 1. HiveHub Integration API

**New Component**: HiveHub integration client that communicates with HiveHub API to:
- Validate API keys and retrieve tenant information
- Fetch quota limits (storage, operations, rate limits)
- Report usage metrics for billing
- Synchronize access control policies

**Files Added**:
- `src/hivehub/client.rs` - HiveHub API client
- `src/hivehub/models.rs` - HiveHub data models (quotas, tenant info)
- `src/hivehub/cache.rs` - Local cache for quota/auth data

### 2. Authentication & Authorization System

**New Component**: Comprehensive auth system with:
- API key validation middleware
- Tenant context extraction
- Permission-based access control
- Separate key types (admin keys vs. MCP keys)

**Files Modified**:
- `src/api/middleware/auth.rs` - Authentication middleware
- `src/models/auth.rs` - Auth models (ApiKey, Tenant, Permissions)

**Permission Levels**:
- `ADMIN`: Full access (collection management, admin operations)
- `READ_WRITE`: Data operations (insert, update, delete, search)
- `READ_ONLY`: Search and read operations only
- `MCP`: Limited to MCP protocol operations (isolated from admin functions)

### 3. Multi-Tenant Data Isolation

**Modified Component**: Core data structures with tenant scoping:
- All collections prefixed with tenant ID: `{tenant_id}:{collection_name}`
- All operations require tenant context
- Storage paths isolated: `data/{tenant_id}/...`

**Files Modified**:
- `src/db/vector_store.rs` - Add tenant context to all operations
- `src/db/collection.rs` - Tenant-scoped collection naming
- `src/persistence/storage.rs` - Tenant-isolated file paths
- `src/models/collection.rs` - Add tenant_id field

### 4. Quota & Rate Limiting System

**New Component**: Quota enforcement system that:
- Tracks storage usage per tenant
- Enforces rate limits per tenant (requests/minute, requests/hour)
- Blocks operations when limits are exceeded
- Reports usage to HiveHub periodically

**Files Added**:
- `src/quota/manager.rs` - Quota management
- `src/quota/rate_limiter.rs` - Per-tenant rate limiting
- `src/quota/storage_tracker.rs` - Storage usage tracking
- `src/quota/reporter.rs` - Usage reporting to HiveHub

### 5. API Changes

**All Endpoints Modified**:
- Add `Authorization: Bearer <api_key>` header requirement
- Extract tenant context from API key
- Validate permissions for each operation
- Enforce rate limits per tenant

**New Endpoints**:
- `GET /api/v1/cluster/health` - Cluster health (admin only)
- `GET /api/v1/cluster/tenants` - List tenants (admin only)
- `GET /api/v1/cluster/usage` - Tenant usage stats (tenant owner or admin)
- `POST /api/v1/cluster/keys/validate` - Validate API key

### 6. MCP Protocol Isolation

**Modified**: MCP server with tenant authentication:
- MCP connections require API key authentication
- MCP tools scoped to tenant's collections only
- MCP keys cannot access admin tools
- Separate MCP key type with limited permissions

**Files Modified**:
- `src/mcp/server.rs` - Add authentication to MCP
- `src/mcp/tools/*.rs` - Add tenant scoping to all tools

### 7. Configuration

**New Configuration Options** (`config.yml`):
```yaml
cluster:
  enabled: true  # Enable cluster mode
  hivehub_api_url: "https://api.hivehub.io"
  hivehub_api_key: "${HIVEHUB_API_KEY}"
  quota_check_interval: 60  # seconds
  usage_report_interval: 300  # seconds
  
auth:
  require_authentication: true  # Enforce auth on all endpoints
  api_key_cache_ttl: 300  # seconds
  
rate_limiting:
  default_requests_per_minute: 1000
  default_requests_per_hour: 10000
```

### 8. Monitoring & Observability

**Enhanced Metrics**:
- Per-tenant request counts
- Per-tenant storage usage
- Per-tenant rate limit violations
- Authentication failures by tenant
- Quota enforcement events

**New Prometheus Metrics**:
- `vectorizer_tenant_requests_total{tenant_id, operation}`
- `vectorizer_tenant_storage_bytes{tenant_id}`
- `vectorizer_tenant_rate_limit_exceeded_total{tenant_id}`
- `vectorizer_tenant_quota_exceeded_total{tenant_id, quota_type}`

## Impact Analysis

### Breaking Changes

**For Single-Tenant Users**: NONE
- Cluster mode is opt-in via configuration
- Default behavior remains unchanged
- Existing deployments work without modification

**For New Cluster Deployments**: 
- All API calls must include `Authorization` header
- Collections are automatically prefixed with tenant ID

### Performance Considerations

- **Overhead**: ~5-10ms per request for auth validation (mitigated by caching)
- **Storage**: Minimal overhead for tenant ID prefix
- **Memory**: Additional memory for quota tracking (~1MB per 1000 tenants)

### Security Improvements

- All data access authenticated and authorized
- Complete tenant isolation at data layer
- Rate limiting prevents abuse
- Audit trail for all operations

## Migration Path

### Phase 1: Core Infrastructure (Weeks 1-2)
- Implement HiveHub client
- Add authentication middleware
- Implement tenant-scoped data structures

### Phase 2: Quota System (Week 3)
- Implement quota manager
- Add rate limiting
- Implement usage tracking

### Phase 3: API Integration (Week 4)
- Update all REST endpoints
- Update MCP server
- Add admin endpoints

### Phase 4: Testing & Hardening (Week 5)
- Multi-tenant integration tests
- Security audit
- Performance testing
- Documentation

### Phase 5: Production Readiness (Week 6)
- Monitoring setup
- Deployment guides
- HiveHub integration testing
- Beta launch

## Success Criteria

1. ✅ Complete data isolation between tenants verified through security testing
2. ✅ Quota enforcement working with <1% error rate
3. ✅ Rate limiting effective with <50ms overhead
4. ✅ Zero breaking changes for single-tenant deployments
5. ✅ HiveHub integration functional with <99ms API call latency
6. ✅ Documentation complete for cluster deployment
7. ✅ 100+ concurrent tenants tested successfully
8. ✅ All existing tests passing
9. ✅ New multi-tenant test suite with 95%+ coverage

## References

- HiveHub API Documentation: (to be provided)
- Multi-tenant Architecture Best Practices
- OAuth2 and API Key Standards (RFC 6750)
- Rate Limiting Strategies (Token Bucket, Leaky Bucket)

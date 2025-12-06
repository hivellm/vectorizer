## 1. Internal SDK Integration
- [x] 1.1 Add hivehub-internal-sdk to Cargo.toml dependencies
- [x] 1.2 Create src/hub/client.rs for SDK wrapper
- [x] 1.3 Initialize Hub client on server startup
- [x] 1.4 Configure service API key from environment (HIVEHUB_SERVICE_API_KEY)
- [x] 1.5 Implement connection health checks and reconnection logic

## 2. Authentication Module
- [x] 2.1 Create src/hub/auth.rs (moved from src/auth/hub_auth.rs)
- [x] 2.2 Implement Hub access key validation middleware
- [x] 2.3 Extract user_id from validated tokens
- [x] 2.4 Add TenantContext struct to request state
- [x] 2.5 Update API routes to require authentication (x-hivehub-service header bypass)

## 3. Multi-Tenant Collection System
- [x] 3.1 Update Collection struct to include owner_id field
- [x] 3.2 Implement collection naming: user_{user_id}:{collection_name}
- [x] 3.3 Add owner_id to collection metadata
- [x] 3.4 Update collection queries to filter by owner_id (list_collections_for_owner, get_collection_for_owner)
- [x] 3.5 Add ownership validation methods (belongs_to, is_collection_owned_by)

## 4. Hub API Integration
- [x] 4.1 Implement get_user_collections() via SDK
- [x] 4.2 Implement validate_collection() with user verification
- [x] 4.3 Implement create_collection() with quota check
- [x] 4.4 Implement update_usage() for usage reporting
- [x] 4.5 Add error handling for Hub API failures

## 5. Quota Management
- [x] 5.1 Check collection count quota before creation (QuotaManager)
- [x] 5.2 Check vector count quota before insert (QuotaManager)
- [x] 5.3 Check storage quota before operations (QuotaManager)
- [x] 5.4 Return 429 Too Many Requests on quota exceeded
- [x] 5.5 Add quota metrics to monitoring (hub_quota_checks_total, hub_quota_exceeded_total, hub_quota_usage)

## 6. Usage Tracking
- [x] 6.1 Track vector insertions per collection (UsageReporter)
- [x] 6.2 Track storage usage per collection (UsageReporter)
- [x] 6.3 Implement periodic usage sync (configurable interval)
- [x] 6.4 Report usage on collection modifications
- [x] 6.5 Add usage dashboard metrics (hub_api_requests_total, hub_usage_reports_total, hub_backup_operations_total)

## 7. API Updates
- [x] 7.1 Add authentication to POST /collections (via global middleware)
- [x] 7.2 Add authentication to POST /collections/{name}/points (via global middleware)
- [x] 7.3 Add authentication to GET /collections (via global middleware)
- [x] 7.4 Add authentication to search endpoints (via global middleware)
- [x] 7.5 Update API documentation with auth requirements

## 8. MCP Integration
- [x] 8.1 Create src/hub/mcp_gateway.rs
- [x] 8.2 Register MCP server with Hub on startup
- [x] 8.3 Filter MCP responses by user_id
- [x] 8.4 Validate MCP access keys through Hub
- [x] 8.5 Add MCP operation logging

## 9. Cluster Mode
- [x] 9.1 Propagate UserContext across cluster nodes
- [x] 9.2 Implement distributed quota checking
- [x] 9.3 Add user-scoped shard routing
- [x] 9.4 Update replication to preserve user_id
- [x] 9.5 Test multi-node user isolation

## 10. Data Migration
- [x] 10.1 Create migration/hub_migration.rs
- [x] 10.2 Scan existing collections without user_id
- [x] 10.3 Map collections to users (interactive or config-based)
- [x] 10.4 Rename collections with user prefix
- [x] 10.5 Update metadata in storage
- [x] 10.6 Create backup before migration
- [x] 10.7 Add rollback capability

## 11. Configuration
- [x] 11.1 Add [hub] section to config.yml (HubConfig)
- [x] 11.2 Add hub.api_url configuration
- [x] 11.3 Add hub.service_api_key (from env)
- [x] 11.4 Add hub.enabled flag
- [x] 11.5 Add hub.usage_report_interval

## 12. Error Handling
- [x] 12.1 Add Hub error variants to VectorizerError
- [x] 12.2 Handle Hub connection failures gracefully
- [x] 12.3 Add retry logic for Hub API calls (HubClientConfig.retries)
- [x] 12.4 Return proper HTTP status codes
- [x] 12.5 Add detailed error logging

## 13. Testing
- [x] 13.1 Add tests/hub/ integration tests
- [x] 13.2 Mock Hub API for testing
- [x] 13.3 Test multi-tenant isolation
- [x] 13.4 Test quota enforcement (unit tests)
- [x] 13.5 Test usage reporting (unit tests)
- [x] 13.6 Test cluster mode with users
- [x] 13.7 Comprehensive test suite (128 tests)

## 14. Documentation
- [x] 14.1 Create docs/HUB_INTEGRATION.md
- [x] 14.2 Document authentication flow
- [x] 14.3 Document multi-tenancy model
- [x] 14.4 Create migration guide (docs/HUB_MIGRATION.md)
- [x] 14.5 Update README with Hub setup (☁️ HiveHub Cloud Integration section)
- [x] 14.6 Add troubleshooting section

## 15. User-Scoped Backup System
- [x] 15.1 Create src/hub/backup.rs with UserBackupManager
- [x] 15.2 Implement user-isolated backup storage (per user_id directory)
- [x] 15.3 Add backup compression with gzip
- [x] 15.4 Implement SHA-256 checksum verification
- [x] 15.5 Add backup retention/cleanup policies
- [x] 15.6 Create REST API routes for backup operations:
  - GET /api/hub/backups - List user backups
  - POST /api/hub/backups - Create backup
  - GET /api/hub/backups/:id - Get backup info
  - GET /api/hub/backups/:id/download - Download backup file
  - POST /api/hub/backups/restore - Restore from backup
  - POST /api/hub/backups/upload - Upload backup file
  - DELETE /api/hub/backups/:id - Delete backup
- [x] 15.7 Integrate UserBackupManager with VectorizerServer
- [x] 15.8 Add backup quota check integration (estimate_backup_size + quota check before create)
- [x] 15.9 Document backup API in HUB_INTEGRATION.md

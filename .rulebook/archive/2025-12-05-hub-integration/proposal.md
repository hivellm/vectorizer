# Proposal: HiveHub.Cloud Integration

## Why

Vectorizer needs to integrate with HiveHub.Cloud to operate as a managed multi-tenant service with proper user isolation, authentication, quota management, and billing integration. Currently, Vectorizer operates standalone without user context. This integration will enable the HiveHub.Cloud platform to provide Vectorizer as a secure managed service with resource isolation and centralized access control.

## What Changes

Implement HiveHub.Cloud integration in Vectorizer including:

- **Internal SDK Client**: Integrate `hivehub-internal-sdk` for Hub API communication
- **Authentication Layer**: Validate users via Hub-issued access keys
- **Multi-Tenant Collections**: User-scoped collections with `user_{user_id}:{name}` naming
- **Quota Enforcement**: Check user limits before operations via Hub API
- **Usage Reporting**: Track and report vectors, storage to Hub for billing
- **MCP Integration**: Register with Hub's MCP gateway for user access
- **Cluster Support**: User context propagation and distributed quota checking
- **Data Migration**: Tools to migrate existing data to user-scoped model

## Impact

- Affected code:
  - New `src/hub/` - Hub integration module
  - New `src/auth/hub_auth.rs` - Hub authentication
  - Modified `src/collections/` - Multi-tenant collection management
  - Modified `src/api/` - Add user context to API
  - Modified `src/cluster/` - User context in cluster operations
- Breaking change: YES - Requires data migration, API changes
- User benefit: Secure multi-tenant vector database with centralized billing


# Standardize Error Handling

**Change ID**: `standardize-error-handling`  
**Status**: Proposed  
**Priority**: Medium  
**Target Version**: 1.3.0 (or 2.0.0 for breaking changes)

---

## Why

Current error handling mixes `anyhow::Error` and `thiserror::Error` inconsistently. This creates problems:
- SDK developers can't rely on stable error contracts
- Error responses lack structure and context
- Troubleshooting is harder without error types
- No standardized error-to-HTTP-status mapping

---

## What Changes

- Create centralized `src/error.rs` module
- Define structured error types using `thiserror`
- Migrate public APIs to use structured errors
- Update REST/MCP responses with structured format
- Add deprecation warnings (grace period in v1.x)
- **BREAKING** in v2.0.0: Remove string-based errors

---

## Impact

### Affected Capabilities
- **error-handling** (NEW capability)
- **api** (MODIFIED - structured error responses)

### Breaking Changes
- Error response format changes in v2.0.0
- Old SDKs expecting strings will need updates
- Deprecation period: v1.3.0 → v2.0.0

---

## Success Criteria

- ✅ All public APIs use `VectorizerError`
- ✅ Error responses include type, message, details
- ✅ Migration guide for SDK developers
- ✅ Comprehensive error documentation


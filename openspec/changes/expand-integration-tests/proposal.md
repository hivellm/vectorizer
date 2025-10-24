# Expand Integration Test Suite

**Change ID**: `expand-integration-tests`  
**Status**: Proposed  
**Priority**: High  
**Target Version**: 1.3.0

---

## Why

Current test coverage is strong for unit tests (447 passing) but **lacks comprehensive integration tests**:
- Only 21 integration tests (14 of which are ignored)
- Missing end-to-end API workflow tests
- No concurrent operation tests
- No multi-collection scenario tests
- CI doesn't validate complex production scenarios

---

## What Changes

- Create `tests/integration/` directory structure
- Add 30+ new integration tests covering critical paths
- Add test helper utilities
- Run all integration tests in CI
- Target: 50+ total integration tests

Test Categories:
- API workflow tests (CRUD operations)
- Replication failover scenarios
- GPU fallback behavior
- Concurrent operations (race conditions)
- Multi-collection scenarios

---

## Impact

### Affected Capabilities
- **testing** (MODIFIED - expanded coverage)

### Affected Code
- `tests/integration/` - NEW directory
- `tests/helpers/` - NEW test utilities
- `.github/workflows/` - Update to run integration tests

### Breaking Changes
None - tests only.

---

## Success Criteria

- ✅ 50+ integration tests passing
- ✅ All critical API paths covered
- ✅ CI runs integration tests on all platforms
- ✅ Test coverage ≥ 95%


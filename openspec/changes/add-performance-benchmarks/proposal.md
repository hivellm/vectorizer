# Add Performance Benchmarks

**Change ID**: `add-performance-benchmarks`  
**Status**: Proposed  
**Priority**: High  
**Target Version**: 1.2.0  
**Depends On**: `improve-production-readiness` (Phase 1)

---

## Why

Currently **15+ benchmark binaries are disabled** in Cargo.toml, leaving the project without performance regression detection. This creates significant risk:
- Cannot detect performance degradations before production
- No baseline for optimization decisions
- Missing historical performance trends
- CI/CD lacks performance validation

---

## What Changes

- Re-enable all 15+ disabled benchmarks using Criterion framework
- Migrate from `[[bin]]` format to proper `benches/` directory
- Add CI/CD workflow with performance budgets
- Create performance tracking dashboard
- Document benchmarking practices

---

## Impact

### Affected Capabilities
- **performance-testing** (NEW capability)
- **ci-cd** (MODIFIED - add benchmark workflow)

### Affected Code
- `Cargo.toml` - Remove commented benchmarks
- `benches/` - NEW directory with all benchmarks
- `.github/workflows/benchmarks.yml` - NEW workflow
- `docs/BENCHMARKING.md` - NEW documentation

### Breaking Changes
None - purely additive changes.

---

## Success Criteria

- ✅ All 15+ benchmarks running with `cargo bench`
- ✅ CI runs benchmarks on main branch commits
- ✅ Performance budgets enforced (search <5ms, index >1000/s)
- ✅ Historical tracking shows no regressions
- ✅ Dashboard deployed and accessible


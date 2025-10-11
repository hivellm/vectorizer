# Text Normalization - Roadmap

**Feature**: FEAT-NORM-002  
**Status**: 🟡 In Progress  
**Start Date**: 2025-10-11  
**Target Release**: v0.8.0  
**Owner**: HiveLLM Team

---

## Overview

Implement intelligent text normalization to reduce storage footprint by 30-50% while improving embedding consistency and search quality.

**Note**: Vector quantization (SQ-8bit) is already implemented in v0.7.0 with 4x compression + 8.9% quality improvement.

**Key Goals**:
- 📉 Reduce text storage by 30-50%
- 🎯 Improve embedding consistency through normalization
- ⚡ Improve cache hit rates and I/O performance
- 📊 Better deduplication through content hashing

---

## Timeline

```
Week 1-2: Text Normalization Implementation
Week 3:   Cache System Enhancement
Week 4:   Integration & Migration
Week 5:   Testing & Documentation
Week 6:   Production Release
```

---

## Phases

### Phase 1: Text Normalization (Weeks 1-2)

**Status**: 🔴 Not Started  
**Estimated Effort**: 80 hours

#### Tasks

| Task | Status | Assignee | Hours | Dependencies |
|------|--------|----------|-------|--------------|
| **1.1** Design ContentTypeDetector | 🔴 TODO | - | 8 | - |
| **1.2** Implement file extension detection | 🔴 TODO | - | 4 | 1.1 |
| **1.3** Implement content heuristics | 🔴 TODO | - | 12 | 1.1 |
| **1.4** Design TextNormalizer API | 🔴 TODO | - | 8 | - |
| **1.5** Implement Conservative normalization | 🔴 TODO | - | 8 | 1.4 |
| **1.6** Implement Moderate normalization | 🔴 TODO | - | 12 | 1.4 |
| **1.7** Implement Aggressive normalization | 🔴 TODO | - | 12 | 1.4 |
| **1.8** Implement ContentHashCalculator (BLAKE3) | 🔴 TODO | - | 6 | - |
| **1.9** Unit tests (>95% coverage) | 🔴 TODO | - | 16 | 1.2-1.8 |
| **1.10** Benchmarks (throughput, compression) | 🔴 TODO | - | 8 | 1.2-1.8 |

**Deliverables**:
- ✅ `src/normalization/mod.rs`
- ✅ `src/normalization/detector.rs`
- ✅ `src/normalization/normalizer.rs`
- ✅ `src/normalization/hasher.rs`
- ✅ Comprehensive test suite
- ✅ Performance benchmarks

---

### Phase 2: Cache System Enhancement (Week 3)

**Status**: 🔴 Not Started  
**Estimated Effort**: 40 hours

#### Tasks

| Task | Status | Assignee | Hours | Dependencies |
|------|--------|----------|-------|--------------|
| **3.1** Design CacheManager API | 🔴 TODO | - | 6 | - |
| **3.2** Implement hot cache (LFU) | 🔴 TODO | - | 8 | 3.1 |
| **3.3** Implement warm store (mmap) | 🔴 TODO | - | 10 | 3.1 |
| **3.4** Implement cold store (Zstd) | 🔴 TODO | - | 8 | 3.1 |
| **3.5** Cache coherency & versioning | 🔴 TODO | - | 8 | 3.2-3.4 |
| **3.6** Monitoring & metrics | 🔴 TODO | - | 6 | 3.2-3.4 |
| **3.7** Unit tests | 🔴 TODO | - | 8 | 3.2-3.5 |
| **3.8** Benchmarks (hit rate, latency) | 🔴 TODO | - | 6 | 3.2-3.5 |

**Deliverables**:
- ✅ `src/cache/mod.rs`
- ✅ `src/cache/lfu.rs`
- ✅ `src/cache/mmap_store.rs`
- ✅ Cache benchmarks
- ✅ Monitoring dashboard

---

### Phase 3: Integration & Migration (Week 4)

**Status**: 🔴 Not Started  
**Estimated Effort**: 32 hours

#### Tasks

| Task | Status | Assignee | Hours | Dependencies |
|------|--------|----------|-------|--------------|
| **3.1** Integrate normalization into ingestion | 🔴 TODO | - | 8 | Phase 1 |
| **3.2** Integrate cache into search pipeline | 🔴 TODO | - | 6 | Phase 2 |
| **3.3** Query normalization consistency | 🔴 TODO | - | 6 | Phase 1 |
| **3.4** Per-collection configuration | 🔴 TODO | - | 4 | - |
| **3.5** Migration tool for existing collections | 🔴 TODO | - | 8 | Phase 1, 2 |
| **3.6** Feature flags & rollout plan | 🔴 TODO | - | 4 | - |
| **3.7** Integration tests | 🔴 TODO | - | 8 | All phases |

**Deliverables**:
- ✅ End-to-end integration
- ✅ Migration CLI tool
- ✅ Configuration guide
- ✅ Rollout plan

---

### Phase 4: Testing & Documentation (Week 5)

**Status**: 🔴 Not Started  
**Estimated Effort**: 40 hours

#### Tasks

| Task | Status | Assignee | Hours | Dependencies |
|------|--------|----------|-------|--------------|
| **5.1** End-to-end quality tests | 🔴 TODO | - | 12 | Phase 4 |
| **5.2** Performance benchmarks (full suite) | 🔴 TODO | - | 8 | Phase 4 |
| **5.3** Load testing (concurrent ingestion) | 🔴 TODO | - | 6 | Phase 4 |
| **5.4** Load testing (concurrent search) | 🔴 TODO | - | 6 | Phase 4 |
| **5.5** User documentation | 🔴 TODO | - | 8 | - |
| **5.6** API documentation (rustdoc) | 🔴 TODO | - | 4 | - |
| **5.7** Migration guide | 🔴 TODO | - | 6 | Phase 4 |
| **5.8** Troubleshooting FAQ | 🔴 TODO | - | 4 | - |

**Deliverables**:
- ✅ Test reports (quality, performance)
- ✅ Complete documentation
- ✅ Migration guide

---

### Phase 5: Production Release (Week 6)

**Status**: 🔴 Not Started  
**Estimated Effort**: 16 hours

#### Tasks

| Task | Status | Assignee | Hours | Dependencies |
|------|--------|----------|-------|--------------|
| **5.1** Staging deployment | 🔴 TODO | - | 4 | Phase 4 |
| **5.2** Smoke tests in staging | 🔴 TODO | - | 2 | 5.1 |
| **5.3** Production deployment (canary) | 🔴 TODO | - | 4 | 5.2 |
| **5.4** Monitor metrics for 48 hours | 🔴 TODO | - | 2 | 5.3 |
| **5.5** Full rollout | 🔴 TODO | - | 2 | 5.4 |
| **5.6** Release notes | 🔴 TODO | - | 2 | - |
| **5.7** Blog post / announcement | 🔴 TODO | - | 4 | - |

**Deliverables**:
- ✅ v0.8.0 release
- ✅ Release notes
- ✅ Announcement

---

## Dependencies

### External Dependencies

- `blake3` crate (content hashing)
- `unicode-normalization` crate (text normalization)
- `zstd` crate (blob compression)

### Internal Dependencies

- Embedding service API (for consistency validation)
- Collection management
- Storage layer
- **Existing quantization system** (v0.7.0 - already implemented)

---

## Success Metrics

### Quantitative

| Metric | Baseline | Target | Actual |
|--------|----------|--------|--------|
| Text storage reduction | 0% | ≥30% | - |
| Embedding consistency improvement | 85% | ≥95% | - |
| Cache hit rate (dedupe) | 60% | ≥80% | - |
| Normalization overhead | 0ms | <5ms/doc | - |
| Search quality (maintained) | 96.3% | ≥96% | - |

**Note**: Vector memory already optimized with SQ-8bit quantization (75% reduction achieved in v0.7.0)

### Qualitative

- [ ] Zero data loss during migration
- [ ] Seamless user experience
- [ ] Positive user feedback
- [ ] No critical bugs in first month

---

## Risks & Mitigation

### High Priority Risks

| Risk | Impact | Mitigation | Owner |
|------|--------|------------|-------|
| Quality degradation from SQ-8 | HIGH | Extensive A/B testing, adjustable precision | TBD |
| Code normalization breaks semantics | HIGH | Preserve whitespace for code/tables | TBD |
| Migration data loss | CRITICAL | Mandatory backups, rollback plan | TBD |
| Performance regression | MEDIUM | SIMD optimization, profiling | TBD |

### Medium Priority Risks

| Risk | Impact | Mitigation | Owner |
|------|--------|------------|-------|
| Cache memory pressure | MEDIUM | Configurable limits, LFU eviction | TBD |
| Increased CPU usage | LOW | Batch processing, caching | TBD |
| Disk I/O bottleneck | MEDIUM | Compression, async writes | TBD |

---

## Milestones

- [ ] **M1**: Phase 1 complete - Text Normalization (Week 2)
- [ ] **M2**: Phase 2 complete - Cache Enhancement (Week 3)
- [ ] **M3**: Phase 3 complete - Integration (Week 4)
- [ ] **M4**: Phase 4 complete - Testing (Week 5)
- [ ] **M5**: Production release v0.8.0 (Week 6)

**Note**: Quantization milestones already achieved in v0.7.0

---

## Team

### Core Team

- **Tech Lead**: TBD
- **Rust Developer 1**: TBD (Normalization + Quantization)
- **Rust Developer 2**: TBD (Cache + Integration)
- **QA Engineer**: TBD (Testing)
- **Technical Writer**: TBD (Documentation)

### Reviewers

- **Specialist 1**: Performance optimization
- **Specialist 2**: Search quality
- **Judge**: Final approval

---

## Communication

### Status Updates

- **Daily**: Standup (async)
- **Weekly**: Progress report to stakeholders
- **Bi-weekly**: Demo to wider team

### Channels

- GitHub Issues: Task tracking
- Discord: `#vectorizer-normalization` channel
- Task Queue: Automated status updates

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-10-11 | Initial roadmap created |

---

**Last Updated**: 2025-10-11  
**Next Review**: 2025-10-14 (Week 1 checkpoint)


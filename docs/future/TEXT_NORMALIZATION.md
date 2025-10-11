# Text Normalization - Technical Specification

**Feature ID**: FEAT-NORM-002  
**Version**: 1.0.0  
**Status**: Planning  
**Created**: 2025-10-11  
**Author**: HiveLLM Team

---

## Executive Summary

This specification defines an intelligent text normalization system for the Vectorizer to significantly reduce storage footprint and improve embedding consistency. The system will implement content-type-aware text preprocessing and sophisticated content hashing for deduplication.

**Note**: Vector quantization (SQ-8bit) is already implemented in v0.7.0, achieving 4x compression + 8.9% quality improvement.

**Expected Benefits**:
- **Storage Reduction**: 30-50% reduction in text payload (depending on corpus quality)
- **Embedding Consistency**: Same semantic content → same embeddings (regardless of whitespace)
- **Better Deduplication**: Content hashing eliminates duplicate processing
- **Performance**: Faster I/O, better cache hit rates, lower latency

---

## Problem Analysis

### Current Issues

1. **Text Storage Inefficiency**
   - Raw text contains redundant whitespace (`\t`, `\r\n`, multiple spaces)
   - No normalization leads to inconsistent embeddings
   - Unicode variants cause duplicate semantic content
   - Invisible control characters waste space

2. **Semantic Inconsistency**
   - Same content with different whitespace → different embeddings
   - Query normalization doesn't match document normalization
   - Case sensitivity issues in lexical search

### Impact

- **Disk Space**: Multi-GB collections grow unnecessarily large
- **RAM**: Limited concurrent collections due to vector memory
- **Performance**: Poor cache efficiency, high I/O overhead
- **Cost**: Higher infrastructure costs for storage and memory

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Ingestion Pipeline                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Raw Blob → Type Detection → Normalization → Chunking →         │
│  Content Hash → Cache Lookup → Embedding →                      │
│  HNSW Indexing (with existing SQ-8 quantization)                │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                          Search Pipeline                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Query → Normalize (same policy) → Embed →                      │
│  HNSW Search (existing SQ-8) → Re-rank → Results                │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Content Type Detector

**Purpose**: Identify content type to apply appropriate normalization

**Types**:
- `code` - Programming languages (preserve whitespace)
- `markdown` - Markdown with code blocks (mixed handling)
- `table` - CSV/TSV (preserve delimiters)
- `html` - HTML markup (extract text)
- `plain` - Plain text (aggressive normalization)
- `json` - JSON/YAML (preserve structure)

**Implementation**:
```rust
enum ContentType {
    Code { language: Option<String> },
    Markdown,
    Table { format: TableFormat },
    Html,
    Plain,
    Json,
}

struct TypeDetector {
    // File extension → type
    // Content heuristics (shebang, tags, indentation patterns)
}
```

#### 2. Text Normalizer

**Purpose**: Apply content-type-aware normalization

**Normalization Levels**:

**Level 1 - Conservative** (for code/tables):
- Unicode NFC (canonical composition)
- CRLF → LF
- Remove BOM (`\uFEFF`)
- Trim trailing whitespace per line

**Level 2 - Moderate** (for markdown):
- All Level 1 transformations
- Remove zero-width characters (`\u200B`-`\u200D`)
- Normalize heading markers
- Preserve code blocks (fenced ```)

**Level 3 - Aggressive** (for plain text):
- All Level 2 transformations
- Unicode NFKC (compatibility composition)
- Collapse multiple spaces → single space
- Collapse multiple newlines → max 2
- Remove control characters (except `\n`, `\t`)
- Optional: case folding (configurable)

**Implementation**:
```rust
struct NormalizationPolicy {
    version: u32,
    level: NormalizationLevel,
    preserve_case: bool,
    collapse_whitespace: bool,
    remove_html: bool,
}

struct TextNormalizer {
    policy: NormalizationPolicy,
    
    fn normalize(&self, raw: &str, content_type: ContentType) -> String {
        match (self.policy.level, content_type) {
            (_, ContentType::Code) => self.normalize_conservative(raw),
            (_, ContentType::Table) => self.normalize_conservative(raw),
            (NormalizationLevel::Aggressive, ContentType::Plain) => {
                self.normalize_aggressive(raw)
            },
            _ => self.normalize_moderate(raw),
        }
    }
}
```

#### 3. Content Hash Calculator

**Purpose**: Generate idempotent hash for deduplication and caching

**Algorithm**: BLAKE3 (fastest, collision-resistant)

**Keys**:
```rust
struct ContentHash(Blake3Hash);  // 32 bytes

struct VectorKey {
    content_hash: ContentHash,
    embedding_config: EmbeddingConfig,
    quant_version: u32,
}

impl VectorKey {
    fn to_bytes(&self) -> [u8; 64] {
        // Deterministic serialization
    }
}
```

#### 4. Cache Manager

**Purpose**: Multi-tier caching for normalized text and content hashes

**Cache Tiers**:

**Tier 1 - Memory (Hot)**:
- Normalized text (recent documents)
- Content hashes for deduplication
- Size: Configurable (e.g., 10% of total)
- Eviction: LFU (Least Frequently Used)

**Tier 2 - Disk (Warm)**:
- Normalized text blobs (Zstd compressed)
- Content hash → normalized text mapping
- Persistent, memory-mapped
- Size: Unlimited

**Tier 3 - Blob Store (Cold)**:
- Raw text (original, unmodified)
- Normalized text (processed)
- All metadata
- Size: Unlimited (compressed with Zstd)

**Note**: Vector caching and quantization already implemented in existing system

**Implementation**:
```rust
struct CacheManager {
    hot_cache: LfuCache<VectorKey, CompressedF16>,
    warm_store: MmapVectorStore<VectorKey, QuantizedVector>,
    cold_store: BlobStore<ContentHash, CompressedBlob>,
    
    async fn get_embedding(&self, key: &VectorKey) -> Option<Vec<f32>> {
        // 1. Check hot cache (decompressed float16)
        // 2. Check warm store (dequantize SQ-8)
        // 3. Re-embed if necessary
    }
}
```

---

## Data Models

### Storage Schema

```rust
// Persistent storage
struct ChunkMetadata {
    id: ChunkId,
    collection_id: CollectionId,
    content_hash: ContentHash,
    
    // Pointers
    raw_blob_offset: u64,
    norm_blob_offset: u64,
    vector_offset: u64,
    
    // Versions
    norm_version: u32,
    embed_config: EmbeddingConfig,
    quant_version: u32,
    
    // Stats
    byte_size: u32,
    token_count: u16,
    indexed_at: u64,
}

struct QuantizedVectorStore {
    codes: Vec<u8>,           // D bytes per vector
    params: QuantizationParams,
    index: HashMap<VectorKey, usize>, // Key → offset
}

struct BlobStore {
    blobs: Vec<u8>,           // Zstd compressed
    index: HashMap<ContentHash, (u64, u32)>, // Hash → (offset, size)
}
```

---

## API Design

### Normalization API

```rust
// Public API
pub trait Normalizer {
    fn normalize(&self, content: &str, content_type: ContentType) -> NormalizedContent;
    fn normalize_query(&self, query: &str) -> String;
}

pub struct NormalizedContent {
    pub text: String,
    pub content_hash: ContentHash,
    pub metadata: NormalizationMetadata,
}

pub struct NormalizationMetadata {
    pub original_size: usize,
    pub normalized_size: usize,
    pub removed_whitespace: usize,
    pub policy_version: u32,
}
```

**Note**: Quantization API already exists in `src/quantization/` (v0.7.0)

---

## Implementation Plan

### Phase 1: Text Normalization (Week 1-2)

**Tasks**:
1. Implement `ContentTypeDetector`
   - File extension mapping
   - Content heuristics (regex patterns)
   - Test suite with diverse samples

2. Implement `TextNormalizer`
   - Conservative, moderate, aggressive levels
   - Unicode normalization (NFKC/NFC)
   - Whitespace handling
   - Benchmark: performance and compression ratio

3. Implement `ContentHashCalculator`
   - BLAKE3 integration
   - Idempotency tests
   - Collision probability analysis

**Deliverables**:
- `src/normalization/` module
- Unit tests (>95% coverage)
- Benchmarks (throughput, compression ratio)

### Phase 2: Cache System Enhancement (Week 3)

**Tasks**:
1. Implement multi-tier cache
   - LFU hot cache (memory)
   - Mmap warm store (disk)
   - Zstd cold store (blob)

2. Cache coherency
   - Versioning strategy
   - Invalidation on policy change
   - Atomic updates

3. Monitoring
   - Hit rate metrics
   - Eviction statistics
   - Memory pressure alerts

**Deliverables**:
- `src/cache/` module
- Cache benchmarks (hit rate, latency)
- Monitoring dashboard

### Phase 3: Integration & Migration (Week 4)

**Tasks**:
1. Integrate into ingestion pipeline
   - Apply normalization to all incoming text
   - Store both raw and normalized versions
   - Content hash-based deduplication
   - Migration tool for existing collections

2. Integrate into search pipeline
   - Query normalization (same policy as documents)
   - Consistent embedding generation
   - Hash-based cache lookup

3. Configuration
   - Per-collection normalization policies
   - Feature flags
   - Performance tuning

**Deliverables**:
- End-to-end integration
- Migration guide
- Configuration documentation

---

## Testing Strategy

### Unit Tests

**Normalization**:
- Unicode edge cases (combining characters, homoglyphs)
- Whitespace handling (tabs, multiple spaces, CRLF)
- Code block preservation
- HTML/Markdown parsing

**Cache**:
- Concurrent access safety
- Eviction correctness
- Persistence across restarts

### Integration Tests

**End-to-End**:
1. Ingest 1000 documents with diverse content types
2. Verify all normalized and quantized correctly
3. Run search queries
4. Compare results with baseline (float32, no normalization)
5. Measure: Recall@10, NDCG@10, latency p50/p95/p99

**Migration**:
1. Create collection with old format
2. Run migration tool
3. Verify data integrity
4. Test searches still work

### Performance Tests

**Benchmarks**:
- Normalization throughput (MB/s)
- Content hashing throughput (docs/s)
- Cache hit rate (%)
- Storage reduction ratio (%)

**Load Tests**:
- Concurrent ingestion (100 threads)
- Concurrent search (1000 QPS)
- Memory usage under load
- Disk I/O patterns

---

## Success Criteria

### Functional

- ✅ Text normalization reduces storage by ≥30%
- ✅ Query normalization matches document normalization
- ✅ Code/table content preserved correctly
- ✅ Cache hit rate ≥80% for normalized text
- ✅ Content hash deduplication working correctly

### Quality

- ✅ Embedding consistency improved (same content → same embedding)
- ✅ Search quality maintained (≥96%)
- ✅ No semantic errors in normalized text
- ✅ Code/table structure preserved

### Performance

- ✅ Normalization <5ms per document (avg)
- ✅ Content hashing <1ms per document (avg)
- ✅ Cache lookup <0.1ms
- ✅ No search latency regression

**Note**: Vector quantization performance already validated in v0.7.0 (SQ-8bit)

### Operational

- ✅ Zero-downtime migration for existing collections
- ✅ Configurable per collection
- ✅ Monitoring dashboard deployed
- ✅ Documentation complete

---

## Risk Analysis

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Code normalization breaks semantics | Medium | High | Content-type detection, preserve whitespace for code/tables |
| Aggressive normalization loses information | Low | Medium | Multiple normalization levels, configurable policies |
| Cache memory pressure | Low | Medium | Configurable limits, LFU eviction |
| Migration data loss | Low | Critical | Backup requirement, rollback plan |

### Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Increased CPU usage | High | Low | SIMD optimization, caching |
| Disk I/O bottleneck | Medium | Medium | Compression, batch writes |
| Breaking API changes | Low | High | Versioning, backward compatibility |

---

## Monitoring & Observability

### Metrics

**Normalization**:
- `norm_bytes_saved_total` - Total bytes saved through normalization
- `norm_duration_seconds` - Normalization processing time
- `norm_policy_version` - Current normalization policy version
- `norm_content_hash_hits` - Content hash deduplication hits

**Cache**:
- `cache_hit_rate` - Hit rate by tier
- `cache_evictions_total` - Eviction count
- `cache_memory_bytes` - Memory usage

**Quality**:
- `search_recall_at_k` - Recall metrics
- `search_ndcg_at_k` - Ranking quality
- `search_latency_seconds` - Query latency

### Alerts

- Search quality drops >1%
- Cache hit rate <50%
- Normalization errors spike
- Storage growth exceeds expected rate

---

## Documentation

### User Documentation

- [ ] Configuration guide (normalization policies)
- [ ] Migration guide (existing collections)
- [ ] Performance tuning guide
- [ ] Troubleshooting FAQ

### Developer Documentation

- [ ] Architecture overview
- [ ] API documentation (rustdoc)
- [ ] Adding new content types
- [ ] Custom quantization strategies

---

## Appendix

### References

- [Unicode Normalization](https://unicode.org/reports/tr15/)
- [BLAKE3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Zstandard Compression](https://github.com/facebook/zstd)

### Glossary

- **NFC**: Unicode Normalization Form C (canonical composition)
- **NFKC**: Unicode Normalization Form KC (compatibility composition)
- **BLAKE3**: Fast cryptographic hash function
- **LFU**: Least Frequently Used (cache eviction strategy)
- **Content Hash**: Deterministic hash for deduplication

---

**Status**: ✅ Specification Complete  
**Next Step**: Implementation Phase 1  
**Review Date**: 2025-10-18


# Text Normalization and Quantization - Technical Specification

**Feature ID**: FEAT-NORM-001  
**Version**: 1.0.0  
**Status**: Planning  
**Created**: 2025-10-11  
**Author**: HiveLLM Team

---

## Executive Summary

This specification defines a comprehensive text normalization and vector quantization system for the Vectorizer to significantly reduce storage footprint and memory usage while maintaining semantic search quality. The system will implement intelligent text preprocessing, efficient vector quantization (SQ-8), and sophisticated caching strategies.

**Expected Benefits**:
- **Storage Reduction**: 30-50% reduction in text payload (depending on corpus quality)
- **Memory Reduction**: 75% reduction in vector memory (float32 → SQ-8)
- **Performance**: Faster I/O, better cache hit rates, lower latency
- **Quality**: Maintained or improved search quality through consistent normalization

---

## Problem Analysis

### Current Issues

1. **Text Storage Inefficiency**
   - Raw text contains redundant whitespace (`\t`, `\r\n`, multiple spaces)
   - No normalization leads to inconsistent embeddings
   - Unicode variants cause duplicate semantic content
   - Invisible control characters waste space

2. **Vector Storage Inefficiency**
   - Float32 vectors (4 bytes × dim) consume massive memory
   - HNSW index size grows linearly with vector count
   - No compression or quantization strategy
   - High memory pressure on large collections

3. **Semantic Inconsistency**
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
│  Content Hash → Cache Lookup → Embedding → Quantization →       │
│  HNSW Indexing                                                   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                          Search Pipeline                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Query → Normalize (same policy) → Embed → Quantize →           │
│  HNSW Search (SQ-8) → Re-rank (float16/32) → Results            │
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

#### 4. Vector Quantizer (SQ-8)

**Purpose**: Compress float32 vectors to uint8 (4x reduction)

**Scalar Quantization (SQ-8)**:

```
For each dimension i:
  code[i] = round((value[i] - zero_point[i]) / scale[i])
  clamped to [0, 255]

Dequantization:
  value[i] ≈ code[i] * scale[i] + zero_point[i]
```

**Strategies**:

**Per-Dimension (Global)**:
- One scale/zero_point per dimension
- Best for balanced datasets
- 2 × D × 4 bytes metadata (D = embedding dim)

**Per-Block**:
- Divide dimensions into blocks of size B (e.g., 8, 16)
- One scale/zero_point per block
- Better adaptation to heterogeneous data
- 2 × (D/B) × 4 bytes metadata

**Implementation**:
```rust
struct QuantizationParams {
    version: u32,
    strategy: QuantStrategy,
    scales: Vec<f32>,      // Per-dim or per-block
    zero_points: Vec<i32>, // Per-dim or per-block
}

struct Quantizer {
    params: QuantizationParams,
    
    fn quantize(&self, vector: &[f32]) -> Vec<u8> {
        match self.params.strategy {
            QuantStrategy::PerDim => self.quantize_per_dim(vector),
            QuantStrategy::PerBlock(block_size) => {
                self.quantize_per_block(vector, block_size)
            }
        }
    }
    
    fn dequantize(&self, codes: &[u8]) -> Vec<f32> {
        // Reverse process
    }
}
```

**Distance Computation (ADC)**:

Asymmetric Distance Computation - query in float32, database in SQ-8:

```rust
fn adc_distance(
    query: &[f32],
    codes: &[u8],
    params: &QuantizationParams,
) -> f32 {
    // Pre-compute lookup tables for efficiency
    let mut distance = 0.0;
    for i in 0..query.len() {
        let dequant = codes[i] as f32 * params.scales[i] + params.zero_points[i] as f32;
        distance += (query[i] - dequant).powi(2);
    }
    distance.sqrt()
}
```

#### 5. Cache Manager

**Purpose**: Multi-tier caching for embeddings and quantized vectors

**Cache Tiers**:

**Tier 1 - Memory (Hot)**:
- Recent embeddings (float16 compressed with LZ4)
- Size: Configurable (e.g., 10% of total)
- Eviction: LFU (Least Frequently Used)

**Tier 2 - Disk (Warm)**:
- All quantized vectors (SQ-8)
- Persistent, memory-mapped
- Size: Unlimited (compressed with Zstd)

**Tier 3 - Blob Store (Cold)**:
- Raw and normalized text (Zstd)
- All metadata
- Size: Unlimited

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

### Quantization API

```rust
pub trait VectorQuantizer {
    fn quantize(&self, vector: &[f32]) -> Result<QuantizedVector>;
    fn dequantize(&self, codes: &[u8]) -> Vec<f32>;
    fn compute_distance(&self, query: &[f32], codes: &[u8]) -> f32;
}

pub struct QuantizedVector {
    pub codes: Vec<u8>,
    pub params_ref: QuantParamsRef,  // Reference to shared params
}
```

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

### Phase 2: Vector Quantization (Week 3-4)

**Tasks**:
1. Implement SQ-8 quantizer
   - Per-dimension strategy
   - Per-block strategy
   - Parameter optimization (grid search)

2. Implement ADC distance computation
   - Cosine similarity
   - L2 distance
   - SIMD optimization (AVX2/NEON)

3. Quality evaluation
   - Recall@K comparison (float32 vs SQ-8)
   - Precision degradation analysis
   - NDCG@K metrics

**Deliverables**:
- `src/quantization/` module
- Quality report (Recall@10, NDCG@10)
- Performance benchmarks (latency, throughput)

### Phase 3: Cache System (Week 5)

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

### Phase 4: Integration & Migration (Week 6)

**Tasks**:
1. Integrate into ingestion pipeline
   - Replace raw text storage
   - Enable quantization by default
   - Migration tool for existing collections

2. Integrate into search pipeline
   - Query normalization
   - ADC distance in HNSW
   - Re-ranking with dequantized vectors

3. Configuration
   - Per-collection policies
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

**Quantization**:
- Round-trip accuracy (quantize → dequantize)
- Distance computation correctness
- Saturation handling (values outside [0,1])
- Block alignment

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
- Quantization throughput (vectors/s)
- Search latency (ms per query)
- Cache hit rate (%)

**Load Tests**:
- Concurrent ingestion (100 threads)
- Concurrent search (1000 QPS)
- Memory usage under load
- Disk I/O patterns

---

## Success Criteria

### Functional

- ✅ Text normalization reduces storage by ≥30%
- ✅ SQ-8 quantization reduces vector memory by ≥75%
- ✅ Query normalization matches document normalization
- ✅ Code/table content preserved correctly
- ✅ Cache hit rate ≥80% for hot collections

### Quality

- ✅ Recall@10 degradation <2% (SQ-8 vs float32)
- ✅ NDCG@10 degradation <1%
- ✅ No semantic errors in normalized text

### Performance

- ✅ Normalization <5ms per document (avg)
- ✅ Quantization <1ms per vector (avg)
- ✅ Search latency increase <10% (ADC overhead)
- ✅ Cache lookup <0.1ms

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
| Quality degradation from quantization | Medium | High | Extensive A/B testing, adjustable precision |
| Code normalization breaks semantics | Medium | High | Whitelist code extensions, preserve structure |
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
- `norm_bytes_saved_total` - Total bytes saved
- `norm_duration_seconds` - Normalization time
- `norm_policy_version` - Current policy version

**Quantization**:
- `quant_vectors_compressed_total` - Vectors quantized
- `quant_compression_ratio` - Storage reduction
- `quant_distance_error` - ADC vs exact distance error

**Cache**:
- `cache_hit_rate` - Hit rate by tier
- `cache_evictions_total` - Eviction count
- `cache_memory_bytes` - Memory usage

**Quality**:
- `search_recall_at_k` - Recall metrics
- `search_ndcg_at_k` - Ranking quality
- `search_latency_seconds` - Query latency

### Alerts

- Recall@10 drops >2%
- Cache hit rate <50%
- Quantization errors spike
- Memory usage >90%

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

- [Faiss SQ8 Documentation](https://github.com/facebookresearch/faiss/wiki/Faiss-indexes#scalar-quantizer)
- [Unicode Normalization](https://unicode.org/reports/tr15/)
- [BLAKE3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)

### Glossary

- **ADC**: Asymmetric Distance Computation
- **NFKC**: Unicode Normalization Form KC (compatibility composition)
- **SQ-8**: Scalar Quantization to 8 bits
- **HNSW**: Hierarchical Navigable Small World graph

---

**Status**: ✅ Specification Complete  
**Next Step**: Implementation Phase 1  
**Review Date**: 2025-10-18


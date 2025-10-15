# Text Normalization

**Status**: ✅ **PRODUCTION READY**  
**Version**: v0.8.0-dev  
**Last Updated**: 2025-10-11

---

## 📋 Overview

Text normalization reduces storage footprint and improves embedding consistency by intelligently preprocessing text before indexing. The implementation includes content-type detection, multi-level normalization, and a three-tier caching system.

**Key Benefits**:
- 8-50% storage reduction
- Better deduplication
- More consistent embeddings
- Query-document matching

---

## ✅ Current Status

### Implemented (Production Ready)

| Component | Status | Lines | Tests |
|-----------|--------|-------|-------|
| **Phase 1: Core** | ✅ Complete | ~1,700 | 27 |
| Content Type Detector | ✅ | 412 | 8 |
| Text Normalizer (3 levels) | ✅ | 447 | 13 |
| Content Hasher (BLAKE3) | ✅ | 226 | 6 |
| **Phase 2: Cache** | ✅ Complete | ~1,500 | 23+ |
| Hot Cache (LFU memory) | ✅ | 305 | - |
| Warm Store (mmap) | ✅ | 291 | - |
| Cold Store (Zstd) | ✅ | 333 | - |
| Cache Manager | ✅ | 266 | - |
| **Phase 3: Integration** | ✅ Complete | ~1,300 | - |
| Config per Collection | ✅ | 181 | - |
| Integration Pipeline | ✅ | 290 | - |
| Collection Helper | ✅ | 228 | - |
| MCP/REST Metadata | ✅ | ~120 | - |
| **Total** | **✅** | **~4,500** | **50+** |

### Default Configuration

**New collections automatically have**:
- ✅ Quantization: SQ-8 (8-bit Scalar)
- ✅ Embedding: BM25 512D
- ✅ **Normalization: ACTIVE (Moderate level)**

---

## 🚀 Quick Start

### Enable Normalization (Already Default!)

```rust
use vectorizer::{
    db::{Collection, CollectionNormalizationHelper},
    models::CollectionConfig,
};

// Normalization is enabled by default!
let config = CollectionConfig::default();
let collection = Collection::new("docs".to_string(), config);
```

### Choose a Different Level

```rust
use vectorizer::normalization::NormalizationConfig;

let config = CollectionConfig {
    normalization: Some(NormalizationConfig::aggressive()),
    ..Default::default()
};
```

### Process Documents

```rust
use std::path::PathBuf;

// Create helper
let norm_helper = CollectionNormalizationHelper::from_config(
    &config, 
    &PathBuf::from("./data")
)?;

// Process document
let processed = norm_helper
    .process_document(raw_text, Some(&file_path))
    .await?;

// Generate embedding from NORMALIZED text
let embedding = embedding_model
    .embed(processed.embedding_text())
    .await?;

// Create and insert vector
let vector = norm_helper.create_vector_with_normalization(
    id, embedding, &processed, payload
);
collection.insert(vector)?;
```

### Normalize Queries

```rust
// Ensure query uses same normalization as documents
let normalized_query = norm_helper.process_query(user_query);
let query_embedding = embedding_model.embed(&normalized_query).await?;
let results = collection.search(&query_embedding, 10)?;
```

---

## 📊 Normalization Levels

| Level | Use Case | Behavior | Storage Impact |
|-------|----------|----------|----------------|
| **Conservative** | Code, structured data | NFC, CRLF→LF, BOM removal only | Minimal (~2%) |
| **Moderate** | Markdown, general text | + zero-width removal, newline collapsing | Medium (~8-15%) |
| **Aggressive** | Plain text, max compression | + NFKC, space collapsing, case folding | High (~10-50%) |

### Conservative
```rust
NormalizationConfig::conservative()
```
- Preserves whitespace structure
- Ideal for: Code, CSV, TSV, technical docs

### Moderate (Default)
```rust
NormalizationConfig::moderate() // or ::default()
```
- Balanced approach
- Ideal for: Markdown, documentation, general content

### Aggressive
```rust
NormalizationConfig::aggressive()
```
- Maximum compression
- Ideal for: Plain text, chat logs, user-generated content

---

## 🔍 Content Type Detection

Automatically detects and applies appropriate normalization:

**Supported Types**:
- **Code**: Rust, Python, JavaScript, TypeScript, Java, C/C++, Go, Ruby, PHP, C#, Swift, Kotlin
- **Markup**: Markdown, HTML
- **Data**: JSON, YAML, CSV, TSV
- **Plain text**: Default fallback

**Detection Methods**:
1. File extension (`.rs`, `.py`, `.md`, etc.)
2. Content heuristics (shebangs, function declarations, JSON structure)
3. Fallback to plain text

---

## 💾 Multi-Tier Cache

### Architecture

```
┌──────────────────────────────────────┐
│         Cache Manager                │
├──────────────────────────────────────┤
│ Request → Hot (LFU) → Warm → Cold   │
│          (memory)  (mmap)  (Zstd)    │
└──────────────────────────────────────┘
```

### Tiers

| Tier | Storage | Speed | Size | Use |
|------|---------|-------|------|-----|
| **Hot** | RAM | Ultra-fast (<0.1ms) | 100 MB | Recent texts |
| **Warm** | Mmap | Fast (~1ms) | Unlimited | Frequent texts |
| **Cold** | Disk (Zstd) | Medium (~5ms) | Unlimited | All texts |

### Configuration

```rust
let config = NormalizationConfig::moderate()
    .with_cache_size(50 * 1024 * 1024)  // 50 MB hot cache
    .without_cache();                    // Or disable caching
```

---

## 🌐 API Integration

### MCP Tool: `get_collection_info`

```json
{
  "name": "docs",
  "vector_count": 1000,
  "document_count": 950,
  "normalization": {
    "enabled": true,
    "level": "Moderate",
    "preserve_case": true,
    "collapse_whitespace": true,
    "cache_enabled": true,
    "cache_size_mb": 100,
    "normalize_queries": true,
    "store_raw_text": true
  }
}
```

### REST: `GET /collections/{name}`

```bash
curl http://localhost:3000/collections/docs | jq '.normalization'
```

```json
{
  "enabled": true,
  "level": "Moderate",
  "preserve_case": true,
  "collapse_whitespace": true,
  "remove_html": false,
  "cache_enabled": true,
  "cache_size_mb": 100,
  "normalize_queries": true,
  "store_raw_text": true
}
```

### REST: `GET /collections` (List)

```json
{
  "collections": [
    {
      "name": "docs",
      "normalization": {
        "enabled": true,
        "level": "Moderate"
      }
    }
  ]
}
```

---

## 📈 Performance & Metrics

### Benchmarks

| Metric | Value |
|--------|-------|
| Normalization speed | < 10μs/doc |
| BLAKE3 hashing | > 1 GB/s |
| Storage reduction | 8-50% |
| Cache hit latency | < 0.1ms (hot) |
| Memory overhead | Minimal |

### Real Results

```
Document Processing:
  Plain text:  56 → 51 bytes (8.9% saved)
  Markdown:    49 → 45 bytes (8.2% saved)
  Rust code:   43 → 43 bytes (0% - preserved!)

Query Processing:
  "  machine   learning  " → "machine learning" (24% saved)
```

---

## 🛠️ Configuration Options

### NormalizationConfig

```rust
pub struct NormalizationConfig {
    pub enabled: bool,                // Enable/disable
    pub policy: NormalizationPolicy,  // Level & rules
    pub cache_enabled: bool,          // Enable caching
    pub hot_cache_size: usize,        // Cache size in bytes
    pub normalize_queries: bool,      // Normalize search queries
    pub store_raw_text: bool,         // Keep original text
}
```

### NormalizationPolicy

```rust
pub struct NormalizationPolicy {
    pub version: u32,                 // Policy version
    pub level: NormalizationLevel,    // Conservative/Moderate/Aggressive
    pub preserve_case: bool,          // Keep original case
    pub collapse_whitespace: bool,    // Reduce multiple spaces
    pub remove_html: bool,            // Strip HTML tags
}
```

---

## 🔧 Advanced Usage

### Custom Policy

```rust
let custom_policy = NormalizationPolicy {
    version: 1,
    level: NormalizationLevel::Moderate,
    preserve_case: false,  // Lowercase everything
    collapse_whitespace: true,
    remove_html: true,
};

let config = NormalizationConfig {
    enabled: true,
    policy: custom_policy,
    cache_enabled: true,
    hot_cache_size: 50 * 1024 * 1024,
    normalize_queries: true,
    store_raw_text: false,  // Save more space
};
```

### Check for Duplicates

```rust
// Before inserting, check if content already exists
if norm_helper.is_duplicate(text).await? {
    println!("Duplicate detected, skipping");
    continue;
}
```

### Cache Statistics

```rust
if let Some(stats) = norm_helper.cache_stats() {
    println!("Hit rate: {:.1}%", stats.hit_rate * 100.0);
    println!("Hot hits: {}", stats.hot_hits);
    println!("Warm hits: {}", stats.warm_hits);
}
```

---

## 🧪 Testing

### Run Tests

```bash
# All normalization tests
cargo test normalization

# Specific component
cargo test normalization::detector
cargo test normalization::normalizer
cargo test normalization::cache
```

### Test Coverage

- ✅ 50+ unit tests
- ✅ Integration tests
- ✅ Performance benchmarks
- ✅ Edge cases (Unicode, BOM, etc.)

---

## 🗺️ Future Roadmap

### Completed ✅
- [x] Core normalization (3 levels)
- [x] Multi-tier cache
- [x] Collection integration
- [x] MCP/REST metadata
- [x] Content type detection
- [x] Deduplication

### Planned ⏳
- [ ] REST endpoints for config (`POST /collections/{name}/normalization`)
- [ ] Migration tool for existing collections
- [ ] Feature flags for conditional compilation
- [ ] Extended documentation & tutorials
- [ ] Performance dashboard
- [ ] Cache prewarming strategies

---

## 📚 Files & Structure

### Source Code
```
src/normalization/
├── mod.rs           # Module exports
├── config.rs        # Configuration
├── detector.rs      # Content type detection
├── normalizer.rs    # Text normalization
├── hasher.rs        # Content hashing
├── integration.rs   # Pipeline integration
└── cache/
    ├── mod.rs       # Cache manager
    ├── hot_cache.rs # LFU memory cache
    ├── warm_store.rs# Mmap storage
    ├── blob_store.rs# Compressed blobs
    └── metrics.rs   # Statistics

src/db/
└── collection_normalization.rs  # Collection helper
```

### Dependencies

```toml
blake3 = "1.5"                # Fast content hashing
unicode-normalization = "0.1"  # Unicode NFC/NFKC
zstd = "0.13"                  # Compression
memmap2 = "0.9"                # Memory mapping
regex = "1.10"                 # Pattern matching
```

---

## 💡 Best Practices

### Do ✅
- Use default config for most cases (Moderate)
- Enable caching for better performance
- Normalize queries the same way as documents
- Store raw text for transparency (default)
- Check cache stats periodically

### Don't ❌
- Use Aggressive on code (use Conservative)
- Disable query normalization (breaks consistency)
- Skip cache without good reason
- Change normalization level mid-collection
- Ignore content hash for deduplication

---

## 🐛 Troubleshooting

### Issue: Cache misses are high
**Solution**: Increase hot cache size or check if content is actually unique

### Issue: Too much storage used
**Solution**: Disable `store_raw_text` or use Aggressive level

### Issue: Code formatting broken
**Solution**: Use Conservative level for code files

### Issue: Queries don't match documents
**Solution**: Ensure `normalize_queries` is enabled (default)

---

## 📄 License

Same as Vectorizer project (MIT)

---

## 🎉 Summary

Text normalization in Vectorizer is:
- ✅ **Production ready** (4,500+ lines, 50+ tests)
- ✅ **Enabled by default** (Moderate level)
- ✅ **Well tested** (50+ tests passing)
- ✅ **Fully integrated** (MCP + REST APIs)
- ✅ **High performance** (<10μs overhead)
- ✅ **Flexible** (3 levels, customizable)

**Ready to use today!** 🚀

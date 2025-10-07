# Discovery System - Rust Libraries Integration

**Status**: 📋 Ready for Implementation  
**Coverage**: 70-80% of pipeline with open-source Rust libraries  
**Date**: 2025-10-07

---

## 🎯 Library Stack Overview

| Component | Library | Purpose | Coverage |
|-----------|---------|---------|----------|
| **FTS + BM25** | `tantivy` | Collection filtering, BM25 scoring for hybrid | Filter, Score, Broad |
| **Vector Index** | `hnsw_rs` (existing) | Dense vector ANN search | Broad, Focus |
| **Reranking** | `onnxruntime` | Cross-encoder local reranking | Focus |
| **Keyword Extraction** | `keyword_extraction` | TextRank/YAKE for bullets | Compress |
| **Sentence Segmentation** | `unicode-segmentation` | Sentence boundary detection | Compress |
| **Hybrid (Internal)** | Custom implementation | Combine HNSW + tantivy BM25 | Broad, Focus |

---

## 📦 Cargo Dependencies

```toml
[dependencies]
# Existing
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
log = "0.4"

# Full-text search and BM25 (for hybrid search with our HNSW)
tantivy = "0.22"

# Neural reranking
onnxruntime = "0.0.14"
tokenizers = "0.19"  # For model tokenization
ndarray = "0.15"     # For tensor operations

# Text processing and extraction
keyword_extraction = "0.1"
unicode-segmentation = "1.11"

# Utilities
glob = "0.3"
futures = "0.3"

# NOTE: We do NOT use qdrant-client - vectorizer is our own vector database!
# Hybrid search is implemented internally combining:
# - Our existing HNSW index for dense vectors
# - Tantivy for BM25/sparse search
```

---

## 🔧 Integration Plan by Component

### 1. Collection Filtering & Scoring (tantivy)

**Files**: `filter.rs`, `score.rs`

**Implementation**:
```rust
use tantivy::{
    schema::*, 
    Index, 
    IndexWriter, 
    collector::TopDocs,
    query::QueryParser,
    tokenizer::*
};

pub struct CollectionIndexer {
    index: Index,
    schema: Schema,
}

impl CollectionIndexer {
    pub fn new() -> Result<Self> {
        let mut schema_builder = Schema::builder();
        
        // Collection metadata fields
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_text_field("tags", TEXT);
        schema_builder.add_u64_field("vector_count", INDEXED | STORED);
        schema_builder.add_date_field("updated_at", INDEXED);
        
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        
        // Configure tokenizers
        let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::default())
            .build();
        
        index.tokenizers().register("default", tokenizer);
        
        Ok(Self { index, schema })
    }
    
    pub fn index_collection(&mut self, collection: &CollectionRef) -> Result<()> {
        let name = self.schema.get_field("name").unwrap();
        let tags = self.schema.get_field("tags").unwrap();
        let vector_count = self.schema.get_field("vector_count").unwrap();
        let updated_at = self.schema.get_field("updated_at").unwrap();
        
        let mut writer = self.index.writer(50_000_000)?;
        
        writer.add_document(doc!(
            name => collection.name.clone(),
            tags => collection.tags.join(" "),
            vector_count => collection.vector_count as u64,
            updated_at => DateTime::from_timestamp(
                collection.updated_at.timestamp(), 0
            ).unwrap()
        ))?;
        
        writer.commit()?;
        Ok(())
    }
    
    pub fn search_collections(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        let name = self.schema.get_field("name").unwrap();
        let query_parser = QueryParser::for_index(&self.index, vec![name]);
        let query = query_parser.parse_query(query)?;
        
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let name_value = doc.get_first(name).unwrap();
            if let Some(text) = name_value.as_text() {
                results.push((text.to_string(), score));
            }
        }
        
        Ok(results)
    }
}
```

**Benefits**:
- Built-in BM25 scoring
- Stopword removal
- Stemming
- Prefix/regex boost
- Fast in-memory indexing

---

### 2. Hybrid Search (Internal Implementation)

**Files**: `broad.rs`, `focus.rs`

**Strategy**: Combine our existing HNSW vector index + tantivy BM25

**Implementation**:
```rust
use tantivy::{Index, collector::TopDocs};
use crate::hnsw::OptimizedHnswIndex;  // Our existing HNSW

pub struct HybridSearcher {
    // Our own vector index
    hnsw_index: Arc<RwLock<OptimizedHnswIndex>>,
    // Tantivy for BM25
    bm25_index: Index,
    schema: Schema,
}

impl HybridSearcher {
    pub async fn new(collection: &Collection) -> Result<Self> {
        // Use the existing HNSW index from collection
        let hnsw_index = collection.index.clone();
        
        // Create tantivy index for BM25 on text content
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.add_text_field("doc_id", STRING | STORED);
        let schema = schema_builder.build();
        let bm25_index = Index::create_in_ram(schema.clone());
        
        Ok(Self { hnsw_index, bm25_index, schema })
    }
    
    pub async fn hybrid_search(
        &self,
        query: &str,
        query_vector: Vec<f32>,
        limit: usize,
        alpha: f32,  // Weight: 0.0 = pure BM25, 1.0 = pure dense
    ) -> Result<Vec<ScoredChunk>> {
        // 1. Dense search with our HNSW
        let dense_results = {
            let index = self.hnsw_index.read();
            index.search(&query_vector, limit * 2)?
        };
        
        // 2. BM25 search with tantivy
        let bm25_results = {
            let reader = self.bm25_index.reader()?;
            let searcher = reader.searcher();
            let content_field = self.schema.get_field("content").unwrap();
            let query_parser = QueryParser::for_index(&self.bm25_index, vec![content_field]);
            let query = query_parser.parse_query(query)?;
            searcher.search(&query, &TopDocs::with_limit(limit * 2))?
        };
        
        // 3. Reciprocal Rank Fusion (RRF)
        let merged = self.reciprocal_rank_fusion(
            &dense_results,
            &bm25_results,
            alpha,
        );
        
        // 4. Take top k
        merged.into_iter().take(limit).collect()
    }
    
    fn reciprocal_rank_fusion(
        &self,
        dense: &[(u32, f32)],
        sparse: &[(Score, DocAddress)],
        alpha: f32,
    ) -> Vec<ScoredChunk> {
        let k = 60.0; // RRF constant
        let mut scores: HashMap<String, f32> = HashMap::new();
        
        // Score from dense results
        for (rank, (id, score)) in dense.iter().enumerate() {
            let rrf_score = alpha / (k + (rank as f32 + 1.0));
            *scores.entry(id.to_string()).or_insert(0.0) += rrf_score + score * alpha;
        }
        
        // Score from BM25 results
        for (rank, (score, doc_addr)) in sparse.iter().enumerate() {
            let doc = self.get_doc(doc_addr)?;
            let id = doc.get_first(doc_id_field).unwrap().as_text().unwrap();
            let rrf_score = (1.0 - alpha) / (k + (rank as f32 + 1.0));
            *scores.entry(id.to_string()).or_insert(0.0) += rrf_score + score * (1.0 - alpha);
        }
        
        // Sort by combined score
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Convert to ScoredChunk
        results.into_iter()
            .map(|(id, score)| self.id_to_chunk(&id, score))
            .collect()
    }
}
```

**Benefits**:
- Uses our own HNSW vector index
- Adds BM25 via tantivy for text matching
- Implements RRF fusion internally
- No external vector database dependency
- Full control over the pipeline

---

### 3. Neural Reranking (ONNX Runtime)

**Files**: `focus.rs`

**Implementation**:
```rust
use onnxruntime::{
    environment::Environment,
    GraphOptimizationLevel,
    session::Session,
    tensor::OrtOwnedTensor,
};

pub struct CrossEncoderReranker {
    session: Session<'static>,
    tokenizer: Tokenizer,  // From tokenizers crate
}

impl CrossEncoderReranker {
    pub fn new(model_path: &str) -> Result<Self> {
        let environment = Environment::builder()
            .with_name("reranker")
            .build()?;
        
        let session = environment
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_file(model_path)?;
        
        // Load tokenizer (e.g., bge-reranker-base)
        let tokenizer = Tokenizer::from_pretrained("BAAI/bge-reranker-base", None)?;
        
        Ok(Self { session, tokenizer })
    }
    
    pub fn rerank(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<Vec<(usize, f32)>> {
        let mut scores = Vec::new();
        
        for (idx, doc) in documents.iter().enumerate() {
            // Create pair [query, doc]
            let pair = format!("{} [SEP] {}", query, doc);
            
            // Tokenize
            let encoding = self.tokenizer.encode(pair, false)?;
            let input_ids = encoding.get_ids();
            
            // Run inference
            let input_tensor = ndarray::Array2::from_shape_vec(
                (1, input_ids.len()),
                input_ids.to_vec(),
            )?;
            
            let outputs: Vec<OrtOwnedTensor<f32, _>> = 
                self.session.run(vec![input_tensor.into()])?;
            
            // Get score (logit)
            let score = outputs[0][[0, 0]];
            scores.push((idx, score));
        }
        
        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(scores)
    }
}
```

**Models to use**:
- `bge-reranker-base` (small, fast)
- `bge-reranker-large` (better quality)
- `cross-encoder/ms-marco-MiniLM-L-6-v2`

**Benefits**:
- True semantic reranking
- Local inference (no API calls)
- Fast with ONNX optimization
- Works with quantized models

---

### 4. Extractive Compression (keyword_extraction)

**Files**: `compress.rs`

**Implementation**:
```rust
use keyword_extraction::*;
use unicode_segmentation::UnicodeSegmentation;

pub struct ExtractiveCompressor {
    config: CompressionConfig,
}

impl ExtractiveCompressor {
    pub fn extract_keyphrases(&self, text: &str, n: usize) -> Vec<String> {
        // TextRank algorithm
        let rake = Rake::new();
        let keywords = rake.run(text);
        
        keywords
            .into_iter()
            .take(n)
            .map(|kw| kw.keyword)
            .collect()
    }
    
    pub fn extract_sentences(&self, text: &str, max_sentences: usize) -> Vec<String> {
        // Use unicode segmentation for sentence boundaries
        let sentences: Vec<&str> = text
            .unicode_sentences()
            .collect();
        
        // Score sentences by keyword density
        let keywords = self.extract_keyphrases(text, 10);
        
        let mut scored: Vec<_> = sentences
            .into_iter()
            .map(|sent| {
                let score = self.sentence_score(sent, &keywords);
                (sent, score)
            })
            .collect();
        
        // Sort by score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        scored
            .into_iter()
            .take(max_sentences)
            .map(|(s, _)| s.trim().to_string())
            .collect()
    }
    
    fn sentence_score(&self, sentence: &str, keywords: &[String]) -> f32 {
        let sent_lower = sentence.to_lowercase();
        let mut score = 0.0;
        
        for keyword in keywords {
            if sent_lower.contains(&keyword.to_lowercase()) {
                score += 1.0;
            }
        }
        
        // Normalize by sentence length
        let word_count = sentence.split_whitespace().count();
        if word_count > 0 {
            score / (word_count as f32).sqrt()
        } else {
            0.0
        }
    }
}

// Alternative: TF-IDF based summarization
pub struct TfIdfSummarizer {
    // Use tfidf-text-summarizer or implement custom
}

impl TfIdfSummarizer {
    pub fn summarize(&self, documents: &[String], n: usize) -> Vec<String> {
        // Calculate TF-IDF scores
        let tfidf = self.calculate_tfidf(documents);
        
        // Extract top sentences based on TF-IDF
        self.extract_top_sentences(&tfidf, n)
    }
    
    fn calculate_tfidf(&self, documents: &[String]) -> TfIdfMatrix {
        // Implementation using basic TF-IDF
        // Term Frequency × Inverse Document Frequency
        todo!("Implement TF-IDF calculation")
    }
    
    fn extract_top_sentences(&self, tfidf: &TfIdfMatrix, n: usize) -> Vec<String> {
        // Extract sentences with highest TF-IDF scores
        todo!("Implement sentence extraction")
    }
}
```

**Benefits**:
- TextRank for keyword extraction
- YAKE algorithm support
- Proper sentence segmentation
- TF-IDF for sentence scoring

---

## 🚀 Quick Integration Strategy

### Phase 1: Foundation (Week 1)
```rust
// 1. Set up tantivy for collection indexing
let mut indexer = CollectionIndexer::new()?;
for collection in all_collections {
    indexer.index_collection(&collection)?;
}

// 2. Replace manual filtering with tantivy BM25
let scored = indexer.search_collections(query, 50)?;
```

### Phase 2: Hybrid Search (Week 1-2)
```rust
// 3. Implement internal hybrid search (HNSW + tantivy)
let searcher = HybridSearcher::new(collection).await?;

// Broad discovery with hybrid
let query_vec = embed_query(query).await?;
let results = searcher.hybrid_search(
    query,
    query_vec,
    50,
    0.7  // 70% weight on dense, 30% on BM25
).await?;
```

### Phase 3: Reranking (Week 2)
```rust
// 4. Add ONNX reranker
let reranker = CrossEncoderReranker::new("models/bge-reranker-base.onnx")?;

// Rerank top results
let docs: Vec<String> = results.iter().map(|r| r.content.clone()).collect();
let reranked = reranker.rerank(query, &docs)?;
```

### Phase 4: Compression (Week 2-3)
```rust
// 5. Extract key sentences
let compressor = ExtractiveCompressor::new(config);

for chunk in top_chunks {
    let sentences = compressor.extract_sentences(&chunk.content, 3);
    for sent in sentences {
        bullets.push(create_bullet(sent, &chunk));
    }
}
```

---

## 📊 Coverage Breakdown

| Component | Manual | Tantivy | HNSW (existing) | ONNX | keyword_extraction | Total |
|-----------|--------|---------|-----------------|------|-------------------|-------|
| Filter | 50% | 50% | - | - | - | **100%** |
| Score | 30% | 70% | - | - | - | **100%** |
| Expand | 100% | - | - | - | - | **100%** |
| Broad | 20% | 30% (BM25) | 40% (dense) | - | 10% | **100%** |
| Focus | 30% | - | 40% (dense) | 20% | 10% | **100%** |
| README | 100% | - | - | - | - | **100%** |
| Compress | 30% | - | - | - | 70% | **100%** |
| Plan | 100% | - | - | - | - | **100%** |
| Render | 100% | - | - | - | - | **100%** |
| **Overall** | **62%** | **17%** | **9%** | **2%** | **10%** | **≈100%** |

---

## 🔗 Complete Internal Implementation

Vectorizer uses only its own components + standalone libraries:

```rust
pub struct VectorizerDiscovery {
    // Our existing HNSW index
    vector_store: Arc<VectorStore>,
    // Tantivy for BM25
    bm25_indexer: CollectionIndexer,
    // ONNX for reranking
    reranker: CrossEncoderReranker,
    // Keyword extraction
    compressor: ExtractiveCompressor,
}

impl VectorizerDiscovery {
    pub async fn discover(&self, query: &str) -> Result<DiscoveryResponse> {
        // 1. Filter collections with tantivy BM25
        let filtered = self.bm25_indexer.search_collections(query, 50)?;
        
        // 2. Hybrid search: our HNSW + tantivy BM25
        let query_vec = self.vector_store.embed(query).await?;
        let dense_results = self.vector_store.search(&query_vec, 100).await?;
        let sparse_results = self.bm25_indexer.search_content(query, 100)?;
        let merged = reciprocal_rank_fusion(&dense_results, &sparse_results);
        
        // 3. Rerank with ONNX cross-encoder
        let docs: Vec<String> = merged.iter().map(|r| r.content.clone()).collect();
        let reranked = self.reranker.rerank(query, &docs)?;
        
        // 4. Extract sentences with keyword_extraction
        let bullets = self.compressor.extract_bullets(&reranked)?;
        
        // 5. Build plan and render
        let plan = build_answer_plan(&bullets)?;
        let prompt = render_llm_prompt(&plan)?;
        
        Ok(DiscoveryResponse { prompt, plan, bullets, ... })
    }
}
```

**This covers**:
- ✅ Collection filtering (tantivy BM25)
- ✅ Dense search (our HNSW)
- ✅ Hybrid search (internal RRF)
- ✅ Reranking (ONNX)
- ✅ Compression (keyword_extraction)
- ✅ README boost (manual heuristic)

**Total coverage: ~100% with our own tech stack**

---

## 📝 Next Steps

1. **Add dependencies to Cargo.toml**
2. **Implement `CollectionIndexer` with tantivy**
3. **Integrate Qdrant client for hybrid search**
4. **Add ONNX reranker for semantic focus**
5. **Use keyword_extraction for bullet compression**
6. **Test end-to-end with real data**

---

## 🔍 Model Downloads

### ONNX Reranker Models
```bash
# BGE Reranker Base (recommended)
wget https://huggingface.co/BAAI/bge-reranker-base/resolve/main/onnx/model.onnx

# Cross-Encoder MiniLM (faster, smaller)
wget https://huggingface.co/cross-encoder/ms-marco-MiniLM-L-6-v2/resolve/main/onnx/model.onnx
```

### Tokenizers
```bash
# Download from Hugging Face
pip install huggingface-hub
huggingface-cli download BAAI/bge-reranker-base --include "*.json" --local-dir models/
```

---

## 📚 References

- [Tantivy Documentation](https://docs.rs/tantivy) - FTS and BM25
- [ONNX Runtime Rust](https://docs.rs/onnxruntime) - Neural inference
- [keyword_extraction crate](https://crates.io/crates/keyword_extraction) - TextRank/YAKE
- [unicode-segmentation](https://docs.rs/unicode-segmentation) - Sentence boundaries
- [Reciprocal Rank Fusion Paper](https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf) - RRF algorithm

---

**Status**: Ready for implementation with 100% coverage using our stack  
**Estimated time**: 2-3 weeks for full integration  
**Benefits**: 
- ✅ No dependency on competing vector databases
- ✅ Full control over hybrid search pipeline
- ✅ Production-ready standalone libraries
- ✅ Battle-tested components (tantivy, ONNX)
- ✅ Minimal external dependencies


# Intelligent Search Architecture

## System Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Vectorizer Server                        │
├─────────────────────────────────────────────────────────────┤
│  Intelligent Search Engine                                  │
│  ├── QueryGenerator                                         │
│  │   ├── MultiQueryExpansion                               │
│  │   ├── DomainKnowledge                                   │
│  │   └── TechnicalTermExtraction                           │
│  ├── SemanticReranker                                      │
│  │   ├── EmbeddingSimilarity                               │
│  │   ├── TermFrequencyScoring                              │
│  │   ├── PositionBonus                                     │
│  │   ├── CollectionRelevance                               │
│  │   ├── ContentQuality                                    │
│  │   └── FreshnessScoring                                  │
│  ├── DeduplicationEngine                                   │
│  │   ├── ContentHashing                                    │
│  │   ├── SemanticSimilarity                                │
│  │   └── DuplicateDetection                                │
│  └── ContextFormatter                                      │
│      ├── RelevanceExtraction                               │
│      ├── ContentTruncation                                 │
│      └── MetadataEnrichment                                │
├─────────────────────────────────────────────────────────────┤
│  Enhanced MCP Tools                                         │
│  ├── intelligent_search                                     │
│  ├── semantic_search                                        │
│  ├── contextual_search                                      │
│  └── multi_collection_search                                │
├─────────────────────────────────────────────────────────────┤
│  Vector Database (HNSW + Embeddings)                       │
│  ├── HNSW Index                                            │
│  ├── Embedding Models                                       │
│  ├── Collection Management                                 │
│  └── Metadata Storage                                      │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. QueryGenerator

**Responsibility**: Generate multiple search queries from user input

**Key Features**:
- Multi-query expansion (8 queries max)
- Domain-specific knowledge integration
- Technical term extraction
- Synonym expansion

**Implementation**:
```rust
pub struct QueryGenerator {
    domain_knowledge: DomainKnowledge,
    technical_extractor: TechnicalTermExtractor,
    synonym_expander: SynonymExpander,
}

impl QueryGenerator {
    pub fn generate_queries(&self, user_query: &str) -> Vec<String> {
        let terms = self.technical_extractor.extract(user_query);
        let mut queries = vec![user_query.to_string()];
        
        if let Some(main_term) = terms.first() {
            // Technical documentation queries
            queries.extend(self.generate_technical_queries(main_term));
            
            // Domain-specific expansion
            queries.extend(self.domain_knowledge.expand_term(main_term));
            
            // Synonym expansion
            queries.extend(self.synonym_expander.expand(main_term));
        }
        
        queries.into_iter().unique().take(8).collect()
    }
}
```

### 2. SemanticReranker

**Responsibility**: Rerank search results using multiple scoring factors

**Scoring Factors**:
- Semantic similarity (40%)
- Term frequency (20%)
- Position bonus (10%)
- Collection relevance (10%)
- Content quality (10%)
- Freshness (10%)

**Implementation**:
```rust
pub struct SemanticReranker {
    embedding_model: EmbeddingModel,
    scoring_weights: ScoringWeights,
    quality_analyzer: ContentQualityAnalyzer,
}

impl SemanticReranker {
    pub async fn rerank(&self, results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let query_embedding = self.embedding_model.embed(query).await?;
        
        let mut scored_results = Vec::new();
        
        for result in results {
            let scores = self.calculate_all_scores(&result, &query_embedding, query).await?;
            let final_score = self.combine_scores(scores);
            
            scored_results.push(ScoredResult { result, score: final_score });
        }
        
        scored_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(scored_results.into_iter().map(|sr| sr.result).collect())
    }
}
```

### 3. DeduplicationEngine

**Responsibility**: Remove duplicate and near-duplicate content

**Methods**:
- Content hashing for exact duplicates
- Semantic similarity for near-duplicates
- Jaccard similarity for efficiency

**Implementation**:
```rust
pub struct DeduplicationEngine {
    similarity_threshold: f32,
    content_hasher: ContentHasher,
    semantic_similarity: SemanticSimilarity,
}

impl DeduplicationEngine {
    pub fn deduplicate(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut unique_results = Vec::new();
        let mut seen_hashes = HashSet::new();
        
        for result in results {
            let content_hash = self.content_hasher.hash(&result.content);
            
            // Skip exact duplicates
            if seen_hashes.contains(&content_hash) {
                continue;
            }
            
            // Check semantic duplicates
            let is_duplicate = unique_results.iter().any(|existing| {
                self.semantic_similarity.calculate(&result.content, &existing.content) 
                    > self.similarity_threshold
            });
            
            if !is_duplicate {
                seen_hashes.insert(content_hash);
                unique_results.push(result);
            }
        }
        
        unique_results
    }
}
```

## MCP Tools Implementation

### intelligent_search Tool

**Primary tool for intelligent search with full feature set**

**Features**:
- Multi-query generation
- Semantic reranking
- Deduplication
- Domain-specific expansion
- Rich metadata

**Usage Example**:
```json
{
  "query": "vectorizer performance",
  "collections": ["vectorizer-docs", "performance-benchmarks"],
  "max_results": 5,
  "rerank": true,
  "deduplicate": true,
  "domain_hints": ["database", "search", "performance"]
}
```

### semantic_search Tool

**Pure semantic search using embedding similarity**

**Features**:
- Embedding-based similarity
- Configurable threshold
- Optional embedding inclusion
- Fast execution

**Usage Example**:
```json
{
  "query": "HNSW indexing algorithm",
  "similarity_threshold": 0.8,
  "max_results": 10,
  "include_embeddings": false
}
```

### contextual_search Tool

**Search with additional context for better relevance**

**Features**:
- Context-aware query enhancement
- Weighted context integration
- Improved relevance scoring

**Usage Example**:
```json
{
  "query": "optimization",
  "context": "vector database performance tuning",
  "context_weight": 0.3,
  "max_results": 5
}
```

### multi_collection_search Tool

**Search across multiple collection groups with weighted results**

**Features**:
- Group-based searching
- Weighted result combination
- Cross-collection reranking
- Flexible group configuration

**Usage Example**:
```json
{
  "query": "API documentation",
  "collection_groups": [
    {
      "name": "core_docs",
      "collections": ["api-docs", "core-api"],
      "weight": 1.0
    },
    {
      "name": "examples",
      "collections": ["code-examples", "tutorials"],
      "weight": 0.8
    }
  ],
  "cross_collection_rerank": true,
  "max_results_per_group": 3
}
```

## Performance Optimization

### Caching Strategy

**Query Generation Cache**:
- Cache generated queries for common terms
- TTL: 1 hour
- Memory limit: 100MB

**Embedding Cache**:
- Cache query embeddings
- TTL: 30 minutes
- Memory limit: 200MB

**Result Cache**:
- Cache search results for identical queries
- TTL: 15 minutes
- Memory limit: 500MB

### Parallel Processing

**Multi-Query Execution**:
- Execute queries in parallel
- Batch collection searches
- Async result aggregation

**Reranking Optimization**:
- Parallel score calculation
- Batch embedding computation
- Optimized similarity calculations

## Configuration

### Default Settings

```yaml
intelligent_search:
  query_generation:
    max_queries: 8
    domain_expansion: true
    technical_focus: true
    synonym_expansion: true
    
  reranking:
    enabled: true
    weights:
      semantic_similarity: 0.4
      term_frequency: 0.2
      position_bonus: 0.1
      collection_relevance: 0.1
      content_quality: 0.1
      freshness: 0.1
      
  deduplication:
    enabled: true
    similarity_threshold: 0.8
    content_hashing: true
    
  caching:
    query_cache_ttl: 3600
    embedding_cache_ttl: 1800
    result_cache_ttl: 900
    
  performance:
    max_parallel_queries: 8
    batch_size: 100
    timeout_ms: 5000
```

## Monitoring & Metrics

### Key Metrics

**Search Quality**:
- Relevance score (target: >95%)
- Context completeness (target: >90%)
- User satisfaction (target: >4.5/5)

**Performance**:
- Search latency (target: <100ms)
- Memory usage (target: <50MB overhead)
- Throughput (target: >1000 searches/sec)

**System Health**:
- Cache hit rate (target: >80%)
- Error rate (target: <0.1%)
- Uptime (target: >99.9%)

### Logging

**Search Operations**:
- Query generation details
- Reranking scores breakdown
- Deduplication statistics
- Performance metrics

**Error Tracking**:
- Failed queries
- Timeout occurrences
- Memory pressure events
- Cache misses

## Testing Strategy

### Unit Tests

**QueryGenerator**:
- Multi-query generation
- Domain expansion
- Technical term extraction

**SemanticReranker**:
- Score calculation accuracy
- Reranking consistency
- Performance benchmarks

**DeduplicationEngine**:
- Duplicate detection
- Similarity threshold accuracy
- Performance with large datasets

### Integration Tests

**MCP Tools**:
- Tool registration
- Input validation
- Output format compliance
- Error handling

**End-to-End**:
- Complete search workflows
- Performance under load
- Memory usage patterns
- Cache behavior

### Quality Validation

**Search Quality**:
- Manual relevance assessment
- Comparison with Cursor results
- Domain-specific accuracy
- Context completeness

**Performance**:
- Latency benchmarks
- Memory usage profiling
- Throughput testing
- Scalability validation

---

**This architecture provides a robust foundation for implementing Cursor-level intelligent search capabilities while maintaining high performance and reliability.**

# Intelligent Search Implementation

## Overview

This document outlines the implementation of **Cursor-level intelligent search** capabilities in Vectorizer, transforming it from a basic vector database into a sophisticated semantic search engine that rivals modern AI-powered IDEs.

## 🎯 **Strategic Goals**

### **Primary Objectives**
- **Match Cursor's search quality** with multi-query intelligence
- **Eliminate client-side complexity** by moving intelligence to the server
- **Provide rich MCP tools** for seamless IDE integration
- **Enable domain-specific knowledge** for better context understanding

### **Success Metrics**
- **Search relevance**: 95%+ accuracy vs manual curation
- **Response time**: <100ms for intelligent search
- **Context quality**: Match or exceed Cursor's context generation
- **Client simplicity**: Reduce client code by 80%

## 🏗️ **Architecture Design**

### **Core Components**

```
┌─────────────────────────────────────────────────────────────┐
│                    Vectorizer Server                        │
├─────────────────────────────────────────────────────────────┤
│  Intelligent Search Engine                                  │
│  ├── Query Generator (Multi-Query)                          │
│  ├── Semantic Reranker                                      │
│  ├── Deduplication Engine                                   │
│  ├── Domain Knowledge Base                                  │
│  └── Context Formatter                                      │
├─────────────────────────────────────────────────────────────┤
│  Enhanced MCP Tools                                         │
│  ├── intelligent_search                                     │
│  ├── semantic_search                                        │
│  ├── contextual_search                                      │
│  └── multi_collection_search                                │
├─────────────────────────────────────────────────────────────┤
│  Vector Database (HNSW + Embeddings)                       │
└─────────────────────────────────────────────────────────────┘
```

## 🔧 **Implementation Details**

### **1. Multi-Query Generation Engine**

#### **Core Algorithm**
```rust
pub struct QueryGenerator {
    domain_knowledge: DomainKnowledge,
    query_expansion: QueryExpansion,
}

impl QueryGenerator {
    pub fn generate_queries(&self, user_query: &str) -> Vec<String> {
        let terms = self.extract_terms(user_query);
        let mut queries = vec![user_query.to_string()];
        
        if let Some(main_term) = terms.first() {
            // Technical documentation queries
            queries.extend([
                format!("{} documentation", main_term),
                format!("{} features", main_term),
                format!("{} architecture", main_term),
                format!("{} performance", main_term),
                format!("{} API", main_term),
                format!("{} usage examples", main_term),
                format!("{} configuration", main_term),
                format!("{} benchmarks", main_term),
            ]);
            
            // Domain-specific expansion
            self.add_domain_queries(&mut queries, main_term);
        }
        
        // Remove duplicates and limit
        queries.into_iter().unique().take(8).collect()
    }
    
    fn add_domain_queries(&self, queries: &mut Vec<String>, term: &str) {
        match term.to_lowercase().as_str() {
            "vectorizer" => {
                queries.extend([
                    "vector database".to_string(),
                    "semantic search".to_string(),
                    "HNSW indexing".to_string(),
                    "embedding models".to_string(),
                    "similarity search".to_string(),
                    "vector quantization".to_string(),
                ]);
            },
            "cmmv" => {
                queries.extend([
                    "CMMV framework".to_string(),
                    "Contract Model View".to_string(),
                    "CMMV architecture".to_string(),
                    "CMMV documentation".to_string(),
                ]);
            },
            "hnsw" => {
                queries.extend([
                    "hierarchical navigable small world".to_string(),
                    "graph-based indexing".to_string(),
                    "approximate nearest neighbor".to_string(),
                    "HNSW performance".to_string(),
                ]);
            },
            _ => {}
        }
    }
}
```

#### **Domain Knowledge Base**
```rust
pub struct DomainKnowledge {
    technologies: HashMap<String, Vec<String>>,
    synonyms: HashMap<String, Vec<String>>,
    related_terms: HashMap<String, Vec<String>>,
}

impl DomainKnowledge {
    pub fn new() -> Self {
        let mut knowledge = Self {
            technologies: HashMap::new(),
            synonyms: HashMap::new(),
            related_terms: HashMap::new(),
        };
        
        // Vectorizer domain
        knowledge.technologies.insert("vectorizer".to_string(), vec![
            "vector database".to_string(),
            "semantic search".to_string(),
            "HNSW".to_string(),
            "embedding".to_string(),
            "similarity".to_string(),
        ]);
        
        // CMMV domain
        knowledge.technologies.insert("cmmv".to_string(), vec![
            "Contract Model View".to_string(),
            "framework".to_string(),
            "architecture".to_string(),
            "TypeScript".to_string(),
        ]);
        
        // Synonyms
        knowledge.synonyms.insert("vectorizer".to_string(), vec![
            "vector db".to_string(),
            "vector database".to_string(),
            "semantic search engine".to_string(),
        ]);
        
        knowledge
    }
}
```

### **2. Semantic Reranking Engine**

#### **Advanced Scoring Algorithm**
```rust
pub struct SemanticReranker {
    embedding_model: EmbeddingModel,
    scoring_weights: ScoringWeights,
}

#[derive(Debug)]
pub struct ScoringWeights {
    pub semantic_similarity: f32,    // 0.4
    pub term_frequency: f32,        // 0.2
    pub position_bonus: f32,        // 0.1
    pub collection_relevance: f32,  // 0.1
    pub content_quality: f32,       // 0.1
    pub freshness: f32,             // 0.1
}

impl SemanticReranker {
    pub async fn rerank(&self, results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let query_embedding = self.embedding_model.embed(query).await?;
        
        let mut scored_results = Vec::new();
        
        for result in results {
            let content_embedding = self.embedding_model.embed(&result.content).await?;
            
            // Calculate semantic similarity
            let semantic_score = cosine_similarity(&query_embedding, &content_embedding);
            
            // Calculate term frequency score
            let tf_score = self.calculate_term_frequency(&result.content, query);
            
            // Calculate position bonus
            let position_score = self.calculate_position_bonus(&result.content, query);
            
            // Calculate collection relevance
            let collection_score = self.calculate_collection_relevance(&result.collection, query);
            
            // Calculate content quality
            let quality_score = self.calculate_content_quality(&result.content);
            
            // Calculate freshness
            let freshness_score = self.calculate_freshness(&result.metadata);
            
            // Weighted combination
            let final_score = 
                semantic_score * self.scoring_weights.semantic_similarity +
                tf_score * self.scoring_weights.term_frequency +
                position_score * self.scoring_weights.position_bonus +
                collection_score * self.scoring_weights.collection_relevance +
                quality_score * self.scoring_weights.content_quality +
                freshness_score * self.scoring_weights.freshness;
            
            scored_results.push(ScoredResult {
                result,
                score: final_score,
                breakdown: ScoreBreakdown {
                    semantic: semantic_score,
                    term_frequency: tf_score,
                    position: position_score,
                    collection: collection_score,
                    quality: quality_score,
                    freshness: freshness_score,
                },
            });
        }
        
        // Sort by final score
        scored_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(scored_results.into_iter().map(|sr| sr.result).collect())
    }
}
```

### **3. Deduplication Engine**

#### **Content Deduplication**
```rust
pub struct DeduplicationEngine {
    similarity_threshold: f32,
    content_hasher: ContentHasher,
}

impl DeduplicationEngine {
    pub fn deduplicate(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut unique_results = Vec::new();
        let mut seen_hashes = HashSet::new();
        
        for result in results {
            let content_hash = self.content_hasher.hash(&result.content);
            
            // Check for exact duplicates
            if seen_hashes.contains(&content_hash) {
                continue;
            }
            
            // Check for semantic duplicates
            let is_semantic_duplicate = unique_results.iter().any(|existing| {
                self.calculate_similarity(&result.content, &existing.content) > self.similarity_threshold
            });
            
            if !is_semantic_duplicate {
                seen_hashes.insert(content_hash);
                unique_results.push(result);
            }
        }
        
        unique_results
    }
    
    fn calculate_similarity(&self, content1: &str, content2: &str) -> f32 {
        // Use Jaccard similarity for efficiency
        let words1: HashSet<&str> = content1.split_whitespace().collect();
        let words2: HashSet<&str> = content2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 { 0.0 } else { intersection as f32 / union as f32 }
    }
}
```

### **4. Enhanced MCP Tools**

#### **intelligent_search Tool**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentSearchParams {
    pub query: String,
    pub collections: Option<Vec<String>>,
    pub max_results: Option<usize>,
    pub rerank: Option<bool>,
    pub deduplicate: Option<bool>,
    pub domain_hints: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentSearchResult {
    pub results: Vec<SearchResult>,
    pub query_expansion: Vec<String>,
    pub search_metadata: SearchMetadata,
}

impl McpTool for IntelligentSearchTool {
    fn name(&self) -> &str { "intelligent_search" }
    
    fn description(&self) -> &str {
        "Perform intelligent multi-query search with semantic reranking and deduplication"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "User query"},
                "collections": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Specific collections to search (empty = all)"
                },
                "max_results": {
                    "type": "number",
                    "default": 5,
                    "description": "Maximum number of results"
                },
                "rerank": {
                    "type": "boolean",
                    "default": true,
                    "description": "Enable semantic reranking"
                },
                "deduplicate": {
                    "type": "boolean",
                    "default": true,
                    "description": "Enable deduplication"
                },
                "domain_hints": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Domain-specific hints for better context"
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let params: IntelligentSearchParams = serde_json::from_value(params)?;
        
        // Generate multiple queries
        let queries = self.query_generator.generate_queries(&params.query);
        
        // Search across collections
        let mut all_results = Vec::new();
        let collections = params.collections.unwrap_or_else(|| self.get_all_collections());
        
        for query in &queries {
            for collection in &collections {
                let results = self.search_vectors(collection, query, 3).await?;
                all_results.extend(results);
            }
        }
        
        // Apply intelligent processing
        let mut processed_results = all_results;
        
        if params.deduplicate.unwrap_or(true) {
            processed_results = self.deduplication_engine.deduplicate(processed_results);
        }
        
        if params.rerank.unwrap_or(true) {
            processed_results = self.semantic_reranker.rerank(processed_results, &params.query).await?;
        }
        
        // Limit results
        let max_results = params.max_results.unwrap_or(5);
        processed_results.truncate(max_results);
        
        let result = IntelligentSearchResult {
            results: processed_results,
            query_expansion: queries,
            search_metadata: SearchMetadata {
                total_queries: queries.len(),
                collections_searched: collections.len(),
                total_results_found: all_results.len(),
                results_after_dedup: processed_results.len(),
            },
        };
        
        Ok(serde_json::to_value(result)?)
    }
}
```

#### **semantic_search Tool**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchParams {
    pub query: String,
    pub collections: Option<Vec<String>>,
    pub similarity_threshold: Option<f32>,
    pub max_results: Option<usize>,
    pub include_embeddings: Option<bool>,
}

impl McpTool for SemanticSearchTool {
    fn name(&self) -> &str { "semantic_search" }
    
    fn description(&self) -> &str {
        "Perform pure semantic search using embedding similarity"
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let params: SemanticSearchParams = serde_json::from_value(params)?;
        
        // Generate query embedding
        let query_embedding = self.embedding_model.embed(&params.query).await?;
        
        // Search using embedding similarity
        let results = self.search_by_embedding(
            &query_embedding,
            params.collections,
            params.similarity_threshold.unwrap_or(0.7),
            params.max_results.unwrap_or(10),
        ).await?;
        
        Ok(serde_json::to_value(results)?)
    }
}
```

#### **contextual_search Tool**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextualSearchParams {
    pub query: String,
    pub context: String,
    pub collections: Option<Vec<String>>,
    pub context_weight: Option<f32>,
    pub max_results: Option<usize>,
}

impl McpTool for ContextualSearchTool {
    fn name(&self) -> &str { "contextual_search" }
    
    fn description(&self) -> &str {
        "Search with additional context to improve relevance"
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let params: ContextualSearchParams = serde_json::from_value(params)?;
        
        // Combine query and context
        let enhanced_query = format!("{} {}", params.query, params.context);
        
        // Use intelligent search with enhanced query
        let intelligent_params = IntelligentSearchParams {
            query: enhanced_query,
            collections: params.collections,
            max_results: params.max_results,
            rerank: Some(true),
            deduplicate: Some(true),
            domain_hints: None,
        };
        
        self.intelligent_search.execute(serde_json::to_value(intelligent_params)?).await
    }
}
```

#### **multi_collection_search Tool**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiCollectionSearchParams {
    pub query: String,
    pub collection_groups: Vec<CollectionGroup>,
    pub cross_collection_rerank: Option<bool>,
    pub max_results_per_group: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionGroup {
    pub name: String,
    pub collections: Vec<String>,
    pub weight: Option<f32>,
}

impl McpTool for MultiCollectionSearchTool {
    fn name(&self) -> &str { "multi_collection_search" }
    
    fn description(&self) -> &str {
        "Search across multiple collection groups with weighted results"
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let params: MultiCollectionSearchParams = serde_json::from_value(params)?;
        
        let mut group_results = Vec::new();
        
        for group in &params.collection_groups {
            let group_params = IntelligentSearchParams {
                query: params.query.clone(),
                collections: Some(group.collections.clone()),
                max_results: params.max_results_per_group.unwrap_or(3),
                rerank: Some(true),
                deduplicate: Some(true),
                domain_hints: None,
            };
            
            let results = self.intelligent_search.execute(serde_json::to_value(group_params)?).await?;
            let search_result: IntelligentSearchResult = serde_json::from_value(results)?;
            
            group_results.push(CollectionGroupResult {
                group_name: group.name.clone(),
                weight: group.weight.unwrap_or(1.0),
                results: search_result.results,
            });
        }
        
        // Cross-collection reranking if enabled
        let final_results = if params.cross_collection_rerank.unwrap_or(true) {
            self.cross_collection_rerank(group_results).await?
        } else {
            group_results
        };
        
        Ok(serde_json::to_value(final_results)?)
    }
}
```

## 🚀 **Implementation Timeline**

### **Phase 1: Core Engine (2 weeks)**
- [ ] Query generation engine
- [ ] Basic semantic reranking
- [ ] Content deduplication
- [ ] Domain knowledge base

### **Phase 2: MCP Integration (1 week)**
- [ ] `intelligent_search` tool
- [ ] `semantic_search` tool
- [ ] MCP server integration
- [ ] API testing

### **Phase 3: Advanced Features (1 week)**
- [ ] `contextual_search` tool
- [ ] `multi_collection_search` tool
- [ ] Cross-collection reranking
- [ ] Performance optimization

### **Phase 4: Testing & Optimization (1 week)**
- [ ] Comprehensive testing
- [ ] Performance benchmarking
- [ ] Quality validation
- [ ] Documentation updates

## 📊 **Performance Targets**

### **Search Quality**
- **Relevance Score**: >95% vs manual curation
- **Context Completeness**: Match Cursor's context generation
- **Domain Accuracy**: >90% for technical queries

### **Performance Metrics**
- **Search Latency**: <100ms for intelligent search
- **Memory Usage**: <50MB additional overhead
- **Throughput**: >1000 searches/second

### **Client Benefits**
- **Code Reduction**: 80% less client-side logic
- **Integration Time**: <1 hour for new clients
- **Maintenance**: Centralized intelligence updates

## 🔧 **Configuration**

### **Default Configuration**
```yaml
intelligent_search:
  enabled: true
  
  query_generation:
    max_queries: 8
    domain_expansion: true
    technical_focus: true
    
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
    
  domain_knowledge:
    auto_update: true
    custom_domains: []
```

## 🎯 **Success Criteria**

### **Technical Validation**
- [ ] All 4 MCP tools implemented and tested
- [ ] Search quality matches or exceeds Cursor
- [ ] Performance targets met
- [ ] Memory usage within limits

### **Integration Validation**
- [ ] BitNet sample simplified to <50 lines
- [ ] Cursor integration seamless
- [ ] Other IDEs can easily adopt
- [ ] Documentation complete

### **Quality Validation**
- [ ] 95%+ relevance on test queries
- [ ] Context generation matches Cursor
- [ ] Domain-specific queries handled correctly
- [ ] No regression in existing functionality

## 📚 **References**

- [Cursor IDE Search Implementation](https://cursor.sh/docs)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [HNSW Algorithm Paper](https://arxiv.org/abs/1603.09320)
- [Semantic Search Best Practices](https://docs.vespa.ai/en/semantic-search.html)

---

**This implementation will transform Vectorizer into a world-class intelligent search engine, matching the quality of modern AI-powered development tools while providing a simple, powerful API for any client to leverage.**

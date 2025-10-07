# Cursor-like Discovery System - Technical Specification

**Status**: 📋 Planning  
**Priority**: P1 - High  
**Target Version**: v0.4.0  
**Estimated Effort**: 3-4 weeks  
**Created**: 2025-10-07

---

## 🎯 Objective

Implement a Cursor-inspired discovery system that mirrors how Cursor performs intelligent context retrieval:
- Pre-filtering collections by relevance
- Ranking collections by multiple signals
- Query expansion with semantic focus
- Evidence compression with citations
- Answer plan generation for LLM prompts

This system will create a **unified discovery tool** that chains all operations efficiently.

---

## 📋 Core Functions Overview

### Function Pipeline
```
User Query
    ↓
1. filter_collections       → Filter by name/pattern
    ↓
2. score_collections        → Rank by relevance signals
    ↓
3. expand_queries_baseline  → Generate query variations
    ↓
4. broad_discovery          → Multi-query search with MMR
    ↓
5. semantic_focus           → Deep search in top collections
    ↓
6. promote_readme           → Boost README files
    ↓
7. compress_evidence        → Extract key sentences
    ↓
8. build_answer_plan        → Structure sections
    ↓
9. render_llm_prompt        → Generate final prompt
    ↓
LLM Response (formatted)
```

---

## 🔧 Function Specifications

### 1. `filter_collections`

**Purpose**: Pre-filter collections by name patterns with stopword removal.

**Signature**:
```rust
pub fn filter_collections(
    query: &str,
    include: &[&str],
    exclude: &[&str],
    all_collections: &[CollectionRef]
) -> Result<Vec<CollectionRef>, DiscoveryError>
```

**Parameters**:
- `query`: User's original query (used for stopword removal)
- `include`: Glob patterns to include (e.g., `["vectorizer*", "*-docs"]`)
- `exclude`: Glob patterns to exclude (e.g., `["test-*", "*-backup"]`)
- `all_collections`: All available collections

**Algorithm**:
```rust
impl CollectionFilter {
    fn filter(&self, query: &str, include: &[&str], exclude: &[&str]) -> Vec<CollectionRef> {
        // 1. Extract terms from query (remove stopwords)
        let query_terms = self.extract_terms(query);
        
        // 2. Match include patterns
        let mut candidates = Vec::new();
        for collection in &self.all_collections {
            if self.matches_any_pattern(&collection.name, include) {
                candidates.push(collection.clone());
            }
        }
        
        // 3. Remove exclude patterns
        candidates.retain(|c| !self.matches_any_pattern(&c.name, exclude));
        
        // 4. If no include patterns, use query-based filtering
        if include.is_empty() {
            candidates = self.filter_by_query_terms(&query_terms);
        }
        
        candidates
    }
    
    fn extract_terms(&self, query: &str) -> Vec<String> {
        let stopwords = ["o", "que", "é", "the", "is", "a", "an", "what", "how"];
        query.split_whitespace()
            .filter(|term| !stopwords.contains(&term.to_lowercase().as_str()))
            .map(|s| s.to_string())
            .collect()
    }
    
    fn matches_any_pattern(&self, name: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(name))
                .unwrap_or(false)
        })
    }
}
```

**Output**:
```rust
pub struct CollectionRef {
    pub name: String,
    pub dimension: usize,
    pub vector_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}
```

**Example**:
```rust
let query = "O que é o vectorizer";
let include = ["vectorizer*", "*-docs"];
let exclude = ["*-test", "*-backup"];
let filtered = filter_collections(query, include, exclude, &all_collections)?;
// Returns: ["vectorizer-docs", "vectorizer-source", "vectorizer-sdk-python", ...]
```

---

### 2. `score_collections`

**Purpose**: Score collections by name match, term boost, and signal boost.

**Signature**:
```rust
pub fn score_collections(
    query_terms: &[&str],
    collections: &[CollectionRef],
    config: &ScoringConfig
) -> Result<Vec<(CollectionRef, f32)>, DiscoveryError>
```

**Scoring Algorithm**:
```rust
pub struct ScoringConfig {
    pub name_match_weight: f32,      // Default: 0.4
    pub term_boost_weight: f32,      // Default: 0.3
    pub signal_boost_weight: f32,    // Default: 0.3
    pub recency_decay_days: f32,     // Default: 90.0
}

impl CollectionScorer {
    fn score(&self, collection: &CollectionRef, query_terms: &[&str]) -> f32 {
        let name_score = self.name_match_score(&collection.name, query_terms);
        let term_score = self.term_boost_score(&collection.name, query_terms);
        let signal_score = self.signal_boost_score(collection);
        
        name_score * self.config.name_match_weight +
        term_score * self.config.term_boost_weight +
        signal_score * self.config.signal_boost_weight
    }
    
    fn name_match_score(&self, name: &str, terms: &[&str]) -> f32 {
        // Exact match boost
        let exact_matches = terms.iter()
            .filter(|term| name.to_lowercase().contains(&term.to_lowercase()))
            .count();
        
        let score = (exact_matches as f32) / (terms.len() as f32);
        
        // Boost if name starts with query term
        if terms.iter().any(|t| name.starts_with(t)) {
            score * 1.5
        } else {
            score
        }
    }
    
    fn term_boost_score(&self, name: &str, terms: &[&str]) -> f32 {
        let boost_terms = ["docs", "source", "api", "sdk", "core"];
        let matches = boost_terms.iter()
            .filter(|term| name.contains(term))
            .count();
        
        (matches as f32) / (boost_terms.len() as f32)
    }
    
    fn signal_boost_score(&self, collection: &CollectionRef) -> f32 {
        // Size signal (normalize by 1M vectors)
        let size_score = (collection.vector_count as f32 / 1_000_000.0).min(1.0);
        
        // Recency signal (exponential decay)
        let days_old = (Utc::now() - collection.updated_at).num_days() as f32;
        let recency_score = (-days_old / self.config.recency_decay_days).exp();
        
        // Tag signal
        let important_tags = ["documentation", "code", "api"];
        let tag_score = collection.tags.iter()
            .filter(|t| important_tags.contains(&t.as_str()))
            .count() as f32 / important_tags.len() as f32;
        
        (size_score + recency_score + tag_score) / 3.0
    }
}
```

**Output**:
```rust
// Sorted by score (highest first)
vec![
    (CollectionRef { name: "vectorizer-docs", ... }, 0.87),
    (CollectionRef { name: "vectorizer-source", ... }, 0.82),
    (CollectionRef { name: "vectorizer-sdk-python", ... }, 0.65),
    ...
]
```

---

### 3. `expand_queries_baseline`

**Purpose**: Deterministic query expansion with semantic variations.

**Signature**:
```rust
pub fn expand_queries_baseline(
    query: &str,
    config: &ExpansionConfig
) -> Result<Vec<String>, DiscoveryError>
```

**Expansion Templates**:
```rust
pub struct ExpansionConfig {
    pub include_definition: bool,     // Default: true
    pub include_features: bool,       // Default: true
    pub include_architecture: bool,   // Default: true
    pub include_api: bool,            // Default: true
    pub include_performance: bool,    // Default: true
    pub include_use_cases: bool,      // Default: true
    pub max_expansions: usize,        // Default: 8
}

impl QueryExpander {
    fn expand(&self, query: &str) -> Vec<String> {
        let mut expansions = vec![query.to_string()];
        let base_term = self.extract_main_term(query);
        
        if self.config.include_definition {
            expansions.push(format!("{} definition", base_term));
            expansions.push(format!("what is {}", base_term));
        }
        
        if self.config.include_features {
            expansions.push(format!("{} features", base_term));
            expansions.push(format!("{} capabilities", base_term));
            expansions.push(format!("{} main functionality", base_term));
        }
        
        if self.config.include_architecture {
            expansions.push(format!("{} architecture", base_term));
            expansions.push(format!("{} components", base_term));
            expansions.push(format!("{} system design", base_term));
        }
        
        if self.config.include_api {
            expansions.push(format!("{} API", base_term));
            expansions.push(format!("{} usage", base_term));
        }
        
        if self.config.include_performance {
            expansions.push(format!("{} performance", base_term));
            expansions.push(format!("{} benchmarks", base_term));
        }
        
        if self.config.include_use_cases {
            expansions.push(format!("{} use cases", base_term));
            expansions.push(format!("{} examples", base_term));
        }
        
        expansions.truncate(self.config.max_expansions);
        expansions
    }
    
    fn extract_main_term(&self, query: &str) -> String {
        // Remove stopwords and get main term
        let stopwords = ["o", "que", "é", "the", "is", "a", "what"];
        query.split_whitespace()
            .filter(|w| !stopwords.contains(&w.to_lowercase().as_str()))
            .next()
            .unwrap_or(query)
            .to_string()
    }
}
```

**Example Output**:
```rust
Input: "O que é o vectorizer"
Output: [
    "O que é o vectorizer",
    "vectorizer definition",
    "what is vectorizer",
    "vectorizer features",
    "vectorizer capabilities",
    "vectorizer architecture",
    "vectorizer API",
    "vectorizer performance"
]
```

---

### 4. `broad_discovery`

**Purpose**: Multi-query broad search with MMR deduplication.

**Signature**:
```rust
pub fn broad_discovery(
    queries: &[String],
    collections: &[CollectionRef],
    k: usize,
    config: &BroadDiscoveryConfig
) -> Result<Vec<ScoredChunk>, DiscoveryError>
```

**Configuration**:
```rust
pub struct BroadDiscoveryConfig {
    pub k_per_query: usize,           // Default: 10
    pub mmr_lambda: f32,              // Default: 0.7
    pub similarity_threshold: f32,    // Default: 0.3
    pub enable_deduplication: bool,   // Default: true
    pub dedup_threshold: f32,         // Default: 0.85
}

pub struct ScoredChunk {
    pub collection: String,
    pub doc_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: ChunkMetadata,
}

pub struct ChunkMetadata {
    pub file_path: String,
    pub chunk_index: usize,
    pub file_extension: String,
    pub line_range: Option<(usize, usize)>,
}
```

**Algorithm**:
```rust
impl BroadDiscovery {
    async fn discover(&self, queries: &[String], collections: &[CollectionRef]) -> Vec<ScoredChunk> {
        let mut all_results = Vec::new();
        
        // 1. Execute all queries in parallel
        let mut handles = Vec::new();
        for query in queries {
            for collection in collections {
                let handle = self.search_collection(query, collection, self.config.k_per_query);
                handles.push(handle);
            }
        }
        
        let results = futures::future::join_all(handles).await;
        for result in results {
            if let Ok(chunks) = result {
                all_results.extend(chunks);
            }
        }
        
        // 2. Filter by similarity threshold
        all_results.retain(|chunk| chunk.score >= self.config.similarity_threshold);
        
        // 3. Deduplicate by content similarity
        if self.config.enable_deduplication {
            all_results = self.deduplicate(all_results);
        }
        
        // 4. Apply MMR for diversity
        let final_results = self.apply_mmr(all_results, self.k);
        
        final_results
    }
    
    fn deduplicate(&self, chunks: Vec<ScoredChunk>) -> Vec<ScoredChunk> {
        let mut unique = Vec::new();
        
        for chunk in chunks {
            let is_duplicate = unique.iter().any(|existing: &ScoredChunk| {
                self.content_similarity(&chunk.content, &existing.content) > self.config.dedup_threshold
            });
            
            if !is_duplicate {
                unique.push(chunk);
            } else {
                // Keep the one with higher score
                if let Some(pos) = unique.iter().position(|c| 
                    self.content_similarity(&chunk.content, &c.content) > self.config.dedup_threshold
                ) {
                    if chunk.score > unique[pos].score {
                        unique[pos] = chunk;
                    }
                }
            }
        }
        
        unique
    }
    
    fn apply_mmr(&self, chunks: Vec<ScoredChunk>, k: usize) -> Vec<ScoredChunk> {
        let mut selected = Vec::new();
        let mut candidates = chunks;
        
        // Select first item (highest score)
        if let Some(first) = candidates.iter().max_by(|a, b| 
            a.score.partial_cmp(&b.score).unwrap()
        ) {
            selected.push(first.clone());
            candidates.retain(|c| c.doc_id != first.doc_id);
        }
        
        // MMR selection loop
        while selected.len() < k && !candidates.is_empty() {
            let mut best_mmr_score = f32::MIN;
            let mut best_idx = 0;
            
            for (idx, candidate) in candidates.iter().enumerate() {
                // Relevance score
                let relevance = candidate.score;
                
                // Max similarity to already selected
                let max_sim = selected.iter()
                    .map(|s| self.content_similarity(&candidate.content, &s.content))
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                
                // MMR score
                let mmr_score = self.config.mmr_lambda * relevance - 
                                (1.0 - self.config.mmr_lambda) * max_sim;
                
                if mmr_score > best_mmr_score {
                    best_mmr_score = mmr_score;
                    best_idx = idx;
                }
            }
            
            let best = candidates.remove(best_idx);
            selected.push(best);
        }
        
        selected
    }
}
```

---

### 5. `semantic_focus`

**Purpose**: Deep semantic search within specific high-priority collections.

**Signature**:
```rust
pub fn semantic_focus(
    collection: &CollectionRef,
    queries: &[String],
    k: usize,
    config: &SemanticFocusConfig
) -> Result<Vec<ScoredChunk>, DiscoveryError>
```

**Configuration**:
```rust
pub struct SemanticFocusConfig {
    pub semantic_reranking: bool,     // Default: true
    pub cross_encoder: bool,          // Default: false
    pub similarity_threshold: f32,    // Default: 0.35 (higher than broad)
    pub context_window: usize,        // Default: 3 (chunks before/after)
}
```

**Algorithm**:
```rust
impl SemanticFocus {
    async fn focus_search(
        &self,
        collection: &CollectionRef,
        queries: &[String],
        k: usize
    ) -> Vec<ScoredChunk> {
        let mut all_chunks = Vec::new();
        
        // 1. Search with all query variations
        for query in queries {
            let results = self.semantic_search(collection, query, k * 2).await?;
            all_chunks.extend(results);
        }
        
        // 2. Filter by higher threshold
        all_chunks.retain(|c| c.score >= self.config.similarity_threshold);
        
        // 3. Semantic reranking
        if self.config.semantic_reranking {
            all_chunks = self.rerank_semantically(queries[0], all_chunks).await?;
        }
        
        // 4. Add context chunks
        if self.config.context_window > 0 {
            all_chunks = self.add_context_chunks(all_chunks).await?;
        }
        
        // 5. Sort and truncate
        all_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_chunks.truncate(k);
        
        all_chunks
    }
    
    async fn rerank_semantically(&self, query: &str, chunks: Vec<ScoredChunk>) -> Vec<ScoredChunk> {
        // Use multiple semantic signals for reranking
        chunks.into_iter().map(|mut chunk| {
            let base_score = chunk.score;
            
            // Term frequency boost
            let tf_boost = self.term_frequency_score(query, &chunk.content);
            
            // Sentence quality boost
            let quality_boost = self.sentence_quality_score(&chunk.content);
            
            // Position boost (earlier chunks = more important)
            let position_boost = 1.0 / (1.0 + chunk.metadata.chunk_index as f32 * 0.1);
            
            chunk.score = base_score * 0.6 + tf_boost * 0.2 + quality_boost * 0.1 + position_boost * 0.1;
            chunk
        }).collect()
    }
    
    async fn add_context_chunks(&self, chunks: Vec<ScoredChunk>) -> Vec<ScoredChunk> {
        let mut with_context = chunks.clone();
        
        for chunk in &chunks {
            // Get surrounding chunks
            for offset in 1..=self.config.context_window {
                // Previous chunk
                if chunk.metadata.chunk_index >= offset {
                    if let Ok(prev) = self.get_chunk_by_index(
                        &chunk.collection,
                        &chunk.metadata.file_path,
                        chunk.metadata.chunk_index - offset
                    ).await {
                        with_context.push(prev);
                    }
                }
                
                // Next chunk
                if let Ok(next) = self.get_chunk_by_index(
                    &chunk.collection,
                    &chunk.metadata.file_path,
                    chunk.metadata.chunk_index + offset
                ).await {
                    with_context.push(next);
                }
            }
        }
        
        // Deduplicate
        with_context.sort_by(|a, b| {
            (&a.collection, &a.doc_id).cmp(&(&b.collection, &b.doc_id))
        });
        with_context.dedup_by(|a, b| {
            a.collection == b.collection && a.doc_id == b.doc_id
        });
        
        with_context
    }
}
```

---

### 6. `promote_readme`

**Purpose**: Boost README files to the top of results.

**Signature**:
```rust
pub fn promote_readme(
    hits: &[ScoredChunk],
    config: &ReadmePromotionConfig
) -> Result<Vec<ScoredChunk>, DiscoveryError>
```

**Configuration**:
```rust
pub struct ReadmePromotionConfig {
    pub readme_boost: f32,            // Default: 1.5x
    pub readme_patterns: Vec<String>, // ["README.md", "README", "readme.md"]
    pub always_top: bool,             // Default: true
}
```

**Algorithm**:
```rust
impl ReadmePromoter {
    fn promote(&self, hits: Vec<ScoredChunk>) -> Vec<ScoredChunk> {
        let mut readme_chunks = Vec::new();
        let mut other_chunks = Vec::new();
        
        for chunk in hits {
            if self.is_readme(&chunk.metadata.file_path) {
                let mut promoted = chunk.clone();
                promoted.score *= self.config.readme_boost;
                readme_chunks.push(promoted);
            } else {
                other_chunks.push(chunk);
            }
        }
        
        if self.config.always_top {
            // READMEs always at top, sorted by score
            readme_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            other_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            readme_chunks.extend(other_chunks);
            readme_chunks
        } else {
            // READMEs get boost but mixed with others by score
            let mut all = readme_chunks;
            all.extend(other_chunks);
            all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            all
        }
    }
    
    fn is_readme(&self, file_path: &str) -> bool {
        let filename = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        self.config.readme_patterns.iter().any(|pattern| {
            filename.eq_ignore_ascii_case(pattern)
        })
    }
}
```

---

### 7. `compress_evidence`

**Purpose**: Extract key sentences with citations for evidence compression.

**Signature**:
```rust
pub fn compress_evidence(
    chunks: &[ScoredChunk],
    max_bullets: usize,
    max_per_doc: usize,
    config: &CompressionConfig
) -> Result<Vec<Bullet>, DiscoveryError>
```

**Configuration**:
```rust
pub struct CompressionConfig {
    pub min_sentence_words: usize,    // Default: 8
    pub max_sentence_words: usize,    // Default: 30
    pub prefer_starts: bool,          // Default: true (first sentences)
    pub include_citations: bool,      // Default: true
}

pub struct Bullet {
    pub text: String,
    pub source_id: String,
    pub collection: String,
    pub file_path: String,
    pub score: f32,
    pub category: BulletCategory,
}

pub enum BulletCategory {
    Definition,
    Feature,
    Architecture,
    Performance,
    Integration,
    UseCase,
    Other,
}
```

**Algorithm**:
```rust
impl EvidenceCompressor {
    fn compress(&self, chunks: &[ScoredChunk], max_bullets: usize, max_per_doc: usize) -> Vec<Bullet> {
        let mut bullets = Vec::new();
        let mut doc_counts: HashMap<String, usize> = HashMap::new();
        
        for chunk in chunks {
            let doc_key = format!("{}::{}", chunk.collection, chunk.metadata.file_path);
            let count = doc_counts.entry(doc_key.clone()).or_insert(0);
            
            if *count >= max_per_doc {
                continue;
            }
            
            // Extract sentences
            let sentences = self.extract_sentences(&chunk.content);
            
            // Score and filter sentences
            for sentence in sentences {
                let word_count = sentence.split_whitespace().count();
                
                if word_count < self.config.min_sentence_words 
                    || word_count > self.config.max_sentence_words {
                    continue;
                }
                
                let category = self.categorize_sentence(&sentence);
                let source_id = format!("{}#{}", chunk.collection, chunk.metadata.chunk_index);
                
                let bullet = Bullet {
                    text: self.clean_sentence(sentence),
                    source_id,
                    collection: chunk.collection.clone(),
                    file_path: chunk.metadata.file_path.clone(),
                    score: chunk.score,
                    category,
                };
                
                bullets.push(bullet);
                *count += 1;
                
                if bullets.len() >= max_bullets {
                    break;
                }
            }
            
            if bullets.len() >= max_bullets {
                break;
            }
        }
        
        // Sort by score and truncate
        bullets.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        bullets.truncate(max_bullets);
        
        bullets
    }
    
    fn extract_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting (can be improved with NLP)
        text.split(&['.', '!', '?'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
    
    fn categorize_sentence(&self, sentence: &str) -> BulletCategory {
        let lower = sentence.to_lowercase();
        
        if lower.contains("is a") || lower.contains("defines") || lower.contains("represents") {
            BulletCategory::Definition
        } else if lower.contains("feature") || lower.contains("support") || lower.contains("provides") {
            BulletCategory::Feature
        } else if lower.contains("architecture") || lower.contains("component") || lower.contains("module") {
            BulletCategory::Architecture
        } else if lower.contains("performance") || lower.contains("speed") || lower.contains("latency") {
            BulletCategory::Performance
        } else if lower.contains("integration") || lower.contains("api") || lower.contains("sdk") {
            BulletCategory::Integration
        } else if lower.contains("use case") || lower.contains("example") || lower.contains("application") {
            BulletCategory::UseCase
        } else {
            BulletCategory::Other
        }
    }
    
    fn clean_sentence(&self, sentence: String) -> String {
        // Remove markdown artifacts, extra spaces, etc.
        sentence
            .replace("\\r\\n", " ")
            .replace("\\n", " ")
            .trim()
            .to_string()
    }
}
```

---

### 8. `build_answer_plan`

**Purpose**: Structure bullets into organized sections.

**Signature**:
```rust
pub fn build_answer_plan(
    bullets: &[Bullet],
    config: &AnswerPlanConfig
) -> Result<AnswerPlan, DiscoveryError>
```

**Configuration**:
```rust
pub struct AnswerPlanConfig {
    pub sections: Vec<SectionType>,
    pub min_bullets_per_section: usize,  // Default: 1
    pub max_bullets_per_section: usize,  // Default: 5
}

pub enum SectionType {
    Definition,
    Features,
    Architecture,
    Performance,
    Integrations,
    UseCases,
}

pub struct AnswerPlan {
    pub sections: Vec<Section>,
    pub total_bullets: usize,
    pub sources: Vec<String>,
}

pub struct Section {
    pub title: String,
    pub section_type: SectionType,
    pub bullets: Vec<Bullet>,
    pub priority: usize,
}
```

**Algorithm**:
```rust
impl AnswerPlanBuilder {
    fn build(&self, bullets: &[Bullet]) -> AnswerPlan {
        let mut sections = Vec::new();
        let mut sources = HashSet::new();
        
        // Group bullets by category
        let mut bullets_by_category: HashMap<BulletCategory, Vec<Bullet>> = HashMap::new();
        for bullet in bullets {
            bullets_by_category
                .entry(bullet.category.clone())
                .or_insert_with(Vec::new)
                .push(bullet.clone());
            
            sources.insert(format!("[{}]", bullet.source_id));
        }
        
        // Create sections in priority order
        for section_type in &self.config.sections {
            let category = match section_type {
                SectionType::Definition => BulletCategory::Definition,
                SectionType::Features => BulletCategory::Feature,
                SectionType::Architecture => BulletCategory::Architecture,
                SectionType::Performance => BulletCategory::Performance,
                SectionType::Integrations => BulletCategory::Integration,
                SectionType::UseCases => BulletCategory::UseCase,
            };
            
            if let Some(mut section_bullets) = bullets_by_category.remove(&category) {
                // Sort by score and limit
                section_bullets.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                section_bullets.truncate(self.config.max_bullets_per_section);
                
                if section_bullets.len() >= self.config.min_bullets_per_section {
                    sections.push(Section {
                        title: self.section_title(section_type),
                        section_type: section_type.clone(),
                        bullets: section_bullets,
                        priority: sections.len() + 1,
                    });
                }
            }
        }
        
        AnswerPlan {
            sections,
            total_bullets: bullets.len(),
            sources: sources.into_iter().collect(),
        }
    }
    
    fn section_title(&self, section_type: &SectionType) -> String {
        match section_type {
            SectionType::Definition => "📋 Definition".to_string(),
            SectionType::Features => "✨ Key Features".to_string(),
            SectionType::Architecture => "🏗️ Architecture".to_string(),
            SectionType::Performance => "⚡ Performance".to_string(),
            SectionType::Integrations => "🔗 Integrations".to_string(),
            SectionType::UseCases => "🎯 Use Cases".to_string(),
        }
    }
}
```

---

### 9. `render_llm_prompt`

**Purpose**: Generate compact prompt for LLM formatting.

**Signature**:
```rust
pub fn render_llm_prompt(
    plan: &AnswerPlan,
    bullets: &[Bullet],
    config: &PromptRenderConfig
) -> Result<String, DiscoveryError>
```

**Configuration**:
```rust
pub struct PromptRenderConfig {
    pub include_sources: bool,        // Default: true
    pub include_metadata: bool,       // Default: false
    pub format_style: FormatStyle,    // Default: Markdown
    pub max_prompt_tokens: usize,     // Default: 4000
}

pub enum FormatStyle {
    Markdown,
    Plain,
    Json,
}
```

**Algorithm**:
```rust
impl PromptRenderer {
    fn render(&self, plan: &AnswerPlan) -> String {
        let mut prompt = String::new();
        
        // Header
        prompt.push_str("# Context from Vector Database\n\n");
        prompt.push_str(&format!("Found {} relevant pieces of information from {} sources.\n\n", 
                                 plan.total_bullets, plan.sources.len()));
        
        // Instructions
        prompt.push_str("## Instructions\n");
        prompt.push_str("Format the following information into a clear, concise answer. ");
        prompt.push_str("Keep citations [source_id] intact. Organize by the sections provided.\n\n");
        
        // Sections
        prompt.push_str("## Evidence\n\n");
        for section in &plan.sections {
            prompt.push_str(&format!("### {}\n\n", section.title));
            
            for (idx, bullet) in section.bullets.iter().enumerate() {
                if self.config.include_sources {
                    prompt.push_str(&format!("{}. {} [{}]\n", 
                                           idx + 1, 
                                           bullet.text, 
                                           bullet.source_id));
                } else {
                    prompt.push_str(&format!("{}. {}\n", idx + 1, bullet.text));
                }
            }
            prompt.push_str("\n");
        }
        
        // Sources index
        if self.config.include_sources {
            prompt.push_str("## Sources\n\n");
            for source in &plan.sources {
                prompt.push_str(&format!("- {}\n", source));
            }
        }
        
        // Truncate if too long
        self.truncate_to_token_limit(prompt)
    }
    
    fn truncate_to_token_limit(&self, prompt: String) -> String {
        // Rough token estimation: 1 token ≈ 4 characters
        let estimated_tokens = prompt.len() / 4;
        
        if estimated_tokens > self.config.max_prompt_tokens {
            let max_chars = self.config.max_prompt_tokens * 4;
            let truncated = prompt.chars().take(max_chars).collect::<String>();
            format!("{}\n\n[... truncated to {} tokens ...]", truncated, self.config.max_prompt_tokens)
        } else {
            prompt
        }
    }
}
```

**Example Output**:
```markdown
# Context from Vector Database

Found 15 relevant pieces of information from 5 sources.

## Instructions
Format the following information into a clear, concise answer. Keep citations [source_id] intact. Organize by the sections provided.

## Evidence

### 📋 Definition

1. Vectorizer is a high-performance vector database and search engine built in Rust [vectorizer-docs#0]
2. Designed for semantic search, document indexing, and AI-powered applications [vectorizer-docs#1]

### ✨ Key Features

1. Sub-3ms search times with optimized HNSW indexing [vectorizer-docs#5]
2. Real-time file monitoring and automatic indexing [vectorizer-source#12]
3. Support for TF-IDF, BM25, BERT, and MiniLM embeddings [vectorizer-docs#8]

### 🏗️ Architecture

1. Single unified server with REST API and MCP integration [vectorizer-source#3]
2. Automatic persistence with background auto-save every 30 seconds [vectorizer-docs#15]

### ⚡ Performance

1. Tested with 107+ collections in production [vectorizer-docs#20]
2. Automatic quantization for memory optimization [vectorizer-source#45]

### 🔗 Integrations

1. Model Context Protocol (MCP) for IDE integration [vectorizer-docs#18]
2. SDKs available for Python, TypeScript, and Rust [vectorizer-sdk-python#0]

## Sources

- [vectorizer-docs#0]
- [vectorizer-docs#1]
- [vectorizer-docs#5]
- [vectorizer-source#3]
- [vectorizer-sdk-python#0]
```

---

## 🎯 Unified Discovery Tool

### `cursor_discover`

**Purpose**: Chain all functions into a single unified tool.

**Signature**:
```rust
pub async fn cursor_discover(
    query: &str,
    config: CursorDiscoveryConfig
) -> Result<DiscoveryResponse, DiscoveryError>
```

**Full Configuration**:
```rust
pub struct CursorDiscoveryConfig {
    // Step 1: Filter
    pub include_collections: Vec<String>,
    pub exclude_collections: Vec<String>,
    
    // Step 2: Score
    pub scoring: ScoringConfig,
    
    // Step 3: Expand
    pub expansion: ExpansionConfig,
    
    // Step 4: Broad Discovery
    pub broad: BroadDiscoveryConfig,
    pub broad_k: usize,
    
    // Step 5: Semantic Focus
    pub focus: SemanticFocusConfig,
    pub focus_k: usize,
    pub focus_top_n_collections: usize,
    
    // Step 6: README Promotion
    pub readme: ReadmePromotionConfig,
    
    // Step 7: Evidence Compression
    pub compression: CompressionConfig,
    pub max_bullets: usize,
    pub max_per_doc: usize,
    
    // Step 8: Answer Plan
    pub plan: AnswerPlanConfig,
    
    // Step 9: Prompt Rendering
    pub render: PromptRenderConfig,
}

pub struct DiscoveryResponse {
    pub answer_prompt: String,
    pub plan: AnswerPlan,
    pub bullets: Vec<Bullet>,
    pub chunks: Vec<ScoredChunk>,
    pub metrics: DiscoveryMetrics,
}

pub struct DiscoveryMetrics {
    pub total_time_ms: u64,
    pub collections_searched: usize,
    pub queries_generated: usize,
    pub chunks_found: usize,
    pub chunks_after_dedup: usize,
    pub bullets_extracted: usize,
    pub final_prompt_tokens: usize,
}
```

**Full Pipeline Implementation**:
```rust
impl CursorDiscovery {
    pub async fn discover(&self, query: &str) -> Result<DiscoveryResponse, DiscoveryError> {
        let start_time = std::time::Instant::now();
        let mut metrics = DiscoveryMetrics::default();
        
        // Step 1: Filter collections
        let all_collections = self.store.list_collections().await?;
        let filtered = filter_collections(
            query,
            &self.config.include_collections.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &self.config.exclude_collections.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &all_collections
        )?;
        metrics.collections_searched = filtered.len();
        info!("Step 1: Filtered to {} collections", filtered.len());
        
        // Step 2: Score collections
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut scored = score_collections(&query_terms, &filtered, &self.config.scoring)?;
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        info!("Step 2: Scored {} collections", scored.len());
        
        // Step 3: Expand queries
        let queries = expand_queries_baseline(query, &self.config.expansion)?;
        metrics.queries_generated = queries.len();
        info!("Step 3: Expanded to {} queries", queries.len());
        
        // Step 4: Broad discovery (all collections)
        let broad_chunks = broad_discovery(
            &queries,
            &filtered,
            self.config.broad_k,
            &self.config.broad
        ).await?;
        metrics.chunks_found = broad_chunks.len();
        info!("Step 4: Broad discovery found {} chunks", broad_chunks.len());
        
        // Step 5: Semantic focus (top N collections)
        let top_collections: Vec<CollectionRef> = scored.iter()
            .take(self.config.focus_top_n_collections)
            .map(|(c, _)| c.clone())
            .collect();
        
        let mut focus_chunks = Vec::new();
        for collection in &top_collections {
            let chunks = semantic_focus(
                collection,
                &queries,
                self.config.focus_k,
                &self.config.focus
            ).await?;
            focus_chunks.extend(chunks);
        }
        info!("Step 5: Semantic focus found {} chunks from {} collections", 
              focus_chunks.len(), top_collections.len());
        
        // Merge broad + focus results
        let mut all_chunks = broad_chunks;
        all_chunks.extend(focus_chunks);
        all_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_chunks.dedup_by(|a, b| a.doc_id == b.doc_id);
        metrics.chunks_after_dedup = all_chunks.len();
        
        // Step 6: Promote READMEs
        all_chunks = promote_readme(&all_chunks, &self.config.readme)?;
        info!("Step 6: Promoted README files");
        
        // Step 7: Compress evidence
        let bullets = compress_evidence(
            &all_chunks,
            self.config.max_bullets,
            self.config.max_per_doc,
            &self.config.compression
        )?;
        metrics.bullets_extracted = bullets.len();
        info!("Step 7: Compressed to {} bullets", bullets.len());
        
        // Step 8: Build answer plan
        let plan = build_answer_plan(&bullets, &self.config.plan)?;
        info!("Step 8: Built plan with {} sections", plan.sections.len());
        
        // Step 9: Render prompt
        let answer_prompt = render_llm_prompt(&plan, &bullets, &self.config.render)?;
        metrics.final_prompt_tokens = answer_prompt.len() / 4; // rough estimate
        info!("Step 9: Rendered prompt (~{} tokens)", metrics.final_prompt_tokens);
        
        metrics.total_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(DiscoveryResponse {
            answer_prompt,
            plan,
            bullets,
            chunks: all_chunks,
            metrics,
        })
    }
}
```

**REST API Endpoint**:
```rust
#[post("/cursor_discover")]
async fn cursor_discover_endpoint(
    query: web::Json<CursorDiscoveryRequest>,
    discovery: web::Data<Arc<CursorDiscovery>>,
) -> Result<HttpResponse, Error> {
    let response = discovery.discover(&query.query).await
        .map_err(|e| ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
struct CursorDiscoveryRequest {
    query: String,
    #[serde(default)]
    config: Option<CursorDiscoveryConfig>,
}
```

**MCP Tool Definition**:
```json
{
  "name": "cursor_discover",
  "description": "Cursor-like discovery system that finds, ranks, and structures relevant information for LLM consumption",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "User's question or search query"
      },
      "include_collections": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Collections to include (glob patterns)"
      },
      "exclude_collections": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Collections to exclude (glob patterns)"
      },
      "max_bullets": {
        "type": "integer",
        "default": 20,
        "description": "Maximum evidence bullets to extract"
      }
    },
    "required": ["query"]
  }
}
```

---

## 📊 Example Usage

### Complete Flow Example
```rust
// Configuration
let config = CursorDiscoveryConfig {
    include_collections: vec!["vectorizer*".to_string()],
    exclude_collections: vec!["*-test".to_string()],
    broad_k: 50,
    focus_k: 15,
    focus_top_n_collections: 3,
    max_bullets: 20,
    max_per_doc: 3,
    ..Default::default()
};

// Execute discovery
let discovery = CursorDiscovery::new(vector_store, config);
let response = discovery.discover("O que é o vectorizer").await?;

// Results
println!("📊 Metrics:");
println!("  Collections: {}", response.metrics.collections_searched);
println!("  Queries: {}", response.metrics.queries_generated);
println!("  Chunks: {} → {}", response.metrics.chunks_found, response.metrics.chunks_after_dedup);
println!("  Bullets: {}", response.metrics.bullets_extracted);
println!("  Time: {}ms", response.metrics.total_time_ms);
println!("\n📝 Prompt:\n{}", response.answer_prompt);
```

**Expected Output**:
```
📊 Metrics:
  Collections: 5
  Queries: 8
  Chunks: 127 → 45
  Bullets: 18
  Time: 342ms

📝 Prompt:
# Context from Vector Database

Found 18 relevant pieces of information from 5 sources.

## Instructions
Format the following information into a clear, concise answer...

[... full prompt as shown earlier ...]
```

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_collections() {
        let collections = vec![
            CollectionRef { name: "vectorizer-docs".to_string(), ... },
            CollectionRef { name: "vectorizer-source".to_string(), ... },
            CollectionRef { name: "test-collection".to_string(), ... },
        ];
        
        let filtered = filter_collections(
            "vectorizer features",
            &["vectorizer*"],
            &["*-test"],
            &collections
        ).unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|c| c.name == "vectorizer-docs"));
        assert!(filtered.iter().all(|c| !c.name.contains("test")));
    }
    
    #[test]
    fn test_score_collections() {
        let collections = vec![
            CollectionRef { 
                name: "vectorizer-docs".to_string(),
                vector_count: 1000,
                updated_at: Utc::now(),
                ...
            },
        ];
        
        let config = ScoringConfig::default();
        let scored = score_collections(&["vectorizer"], &collections, &config).unwrap();
        
        assert!(scored[0].1 > 0.0);
    }
    
    #[test]
    fn test_expand_queries() {
        let config = ExpansionConfig::default();
        let queries = expand_queries_baseline("vectorizer", &config).unwrap();
        
        assert!(queries.len() >= 6);
        assert!(queries.iter().any(|q| q.contains("definition")));
        assert!(queries.iter().any(|q| q.contains("features")));
    }
    
    #[tokio::test]
    async fn test_compress_evidence() {
        let chunks = vec![
            ScoredChunk {
                content: "Vectorizer is a high-performance vector database. It supports semantic search.".to_string(),
                score: 0.85,
                ...
            },
        ];
        
        let config = CompressionConfig::default();
        let bullets = compress_evidence(&chunks, 10, 3, &config).unwrap();
        
        assert!(!bullets.is_empty());
        assert!(bullets[0].text.len() > 0);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_full_discovery_pipeline() {
    let store = setup_test_store().await;
    let config = CursorDiscoveryConfig::default();
    let discovery = CursorDiscovery::new(store, config);
    
    let response = discovery.discover("What is vectorizer").await.unwrap();
    
    assert!(response.metrics.collections_searched > 0);
    assert!(response.metrics.queries_generated >= 6);
    assert!(response.metrics.bullets_extracted > 0);
    assert!(!response.answer_prompt.is_empty());
    assert!(response.answer_prompt.contains("Definition"));
}
```

---

## 📈 Performance Targets

| Operation | Target | Acceptable | Notes |
|-----------|--------|------------|-------|
| filter_collections | <5ms | <10ms | In-memory filtering |
| score_collections | <10ms | <20ms | 100+ collections |
| expand_queries | <1ms | <5ms | Deterministic expansion |
| broad_discovery | <200ms | <500ms | Parallel search across collections |
| semantic_focus | <150ms | <300ms | Deep search in top 3 collections |
| promote_readme | <5ms | <10ms | Simple pattern matching |
| compress_evidence | <50ms | <100ms | Sentence extraction |
| build_answer_plan | <10ms | <20ms | Grouping and sorting |
| render_llm_prompt | <5ms | <10ms | String formatting |
| **TOTAL** | **<450ms** | **<1000ms** | End-to-end |

---

## 🗓️ Implementation Timeline

### Phase 1: Core Functions (Week 1-2)
- [ ] Implement `filter_collections`
- [ ] Implement `score_collections`
- [ ] Implement `expand_queries_baseline`
- [ ] Unit tests for Phase 1
- [ ] Documentation

### Phase 2: Search Operations (Week 2-3)
- [ ] Implement `broad_discovery`
- [ ] Implement `semantic_focus`
- [ ] Implement `promote_readme`
- [ ] Integration with existing search
- [ ] Unit tests for Phase 2

### Phase 3: Evidence Processing (Week 3)
- [ ] Implement `compress_evidence`
- [ ] Implement `build_answer_plan`
- [ ] Implement `render_llm_prompt`
- [ ] Unit tests for Phase 3

### Phase 4: Integration (Week 4)
- [ ] Implement `cursor_discover` unified tool
- [ ] REST API endpoints
- [ ] MCP tool integration
- [ ] End-to-end integration tests
- [ ] Performance optimization
- [ ] Documentation and examples

---

## 🔗 API Endpoints

### REST API
```
POST /api/v1/cursor_discover
POST /api/v1/filter_collections
POST /api/v1/score_collections
POST /api/v1/expand_queries
POST /api/v1/broad_discovery
POST /api/v1/semantic_focus
POST /api/v1/compress_evidence
POST /api/v1/build_answer_plan
```

### MCP Tools
```
- cursor_discover
- filter_collections
- score_collections
- expand_queries_baseline
- broad_discovery
- semantic_focus
- compress_evidence
- build_answer_plan
- render_llm_prompt
```

---

## 📝 Configuration Files

### Default Configuration
```yaml
# config/cursor_discovery.yml
cursor_discovery:
  filter:
    default_include: ["*-docs", "*-source"]
    default_exclude: ["*-test", "*-backup"]
  
  scoring:
    name_match_weight: 0.4
    term_boost_weight: 0.3
    signal_boost_weight: 0.3
    recency_decay_days: 90
  
  expansion:
    include_definition: true
    include_features: true
    include_architecture: true
    include_api: true
    include_performance: true
    include_use_cases: true
    max_expansions: 8
  
  broad:
    k_per_query: 10
    mmr_lambda: 0.7
    similarity_threshold: 0.3
    enable_deduplication: true
    dedup_threshold: 0.85
  
  focus:
    semantic_reranking: true
    cross_encoder: false
    similarity_threshold: 0.35
    context_window: 3
  
  readme:
    readme_boost: 1.5
    always_top: true
  
  compression:
    min_sentence_words: 8
    max_sentence_words: 30
    prefer_starts: true
    include_citations: true
  
  plan:
    min_bullets_per_section: 1
    max_bullets_per_section: 5
  
  render:
    include_sources: true
    include_metadata: false
    format_style: markdown
    max_prompt_tokens: 4000
```

---

## 🎓 Usage Examples

### Example 1: Basic Discovery
```rust
use vectorizer::cursor_discovery::CursorDiscovery;

let discovery = CursorDiscovery::new(store, Default::default());
let response = discovery.discover("What is vectorizer?").await?;
println!("{}", response.answer_prompt);
```

### Example 2: Custom Configuration
```rust
let config = CursorDiscoveryConfig {
    include_collections: vec!["vectorizer*".to_string(), "cmmv-*-docs".to_string()],
    broad_k: 100,
    focus_k: 20,
    max_bullets: 30,
    ..Default::default()
};

let discovery = CursorDiscovery::new(store, config);
let response = discovery.discover("vectorizer architecture").await?;
```

### Example 3: Via REST API
```bash
curl -X POST http://localhost:15002/api/v1/cursor_discover \
  -H "Content-Type: application/json" \
  -d '{
    "query": "O que é o vectorizer",
    "include_collections": ["vectorizer*"],
    "max_bullets": 20
  }'
```

### Example 4: Via MCP
```json
{
  "tool": "cursor_discover",
  "arguments": {
    "query": "O que é o vectorizer",
    "include_collections": ["vectorizer*"],
    "max_bullets": 20
  }
}
```

---

## 🔍 Monitoring & Metrics

### Metrics to Track
- Discovery requests per minute
- Average response time
- Collections searched per request
- Queries generated per request
- Bullets extracted per request
- Cache hit rate
- Error rate

### Prometheus Metrics
```rust
lazy_static! {
    static ref DISCOVERY_REQUESTS: Counter = register_counter!(
        "cursor_discovery_requests_total",
        "Total cursor discovery requests"
    ).unwrap();
    
    static ref DISCOVERY_DURATION: Histogram = register_histogram!(
        "cursor_discovery_duration_seconds",
        "Cursor discovery request duration"
    ).unwrap();
    
    static ref COLLECTIONS_SEARCHED: Histogram = register_histogram!(
        "cursor_discovery_collections_searched",
        "Number of collections searched per request"
    ).unwrap();
}
```

---

## 🚀 Future Enhancements

### Phase 2 Features
1. **Machine Learning Query Expansion**
   - Train models on successful queries
   - Personalized expansion based on user history

2. **Adaptive Scoring**
   - Learn from user feedback
   - Adjust collection scores dynamically

3. **Caching Layer**
   - Cache query expansions
   - Cache collection scores
   - Cache frequently accessed bullets

4. **Cross-Language Support**
   - Multi-language query expansion
   - Translation of evidence bullets

5. **Real-time Updates**
   - WebSocket for streaming results
   - Progressive result refinement

---

## 📚 References

- [Cursor AI Context Retrieval](https://docs.cursor.com)
- [MMR Algorithm](https://www.cs.cmu.edu/~jgc/publication/The_Use_MMR_Diversity_Based_LTMIR_1998.pdf)
- [Query Expansion Techniques](https://nlp.stanford.edu/IR-book/html/htmledition/query-expansion-1.html)
- [Semantic Search Best Practices](https://www.pinecone.io/learn/semantic-search/)

---

**Status**: Ready for implementation  
**Next Steps**: Begin Phase 1 development  
**Contact**: [Your Team]


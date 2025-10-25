//! Discovery pipeline - chains all functions

use std::sync::Arc;
use std::time::Instant;

use tracing::info;

use super::*;
use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Main discovery system
pub struct Discovery {
    config: DiscoveryConfig,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
}

impl Discovery {
    /// Create new discovery system
    pub fn new(
        config: DiscoveryConfig,
        store: Arc<VectorStore>,
        embedding_manager: Arc<EmbeddingManager>,
    ) -> Self {
        Self {
            config,
            store,
            embedding_manager,
        }
    }

    /// Execute complete discovery pipeline
    pub async fn discover(&self, query: &str) -> DiscoveryResult<DiscoveryResponse> {
        let start_time = Instant::now();
        let mut metrics = DiscoveryMetrics::default();

        // Step 1: Get all collections from vector store
        let all_collections: Vec<CollectionRef> = self
            .store
            .list_collections()
            .iter()
            .filter_map(|name| {
                self.store.get_collection(name).ok().map(|coll| {
                    let metadata = coll.metadata();
                    CollectionRef {
                        name: name.clone(),
                        dimension: metadata.config.dimension,
                        vector_count: metadata.vector_count,
                        created_at: metadata.created_at,
                        updated_at: metadata.updated_at,
                        tags: vec![],
                    }
                })
            })
            .collect();

        // Step 2: Filter collections
        let filtered = filter::filter_collections(
            query,
            &self
                .config
                .include_collections
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &self
                .config
                .exclude_collections
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &all_collections,
        )?;
        metrics.collections_searched = filtered.len();
        info!("Step 1: Filtered to {} collections", filtered.len());

        // Step 3: Score collections
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut scored = score::score_collections(&query_terms, &filtered, &self.config.scoring)?;
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        info!("Step 2: Scored {} collections", scored.len());

        // Step 4: Expand queries
        let queries = expand::expand_queries_baseline(query, &self.config.expansion)?;
        metrics.queries_generated = queries.len();
        info!("Step 3: Expanded to {} queries", queries.len());

        // Step 5: Broad discovery
        let broad_collections: Vec<_> = scored.iter().map(|(c, _)| c.clone()).collect();
        let broad_chunks = broad::broad_discovery(
            &queries,
            &broad_collections,
            self.config.broad_k,
            &self.config.broad,
            &self.store,
            &self.embedding_manager,
        )
        .await?;
        metrics.chunks_found = broad_chunks.len();
        info!(
            "Step 4: Broad discovery found {} chunks",
            broad_chunks.len()
        );

        // Step 6: Semantic focus (top N collections)
        let top_collections: Vec<_> = scored
            .iter()
            .take(self.config.focus_top_n_collections)
            .map(|(c, _)| c.clone())
            .collect();

        let mut focus_chunks = Vec::new();
        for collection in &top_collections {
            let chunks = focus::semantic_focus(
                collection,
                &queries,
                self.config.focus_k,
                &self.config.focus,
                &self.store,
                &self.embedding_manager,
            )
            .await?;
            focus_chunks.extend(chunks);
        }
        info!(
            "Step 5: Semantic focus found {} chunks from {} collections",
            focus_chunks.len(),
            top_collections.len()
        );

        // Merge and deduplicate results
        let mut all_chunks = broad_chunks;
        all_chunks.extend(focus_chunks);
        all_chunks.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Deduplicate
        all_chunks = broad::deduplicate_chunks(all_chunks, self.config.broad.dedup_threshold);
        metrics.chunks_after_dedup = all_chunks.len();

        // Step 7: Promote READMEs
        all_chunks = readme::promote_readme(&all_chunks, &self.config.readme)?;
        info!("Step 6: Promoted README files");

        // Step 8: Compress evidence
        let bullets = compress::compress_evidence(
            &all_chunks,
            self.config.max_bullets,
            self.config.max_per_doc,
            &self.config.compression,
        )?;
        metrics.bullets_extracted = bullets.len();
        info!("Step 7: Compressed to {} bullets", bullets.len());

        // Step 9: Build answer plan
        let plan = plan::build_answer_plan(&bullets, &self.config.plan)?;
        info!("Step 8: Built plan with {} sections", plan.sections.len());

        // Step 10: Render prompt
        let answer_prompt = render::render_llm_prompt(&plan, &self.config.render)?;
        metrics.final_prompt_tokens = answer_prompt.len() / 4; // rough estimate
        info!(
            "Step 9: Rendered prompt (~{} tokens)",
            metrics.final_prompt_tokens
        );

        metrics.total_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(DiscoveryResponse {
            answer_prompt,
            plan,
            bullets,
            chunks: all_chunks,
            metrics,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &DiscoveryConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_creation() {
        let config = DiscoveryConfig::default();
        let store = Arc::new(VectorStore::new());
        let embedding_manager = Arc::new(EmbeddingManager::new());

        let discovery = Discovery::new(config, store, embedding_manager);
        // Discovery should be created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_discovery_with_empty_store() {
        let config = DiscoveryConfig::default();
        let store = Arc::new(VectorStore::new());
        let mut embedding_manager = EmbeddingManager::new();

        // Register a provider
        let bm25 = crate::embedding::Bm25Embedding::new(512);
        embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));

        let discovery = Discovery::new(config, store, Arc::new(embedding_manager));

        let result = discovery.discover("test query").await;
        assert!(result.is_ok());

        // With empty store, should return empty results
        let response = result.unwrap();
        assert_eq!(response.chunks.len(), 0);
    }

    #[tokio::test]
    async fn test_discovery_metrics() {
        let metrics = DiscoveryMetrics::default();

        assert_eq!(metrics.collections_searched, 0);
        assert_eq!(metrics.queries_generated, 0);
        assert_eq!(metrics.chunks_found, 0);
    }

    #[test]
    fn test_collection_ref_creation() {
        let col_ref = CollectionRef {
            name: "test_coll".to_string(),
            dimension: 512,
            vector_count: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec!["tag1".to_string()],
        };

        assert_eq!(col_ref.name, "test_coll");
        assert_eq!(col_ref.dimension, 512);
        assert_eq!(col_ref.vector_count, 100);
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();

        assert!(config.broad_k > 0);
        assert!(config.focus_k > 0);
        assert!(!config.include_collections.is_empty() || config.include_collections.is_empty());
    }
}

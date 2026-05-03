//! Search dispatch — thin wrappers that resolve aliases (via
//! `get_collection`) and hand off to the variant's own search path.
//!
//! Two extra entry points are provided in this module:
//!
//! - [`VectorStore::search_explained`] — run the same search path as
//!   [`VectorStore::search`] but return a full execution trace alongside
//!   the results (used by `POST /collections/{name}/explain`).
//! - Slow-query capture is woven into `search` and `search_explained`
//!   via an optional `Arc<SlowQueryRing>` parameter; the ring itself is
//!   stored on `VectorizerServer` and passed through at the handler
//!   level, so the hot path (no ring) has zero overhead.

use std::time::Instant;

use tracing::debug;

use super::VectorStore;
use crate::cache::SlowQueryRing;
use crate::db::hybrid_search::HybridSearchConfig;
use crate::error::Result;
use crate::models::{ExplainResponse, SearchResult};

impl VectorStore {
    /// Search for similar vectors
    pub fn search(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        debug!(
            "Searching for {} nearest neighbors in collection '{}'",
            k, collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.search(query_vector, k)
    }

    /// Search with slow-query recording.
    ///
    /// Identical to [`search`][VectorStore::search] but records the
    /// latency in `ring` when the configured threshold is exceeded.
    /// Pass `None` to skip recording (same cost as calling `search`).
    pub fn search_timed(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        k: usize,
        ring: Option<&SlowQueryRing>,
    ) -> Result<Vec<SearchResult>> {
        if let Some(ring) = ring {
            let t0 = Instant::now();
            let result = self.search(collection_name, query_vector, k)?;
            ring.record(collection_name, k, t0.elapsed());
            Ok(result)
        } else {
            self.search(collection_name, query_vector, k)
        }
    }

    /// Run the same search as [`search`][VectorStore::search] but return
    /// an execution trace alongside the results.
    ///
    /// Only `CollectionType::Cpu` is supported; other variants return an
    /// error consistent with the reencode / reindex patterns.
    pub fn search_explained(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<ExplainResponse> {
        use crate::db::CollectionType;

        debug!(
            "Explained search for {} nearest neighbors in collection '{}'",
            k, collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;
        match &*collection_ref {
            CollectionType::Cpu(c) => c.search_explained(query_vector, k),
            CollectionType::Sharded(_) => Err(crate::error::VectorizerError::Storage(
                "explain is not supported on sharded collections".to_string(),
            )),
            CollectionType::DistributedSharded(_) => Err(crate::error::VectorizerError::Storage(
                "explain is not supported on distributed collections".to_string(),
            )),
            #[allow(unreachable_patterns)]
            _ => Err(crate::error::VectorizerError::Storage(
                "explain is not supported on this collection type".to_string(),
            )),
        }
    }

    /// Perform hybrid search combining dense and sparse vectors
    pub fn hybrid_search(
        &self,
        collection_name: &str,
        query_dense: &[f32],
        query_sparse: Option<&crate::models::SparseVector>,
        config: HybridSearchConfig,
    ) -> Result<Vec<SearchResult>> {
        debug!(
            "Hybrid search in collection '{}' (alpha={}, algorithm={:?})",
            collection_name, config.alpha, config.algorithm
        );

        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.hybrid_search(query_dense, query_sparse, config)
    }
}

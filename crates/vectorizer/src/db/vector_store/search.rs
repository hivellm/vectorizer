//! Search dispatch — thin wrappers that resolve aliases (via
//! `get_collection`) and hand off to the variant's own search path.

use tracing::debug;

use super::VectorStore;
use crate::db::hybrid_search::HybridSearchConfig;
use crate::error::Result;
use crate::models::SearchResult;

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

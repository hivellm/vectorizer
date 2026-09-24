//! `EmbeddingManager` — multi-provider registry + dispatch facade.
//! Extracted from the monolithic `embedding/mod.rs` under
//! phase4_split-embedding-providers. No behavior change.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::any::Any;
use std::collections::HashMap;
use std::path::Path;

use crate::db::VectorStore;
use crate::embedding::EmbeddingProvider;
use crate::error::{Result, VectorizerError};
use crate::models::CollectionConfig;

pub struct EmbeddingManager {
    providers: HashMap<String, Box<dyn EmbeddingProvider>>,
    default_provider: Option<String>,
}

impl EmbeddingManager {
    /// Create a new embedding manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register an embedding provider
    pub fn register_provider(&mut self, name: String, provider: Box<dyn EmbeddingProvider>) {
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }
        self.providers.insert(name, provider);
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: &str) -> Result<()> {
        if self.providers.contains_key(name) {
            self.default_provider = Some(name.to_string());
            Ok(())
        } else {
            Err(VectorizerError::Other(format!(
                "Provider '{}' not found",
                name
            )))
        }
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Result<&dyn EmbeddingProvider> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| VectorizerError::Other(format!("Provider '{}' not found", name)))
    }

    /// Get a mutable provider by name
    pub fn get_provider_mut(&mut self, name: &str) -> Option<&mut Box<dyn EmbeddingProvider>> {
        self.providers.get_mut(name)
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Result<&dyn EmbeddingProvider> {
        let provider_name = self
            .default_provider
            .as_ref()
            .ok_or_else(|| VectorizerError::Other("No default provider set".to_string()))?;

        self.get_provider(provider_name)
    }

    /// Get the default provider name
    pub fn get_default_provider_name(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    /// Embed text using the default provider
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.get_default_provider()?.embed(text)
    }

    /// Embed batch of texts using the default provider
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.get_default_provider()?.embed_batch(texts)
    }

    /// Embed text using a specific provider by name
    pub fn embed_with_provider(&self, provider_name: &str, text: &str) -> Result<Vec<f32>> {
        let provider = self.get_provider(provider_name)?;
        provider.embed(text)
    }

    /// Embed batch of texts using a specific provider by name
    pub fn embed_batch_with_provider(
        &self,
        texts: &[&str],
        provider_name: &str,
    ) -> Result<Vec<Vec<f32>>> {
        self.get_provider(provider_name)?.embed_batch(texts)
    }

    /// Embed a search query using the default provider (see
    /// [`EmbeddingProvider::embed_query`]).
    pub fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        self.get_default_provider()?.embed_query(text)
    }

    /// Embed a batch of search queries using the default provider.
    pub fn embed_query_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.get_default_provider()?.embed_query_batch(texts)
    }

    /// Embed a search query using a specific provider by name.
    pub fn embed_query_with_provider(&self, provider_name: &str, text: &str) -> Result<Vec<f32>> {
        self.get_provider(provider_name)?.embed_query(text)
    }

    /// Name of the provider that vectorizes text for a collection.
    ///
    /// The collection's own `embedding_provider` wins when it is
    /// registered here *and* produces vectors of the collection's
    /// dimension. Otherwise the server default is used, which is the
    /// pre-3.8 behavior every text path had: collections created without
    /// an explicit provider, legacy `.vecdb` entries whose provider was
    /// back-filled as `bm25` regardless of their width, and names this
    /// server does not register all keep embedding exactly as before.
    ///
    /// Raw-vector collections (`embedding_provider: "none"`) never reach
    /// a registered provider through here — the sentinel is not in the
    /// registry — so callers must keep rejecting text on them *before*
    /// resolving, as they already do.
    pub fn provider_name_for_collection(&self, config: &CollectionConfig) -> Option<&str> {
        if let Some((name, provider)) = self.providers.get_key_value(&config.embedding_provider)
            && provider.dimension() == config.dimension
        {
            return Some(name.as_str());
        }
        self.default_provider.as_deref()
    }

    /// Whether the collection's own `embedding_provider` is registered
    /// here with the collection's dimension — i.e. whether
    /// [`Self::provider_name_for_collection`] resolves to it rather than
    /// falling back to the default.
    pub fn serves_collection(&self, config: &CollectionConfig) -> bool {
        self.providers
            .get(&config.embedding_provider)
            .is_some_and(|provider| provider.dimension() == config.dimension)
    }

    /// Provider that vectorizes text for a collection (see
    /// [`Self::provider_name_for_collection`]).
    pub fn provider_for_collection(
        &self,
        config: &CollectionConfig,
    ) -> Result<&dyn EmbeddingProvider> {
        let name = self
            .provider_name_for_collection(config)
            .ok_or_else(|| VectorizerError::Other("No default provider set".to_string()))?;
        self.get_provider(name)
    }

    /// Embed a document for insertion into a collection, using the
    /// collection's provider (see [`Self::provider_name_for_collection`]).
    pub fn embed_for_collection(&self, config: &CollectionConfig, text: &str) -> Result<Vec<f32>> {
        self.provider_for_collection(config)?.embed(text)
    }

    /// Embed a search query against a collection, using the collection's
    /// provider (see [`Self::provider_name_for_collection`]).
    pub fn embed_query_for_collection(
        &self,
        config: &CollectionConfig,
        text: &str,
    ) -> Result<Vec<f32>> {
        self.provider_for_collection(config)?.embed_query(text)
    }

    /// Config of the named collection, cloned so no store `Ref` is held
    /// while a (possibly slow) model runs. `None` when the collection
    /// does not exist — the caller's own lookup owns that error.
    fn collection_config(store: &VectorStore, collection: &str) -> Option<CollectionConfig> {
        store
            .get_collection(collection)
            .ok()
            .map(|c| c.config().clone())
    }

    /// Embed a document for the named collection of `store` with that
    /// collection's provider (see [`Self::provider_name_for_collection`]).
    /// A missing collection embeds with the default provider, as every
    /// text path did before per-collection providers existed.
    pub fn embed_for_named_collection(
        &self,
        store: &VectorStore,
        collection: &str,
        text: &str,
    ) -> Result<Vec<f32>> {
        match Self::collection_config(store, collection) {
            Some(config) => self.embed_for_collection(&config, text),
            None => self.embed(text),
        }
    }

    /// Embed a search query for the named collection of `store` with that
    /// collection's provider; the query-side twin of
    /// [`Self::embed_for_named_collection`].
    pub fn embed_query_for_named_collection(
        &self,
        store: &VectorStore,
        collection: &str,
        text: &str,
    ) -> Result<Vec<f32>> {
        match Self::collection_config(store, collection) {
            Some(config) => self.embed_query_for_collection(&config, text),
            None => self.embed_query(text),
        }
    }

    /// Get the dimension of a specific provider
    pub fn get_provider_dimension(&self, provider_name: &str) -> Result<usize> {
        Ok(self.get_provider(provider_name)?.dimension())
    }

    /// List all available provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if a provider exists
    pub fn has_provider(&self, provider_name: &str) -> bool {
        self.providers.contains_key(provider_name)
    }

    /// Save vocabulary for a specific provider.
    ///
    /// Dispatches through `EmbeddingProvider::save_vocabulary_json`; providers
    /// that don't override the trait default return an error. Adding a new
    /// vocabulary-bearing provider is a single-file change (the impl block),
    /// not a two-file change as before.
    pub fn save_vocabulary_json<P: AsRef<Path>>(&self, provider_name: &str, path: P) -> Result<()> {
        let provider = self.get_provider(provider_name)?;
        provider.save_vocabulary_json(path.as_ref()).map_err(|e| {
            // Wrap the generic "not supported" message with the provider name
            // so the HTTP/MCP error stays actionable.
            VectorizerError::Other(format!("Provider '{}': {}", provider_name, e))
        })
    }

    /// Restore vocabulary for a specific provider from a tokenizer JSON
    /// file (the inverse of [`Self::save_vocabulary_json`]).
    pub fn load_vocabulary_json<P: AsRef<Path>>(
        &mut self,
        provider_name: &str,
        path: P,
    ) -> Result<()> {
        let provider = self.get_provider_mut(provider_name).ok_or_else(|| {
            VectorizerError::Other(format!("Provider '{provider_name}' not found"))
        })?;
        provider
            .load_vocabulary_json(path.as_ref())
            .map_err(|e| VectorizerError::Other(format!("Provider '{}': {}", provider_name, e)))
    }

    /// Restore the provider's vocabulary from the persisted tokenizer
    /// snapshots in `data_dir` (phase37: without this, every text query
    /// after a restart embeds in the hash-fallback space and matches
    /// nothing until a full re-index).
    ///
    /// Sources scanned, both produced by auto-save:
    /// - raw `*_tokenizer.json` files (legacy storage format)
    /// - `_tokenizer.json` entries inside `vectorizer.vecdb` (compact
    ///   format — the compactor deletes the raw files after packing)
    ///
    /// Snapshot files are written from the single global provider, one
    /// per collection, so they are point-in-time copies of the same
    /// vocabulary. The archive format carries no timestamps, so the
    /// snapshot with the largest `(total_docs, vocabulary size)` is
    /// selected — the final rebuild of a session has seen the most
    /// documents. Snapshots whose `type` doesn't match the provider or
    /// whose vocabulary is empty (legacy minimal tokenizers) are
    /// ignored.
    ///
    /// Returns the names of collections whose tokenizer snapshot was
    /// missing or empty — the caller surfaces those as degraded (their
    /// stored vectors may predate the restored vocabulary).
    pub fn restore_vocabulary_from_disk(
        &mut self,
        provider_name: &str,
        data_dir: &Path,
    ) -> Result<VocabularyRestoreReport> {
        let mut candidates: Vec<(String, serde_json::Value)> = Vec::new();
        let mut degraded: Vec<String> = Vec::new();

        // Raw tokenizer files (legacy format).
        if let Ok(entries) = std::fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if let Some(collection) = name.strip_suffix("_tokenizer.json") {
                    match std::fs::read_to_string(entry.path())
                        .ok()
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    {
                        Some(v) => candidates.push((collection.to_string(), v)),
                        None => degraded.push(collection.to_string()),
                    }
                }
            }
        }

        // Tokenizer entries inside vectorizer.vecdb (compact format).
        if data_dir.join("vectorizer.vecdb").exists() {
            let reader = crate::storage::StorageReader::new(data_dir)?;
            for collection in reader.list_collections()? {
                let Ok(files) = reader.read_collection_files(&collection) else {
                    degraded.push(collection);
                    continue;
                };
                let mut found = false;
                for (path, bytes) in files {
                    if path.ends_with("_tokenizer.json") {
                        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                            candidates.push((collection.clone(), v));
                            found = true;
                        }
                    }
                }
                if !found {
                    degraded.push(collection);
                }
            }
        }

        // Keep only real snapshots for THIS provider kind.
        let usable = |v: &serde_json::Value| -> bool {
            let type_matches = v
                .get("type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t == provider_name);
            let vocab_nonempty = v
                .get("vocabulary")
                .and_then(|m| m.as_object())
                .is_some_and(|m| !m.is_empty());
            type_matches && vocab_nonempty
        };

        let mut best: Option<(String, serde_json::Value)> = None;
        for (collection, v) in candidates {
            if !usable(&v) {
                degraded.push(collection);
                continue;
            }
            let rank = |val: &serde_json::Value| {
                (
                    val.get("total_docs").and_then(|d| d.as_u64()).unwrap_or(0),
                    val.get("vocabulary")
                        .and_then(|m| m.as_object())
                        .map_or(0, |m| m.len()),
                )
            };
            let better = best.as_ref().is_none_or(|(_, b)| rank(&v) > rank(b));
            if better {
                best = Some((collection, v));
            }
        }

        let Some((source_collection, snapshot)) = best else {
            return Ok(VocabularyRestoreReport {
                restored_from: None,
                degraded_collections: degraded,
            });
        };

        // Provider loaders read from a file path; round-trip the chosen
        // snapshot through a temp file rather than widening the trait.
        let tmp = tempfile::NamedTempFile::new().map_err(|e| {
            VectorizerError::Other(format!("Failed to create temp tokenizer file: {e}"))
        })?;
        std::fs::write(tmp.path(), snapshot.to_string()).map_err(|e| {
            VectorizerError::Other(format!("Failed to write temp tokenizer file: {e}"))
        })?;
        self.load_vocabulary_json(provider_name, tmp.path())?;

        degraded.sort();
        degraded.dedup();
        Ok(VocabularyRestoreReport {
            restored_from: Some(source_collection),
            degraded_collections: degraded,
        })
    }
}

/// Outcome of [`EmbeddingManager::restore_vocabulary_from_disk`].
#[derive(Debug)]
pub struct VocabularyRestoreReport {
    /// Collection whose tokenizer snapshot was loaded, if any.
    pub restored_from: Option<String>,
    /// Collections without a usable vocabulary snapshot — their text
    /// search may be degraded until re-indexed.
    pub degraded_collections: Vec<String>,
}

impl Default for EmbeddingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::embedding::{
        BagOfWordsEmbedding, Bm25Embedding, CharNGramEmbedding, SvdEmbedding, TfIdfEmbedding,
    };

    #[test]
    fn test_tfidf_embedding() {
        let mut tfidf = TfIdfEmbedding::new(10);

        let corpus = vec![
            "machine learning is great",
            "deep learning is better",
            "vector databases store embeddings",
            "embeddings represent text as vectors",
        ];

        tfidf.build_vocabulary(&corpus);

        let embedding = tfidf.embed("machine learning vectors").unwrap();
        assert_eq!(embedding.len(), 10);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bag_of_words() {
        let mut bow = BagOfWordsEmbedding::new(5);

        let corpus = vec!["hello world", "hello machine learning", "world of vectors"];

        bow.build_vocabulary(&corpus);

        let embedding = bow.embed("hello world").unwrap();
        assert_eq!(embedding.len(), 5);

        // Should have non-zero values for "hello" and "world"
        assert!(embedding.iter().any(|&x| x > 0.0));
    }

    #[test]
    fn test_char_ngram() {
        let mut ngram = CharNGramEmbedding::new(10, 3);

        let corpus = vec!["hello", "world", "hello world"];

        ngram.build_vocabulary(&corpus);

        let embedding = ngram.embed("hello").unwrap();
        assert_eq!(embedding.len(), 10);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6 || norm == 0.0);
    }

    #[test]
    fn test_embedding_manager() {
        let mut manager = EmbeddingManager::new();

        let tfidf = Box::new(TfIdfEmbedding::new(10));
        let bow = Box::new(BagOfWordsEmbedding::new(5));

        manager.register_provider("tfidf".to_string(), tfidf);
        manager.register_provider("bow".to_string(), bow);

        manager.set_default_provider("tfidf").unwrap();

        let provider = manager.get_provider("tfidf").unwrap();
        assert_eq!(provider.dimension(), 10);

        let default_provider = manager.get_default_provider().unwrap();
        assert_eq!(default_provider.dimension(), 10);
    }

    /// Deterministic provider whose output encodes which entry point was
    /// used: documents embed to all `1.0`, queries to all `-1.0`.
    struct SideProvider(usize);

    impl EmbeddingProvider for SideProvider {
        fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![1.0; self.0]).collect())
        }
        fn embed_query_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![-1.0; self.0]).collect())
        }
        fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
            Ok(self.embed_query_batch(&[text])?.remove(0))
        }
        fn dimension(&self) -> usize {
            self.0
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn manager_with_side_provider() -> EmbeddingManager {
        let mut manager = EmbeddingManager::new();
        manager.register_provider("bm25".to_string(), Box::new(Bm25Embedding::new(512)));
        manager.register_provider("side".to_string(), Box::new(SideProvider(4)));
        manager.set_default_provider("bm25").unwrap();
        manager
    }

    fn config(provider: &str, dimension: usize) -> CollectionConfig {
        CollectionConfig {
            dimension,
            embedding_provider: provider.to_string(),
            ..CollectionConfig::default()
        }
    }

    #[test]
    fn query_methods_default_to_document_embedding() {
        let manager = manager_with_side_provider();
        // BM25 does not override the query methods: a query embeds
        // exactly like a document.
        assert_eq!(
            manager.embed_query("hello world").unwrap(),
            manager.embed("hello world").unwrap()
        );
        assert_eq!(
            manager.embed_query_batch(&["a", "b"]).unwrap(),
            manager.embed_batch(&["a", "b"]).unwrap()
        );
        // An overriding provider is reached through the query entry point.
        assert_eq!(
            manager.embed_query_with_provider("side", "q").unwrap(),
            vec![-1.0; 4]
        );
        assert_eq!(
            manager.embed_with_provider("side", "d").unwrap(),
            vec![1.0; 4]
        );
    }

    #[test]
    fn collection_provider_wins_when_registered_with_matching_dimension() {
        let manager = manager_with_side_provider();
        let cfg = config("side", 4);
        assert!(manager.serves_collection(&cfg));
        assert_eq!(manager.provider_name_for_collection(&cfg), Some("side"));
        assert_eq!(
            manager.embed_for_collection(&cfg, "doc").unwrap(),
            vec![1.0; 4]
        );
        assert_eq!(
            manager.embed_query_for_collection(&cfg, "query").unwrap(),
            vec![-1.0; 4]
        );
    }

    #[test]
    fn collection_falls_back_to_default_for_unknown_or_mismatched_provider() {
        let manager = manager_with_side_provider();
        // Not registered on this server.
        let unknown = config("fastembed:multilingual-e5-small", 384);
        assert_eq!(manager.provider_name_for_collection(&unknown), Some("bm25"));
        // Registered, but the collection's width says its vectors came
        // from somewhere else (legacy back-filled label) — keep the
        // default, as every text path did before.
        let mislabeled = config("side", 512);
        assert!(!manager.serves_collection(&mislabeled));
        assert_eq!(
            manager.provider_name_for_collection(&mislabeled),
            Some("bm25")
        );
        assert_eq!(
            manager
                .embed_for_collection(&mislabeled, "doc")
                .unwrap()
                .len(),
            512
        );
        // The raw-vector sentinel is never a registered provider.
        let raw = config(crate::models::RAW_VECTOR_PROVIDER, 4);
        assert_eq!(manager.provider_name_for_collection(&raw), Some("bm25"));
    }

    #[test]
    fn named_collection_helpers_resolve_through_the_store() {
        let manager = manager_with_side_provider();
        let store = VectorStore::new();
        store
            .create_collection_cpu_only("side_coll", config("side", 4))
            .unwrap();
        assert_eq!(
            manager
                .embed_for_named_collection(&store, "side_coll", "doc")
                .unwrap(),
            vec![1.0; 4]
        );
        assert_eq!(
            manager
                .embed_query_for_named_collection(&store, "side_coll", "q")
                .unwrap(),
            vec![-1.0; 4]
        );
        // Unknown collection: default provider (bm25, 512).
        assert_eq!(
            manager
                .embed_query_for_named_collection(&store, "missing", "q")
                .unwrap()
                .len(),
            512
        );
    }

    #[test]
    fn collection_resolution_errors_without_any_default() {
        let manager = EmbeddingManager::new();
        let err = manager
            .embed_query_for_collection(&config("side", 4), "q")
            .unwrap_err();
        assert!(err.to_string().contains("No default provider"), "{err}");
    }

    #[test]
    fn save_vocabulary_dispatches_through_trait_for_bm25() {
        let mut manager = EmbeddingManager::new();
        let mut bm25 = Bm25Embedding::new(32);
        let corpus: Vec<String> = vec!["hello world".into(), "machine learning".into()];
        bm25.build_vocabulary(&corpus);
        manager.register_provider("bm25".to_string(), Box::new(bm25));

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bm25.json");

        manager
            .save_vocabulary_json("bm25", &path)
            .expect("trait dispatch to BM25 override succeeds");

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("\"type\": \"bm25\""),
            "expected BM25 vocabulary JSON, got: {body}"
        );
    }

    #[test]
    fn save_vocabulary_errors_for_provider_without_override() {
        // SVD provider inherits the trait default, which returns an error.
        let mut manager = EmbeddingManager::new();
        manager.register_provider("svd".to_string(), Box::new(SvdEmbedding::new(16, 16)));

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("svd.json");
        let err = manager.save_vocabulary_json("svd", &path).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("svd") && msg.contains("does not support vocabulary saving"),
            "expected provider-aware error for SVD, got: {msg}"
        );
    }
}

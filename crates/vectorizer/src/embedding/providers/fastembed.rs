//! FastEmbed adapter — wraps `fastembed::TextEmbedding` as an
//! `EmbeddingProvider`.
//!
//! The wrapped `TextEmbedding` requires `&mut self` on its `embed` call
//! (the inference session is a mutable ONNX Runtime handle), so we guard
//! it behind a `parking_lot::Mutex` and expose `EmbeddingProvider` via
//! `&self`. Throughput is still excellent — batching happens inside the
//! single locked call, not per text.
//!
//! This file compiles only when the `fastembed` Cargo feature is enabled.
//! Bootstrap still needs to gate on the same feature via
//! `#[cfg(feature = "fastembed")]` before constructing the provider.

#![cfg(feature = "fastembed")]
#![allow(missing_docs)]

use std::path::{Path, PathBuf};

use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use parking_lot::Mutex;

use crate::embedding::EmbeddingProvider;
use crate::error::{Result, VectorizerError};

/// Provider backed by a `fastembed::TextEmbedding` ONNX session.
pub struct FastEmbedProvider {
    model: Mutex<TextEmbedding>,
    dimension: usize,
    /// Canonical model identifier as the operator wrote it in
    /// `config.embedding.model` (e.g. `fastembed:all-MiniLM-L6-v2`).
    pub name: String,
}

impl std::fmt::Debug for FastEmbedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastEmbedProvider")
            .field("name", &self.name)
            .field("dimension", &self.dimension)
            .finish()
    }
}

impl FastEmbedProvider {
    /// Build a provider from a `fastembed:<id>` model identifier as it
    /// appears in `config.embedding.model`. The `<id>` portion is matched
    /// against the strings in [`parse_model_id`] below.
    ///
    /// `cache_dir` is the directory where fastembed should cache the
    /// downloaded ONNX weights + tokenizer files. Pass
    /// `vectorizer_core::paths::data_dir().join("fastembed")` from the
    /// server bootstrap so the cache lives next to `vectorizer.vecdb`.
    pub fn from_config(model_id: &str, cache_dir: PathBuf) -> Result<Self> {
        let model = parse_model_id(model_id)?;
        let dimension = model_dimension(&model);

        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to create fastembed cache dir {}: {}",
                cache_dir.display(),
                e
            ))
        })?;

        let opts = TextInitOptions::new(model.clone())
            .with_cache_dir(cache_dir.clone())
            .with_show_download_progress(false);

        tracing::info!(
            "🔄 FastEmbed: initializing model {:?} (cache_dir={})",
            model,
            cache_dir.display()
        );

        let text_embedding = TextEmbedding::try_new(opts).map_err(|e| {
            VectorizerError::Other(format!("FastEmbed init failed for {:?}: {}", model, e))
        })?;

        tracing::info!("✅ FastEmbed: model {:?} ready (dim={})", model, dimension);

        Ok(Self {
            model: Mutex::new(text_embedding),
            dimension,
            name: format!("fastembed:{}", model_name(&model)),
        })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let owned: Vec<String> = texts.iter().map(|s| (*s).to_string()).collect();
        let mut guard = self.model.lock();
        guard
            .embed(owned, None)
            .map_err(|e| VectorizerError::Other(format!("FastEmbed inference failed: {}", e)))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn save_vocabulary_json(&self, _path: &Path) -> Result<()> {
        Err(VectorizerError::Other(
            "fastembed models ship vocabulary inside the ONNX archive — no separate JSON file"
                .to_string(),
        ))
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Map the `<id>` portion of a `fastembed:<id>` config string to a
/// `fastembed::EmbeddingModel` variant.
///
/// Accepts both short aliases (e.g. `all-MiniLM-L6-v2`) and the enum
/// `Debug`-name form (e.g. `AllMiniLML6V2`). Returns a typed error when
/// the id is not recognized so operators see the bad config at boot,
/// not on first embed.
pub fn parse_model_id(id: &str) -> Result<EmbeddingModel> {
    let trimmed = id.trim();
    let model = match trimmed {
        // Sentence-transformers.
        "all-MiniLM-L6-v2" | "AllMiniLML6V2" => EmbeddingModel::AllMiniLML6V2,
        "all-MiniLM-L6-v2-q" | "AllMiniLML6V2Q" => EmbeddingModel::AllMiniLML6V2Q,
        "all-MiniLM-L12-v2" | "AllMiniLML12V2" => EmbeddingModel::AllMiniLML12V2,
        "all-MiniLM-L12-v2-q" | "AllMiniLML12V2Q" => EmbeddingModel::AllMiniLML12V2Q,
        "all-mpnet-base-v2" | "AllMpnetBaseV2" => EmbeddingModel::AllMpnetBaseV2,

        // BAAI BGE.
        "bge-base-en-v1.5" | "BGEBaseENV15" => EmbeddingModel::BGEBaseENV15,
        "bge-base-en-v1.5-q" | "BGEBaseENV15Q" => EmbeddingModel::BGEBaseENV15Q,
        "bge-large-en-v1.5" | "BGELargeENV15" => EmbeddingModel::BGELargeENV15,
        "bge-large-en-v1.5-q" | "BGELargeENV15Q" => EmbeddingModel::BGELargeENV15Q,
        "bge-small-en-v1.5" | "BGESmallENV15" => EmbeddingModel::BGESmallENV15,

        other => {
            return Err(VectorizerError::Other(format!(
                "Unknown fastembed model id '{}'. Supported: all-MiniLM-L6-v2, \
                 all-MiniLM-L12-v2, all-mpnet-base-v2, bge-small-en-v1.5 (default), \
                 bge-base-en-v1.5, bge-large-en-v1.5 (each also available with '-q' \
                 suffix for the quantized variant)",
                other
            )));
        }
    };
    Ok(model)
}

/// Canonical short name for a `fastembed::EmbeddingModel`, used when
/// rebuilding the `name` field after parsing.
fn model_name(model: &EmbeddingModel) -> &'static str {
    match model {
        EmbeddingModel::AllMiniLML6V2 => "all-MiniLM-L6-v2",
        EmbeddingModel::AllMiniLML6V2Q => "all-MiniLM-L6-v2-q",
        EmbeddingModel::AllMiniLML12V2 => "all-MiniLM-L12-v2",
        EmbeddingModel::AllMiniLML12V2Q => "all-MiniLM-L12-v2-q",
        EmbeddingModel::AllMpnetBaseV2 => "all-mpnet-base-v2",
        EmbeddingModel::BGEBaseENV15 => "bge-base-en-v1.5",
        EmbeddingModel::BGEBaseENV15Q => "bge-base-en-v1.5-q",
        EmbeddingModel::BGELargeENV15 => "bge-large-en-v1.5",
        EmbeddingModel::BGELargeENV15Q => "bge-large-en-v1.5-q",
        EmbeddingModel::BGESmallENV15 => "bge-small-en-v1.5",
        _ => "unknown",
    }
}

/// Output dimension for each supported model. Matches the
/// `TextEmbedding::list_supported_models` metadata (we hard-code here so
/// the provider can report `dimension()` without loading the model).
fn model_dimension(model: &EmbeddingModel) -> usize {
    match model {
        EmbeddingModel::AllMiniLML6V2 | EmbeddingModel::AllMiniLML6V2Q => 384,
        EmbeddingModel::AllMiniLML12V2 | EmbeddingModel::AllMiniLML12V2Q => 384,
        EmbeddingModel::AllMpnetBaseV2 => 768,
        EmbeddingModel::BGEBaseENV15 | EmbeddingModel::BGEBaseENV15Q => 768,
        EmbeddingModel::BGELargeENV15 | EmbeddingModel::BGELargeENV15Q => 1024,
        EmbeddingModel::BGESmallENV15 => 384,
        // Safety net for future variants we haven't mapped yet. Boot
        // will still succeed; `dimension()` will report 0 until the
        // first `embed` call, which is obviously wrong — favor
        // fail-loud at parse time by NOT advertising any variant here
        // that isn't in `parse_model_id`.
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn parse_model_id_accepts_short_alias_and_debug_name() {
        assert_eq!(
            parse_model_id("all-MiniLM-L6-v2").unwrap(),
            EmbeddingModel::AllMiniLML6V2
        );
        assert_eq!(
            parse_model_id("AllMiniLML6V2").unwrap(),
            EmbeddingModel::AllMiniLML6V2
        );
        assert_eq!(
            parse_model_id("bge-small-en-v1.5").unwrap(),
            EmbeddingModel::BGESmallENV15
        );
    }

    #[test]
    fn parse_model_id_rejects_unknown() {
        let err = parse_model_id("not-a-real-model").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("not-a-real-model"));
        assert!(msg.contains("Supported"));
    }

    #[test]
    fn model_dimension_matches_known_values() {
        assert_eq!(model_dimension(&EmbeddingModel::AllMiniLML6V2), 384);
        assert_eq!(model_dimension(&EmbeddingModel::BGEBaseENV15), 768);
        assert_eq!(model_dimension(&EmbeddingModel::BGELargeENV15), 1024);
    }
}

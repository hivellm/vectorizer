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
    /// Whether the model expects the E5 `"query: "` / `"passage: "`
    /// input prefixes (see [`uses_e5_prefixes`]).
    e5_prefixes: bool,
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
            e5_prefixes: uses_e5_prefixes(&model),
            name: format!("fastembed:{}", model_name(&model)),
        })
    }

    /// Run inference on `texts` after applying the E5 prefix for `kind`
    /// (a no-op for every model that isn't multilingual E5).
    fn run(&self, texts: &[&str], kind: InputKind) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let inputs = prepare_inputs(texts, self.e5_prefixes, kind);
        let mut guard = self.model.lock();
        guard
            .embed(inputs, None)
            .map_err(|e| VectorizerError::Other(format!("FastEmbed inference failed: {}", e)))
    }
}

/// Which side of an asymmetric retrieval pair a text is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    /// A search query.
    Query,
    /// A document being indexed.
    Passage,
}

/// Prefix the multilingual E5 models were trained with for queries.
const E5_QUERY_PREFIX: &str = "query: ";
/// Prefix the multilingual E5 models were trained with for documents.
const E5_PASSAGE_PREFIX: &str = "passage: ";

/// Whether `model` is one of the multilingual E5 models, which expect
/// every input to start with `"query: "` or `"passage: "`. fastembed
/// does not add these itself; without them E5 recall degrades
/// noticeably.
fn uses_e5_prefixes(model: &EmbeddingModel) -> bool {
    matches!(
        model,
        EmbeddingModel::MultilingualE5Small
            | EmbeddingModel::MultilingualE5Base
            | EmbeddingModel::MultilingualE5Large
    )
}

/// Build the owned inference inputs for `texts`. When `e5_prefixes` is
/// set, each text gets the prefix for `kind` unless it already starts
/// with either E5 prefix (callers that follow the E5 model card and
/// prefix their own text are not double-prefixed).
fn prepare_inputs(texts: &[&str], e5_prefixes: bool, kind: InputKind) -> Vec<String> {
    let prefix = match kind {
        InputKind::Query => E5_QUERY_PREFIX,
        InputKind::Passage => E5_PASSAGE_PREFIX,
    };
    texts
        .iter()
        .map(|text| {
            if !e5_prefixes
                || text.starts_with(E5_QUERY_PREFIX)
                || text.starts_with(E5_PASSAGE_PREFIX)
            {
                (*text).to_string()
            } else {
                format!("{prefix}{text}")
            }
        })
        .collect()
}

impl EmbeddingProvider for FastEmbedProvider {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.run(texts, InputKind::Passage)
    }

    fn embed_query_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.run(texts, InputKind::Query)
    }

    fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        self.run(&[text], InputKind::Query)?
            .into_iter()
            .next()
            .ok_or_else(|| VectorizerError::Other("Failed to generate embedding".to_string()))
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

        // Multilingual (100+ languages, incl. Portuguese).
        "multilingual-e5-small" | "MultilingualE5Small" => EmbeddingModel::MultilingualE5Small,
        "multilingual-e5-base" | "MultilingualE5Base" => EmbeddingModel::MultilingualE5Base,
        "multilingual-e5-large" | "MultilingualE5Large" => EmbeddingModel::MultilingualE5Large,
        "paraphrase-multilingual-MiniLM-L12-v2" | "ParaphraseMLMiniLML12V2" => {
            EmbeddingModel::ParaphraseMLMiniLML12V2
        }
        "paraphrase-multilingual-MiniLM-L12-v2-q" | "ParaphraseMLMiniLML12V2Q" => {
            EmbeddingModel::ParaphraseMLMiniLML12V2Q
        }

        other => {
            return Err(VectorizerError::Other(format!(
                "Unknown fastembed model id '{}'. Supported: all-MiniLM-L6-v2, \
                 all-MiniLM-L12-v2, all-mpnet-base-v2, bge-small-en-v1.5 (default), \
                 bge-base-en-v1.5, bge-large-en-v1.5, \
                 paraphrase-multilingual-MiniLM-L12-v2 (each also available with '-q' \
                 suffix for the quantized variant), multilingual-e5-small, \
                 multilingual-e5-base, multilingual-e5-large",
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
        EmbeddingModel::MultilingualE5Small => "multilingual-e5-small",
        EmbeddingModel::MultilingualE5Base => "multilingual-e5-base",
        EmbeddingModel::MultilingualE5Large => "multilingual-e5-large",
        EmbeddingModel::ParaphraseMLMiniLML12V2 => "paraphrase-multilingual-MiniLM-L12-v2",
        EmbeddingModel::ParaphraseMLMiniLML12V2Q => "paraphrase-multilingual-MiniLM-L12-v2-q",
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
        EmbeddingModel::MultilingualE5Small => 384,
        EmbeddingModel::MultilingualE5Base => 768,
        EmbeddingModel::MultilingualE5Large => 1024,
        EmbeddingModel::ParaphraseMLMiniLML12V2 | EmbeddingModel::ParaphraseMLMiniLML12V2Q => 384,
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

    /// Every multilingual id resolves from both its short alias and its
    /// enum Debug name, round-trips through `model_name`, and reports
    /// the dimension fastembed's own model metadata advertises.
    #[test]
    fn multilingual_models_parse_name_and_dimension() {
        let cases = [
            (
                "multilingual-e5-small",
                "MultilingualE5Small",
                EmbeddingModel::MultilingualE5Small,
                384,
            ),
            (
                "multilingual-e5-base",
                "MultilingualE5Base",
                EmbeddingModel::MultilingualE5Base,
                768,
            ),
            (
                "multilingual-e5-large",
                "MultilingualE5Large",
                EmbeddingModel::MultilingualE5Large,
                1024,
            ),
            (
                "paraphrase-multilingual-MiniLM-L12-v2",
                "ParaphraseMLMiniLML12V2",
                EmbeddingModel::ParaphraseMLMiniLML12V2,
                384,
            ),
            (
                "paraphrase-multilingual-MiniLM-L12-v2-q",
                "ParaphraseMLMiniLML12V2Q",
                EmbeddingModel::ParaphraseMLMiniLML12V2Q,
                384,
            ),
        ];
        for (alias, debug_name, model, dim) in cases {
            assert_eq!(parse_model_id(alias).unwrap(), model, "alias {alias}");
            assert_eq!(parse_model_id(debug_name).unwrap(), model, "{debug_name}");
            assert_eq!(model_name(&model), alias);
            assert_eq!(model_dimension(&model), dim, "dimension of {alias}");
            let info = TextEmbedding::get_model_info(&model).unwrap();
            assert_eq!(info.dim, dim, "fastembed metadata for {alias}");
        }
    }

    #[test]
    fn unknown_id_error_lists_multilingual_models() {
        let msg = format!("{}", parse_model_id("e5-nope").unwrap_err());
        assert!(msg.contains("multilingual-e5-small"), "{msg}");
        assert!(
            msg.contains("paraphrase-multilingual-MiniLM-L12-v2"),
            "{msg}"
        );
    }

    #[test]
    fn only_multilingual_e5_uses_prefixes() {
        assert!(uses_e5_prefixes(&EmbeddingModel::MultilingualE5Small));
        assert!(uses_e5_prefixes(&EmbeddingModel::MultilingualE5Base));
        assert!(uses_e5_prefixes(&EmbeddingModel::MultilingualE5Large));
        assert!(!uses_e5_prefixes(&EmbeddingModel::ParaphraseMLMiniLML12V2));
        assert!(!uses_e5_prefixes(&EmbeddingModel::AllMiniLML6V2));
        assert!(!uses_e5_prefixes(&EmbeddingModel::BGESmallENV15));
    }

    #[test]
    fn prepare_inputs_prefixes_queries_and_passages_for_e5() {
        let texts = ["como cancelar meu pedido", "reembolso"];
        assert_eq!(
            prepare_inputs(&texts, true, InputKind::Query),
            vec!["query: como cancelar meu pedido", "query: reembolso"]
        );
        assert_eq!(
            prepare_inputs(&texts, true, InputKind::Passage),
            vec!["passage: como cancelar meu pedido", "passage: reembolso"]
        );
    }

    #[test]
    fn prepare_inputs_does_not_double_prefix() {
        let texts = ["query: já prefixado", "passage: também prefixado"];
        let expected = vec!["query: já prefixado", "passage: também prefixado"];
        assert_eq!(prepare_inputs(&texts, true, InputKind::Query), expected);
        assert_eq!(prepare_inputs(&texts, true, InputKind::Passage), expected);
    }

    #[test]
    fn prepare_inputs_leaves_non_e5_text_unchanged() {
        let texts = ["hello world", "query: literal"];
        assert_eq!(
            prepare_inputs(&texts, false, InputKind::Query),
            vec!["hello world", "query: literal"]
        );
        assert_eq!(
            prepare_inputs(&texts, false, InputKind::Passage),
            vec!["hello world", "query: literal"]
        );
    }
}

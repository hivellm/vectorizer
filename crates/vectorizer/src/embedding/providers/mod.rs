//! Split providers — each embedding implementation lives in its own
//! file. This module declares the sub-modules and re-exports their
//! public surfaces so external callers see
//! `crate::embedding::{TfIdfEmbedding, Bm25Embedding, SvdEmbedding,
//! BertEmbedding, MiniLmEmbedding, BagOfWordsEmbedding, CharNGramEmbedding,
//! EmbeddingManager}` unchanged.

mod bag_of_words;
mod bert;
mod bm25;
mod char_ngram;
#[cfg(feature = "fastembed")]
pub mod fastembed;
mod manager;
mod minilm;
mod svd;
pub(super) mod tfidf;

pub use bag_of_words::BagOfWordsEmbedding;
pub use bert::BertEmbedding;
pub use bm25::Bm25Embedding;
pub use char_ngram::CharNGramEmbedding;
#[cfg(feature = "fastembed")]
pub use fastembed::FastEmbedProvider;

/// Factory that builds a boxed `EmbeddingProvider` for a
/// `fastembed:<id>` config string. Always returns a typed error when
/// the `fastembed` Cargo feature is off so `vectorizer-server` bootstrap
/// can dispatch without its own `#[cfg]` guards.
pub fn try_build_fastembed_provider(
    model_id: &str,
    cache_dir: std::path::PathBuf,
) -> crate::error::Result<Box<dyn crate::embedding::EmbeddingProvider>> {
    #[cfg(feature = "fastembed")]
    {
        let provider = fastembed::FastEmbedProvider::from_config(model_id, cache_dir)?;
        Ok(Box::new(provider))
    }
    #[cfg(not(feature = "fastembed"))]
    {
        let _ = (model_id, cache_dir);
        Err(crate::error::VectorizerError::Other(
            "fastembed Cargo feature is not enabled in this build. Rebuild vectorizer with \
             `cargo build --release --features fastembed` (or set \
             `vectorizer = { default-features = true }`, since `fastembed` is in the default \
             feature set) to use any `fastembed:<model-id>` provider."
                .to_string(),
        ))
    }
}
pub use manager::EmbeddingManager;
pub use minilm::MiniLmEmbedding;
pub use svd::SvdEmbedding;
pub use tfidf::TfIdfEmbedding;

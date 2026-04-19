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
mod manager;
mod minilm;
mod svd;
pub(super) mod tfidf;

pub use bag_of_words::BagOfWordsEmbedding;
pub use bert::BertEmbedding;
pub use bm25::Bm25Embedding;
pub use char_ngram::CharNGramEmbedding;
pub use manager::EmbeddingManager;
pub use minilm::MiniLmEmbedding;
pub use svd::SvdEmbedding;
pub use tfidf::TfIdfEmbedding;

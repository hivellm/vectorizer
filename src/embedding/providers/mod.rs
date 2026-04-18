//! Split providers — gradually migrating each embedding implementation
//! out of the 1,788-line `embedding/mod.rs`. This file declares the
//! sub-modules and re-exports their public surfaces so external
//! callers see `crate::embedding::Bm25Embedding` etc. unchanged.

mod bag_of_words;
mod char_ngram;
mod manager;

pub use bag_of_words::BagOfWordsEmbedding;
pub use char_ngram::CharNGramEmbedding;
pub use manager::EmbeddingManager;

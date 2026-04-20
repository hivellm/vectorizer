//! Document summarisation primitives.
//!
//! Provides four built-in summarisation methods (extractive, keyword,
//! sentence, abstractive) plus a [`SummarizationManager`] that selects
//! among them based on per-call config. The methods produce shorter
//! representations suitable for embedding alongside the original chunk
//! to improve retrieval quality on long documents.

pub mod config;
pub mod manager;
pub mod methods;
pub mod types;

#[cfg(test)]
mod tests;

pub use config::SummarizationConfig;
pub use manager::SummarizationManager;
pub use methods::SummarizationMethodTrait;
pub use types::*;

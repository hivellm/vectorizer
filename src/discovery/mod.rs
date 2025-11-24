//! Discovery system for intelligent context retrieval
//!
//! This module provides a comprehensive discovery system that mirrors
//! intelligent IDE context retrieval patterns, with:
//! - Collection pre-filtering and ranking
//! - Query expansion with semantic focus
//! - Evidence compression with citations
//! - Answer plan generation for LLM prompts

pub mod broad;
pub mod compress;
pub mod config;
pub mod expand;
pub mod filter;
pub mod focus;
pub mod hybrid;
pub mod pipeline;
pub mod plan;
pub mod readme;
pub mod render;
pub mod score;
pub mod types;

// Re-export main types and functions
pub use broad::broad_discovery;
pub use compress::compress_evidence;
pub use config::*;
pub use expand::expand_queries_baseline;
pub use filter::filter_collections;
pub use focus::semantic_focus;
pub use hybrid::{HybridSearcher, reciprocal_rank_fusion};
pub use pipeline::Discovery;
pub use plan::build_answer_plan;
pub use readme::promote_readme;
pub use render::render_llm_prompt;
pub use score::score_collections;
pub use types::*;

#[cfg(test)]
mod tests;

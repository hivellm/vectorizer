//! Discovery system for intelligent context retrieval
//! 
//! This module provides a comprehensive discovery system that mirrors
//! intelligent IDE context retrieval patterns, with:
//! - Collection pre-filtering and ranking
//! - Query expansion with semantic focus
//! - Evidence compression with citations
//! - Answer plan generation for LLM prompts

pub mod types;
pub mod config;
pub mod filter;
pub mod score;
pub mod expand;
pub mod broad;
pub mod focus;
pub mod readme;
pub mod compress;
pub mod plan;
pub mod render;
pub mod pipeline;
pub mod hybrid;

// Re-export main types and functions
pub use types::*;
pub use config::*;
pub use filter::filter_collections;
pub use score::score_collections;
pub use expand::expand_queries_baseline;
pub use broad::broad_discovery;
pub use focus::semantic_focus;
pub use readme::promote_readme;
pub use compress::compress_evidence;
pub use plan::build_answer_plan;
pub use render::render_llm_prompt;
pub use pipeline::Discovery;
pub use hybrid::{HybridSearcher, reciprocal_rank_fusion};

#[cfg(test)]
mod tests;


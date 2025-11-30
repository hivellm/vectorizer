//! Configuration for discovery system

use serde::{Deserialize, Serialize};

use super::types::SectionType;

/// Main configuration for discovery system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Step 1: Filter
    pub include_collections: Vec<String>,
    pub exclude_collections: Vec<String>,

    /// Step 2: Score
    pub scoring: ScoringConfig,

    /// Step 3: Expand
    pub expansion: ExpansionConfig,

    /// Step 4: Broad Discovery
    pub broad: BroadDiscoveryConfig,
    pub broad_k: usize,

    /// Step 5: Semantic Focus
    pub focus: SemanticFocusConfig,
    pub focus_k: usize,
    pub focus_top_n_collections: usize,

    /// Step 6: README Promotion
    pub readme: ReadmePromotionConfig,

    /// Step 7: Evidence Compression
    pub compression: CompressionConfig,
    pub max_bullets: usize,
    pub max_per_doc: usize,

    /// Step 8: Answer Plan
    pub plan: AnswerPlanConfig,

    /// Step 9: Prompt Rendering
    pub render: PromptRenderConfig,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            include_collections: vec![],
            exclude_collections: vec!["*-test".to_string(), "*-backup".to_string()],
            scoring: ScoringConfig::default(),
            expansion: ExpansionConfig::default(),
            broad: BroadDiscoveryConfig::default(),
            broad_k: 50,
            focus: SemanticFocusConfig::default(),
            focus_k: 15,
            focus_top_n_collections: 3,
            readme: ReadmePromotionConfig::default(),
            compression: CompressionConfig::default(),
            max_bullets: 20,
            max_per_doc: 3,
            plan: AnswerPlanConfig::default(),
            render: PromptRenderConfig::default(),
        }
    }
}

/// Configuration for collection scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub name_match_weight: f32,
    pub term_boost_weight: f32,
    pub signal_boost_weight: f32,
    pub recency_decay_days: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            name_match_weight: 0.4,
            term_boost_weight: 0.3,
            signal_boost_weight: 0.3,
            recency_decay_days: 90.0,
        }
    }
}

/// Configuration for query expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionConfig {
    pub include_definition: bool,
    pub include_features: bool,
    pub include_architecture: bool,
    pub include_api: bool,
    pub include_performance: bool,
    pub include_use_cases: bool,
    pub max_expansions: usize,
}

impl Default for ExpansionConfig {
    fn default() -> Self {
        Self {
            include_definition: true,
            include_features: true,
            include_architecture: true,
            include_api: true,
            include_performance: true,
            include_use_cases: true,
            max_expansions: 8,
        }
    }
}

/// Configuration for broad discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadDiscoveryConfig {
    pub k_per_query: usize,
    pub mmr_lambda: f32,
    pub similarity_threshold: f32,
    pub enable_deduplication: bool,
    pub dedup_threshold: f32,
}

impl Default for BroadDiscoveryConfig {
    fn default() -> Self {
        Self {
            k_per_query: 10,
            mmr_lambda: 0.7,
            similarity_threshold: 0.3,
            enable_deduplication: true,
            dedup_threshold: 0.85,
        }
    }
}

/// Configuration for semantic focus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFocusConfig {
    pub semantic_reranking: bool,
    pub cross_encoder: bool,
    pub similarity_threshold: f32,
    pub context_window: usize,
}

impl Default for SemanticFocusConfig {
    fn default() -> Self {
        Self {
            semantic_reranking: true,
            cross_encoder: false,
            similarity_threshold: 0.35,
            context_window: 3,
        }
    }
}

/// Configuration for README promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmePromotionConfig {
    pub readme_boost: f32,
    pub readme_patterns: Vec<String>,
    pub always_top: bool,
}

impl Default for ReadmePromotionConfig {
    fn default() -> Self {
        Self {
            readme_boost: 1.5,
            readme_patterns: vec![
                "README.md".to_string(),
                "README".to_string(),
                "readme.md".to_string(),
            ],
            always_top: true,
        }
    }
}

/// Configuration for evidence compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub min_sentence_words: usize,
    pub max_sentence_words: usize,
    pub prefer_starts: bool,
    pub include_citations: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_sentence_words: 8,
            max_sentence_words: 30,
            prefer_starts: true,
            include_citations: true,
        }
    }
}

/// Configuration for answer plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerPlanConfig {
    pub sections: Vec<SectionType>,
    pub min_bullets_per_section: usize,
    pub max_bullets_per_section: usize,
}

impl Default for AnswerPlanConfig {
    fn default() -> Self {
        Self {
            sections: vec![
                SectionType::Definition,
                SectionType::Features,
                SectionType::Architecture,
                SectionType::Performance,
                SectionType::Integrations,
                SectionType::UseCases,
            ],
            min_bullets_per_section: 1,
            max_bullets_per_section: 5,
        }
    }
}

/// Format style for prompt rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatStyle {
    Markdown,
    Plain,
    Json,
}

/// Configuration for prompt rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRenderConfig {
    pub include_sources: bool,
    pub include_metadata: bool,
    pub format_style: FormatStyle,
    pub max_prompt_tokens: usize,
}

impl Default for PromptRenderConfig {
    fn default() -> Self {
        Self {
            include_sources: true,
            include_metadata: false,
            format_style: FormatStyle::Markdown,
            max_prompt_tokens: 4000,
        }
    }
}

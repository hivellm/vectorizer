//! Discovery surface: orchestrated multi-stage retrieval.
//!
//! `discover` is the headline pipeline (filter → score → expand →
//! search → bullet-summarise); the other three methods expose the
//! individual stages so callers can swap or compose them.

use crate::error::{Result, VectorizerError};

use super::VectorizerClient;

impl VectorizerClient {
    /// End-to-end discovery pipeline with intelligent search and
    /// LLM-style bullet generation.
    #[allow(clippy::too_many_arguments)]
    pub async fn discover(
        &self,
        query: &str,
        include_collections: Option<Vec<String>>,
        exclude_collections: Option<Vec<String>>,
        max_bullets: Option<usize>,
        broad_k: Option<usize>,
        focus_k: Option<usize>,
    ) -> Result<serde_json::Value> {
        if query.trim().is_empty() {
            return Err(VectorizerError::validation("Query cannot be empty"));
        }
        if let Some(max) = max_bullets
            && max == 0
        {
            return Err(VectorizerError::validation(
                "max_bullets must be greater than 0",
            ));
        }

        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        if let Some(inc) = include_collections {
            payload.insert(
                "include_collections".to_string(),
                serde_json::to_value(inc).unwrap(),
            );
        }
        if let Some(exc) = exclude_collections {
            payload.insert(
                "exclude_collections".to_string(),
                serde_json::to_value(exc).unwrap(),
            );
        }
        if let Some(max) = max_bullets {
            payload.insert(
                "max_bullets".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        if let Some(k) = broad_k {
            payload.insert("broad_k".to_string(), serde_json::Value::Number(k.into()));
        }
        if let Some(k) = focus_k {
            payload.insert("focus_k".to_string(), serde_json::Value::Number(k.into()));
        }

        let response = self
            .make_request(
                "POST",
                "/discover",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse discover response: {e}")))
    }

    /// Pre-filter collections by name patterns.
    pub async fn filter_collections(
        &self,
        query: &str,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        if query.trim().is_empty() {
            return Err(VectorizerError::validation("Query cannot be empty"));
        }
        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        if let Some(inc) = include {
            payload.insert("include".to_string(), serde_json::to_value(inc).unwrap());
        }
        if let Some(exc) = exclude {
            payload.insert("exclude".to_string(), serde_json::to_value(exc).unwrap());
        }
        let response = self
            .make_request(
                "POST",
                "/discovery/filter_collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse filter response: {e}")))
    }

    /// Rank collections by relevance to a query. The three weights
    /// must each be in `[0.0, 1.0]` when supplied.
    pub async fn score_collections(
        &self,
        query: &str,
        name_match_weight: Option<f32>,
        term_boost_weight: Option<f32>,
        signal_boost_weight: Option<f32>,
    ) -> Result<serde_json::Value> {
        if let Some(w) = name_match_weight
            && !(0.0..=1.0).contains(&w)
        {
            return Err(VectorizerError::validation(
                "name_match_weight must be between 0.0 and 1.0",
            ));
        }
        if let Some(w) = term_boost_weight
            && !(0.0..=1.0).contains(&w)
        {
            return Err(VectorizerError::validation(
                "term_boost_weight must be between 0.0 and 1.0",
            ));
        }
        if let Some(w) = signal_boost_weight
            && !(0.0..=1.0).contains(&w)
        {
            return Err(VectorizerError::validation(
                "signal_boost_weight must be between 0.0 and 1.0",
            ));
        }

        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        if let Some(w) = name_match_weight {
            payload.insert("name_match_weight".to_string(), serde_json::json!(w));
        }
        if let Some(w) = term_boost_weight {
            payload.insert("term_boost_weight".to_string(), serde_json::json!(w));
        }
        if let Some(w) = signal_boost_weight {
            payload.insert("signal_boost_weight".to_string(), serde_json::json!(w));
        }
        let response = self
            .make_request(
                "POST",
                "/discovery/score_collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse score response: {e}")))
    }

    /// Generate query variations (definition / features /
    /// architecture-style expansions, capped by `max_expansions`).
    pub async fn expand_queries(
        &self,
        query: &str,
        max_expansions: Option<usize>,
        include_definition: Option<bool>,
        include_features: Option<bool>,
        include_architecture: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        if let Some(max) = max_expansions {
            payload.insert(
                "max_expansions".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        if let Some(def) = include_definition {
            payload.insert(
                "include_definition".to_string(),
                serde_json::Value::Bool(def),
            );
        }
        if let Some(feat) = include_features {
            payload.insert(
                "include_features".to_string(),
                serde_json::Value::Bool(feat),
            );
        }
        if let Some(arch) = include_architecture {
            payload.insert(
                "include_architecture".to_string(),
                serde_json::Value::Bool(arch),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/discovery/expand_queries",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse expand response: {e}")))
    }
}

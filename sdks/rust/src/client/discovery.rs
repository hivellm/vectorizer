//! Discovery surface: orchestrated multi-stage retrieval.
//!
//! `discover` is the headline pipeline (filter → score → expand →
//! search → bullet-summarise); the other methods expose individual
//! stages and the new phase12 pipeline steps:
//! - `broad_discovery` — multi-query broad search across collections
//! - `semantic_focus` — focused search within one collection
//! - `promote_readme` — README-quality chunk promotion
//! - `compress_evidence` — evidence compression into bullets
//! - `build_answer_plan` — bullet → section organisation
//! - `render_llm_prompt` — plan → final LLM prompt string

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::{
    AnswerPlan, AnswerPlanRequest, BroadDiscoveryRequest, BroadDiscoveryResponse,
    CompressEvidenceRequest, CompressEvidenceResponse, LlmPrompt, PromoteReadmeRequest,
    PromoteReadmeResponse, RenderPromptRequest, SemanticFocusRequest, SemanticFocusResponse,
};

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

    /// Broad multi-query search across all collections.
    ///
    /// Calls `POST /discovery/broad_discovery` with `{queries, k?}`.
    pub async fn broad_discovery(
        &self,
        request: BroadDiscoveryRequest,
    ) -> Result<BroadDiscoveryResponse> {
        let payload = serde_json::json!({
            "queries": request.queries,
            "k": request.k.unwrap_or(50),
        });
        let response = self
            .make_request("POST", "/discovery/broad_discovery", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse broad_discovery response: {e}"))
        })
    }

    /// Focused semantic search within a single collection.
    ///
    /// Calls `POST /discovery/semantic_focus` with `{collection, queries, k?}`.
    pub async fn semantic_focus(
        &self,
        request: SemanticFocusRequest,
    ) -> Result<SemanticFocusResponse> {
        let payload = serde_json::json!({
            "collection": request.collection,
            "queries": request.queries,
            "k": request.k.unwrap_or(15),
        });
        let response = self
            .make_request("POST", "/discovery/semantic_focus", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse semantic_focus response: {e}"))
        })
    }

    /// Promote README-quality chunks to the top of a result set.
    ///
    /// Calls `POST /discovery/promote_readme` with `{chunks}`.
    pub async fn promote_readme(
        &self,
        request: PromoteReadmeRequest,
    ) -> Result<PromoteReadmeResponse> {
        let payload = serde_json::json!({ "chunks": request.chunks });
        let response = self
            .make_request("POST", "/discovery/promote_readme", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse promote_readme response: {e}"))
        })
    }

    /// Compress a chunk set into a concise bullet list.
    ///
    /// Calls `POST /discovery/compress_evidence` with
    /// `{chunks, max_bullets?, max_per_doc?}`.
    pub async fn compress_evidence(
        &self,
        request: CompressEvidenceRequest,
    ) -> Result<CompressEvidenceResponse> {
        let mut payload = serde_json::json!({ "chunks": request.chunks });
        if let Some(mb) = request.max_bullets {
            payload["max_bullets"] = serde_json::json!(mb);
        }
        if let Some(mpd) = request.max_per_doc {
            payload["max_per_doc"] = serde_json::json!(mpd);
        }
        let response = self
            .make_request("POST", "/discovery/compress_evidence", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse compress_evidence response: {e}"))
        })
    }

    /// Organise bullets into a structured answer plan.
    ///
    /// Calls `POST /discovery/build_answer_plan` with `{bullets}`.
    pub async fn build_answer_plan(&self, request: AnswerPlanRequest) -> Result<AnswerPlan> {
        let payload = serde_json::json!({ "bullets": request.bullets });
        let response = self
            .make_request("POST", "/discovery/build_answer_plan", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse build_answer_plan response: {e}"))
        })
    }

    /// Render an answer plan into a final LLM prompt string.
    ///
    /// Calls `POST /discovery/render_llm_prompt` with `{plan}`.
    pub async fn render_llm_prompt(&self, request: RenderPromptRequest) -> Result<LlmPrompt> {
        let payload = serde_json::json!({ "plan": request.plan });
        let response = self
            .make_request("POST", "/discovery/render_llm_prompt", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse render_llm_prompt response: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use crate::models::{
        AnswerPlan, AnswerPlanRequest, BroadDiscoveryRequest, BroadDiscoveryResponse,
        CompressEvidenceRequest, CompressEvidenceResponse, LlmPrompt, PromoteReadmeRequest,
        PromoteReadmeResponse, RenderPromptRequest, SemanticFocusRequest, SemanticFocusResponse,
    };

    #[test]
    fn broad_discovery_request_serializes() {
        let req = BroadDiscoveryRequest {
            queries: vec!["HNSW index".into(), "embedding model".into()],
            k: Some(30),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["queries"][0], "HNSW index");
        assert_eq!(v["k"], 30);
    }

    #[test]
    fn broad_discovery_response_deserializes() {
        let raw = json!({
            "chunks": [{"collection": "docs", "score": 0.9, "content_preview": "test"}],
            "count": 1
        });
        let resp: BroadDiscoveryResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.count, 1);
        assert_eq!(resp.chunks.len(), 1);
    }

    #[test]
    fn semantic_focus_request_serializes() {
        let req = SemanticFocusRequest {
            collection: "code".into(),
            queries: vec!["async runtime".into()],
            k: None,
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["collection"], "code");
        assert_eq!(v["queries"][0], "async runtime");
    }

    #[test]
    fn semantic_focus_response_deserializes() {
        let raw = json!({ "chunks": [], "count": 0 });
        let resp: SemanticFocusResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.count, 0);
    }

    #[test]
    fn promote_readme_request_serializes() {
        let req = PromoteReadmeRequest {
            chunks: vec![json!({"collection": "docs", "score": 0.8, "content": "README text"})],
        };
        let v = serde_json::to_value(&req).unwrap();
        assert!(v["chunks"].is_array());
    }

    #[test]
    fn promote_readme_response_deserializes() {
        let raw = json!({ "promoted_chunks": [], "count": 0 });
        let resp: PromoteReadmeResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.count, 0);
    }

    #[test]
    fn compress_evidence_round_trip() {
        let req = CompressEvidenceRequest {
            chunks: vec![json!({"collection": "c", "score": 1.0, "content": "x"})],
            max_bullets: Some(5),
            max_per_doc: Some(2),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["max_bullets"], 5);

        let raw = json!({ "bullets": [{"text": "b", "source_id": "s", "category": "Feature", "score": 0.9}], "count": 1 });
        let resp: CompressEvidenceResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.count, 1);
    }

    #[test]
    fn answer_plan_round_trip() {
        let plan = AnswerPlan {
            sections: vec![json!({"title": "Intro", "bullets_count": 1, "bullets": []})],
            total_bullets: 1,
            sources: vec!["docs".into()],
        };
        let serialized = serde_json::to_value(&plan).unwrap();
        let parsed: AnswerPlan = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed.total_bullets, 1);
        assert_eq!(parsed.sources[0], "docs");
    }

    #[test]
    fn llm_prompt_deserializes() {
        let raw = json!({ "prompt": "Answer: ...", "length": 10, "estimated_tokens": 2 });
        let lp: LlmPrompt = serde_json::from_value(raw).unwrap();
        assert_eq!(lp.prompt, "Answer: ...");
        assert_eq!(lp.estimated_tokens, 2);
    }

    #[test]
    fn render_prompt_request_serializes() {
        let req = RenderPromptRequest {
            plan: AnswerPlan {
                sections: vec![],
                total_bullets: 0,
                sources: vec![],
            },
        };
        let v = serde_json::to_value(&req).unwrap();
        assert!(v["plan"].is_object());
    }
}

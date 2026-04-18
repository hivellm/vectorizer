//! Discovery API REST handlers.
//!
//! The `/discover*` endpoints below feed the high-level Discovery
//! pipeline (filter → score → expand → broad search → focus → promote
//! README → compress → plan → render prompt). Each individual step is
//! exposed as its own handler for debugging and composition; the
//! top-level `discover` handler runs the whole pipeline end-to-end.
//!
//! All logic lives in [`crate::discovery`]; these handlers are thin
//! adapters that parse JSON, call the appropriate discovery function,
//! and marshal results back.

use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::error;

use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_validation_error,
};

pub async fn discover(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{Discovery, DiscoveryConfig};

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let mut config = DiscoveryConfig::default();

    if let Some(include) = payload
        .get("include_collections")
        .and_then(|v| v.as_array())
    {
        config.include_collections = include
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }

    if let Some(exclude) = payload
        .get("exclude_collections")
        .and_then(|v| v.as_array())
    {
        config.exclude_collections = exclude
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }

    if let Some(max_bullets) = payload.get("max_bullets").and_then(|v| v.as_u64()) {
        config.max_bullets = max_bullets as usize;
    }

    if let Some(broad_k) = payload.get("broad_k").and_then(|v| v.as_u64()) {
        config.broad_k = broad_k as usize;
    }

    if let Some(focus_k) = payload.get("focus_k").and_then(|v| v.as_u64()) {
        config.focus_k = focus_k as usize;
    }

    let discovery = Discovery::new(config, state.store.clone(), state.embedding_manager.clone());

    match discovery.discover(query).await {
        Ok(response) => Ok(Json(json!({
            "answer_prompt": response.answer_prompt,
            "sections": response.plan.sections.len(),
            "bullets": response.bullets.len(),
            "chunks": response.chunks.len(),
            "metrics": {
                "total_time_ms": response.metrics.total_time_ms,
                "collections_searched": response.metrics.collections_searched,
                "queries_generated": response.metrics.queries_generated,
                "chunks_found": response.metrics.chunks_found,
                "chunks_after_dedup": response.metrics.chunks_after_dedup,
                "bullets_extracted": response.metrics.bullets_extracted,
                "final_prompt_tokens": response.metrics.final_prompt_tokens,
            }
        }))),
        Err(e) => {
            error!("Discovery error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Discovery failed: {}",
                e
            )))
        }
    }
}

pub async fn filter_collections(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::filter_collections as filter_fn;

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let include: Vec<&str> = payload
        .get("include")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let exclude: Vec<&str> = payload
        .get("exclude")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let all_collections: Vec<_> = state
        .store
        .list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                crate::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();

    match filter_fn(query, &include, &exclude, &all_collections) {
        Ok(filtered) => Ok(Json(json!({
            "filtered_collections": filtered.iter().map(|c| json!({
                "name": c.name,
                "vector_count": c.vector_count,
            })).collect::<Vec<_>>(),
            "count": filtered.len(),
        }))),
        Err(e) => {
            error!("Filter collections error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Filter collections failed: {}",
                e
            )))
        }
    }
}

pub async fn score_collections(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{ScoringConfig, score_collections as score_fn};

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let mut config = ScoringConfig::default();

    if let Some(w) = payload.get("name_match_weight").and_then(|v| v.as_f64()) {
        config.name_match_weight = w as f32;
    }
    if let Some(w) = payload.get("term_boost_weight").and_then(|v| v.as_f64()) {
        config.term_boost_weight = w as f32;
    }
    if let Some(w) = payload.get("signal_boost_weight").and_then(|v| v.as_f64()) {
        config.signal_boost_weight = w as f32;
    }

    let all_collections: Vec<_> = state
        .store
        .list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                crate::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();

    let query_terms: Vec<&str> = query.split_whitespace().collect();

    match score_fn(&query_terms, &all_collections, &config) {
        Ok(scored) => Ok(Json(json!({
            "scored_collections": scored.iter().map(|(c, score)| json!({
                "name": c.name,
                "score": score,
                "vector_count": c.vector_count,
            })).collect::<Vec<_>>(),
            "count": scored.len(),
        }))),
        Err(e) => {
            error!("Score collections error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Score collections failed: {}",
                e
            )))
        }
    }
}

pub async fn expand_queries(Json(payload): Json<Value>) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{ExpansionConfig, expand_queries_baseline};

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    let mut config = ExpansionConfig::default();

    if let Some(max) = payload.get("max_expansions").and_then(|v| v.as_u64()) {
        config.max_expansions = max as usize;
    }
    if let Some(def) = payload.get("include_definition").and_then(|v| v.as_bool()) {
        config.include_definition = def;
    }
    if let Some(feat) = payload.get("include_features").and_then(|v| v.as_bool()) {
        config.include_features = feat;
    }
    if let Some(arch) = payload
        .get("include_architecture")
        .and_then(|v| v.as_bool())
    {
        config.include_architecture = arch;
    }

    match expand_queries_baseline(query, &config) {
        Ok(expanded) => Ok(Json(json!({
            "original_query": query,
            "expanded_queries": expanded,
            "count": expanded.len(),
        }))),
        Err(e) => {
            error!("Expand queries error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Expand queries failed: {}",
                e
            )))
        }
    }
}

pub async fn broad_discovery(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{BroadDiscoveryConfig, broad_discovery as broad_fn};

    let queries = payload
        .get("queries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| create_validation_error("queries", "missing or invalid queries parameter"))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let k = payload.get("k").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let config = BroadDiscoveryConfig::default();

    let collections: Vec<_> = state
        .store
        .list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                crate::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();

    match broad_fn(
        &queries,
        &collections,
        k,
        &config,
        &state.store,
        &state.embedding_manager,
    )
    .await
    {
        Ok(chunks) => Ok(Json(json!({
            "chunks": chunks.iter().map(|c| json!({
                "collection": c.collection,
                "score": c.score,
                "content_preview": c.content.chars().take(100).collect::<String>(),
            })).collect::<Vec<_>>(),
            "count": chunks.len(),
        }))),
        Err(e) => {
            error!("Broad discovery error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn semantic_focus(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{SemanticFocusConfig, semantic_focus as focus_fn};

    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    let queries = payload
        .get("queries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| create_validation_error("queries", "missing or invalid queries parameter"))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let k = payload.get("k").and_then(|v| v.as_u64()).unwrap_or(15) as usize;

    let config = SemanticFocusConfig::default();

    let coll = state
        .store
        .get_collection(collection_name)
        .map_err(|e| ErrorResponse::from(e))?;

    let metadata = coll.metadata();
    let collection = crate::discovery::CollectionRef {
        name: collection_name.to_string(),
        dimension: metadata.config.dimension,
        vector_count: metadata.vector_count,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        tags: vec![],
    };

    match focus_fn(
        &collection,
        &queries,
        k,
        &config,
        &state.store,
        &state.embedding_manager,
    )
    .await
    {
        Ok(chunks) => Ok(Json(json!({
            "chunks": chunks.iter().map(|c| json!({
                "collection": c.collection,
                "score": c.score,
                "content_preview": c.content.chars().take(100).collect::<String>(),
            })).collect::<Vec<_>>(),
            "count": chunks.len(),
        }))),
        Err(e) => {
            error!("Semantic focus error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn promote_readme(Json(payload): Json<Value>) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{
        ChunkMetadata, ReadmePromotionConfig, ScoredChunk, promote_readme as promote_fn,
    };

    let chunks_json = payload
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| create_validation_error("chunks", "missing or invalid chunks parameter"))?;

    let chunks: Vec<ScoredChunk> = chunks_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            Some(ScoredChunk {
                collection: obj.get("collection")?.as_str()?.to_string(),
                doc_id: obj.get("doc_id")?.as_str()?.to_string(),
                content: obj.get("content")?.as_str()?.to_string(),
                score: obj.get("score")?.as_f64()? as f32,
                metadata: ChunkMetadata {
                    file_path: obj.get("file_path")?.as_str()?.to_string(),
                    chunk_index: obj.get("chunk_index")?.as_u64()? as usize,
                    file_extension: obj.get("file_extension")?.as_str()?.to_string(),
                    line_range: None,
                },
            })
        })
        .collect();

    let config = ReadmePromotionConfig::default();

    match promote_fn(&chunks, &config) {
        Ok(promoted) => Ok(Json(json!({
            "promoted_chunks": promoted.iter().map(|c| json!({
                "collection": c.collection,
                "file_path": c.metadata.file_path,
                "score": c.score,
            })).collect::<Vec<_>>(),
            "count": promoted.len(),
        }))),
        Err(e) => {
            error!("Promote README error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn compress_evidence(Json(payload): Json<Value>) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{
        ChunkMetadata, CompressionConfig, ScoredChunk, compress_evidence as compress_fn,
    };

    let chunks_json = payload
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| create_validation_error("chunks", "missing or invalid chunks parameter"))?;

    let max_bullets = payload
        .get("max_bullets")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;

    let max_per_doc = payload
        .get("max_per_doc")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;

    let chunks: Vec<ScoredChunk> = chunks_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            Some(ScoredChunk {
                collection: obj.get("collection")?.as_str()?.to_string(),
                doc_id: obj.get("doc_id")?.as_str()?.to_string(),
                content: obj.get("content")?.as_str()?.to_string(),
                score: obj.get("score")?.as_f64()? as f32,
                metadata: ChunkMetadata {
                    file_path: obj.get("file_path")?.as_str()?.to_string(),
                    chunk_index: obj.get("chunk_index")?.as_u64()? as usize,
                    file_extension: obj.get("file_extension")?.as_str()?.to_string(),
                    line_range: None,
                },
            })
        })
        .collect();

    let config = CompressionConfig::default();

    match compress_fn(&chunks, max_bullets, max_per_doc, &config) {
        Ok(bullets) => Ok(Json(json!({
            "bullets": bullets.iter().map(|b| json!({
                "text": b.text,
                "source_id": b.source_id,
                "category": format!("{:?}", b.category),
                "score": b.score,
            })).collect::<Vec<_>>(),
            "count": bullets.len(),
        }))),
        Err(e) => {
            error!("Compress evidence error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn build_answer_plan(Json(payload): Json<Value>) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{
        AnswerPlanConfig, Bullet, BulletCategory, build_answer_plan as build_fn,
    };

    let bullets_json = payload
        .get("bullets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            create_validation_error("bullets", "missing or invalid bullets parameter")
        })?;

    let bullets: Vec<Bullet> = bullets_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let category = match obj.get("category")?.as_str()? {
                "Definition" => BulletCategory::Definition,
                "Feature" => BulletCategory::Feature,
                "Architecture" => BulletCategory::Architecture,
                "Performance" => BulletCategory::Performance,
                "Integration" => BulletCategory::Integration,
                "UseCase" => BulletCategory::UseCase,
                _ => BulletCategory::Other,
            };

            Some(Bullet {
                text: obj.get("text")?.as_str()?.to_string(),
                source_id: obj.get("source_id")?.as_str()?.to_string(),
                collection: obj.get("collection")?.as_str()?.to_string(),
                file_path: obj.get("file_path")?.as_str()?.to_string(),
                score: obj.get("score")?.as_f64()? as f32,
                category,
            })
        })
        .collect();

    let config = AnswerPlanConfig::default();

    match build_fn(&bullets, &config) {
        Ok(plan) => Ok(Json(json!({
            "sections": plan.sections.iter().map(|s| json!({
                "title": s.title,
                "bullets_count": s.bullets.len(),
                "bullets": s.bullets.iter().map(|b| json!({
                    "text": b.text,
                    "source_id": b.source_id,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "total_bullets": plan.total_bullets,
            "sources": plan.sources,
        }))),
        Err(e) => {
            error!("Build answer plan error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

pub async fn render_llm_prompt(Json(payload): Json<Value>) -> Result<Json<Value>, ErrorResponse> {
    use crate::discovery::{
        AnswerPlan, Bullet, BulletCategory, PromptRenderConfig, Section, SectionType,
        render_llm_prompt as render_fn,
    };

    let plan_json = payload
        .get("plan")
        .and_then(|v| v.as_object())
        .ok_or_else(|| create_validation_error("plan", "missing or invalid plan parameter"))?;

    let sections_json = plan_json
        .get("sections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            create_validation_error("sections", "missing or invalid sections parameter")
        })?;

    let sections: Vec<Section> = sections_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let bullets_json = obj.get("bullets")?.as_array()?;

            let bullets: Vec<Bullet> = bullets_json
                .iter()
                .filter_map(|b| {
                    let b_obj = b.as_object()?;
                    let category = match b_obj.get("category")?.as_str()? {
                        "Definition" => BulletCategory::Definition,
                        "Feature" => BulletCategory::Feature,
                        "Architecture" => BulletCategory::Architecture,
                        "Performance" => BulletCategory::Performance,
                        "Integration" => BulletCategory::Integration,
                        "UseCase" => BulletCategory::UseCase,
                        _ => BulletCategory::Other,
                    };

                    Some(Bullet {
                        text: b_obj.get("text")?.as_str()?.to_string(),
                        source_id: b_obj.get("source_id")?.as_str()?.to_string(),
                        collection: b_obj.get("collection")?.as_str()?.to_string(),
                        file_path: b_obj.get("file_path")?.as_str()?.to_string(),
                        score: b_obj.get("score")?.as_f64()? as f32,
                        category,
                    })
                })
                .collect();

            Some(Section {
                title: obj.get("title")?.as_str()?.to_string(),
                section_type: SectionType::Definition,
                bullets,
                priority: obj.get("priority")?.as_u64()? as usize,
            })
        })
        .collect();

    let plan = AnswerPlan {
        sections,
        total_bullets: plan_json
            .get("total_bullets")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        sources: plan_json
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    };

    let config = PromptRenderConfig::default();

    match render_fn(&plan, &config) {
        Ok(prompt) => Ok(Json(json!({
            "prompt": prompt,
            "length": prompt.len(),
            "estimated_tokens": prompt.len() / 4,
        }))),
        Err(e) => {
            error!("Render LLM prompt error: {:?}", e);
            Err(create_bad_request_error(&format!(
                "Operation failed: {}",
                e
            )))
        }
    }
}

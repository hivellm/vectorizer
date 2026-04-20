//! Discovery tool handlers for MCP

use std::sync::Arc;

use rmcp::model::{CallToolRequestParam, CallToolResult, Content, ErrorData};
use serde_json::json;

use crate::VectorStore;
use crate::discovery::{
    AnswerPlan, AnswerPlanConfig, BroadDiscoveryConfig, Bullet, BulletCategory, ChunkMetadata,
    CollectionRef, CompressionConfig, Discovery, DiscoveryConfig, ExpansionConfig,
    PromptRenderConfig, ReadmePromotionConfig, ScoredChunk, ScoringConfig, Section, SectionType,
    SemanticFocusConfig, broad_discovery, build_answer_plan, compress_evidence,
    expand_queries_baseline, filter_collections, promote_readme, render_llm_prompt,
    score_collections, semantic_focus,
};
use crate::embedding::EmbeddingManager;

// Helper to convert store collections to CollectionRef
fn get_collection_refs(store: &Arc<VectorStore>) -> Vec<CollectionRef> {
    store
        .list_collections()
        .iter()
        .filter_map(|name| {
            store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect()
}

pub async fn handle_discover(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let mut config = DiscoveryConfig::default();

    if let Some(include) = args.get("include_collections").and_then(|v| v.as_array()) {
        config.include_collections = include
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }

    if let Some(exclude) = args.get("exclude_collections").and_then(|v| v.as_array()) {
        config.exclude_collections = exclude
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }

    if let Some(max_bullets) = args.get("max_bullets").and_then(|v| v.as_u64()) {
        config.max_bullets = max_bullets as usize;
    }

    if let Some(broad_k) = args.get("broad_k").and_then(|v| v.as_u64()) {
        config.broad_k = broad_k as usize;
    }

    if let Some(focus_k) = args.get("focus_k").and_then(|v| v.as_u64()) {
        config.focus_k = focus_k as usize;
    }

    let discovery = Discovery::new(config, store, embedding_manager);
    let response = discovery
        .discover(query)
        .await
        .map_err(|e| ErrorData::internal_error(format!("Discovery failed: {}", e), None))?;

    let result = json!({
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
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_filter_collections(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let include: Vec<&str> = args
        .get("include")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let exclude: Vec<&str> = args
        .get("exclude")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let all_collections = get_collection_refs(&store);
    let filtered = filter_collections(query, &include, &exclude, &all_collections)
        .map_err(|e| ErrorData::internal_error(format!("Filter failed: {}", e), None))?;

    let result = json!({
        "filtered_collections": filtered.iter().map(|c| json!({
            "name": c.name,
            "vector_count": c.vector_count,
        })).collect::<Vec<_>>(),
        "count": filtered.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_score_collections(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let all_collections = get_collection_refs(&store);

    let mut config = ScoringConfig::default();
    if let Some(w) = args.get("name_match_weight").and_then(|v| v.as_f64()) {
        config.name_match_weight = w as f32;
    }
    if let Some(w) = args.get("term_boost_weight").and_then(|v| v.as_f64()) {
        config.term_boost_weight = w as f32;
    }
    if let Some(w) = args.get("signal_boost_weight").and_then(|v| v.as_f64()) {
        config.signal_boost_weight = w as f32;
    }

    let query_terms: Vec<&str> = query.split_whitespace().collect();
    let scored = score_collections(&query_terms, &all_collections, &config)
        .map_err(|e| ErrorData::internal_error(format!("Scoring failed: {}", e), None))?;

    let result = json!({
        "scored_collections": scored.iter().map(|(c, score)| json!({
            "name": c.name,
            "score": score,
            "vector_count": c.vector_count,
        })).collect::<Vec<_>>(),
        "count": scored.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_expand_queries(
    request: CallToolRequestParam,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let mut config = ExpansionConfig::default();

    if let Some(max) = args.get("max_expansions").and_then(|v| v.as_u64()) {
        config.max_expansions = max as usize;
    }
    if let Some(def) = args.get("include_definition").and_then(|v| v.as_bool()) {
        config.include_definition = def;
    }
    if let Some(feat) = args.get("include_features").and_then(|v| v.as_bool()) {
        config.include_features = feat;
    }
    if let Some(arch) = args.get("include_architecture").and_then(|v| v.as_bool()) {
        config.include_architecture = arch;
    }

    let expanded = expand_queries_baseline(query, &config)
        .map_err(|e| ErrorData::internal_error(format!("Expansion failed: {}", e), None))?;

    let result = json!({
        "original_query": query,
        "expanded_queries": expanded,
        "count": expanded.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_broad_discovery(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let queries = args
        .get("queries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing queries array", None))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let config = BroadDiscoveryConfig::default();
    let collections = get_collection_refs(&store);

    let chunks = broad_discovery(
        &queries,
        &collections,
        k,
        &config,
        &store,
        &embedding_manager,
    )
    .await
    .map_err(|e| ErrorData::internal_error(format!("Broad discovery failed: {}", e), None))?;

    let result = json!({
        "chunks": chunks.iter().map(|c| json!({
            "collection": c.collection,
            "score": c.score,
            "content_preview": c.content.chars().take(100).collect::<String>(),
        })).collect::<Vec<_>>(),
        "count": chunks.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_semantic_focus(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let queries = args
        .get("queries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing queries array", None))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(15) as usize;

    let config = SemanticFocusConfig::default();

    let coll = store
        .get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;

    let metadata = coll.metadata();
    let collection = CollectionRef {
        name: collection_name.to_string(),
        dimension: metadata.config.dimension,
        vector_count: metadata.vector_count,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        tags: vec![],
    };

    let chunks = semantic_focus(
        &collection,
        &queries,
        k,
        &config,
        &store,
        &embedding_manager,
    )
    .await
    .map_err(|e| ErrorData::internal_error(format!("Semantic focus failed: {}", e), None))?;

    let result = json!({
        "chunks": chunks.iter().map(|c| json!({
            "collection": c.collection,
            "score": c.score,
            "content_preview": c.content.chars().take(100).collect::<String>(),
        })).collect::<Vec<_>>(),
        "count": chunks.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_promote_readme(
    request: CallToolRequestParam,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let chunks_json = args
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing chunks array", None))?;

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
    let promoted = promote_readme(&chunks, &config)
        .map_err(|e| ErrorData::internal_error(format!("README promotion failed: {}", e), None))?;

    let result = json!({
        "promoted_chunks": promoted.iter().map(|c| json!({
            "collection": c.collection,
            "file_path": c.metadata.file_path,
            "score": c.score,
        })).collect::<Vec<_>>(),
        "count": promoted.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_compress_evidence(
    request: CallToolRequestParam,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let chunks_json = args
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing chunks array", None))?;

    let max_bullets = args
        .get("max_bullets")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;
    let max_per_doc = args
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
    let bullets = compress_evidence(&chunks, max_bullets, max_per_doc, &config).map_err(|e| {
        ErrorData::internal_error(format!("Evidence compression failed: {}", e), None)
    })?;

    let result = json!({
        "bullets": bullets.iter().map(|b| json!({
            "text": b.text,
            "source_id": b.source_id,
            "category": format!("{:?}", b.category),
            "score": b.score,
        })).collect::<Vec<_>>(),
        "count": bullets.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_build_answer_plan(
    request: CallToolRequestParam,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let bullets_json = args
        .get("bullets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing bullets array", None))?;

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
    let plan = build_answer_plan(&bullets, &config)
        .map_err(|e| ErrorData::internal_error(format!("Plan building failed: {}", e), None))?;

    let result = json!({
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
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

pub async fn handle_render_llm_prompt(
    request: CallToolRequestParam,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let plan_json = args
        .get("plan")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ErrorData::invalid_params("Missing plan object", None))?;

    let sections_json = plan_json
        .get("sections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing sections", None))?;

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
    let prompt = render_llm_prompt(&plan, &config)
        .map_err(|e| ErrorData::internal_error(format!("Prompt rendering failed: {}", e), None))?;

    let result = json!({
        "prompt": prompt,
        "length": prompt.len(),
        "estimated_tokens": prompt.len() / 4,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

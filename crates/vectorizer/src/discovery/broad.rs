//! Broad discovery with multi-query search

use std::collections::HashMap;
use std::sync::Arc;

use super::config::BroadDiscoveryConfig;
use super::types::{ChunkMetadata, CollectionRef, DiscoveryError, DiscoveryResult, ScoredChunk};
use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Multi-query broad search with MMR deduplication
///
/// This function performs a broad search across multiple collections using
/// multiple query variations. It combines results from all queries and applies
/// MMR (Maximal Marginal Relevance) for diversity.
pub async fn broad_discovery(
    queries: &[String],
    collections: &[CollectionRef],
    k: usize,
    config: &BroadDiscoveryConfig,
    store: &Arc<VectorStore>,
    embedding_manager: &Arc<EmbeddingManager>,
) -> DiscoveryResult<Vec<ScoredChunk>> {
    let mut all_results = Vec::new();
    let k_per_query = config.k_per_query;

    // Execute all queries across all collections
    for query in queries {
        // Embed the query
        let query_embedding = embedding_manager
            .embed(query)
            .map_err(|e| DiscoveryError::SearchError(format!("Embedding error: {}", e)))?;

        for collection in collections {
            // Search in this collection
            match store.search(&collection.name, &query_embedding, k_per_query) {
                Ok(results) => {
                    // Convert search results to ScoredChunks
                    for result in results {
                        // Extract metadata from the document
                        let metadata = extract_metadata(&result.id, &collection.name);

                        // Extract text from payload - try both "text" and "content" fields
                        let content = result
                            .payload
                            .as_ref()
                            .and_then(|p| {
                                // Try "content" first (most common)
                                p.data
                                    .get("content")
                                    .or_else(|| p.data.get("text"))
                                    .and_then(|v| v.as_str())
                            })
                            .unwrap_or("")
                            .to_string();

                        // Skip empty content
                        if content.is_empty() {
                            continue;
                        }

                        let chunk = ScoredChunk {
                            collection: collection.name.clone(),
                            doc_id: result.id.clone(),
                            content,
                            score: result.score,
                            metadata,
                        };

                        all_results.push(chunk);
                    }
                }
                Err(e) => {
                    // Log error but continue with other collections
                    tracing::warn!("Search error in collection {}: {}", collection.name, e);
                }
            }
        }
    }

    // Filter by similarity threshold
    all_results.retain(|chunk| chunk.score >= config.similarity_threshold);

    // Deduplicate if enabled
    if config.enable_deduplication {
        all_results = deduplicate_chunks(all_results, config.dedup_threshold);
    }

    // Apply MMR for diversity
    let final_results = apply_mmr(all_results, k, config.mmr_lambda);

    Ok(final_results)
}

/// Extract metadata from document ID
/// Format: "collection_name::file_path::chunk_index"
fn extract_metadata(doc_id: &str, collection_name: &str) -> ChunkMetadata {
    let parts: Vec<&str> = doc_id.split("::").collect();

    let file_path = parts.get(1).unwrap_or(&"unknown").to_string();
    let chunk_index = parts
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let file_extension = std::path::Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt")
        .to_string();

    ChunkMetadata {
        file_path,
        chunk_index,
        file_extension,
        line_range: None,
    }
}

/// Calculate content similarity between two chunks
pub fn content_similarity(a: &str, b: &str) -> f32 {
    // Simple Jaccard similarity on words
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Deduplicate chunks by content similarity
pub fn deduplicate_chunks(mut chunks: Vec<ScoredChunk>, threshold: f32) -> Vec<ScoredChunk> {
    let mut unique = Vec::new();

    for chunk in chunks.drain(..) {
        let is_duplicate = unique.iter().any(|existing: &ScoredChunk| {
            content_similarity(&chunk.content, &existing.content) > threshold
        });

        if !is_duplicate {
            unique.push(chunk);
        } else {
            // Keep the one with higher score
            if let Some(pos) = unique
                .iter()
                .position(|c| content_similarity(&chunk.content, &c.content) > threshold)
            {
                if chunk.score > unique[pos].score {
                    unique[pos] = chunk;
                }
            }
        }
    }

    unique
}

/// Apply MMR (Maximal Marginal Relevance) for diversity
pub fn apply_mmr(chunks: Vec<ScoredChunk>, k: usize, lambda: f32) -> Vec<ScoredChunk> {
    let mut selected = Vec::new();
    let mut candidates = chunks;

    if candidates.is_empty() {
        return selected;
    }

    // Select first item (highest score)
    if let Some(pos) = candidates
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(pos, _)| pos)
    {
        let first = candidates.remove(pos);
        selected.push(first);
    }

    // MMR selection loop
    while selected.len() < k && !candidates.is_empty() {
        let mut best_mmr_score = f32::MIN;
        let mut best_idx = 0;

        for (idx, candidate) in candidates.iter().enumerate() {
            // Relevance score
            let relevance = candidate.score;

            // Max similarity to already selected
            let max_sim = selected
                .iter()
                .map(|s| content_similarity(&candidate.content, &s.content))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            // MMR score
            let mmr_score = lambda * relevance - (1.0 - lambda) * max_sim;

            if mmr_score > best_mmr_score {
                best_mmr_score = mmr_score;
                best_idx = idx;
            }
        }

        let best = candidates.remove(best_idx);
        selected.push(best);
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::ChunkMetadata;

    fn create_test_chunk(content: &str, score: f32) -> ScoredChunk {
        ScoredChunk {
            collection: "test".to_string(),
            doc_id: "doc1".to_string(),
            content: content.to_string(),
            score,
            metadata: ChunkMetadata {
                file_path: "test.md".to_string(),
                chunk_index: 0,
                file_extension: "md".to_string(),
                line_range: None,
            },
        }
    }

    #[test]
    fn test_content_similarity() {
        let sim = content_similarity("hello world", "hello world");
        assert!((sim - 1.0).abs() < 0.01);

        let sim = content_similarity("hello world", "goodbye world");
        assert!(sim > 0.0 && sim < 1.0);

        let sim = content_similarity("hello world", "completely different");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_deduplicate_chunks() {
        let chunks = vec![
            create_test_chunk("hello world", 0.9),
            create_test_chunk("hello world again", 0.8),
            create_test_chunk("completely different", 0.7),
        ];

        let unique = deduplicate_chunks(chunks, 0.5);
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_apply_mmr() {
        let chunks = vec![
            create_test_chunk("first document", 0.9),
            create_test_chunk("first document similar", 0.8),
            create_test_chunk("second document", 0.7),
            create_test_chunk("third document", 0.6),
        ];

        let selected = apply_mmr(chunks, 3, 0.7);
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].score, 0.9);
    }
}

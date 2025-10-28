//! Semantic focus search

use std::sync::Arc;

use futures_util::future::TryFutureExt;

use super::config::SemanticFocusConfig;
use super::types::{ChunkMetadata, CollectionRef, DiscoveryError, DiscoveryResult, ScoredChunk};
use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Deep semantic search within specific high-priority collections
///
/// This function performs focused semantic search in a specific collection,
/// applying semantic reranking and optionally adding context chunks.
pub async fn semantic_focus(
    collection: &CollectionRef,
    queries: &[String],
    k: usize,
    config: &SemanticFocusConfig,
    store: &Arc<VectorStore>,
    embedding_manager: &Arc<EmbeddingManager>,
) -> DiscoveryResult<Vec<ScoredChunk>> {
    let mut all_chunks = Vec::new();

    // Search with all query variations
    for query in queries {
        // Embed the query
        let embedding_result = embedding_manager
            .embed(query)
            .await
            .map_err(|e| DiscoveryError::SearchError(format!("Embedding error: {}", e)))?;
        let query_embedding = &embedding_result.embedding;

        // Search in the collection
        match store.search(&collection.name, &query_embedding, k * 2) {
            Ok(results) => {
                for result in results {
                    let metadata = extract_metadata(&result.id);

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

                    all_chunks.push(chunk);
                }
            }
            Err(e) => {
                tracing::warn!("Search error in collection {}: {}", collection.name, e);
            }
        }
    }

    // Filter by higher threshold
    all_chunks.retain(|c| c.score >= config.similarity_threshold);

    // Semantic reranking
    if config.semantic_reranking && !queries.is_empty() {
        all_chunks = rerank_semantically(&queries[0], all_chunks);
    }

    // Sort and truncate
    all_chunks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    all_chunks.truncate(k);

    Ok(all_chunks)
}

/// Extract metadata from document ID
/// Format: "collection_name::file_path::chunk_index"
fn extract_metadata(doc_id: &str) -> ChunkMetadata {
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

/// Rerank chunks semantically
fn rerank_semantically(query: &str, chunks: Vec<ScoredChunk>) -> Vec<ScoredChunk> {
    chunks
        .into_iter()
        .map(|mut chunk| {
            let base_score = chunk.score;

            // Term frequency boost
            let tf_boost = term_frequency_score(query, &chunk.content);

            // Sentence quality boost
            let quality_boost = sentence_quality_score(&chunk.content);

            // Position boost (earlier chunks = more important)
            let position_boost = 1.0 / (1.0 + chunk.metadata.chunk_index as f32 * 0.1);

            // Combine scores
            chunk.score =
                base_score * 0.6 + tf_boost * 0.2 + quality_boost * 0.1 + position_boost * 0.1;
            chunk
        })
        .collect()
}

/// Score sentence quality
pub fn sentence_quality_score(sentence: &str) -> f32 {
    let mut score: f32 = 0.5;

    // Length score
    let word_count = sentence.split_whitespace().count();
    if (8..=30).contains(&word_count) {
        score += 0.2;
    }

    // Has proper capitalization
    if sentence
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
        score += 0.1;
    }

    // Ends with punctuation
    if sentence.ends_with(['.', '!', '?']) {
        score += 0.1;
    }

    // Contains key indicators
    let indicators = ["is", "provides", "supports", "enables", "allows"];
    if indicators
        .iter()
        .any(|i| sentence.to_lowercase().contains(i))
    {
        score += 0.1;
    }

    score.min(1.0)
}

/// Calculate term frequency score
pub fn term_frequency_score(query: &str, content: &str) -> f32 {
    let query_terms: Vec<&str> = query.split_whitespace().collect();
    if query_terms.is_empty() {
        return 0.0;
    }

    let content_lower = content.to_lowercase();
    let matches = query_terms
        .iter()
        .filter(|term| content_lower.contains(&term.to_lowercase()))
        .count();

    (matches as f32) / (query_terms.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_quality_score() {
        let good = "This is a well-formed sentence with proper structure.";
        let bad = "this no good";

        assert!(sentence_quality_score(good) > sentence_quality_score(bad));
    }

    #[test]
    fn test_term_frequency_score() {
        let query = "vectorizer database";
        let content = "The vectorizer is a database system for vectors";

        let score = term_frequency_score(query, content);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
}

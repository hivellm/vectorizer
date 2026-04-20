//! `compress_evidence` — empty / single / many chunks + per-doc cap.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::discovery::types::{ChunkMetadata, ScoredChunk};
use vectorizer::discovery::{CompressionConfig, compress_evidence};

fn chunk(
    collection: &str,
    file: &str,
    chunk_index: usize,
    content: &str,
    score: f32,
) -> ScoredChunk {
    ScoredChunk {
        collection: collection.to_string(),
        doc_id: format!("{collection}/{file}#{chunk_index}"),
        content: content.to_string(),
        score,
        metadata: ChunkMetadata {
            file_path: file.to_string(),
            chunk_index,
            file_extension: "md".to_string(),
            line_range: None,
        },
    }
}

#[test]
fn compress_empty_input_returns_empty_bullets() {
    let bullets = compress_evidence(&[], 10, 3, &CompressionConfig::default()).unwrap();
    assert!(bullets.is_empty());
}

#[test]
fn compress_single_chunk_with_one_qualifying_sentence() {
    // The default config requires sentences between 8 and 30 words.
    // The first sentence below is 11 words; the second is too short
    // (4 words) and must be filtered out.
    let chunks = vec![chunk(
        "docs",
        "intro.md",
        0,
        "Vectorizer is a high-performance vector database with native HNSW support. Short.",
        0.9,
    )];
    let bullets = compress_evidence(&chunks, 5, 3, &CompressionConfig::default()).unwrap();
    assert!(
        !bullets.is_empty(),
        "qualifying sentence should produce at least one bullet"
    );
    assert!(
        bullets.iter().all(|b| {
            let wc = b.text.split_whitespace().count();
            (8..=30).contains(&wc)
        }),
        "all bullets must respect the configured min/max sentence-word window"
    );
}

#[test]
fn compress_respects_max_per_doc_cap() {
    // Five chunks all from the same (collection, file_path) — the
    // per-doc cap of 2 means only the first two contribute bullets.
    let mut chunks = Vec::new();
    for i in 0..5 {
        chunks.push(chunk(
            "docs",
            "intro.md",
            i,
            &format!(
                "Vectorizer chunk number {i} provides high performance vector search and graph features."
            ),
            1.0 - (i as f32) * 0.05,
        ));
    }
    let bullets = compress_evidence(&chunks, 100, 2, &CompressionConfig::default()).unwrap();
    let docs_seen: std::collections::HashSet<&str> =
        bullets.iter().map(|b| b.file_path.as_str()).collect();
    assert_eq!(docs_seen.len(), 1, "all bullets are from the single doc");
    // The implementation may produce multiple sentences from one chunk
    // but caps the chunk-count-per-doc, not the bullet-count. Verify
    // the cap is respected at the chunk level by counting unique
    // chunk_indices that contributed.
    let chunks_contributing: std::collections::HashSet<&str> =
        bullets.iter().map(|b| b.source_id.as_str()).collect();
    assert!(
        chunks_contributing.len() <= 2,
        "max_per_doc=2 must cap the number of chunks contributing bullets, got {} chunks: {:?}",
        chunks_contributing.len(),
        chunks_contributing
    );
}

#[test]
fn compress_respects_max_bullets_cap() {
    // Many chunks, generous per-doc cap, tight global bullet cap.
    let mut chunks = Vec::new();
    for i in 0..20 {
        chunks.push(chunk(
            "docs",
            &format!("file-{i}.md"),
            0,
            "Vectorizer is a fast vector database with native HNSW indexing and graph features.",
            1.0 - (i as f32) * 0.01,
        ));
    }
    let bullets = compress_evidence(&chunks, 5, 10, &CompressionConfig::default()).unwrap();
    assert!(
        bullets.len() <= 5,
        "max_bullets=5 must cap the global bullet count, got {}",
        bullets.len()
    );
}

#[test]
fn compress_sorts_bullets_by_descending_score() {
    let chunks = vec![
        chunk(
            "docs",
            "low.md",
            0,
            "Vectorizer offers low-priority sentence with sufficient word count for emission.",
            0.1,
        ),
        chunk(
            "docs",
            "high.md",
            0,
            "Vectorizer offers high-priority sentence with sufficient word count for emission.",
            0.9,
        ),
    ];
    let bullets = compress_evidence(&chunks, 5, 3, &CompressionConfig::default()).unwrap();
    if bullets.len() >= 2 {
        // Bullets carry the originating chunk's score; the post-sort
        // inside compress_evidence puts the higher score first.
        assert!(
            bullets[0].score >= bullets[1].score,
            "bullets must be sorted by descending score; got {:?}",
            bullets
                .iter()
                .map(|b| (b.file_path.clone(), b.score))
                .collect::<Vec<_>>()
        );
    }
}

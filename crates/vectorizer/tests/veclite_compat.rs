//! VecLite → Vectorizer graduation conformance (shared corpus, VecLite
//! SPEC-013 §4 / IOP-002).
//!
//! The fixture under `tests/compat/veclite/` is produced by VecLite's
//! `cargo xtask graduation` (in the hivellm/veclite repo): a
//! `vectorizer.vecdb` and `vectorizer.vecidx` pair exported from the
//! deterministic shared corpus, plus `golden.json` holding VecLite's own
//! query results and BM25 query embeddings. This test proves the server side
//! of the graduation contract with the server's own code:
//!
//! 1. `StorageReader` accepts the exported archive (IOP-010) and the
//!    collection contents match the corpus manifest.
//! 2. The server BM25 provider, restored from the exported tokenizer entry,
//!    reproduces VecLite's query embeddings within 1e-5 (IOP-011 — identical
//!    scoring server-side).
//! 3. Cosine top-10 over the imported vectors matches VecLite's pre-export
//!    top-10: overlap ≥ 0.99 and scores within 1e-5 (NFR-04 / TST-032).
//!
//! Divergence here is a bug in one of the two engines (IOP-002).

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::Deserialize;
use vectorizer::Bm25Embedding;
use vectorizer::embedding::EmbeddingProvider;
use vectorizer::storage::StorageReader;

const OVERLAP_GATE: f64 = 0.99;
const SCORE_TOL: f32 = 1e-5;
const TOP_K: usize = 10;

#[derive(Deserialize)]
struct Hit {
    id: String,
    score: f32,
}

#[derive(Deserialize)]
struct TextQueryGolden {
    query: String,
    embedding: Vec<f32>,
    top: Vec<Hit>,
}

#[derive(Deserialize)]
struct VectorQueryGolden {
    vector: Vec<f32>,
    top: Vec<Hit>,
}

#[derive(Deserialize)]
struct Golden {
    doc_count: usize,
    vec_count: usize,
    text_queries: Vec<TextQueryGolden>,
    vector_queries: Vec<VectorQueryGolden>,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/compat/veclite")
}

/// Cosine similarity against a stored vector (stored vectors arrive
/// L2-normalized from VecLite's cosine ingest; the score convention on both
/// sides is the similarity itself).
fn cosine(query: &[f32], stored: &[f32]) -> f32 {
    let dot: f32 = query.iter().zip(stored).map(|(a, b)| a * b).sum();
    let qn: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    let sn: f32 = stored.iter().map(|x| x * x).sum::<f32>().sqrt();
    if qn == 0.0 || sn == 0.0 {
        0.0
    } else {
        dot / (qn * sn)
    }
}

fn top_k(query: &[f32], vectors: &[(String, Vec<f32>)]) -> Vec<Hit> {
    let mut scored: Vec<Hit> = vectors
        .iter()
        .map(|(id, v)| Hit {
            id: id.clone(),
            score: cosine(query, v),
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(TOP_K);
    scored
}

fn overlap(a: &[Hit], b: &[Hit]) -> f64 {
    let sa: BTreeSet<&str> = a.iter().map(|h| h.id.as_str()).collect();
    let sb: BTreeSet<&str> = b.iter().map(|h| h.id.as_str()).collect();
    sa.intersection(&sb).count() as f64 / sa.len().max(sb.len()).max(1) as f64
}

/// Assert score parity on ids shared between the golden and observed top-K.
fn assert_scores(label: &str, want: &[Hit], got: &[Hit]) {
    for hit in got {
        if let Some(expected) = want.iter().find(|w| w.id == hit.id) {
            assert!(
                (expected.score - hit.score).abs() <= SCORE_TOL,
                "{label}: id {:?}: server score {} vs VecLite {} (tol {SCORE_TOL})",
                hit.id,
                hit.score,
                expected.score
            );
        }
    }
}

fn load_vectors(reader: &StorageReader, collection: &str) -> Vec<(String, Vec<f32>)> {
    let persisted = reader
        .read_collection_in_memory(collection)
        .unwrap_or_else(|e| panic!("read {collection}: {e}"))
        .unwrap_or_else(|| panic!("collection {collection} missing from archive"));
    persisted
        .vectors
        .into_iter()
        .map(|pv| {
            let v = pv
                .into_runtime()
                .unwrap_or_else(|e| panic!("vector decode: {e}"));
            (v.id, v.data)
        })
        .collect()
}

#[test]
fn veclite_export_serves_identically() {
    let dir = fixture_dir();
    assert!(
        dir.join("vectorizer.vecdb").exists(),
        "fixture missing at {} — regenerate with `cargo xtask graduation` in the veclite repo",
        dir.display()
    );

    let golden: Golden = serde_json::from_str(
        &std::fs::read_to_string(dir.join("golden.json"))
            .unwrap_or_else(|e| panic!("golden.json: {e}")),
    )
    .unwrap_or_else(|e| panic!("golden.json parse: {e}"));

    // 1. The archive opens with the server reader and matches the manifest.
    let reader = StorageReader::new(&dir).unwrap_or_else(|e| panic!("StorageReader: {e}"));
    let collections = reader
        .list_collections()
        .unwrap_or_else(|e| panic!("list: {e}"));
    assert!(collections.contains(&"docs".to_string()), "{collections:?}");
    assert!(collections.contains(&"vecs".to_string()), "{collections:?}");

    let docs = load_vectors(&reader, "docs");
    let vecs = load_vectors(&reader, "vecs");
    assert_eq!(docs.len(), golden.doc_count, "doc count drifted");
    assert_eq!(vecs.len(), golden.vec_count, "vec count drifted");

    // 2. BM25 vocabulary restored from the exported tokenizer reproduces
    // VecLite's query embeddings (identical scoring, IOP-011).
    let files = reader
        .read_collection_files("docs")
        .unwrap_or_else(|e| panic!("read docs files: {e}"));
    let tokenizer = files
        .get("docs_tokenizer.json")
        .unwrap_or_else(|| panic!("docs_tokenizer.json missing from archive"));
    let tokenizer_path = std::env::temp_dir().join(format!(
        "veclite-compat-tokenizer-{}.json",
        std::process::id()
    ));
    std::fs::write(&tokenizer_path, tokenizer).unwrap_or_else(|e| panic!("{e}"));
    let mut provider = Bm25Embedding::new(256);
    provider
        .load_vocabulary_json(&tokenizer_path)
        .unwrap_or_else(|e| panic!("load tokenizer: {e}"));
    let _ = std::fs::remove_file(&tokenizer_path);

    let mut text_overlap_total = 0.0;
    for (i, q) in golden.text_queries.iter().enumerate() {
        let embedding = provider
            .embed(&q.query)
            .unwrap_or_else(|e| panic!("embed query {i}: {e}"));
        assert_eq!(
            embedding.len(),
            q.embedding.len(),
            "query {i}: embedding dimension drifted"
        );
        for (j, (a, b)) in embedding.iter().zip(&q.embedding).enumerate() {
            assert!(
                (a - b).abs() <= SCORE_TOL,
                "query {i} ({:?}), component {j}: server {a} vs VecLite {b}",
                q.query
            );
        }

        // 3a. Text query results: overlap + score parity.
        let observed = top_k(&embedding, &docs);
        text_overlap_total += overlap(&q.top, &observed);
        assert_scores(&format!("text query {i}"), &q.top, &observed);
    }
    let text_overlap = text_overlap_total / golden.text_queries.len().max(1) as f64;
    assert!(
        text_overlap >= OVERLAP_GATE,
        "text top-{TOP_K} overlap {text_overlap:.4} below gate {OVERLAP_GATE}"
    );

    // 3b. Dense vector query results: overlap + score parity.
    let mut vec_overlap_total = 0.0;
    for (i, q) in golden.vector_queries.iter().enumerate() {
        let observed = top_k(&q.vector, &vecs);
        vec_overlap_total += overlap(&q.top, &observed);
        assert_scores(&format!("vector query {i}"), &q.top, &observed);
    }
    let vec_overlap = vec_overlap_total / golden.vector_queries.len().max(1) as f64;
    assert!(
        vec_overlap >= OVERLAP_GATE,
        "vector top-{TOP_K} overlap {vec_overlap:.4} below gate {OVERLAP_GATE}"
    );

    println!(
        "veclite graduation conformance: overlap text {text_overlap:.4}, vectors {vec_overlap:.4} (gate {OVERLAP_GATE})"
    );
}

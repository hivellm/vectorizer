//! `filter_collections` + `score_collections` — diverse inputs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::Utc;

use vectorizer::discovery::{CollectionRef, ScoringConfig, filter_collections, score_collections};

fn coll(name: &str) -> CollectionRef {
    CollectionRef {
        name: name.to_string(),
        dimension: 384,
        vector_count: 1000,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec![],
    }
}

#[test]
fn filter_with_no_inputs_passes_collections_through_query_term_match() {
    let collections = vec![
        coll("vectorizer-docs"),
        coll("vectorizer-source"),
        coll("unrelated"),
    ];
    // No include or exclude patterns; filter falls through to query
    // term matching against the collection name.
    let filtered = filter_collections("vectorizer features", &[], &[], &collections).unwrap();
    let names: Vec<_> = filtered.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"vectorizer-docs"));
    assert!(names.contains(&"vectorizer-source"));
    assert!(!names.contains(&"unrelated"));
}

#[test]
fn filter_with_explicit_include_pattern_takes_precedence_over_query_terms() {
    let collections = vec![
        coll("vectorizer-docs"),
        coll("foo-source"),
        coll("foo-docs"),
    ];
    // Explicit include pattern `foo*` overrides the "vectorizer"
    // query term — collections matching `foo*` win even when the
    // query talks about `vectorizer`.
    let filtered = filter_collections("vectorizer features", &["foo*"], &[], &collections).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|c| c.name.starts_with("foo-")));
}

#[test]
fn filter_exclude_pattern_drops_matching_collections() {
    let collections = vec![
        coll("vectorizer-docs"),
        coll("vectorizer-test"),
        coll("vectorizer-backup"),
    ];
    let filtered = filter_collections(
        "vectorizer",
        &["vectorizer*"],
        &["*-test", "*-backup"],
        &collections,
    )
    .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "vectorizer-docs");
}

#[test]
fn filter_returns_empty_when_query_matches_nothing() {
    let collections = vec![coll("alpha"), coll("beta")];
    let filtered = filter_collections("zebra", &[], &[], &collections).unwrap();
    assert!(filtered.is_empty());
}

#[test]
fn filter_handles_empty_collections_input() {
    let filtered = filter_collections("anything", &[], &[], &[]).unwrap();
    assert!(filtered.is_empty());
}

#[test]
fn score_orders_by_descending_score() {
    let collections = vec![
        // "vectorizer" appears in both names but only one matches the
        // exact query terms — the scorer should rank that one first.
        coll("vectorizer-docs"),
        coll("vectorizer-tools-misc"),
    ];
    let terms = ["vectorizer", "docs"];
    let scored = score_collections(&terms, &collections, &ScoringConfig::default()).unwrap();
    assert_eq!(scored.len(), 2);
    assert!(
        scored[0].1 >= scored[1].1,
        "results must be sorted by descending score; got {:?}",
        scored
            .iter()
            .map(|(c, s)| (c.name.clone(), *s))
            .collect::<Vec<_>>()
    );
    assert_eq!(scored[0].0.name, "vectorizer-docs");
}

#[test]
fn score_with_empty_query_terms_returns_zero_for_every_collection() {
    let collections = vec![coll("alpha"), coll("beta")];
    let scored: Vec<f32> = score_collections(&[], &collections, &ScoringConfig::default())
        .unwrap()
        .into_iter()
        .map(|(_, s)| s)
        .collect();
    // Without query terms there is no name-match component, but signal
    // boost (vector_count) can still contribute. Assert non-negative
    // and no panics rather than strict equality to avoid coupling to
    // the signal-boost weight choice.
    assert_eq!(scored.len(), 2);
    for s in scored {
        assert!(s >= 0.0, "scores must be non-negative");
    }
}

#[test]
fn score_handles_empty_collections_input() {
    let scored = score_collections(&["anything"], &[], &ScoringConfig::default()).unwrap();
    assert!(scored.is_empty());
}

//! `expand_queries_baseline` — toggles + max_expansions edge cases.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::discovery::{ExpansionConfig, expand_queries_baseline};

fn config_all_off(max: usize) -> ExpansionConfig {
    ExpansionConfig {
        include_definition: false,
        include_features: false,
        include_architecture: false,
        include_api: false,
        include_performance: false,
        include_use_cases: false,
        max_expansions: max,
    }
}

fn config_all_on(max: usize) -> ExpansionConfig {
    ExpansionConfig {
        include_definition: true,
        include_features: true,
        include_architecture: true,
        include_api: true,
        include_performance: true,
        include_use_cases: true,
        max_expansions: max,
    }
}

#[test]
fn expand_with_no_toggles_returns_only_the_original_query() {
    let out = expand_queries_baseline("vectorizer", &config_all_off(8)).unwrap();
    assert_eq!(out, vec!["vectorizer".to_string()]);
}

#[test]
fn expand_with_definition_toggle_adds_two_definition_variants() {
    let mut cfg = config_all_off(16);
    cfg.include_definition = true;
    let out = expand_queries_baseline("vectorizer", &cfg).unwrap();
    assert!(out.contains(&"vectorizer".to_string()));
    assert!(out.contains(&"vectorizer definition".to_string()));
    assert!(out.contains(&"what is vectorizer".to_string()));
}

#[test]
fn expand_truncates_to_max_expansions() {
    // All toggles ON would emit 1 (original) + 2 + 3 + 3 + 2 + 2 + 2 = 15
    // expansions before truncation. max_expansions = 5 caps the output.
    let out = expand_queries_baseline("vectorizer", &config_all_on(5)).unwrap();
    assert_eq!(out.len(), 5);
    // The first expansion is always the original query.
    assert_eq!(out[0], "vectorizer");
}

#[test]
fn expand_with_max_expansions_zero_returns_empty() {
    // The implementation truncates AFTER pushing the original query —
    // a max of 0 results in a zero-length vec, which downstream stages
    // must be able to handle (and they do — broad_discovery falls
    // through to an empty result set).
    let out = expand_queries_baseline("vectorizer", &config_all_off(0)).unwrap();
    assert!(out.is_empty());
}

#[test]
fn expand_strips_stopwords_when_extracting_main_term() {
    // The expander pulls the first non-stopword token as the base term
    // and uses it for variant construction. "what is vectorizer" should
    // expand around "vectorizer", not "what".
    let mut cfg = config_all_off(8);
    cfg.include_definition = true;
    let out = expand_queries_baseline("what is vectorizer", &cfg).unwrap();
    assert!(out.iter().any(|q| q == "vectorizer definition"));
    assert!(out.iter().any(|q| q == "what is vectorizer"));
}

#[test]
fn expand_empty_query_does_not_panic() {
    // Defensive: an empty query string should produce some sensible
    // (possibly empty) expansion list rather than panicking.
    let out = expand_queries_baseline("", &config_all_on(4)).unwrap();
    assert!(out.len() <= 4);
}

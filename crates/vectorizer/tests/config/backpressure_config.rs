//! Integration tests for `BackpressureConfig` (issue #263).
//!
//! Covers defaults, resolution of `max_concurrent_vocab_builds = 0` to
//! `num_cpus`, cross-field validation (inverted watermarks, zero
//! hard_limit, zero retry_after), YAML round-trip, partial-YAML
//! forward-compat, and env-var overrides on `VectorizerConfig::from_env`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::config::{BackpressureConfig, VectorizerConfig};

#[test]
fn default_matches_proposal() {
    let config = BackpressureConfig::default();

    assert!(config.enabled);
    assert_eq!(config.max_concurrent_vocab_builds, 0);
    assert_eq!(config.upsert_queue_high_water, 256);
    assert_eq!(config.upsert_queue_hard_limit, 1024);
    assert_eq!(config.retry_after_seconds, 2);
    assert!(config.read_path_isolated_runtime);
    assert_eq!(config.log_rate_limit_per_5s, 1);
}

#[test]
fn resolved_max_zero_uses_num_cpus() {
    let config = BackpressureConfig::default();
    let resolved = config.resolved_max_concurrent_vocab_builds();

    assert!(resolved >= 1, "must always return at least one permit");
    assert_eq!(resolved, num_cpus::get().max(1));
}

#[test]
fn resolved_max_explicit_value_wins() {
    let config = BackpressureConfig {
        max_concurrent_vocab_builds: 4,
        ..BackpressureConfig::default()
    };

    assert_eq!(config.resolved_max_concurrent_vocab_builds(), 4);
}

#[test]
fn validate_ok_for_default() {
    BackpressureConfig::default()
        .validate()
        .expect("default config must validate");
}

#[test]
fn validate_rejects_inverted_watermarks() {
    let config = BackpressureConfig {
        upsert_queue_high_water: 1024,
        upsert_queue_hard_limit: 256,
        ..BackpressureConfig::default()
    };

    let err = config
        .validate()
        .expect_err("inverted watermarks must fail validation");
    assert!(
        err.contains("must be <"),
        "error message should explain the ordering: {err}",
    );
}

#[test]
fn validate_rejects_zero_hard_limit() {
    let config = BackpressureConfig {
        upsert_queue_high_water: 0,
        upsert_queue_hard_limit: 0,
        ..BackpressureConfig::default()
    };

    let err = config
        .validate()
        .expect_err("hard_limit=0 must fail validation");
    assert!(err.contains("hard_limit"));
}

#[test]
fn validate_rejects_zero_retry_after() {
    let config = BackpressureConfig {
        retry_after_seconds: 0,
        ..BackpressureConfig::default()
    };

    let err = config
        .validate()
        .expect_err("retry_after_seconds=0 must fail validation");
    assert!(err.contains("retry_after_seconds"));
}

#[test]
fn yaml_round_trip() {
    let config = BackpressureConfig {
        enabled: true,
        max_concurrent_vocab_builds: 8,
        upsert_queue_high_water: 100,
        upsert_queue_hard_limit: 500,
        retry_after_seconds: 5,
        read_path_isolated_runtime: false,
        log_rate_limit_per_5s: 3,
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: BackpressureConfig = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(parsed.enabled, config.enabled);
    assert_eq!(
        parsed.max_concurrent_vocab_builds,
        config.max_concurrent_vocab_builds
    );
    assert_eq!(
        parsed.upsert_queue_high_water,
        config.upsert_queue_high_water
    );
    assert_eq!(
        parsed.upsert_queue_hard_limit,
        config.upsert_queue_hard_limit
    );
    assert_eq!(parsed.retry_after_seconds, config.retry_after_seconds);
    assert_eq!(
        parsed.read_path_isolated_runtime,
        config.read_path_isolated_runtime
    );
    assert_eq!(parsed.log_rate_limit_per_5s, config.log_rate_limit_per_5s);
}

#[test]
fn partial_yaml_uses_defaults() {
    // Operator only overrides one field — everything else stays at
    // proposal defaults. Guarantees forward-compat for existing
    // config.yml files that don't yet know about backpressure.
    let yaml = r"
        max_concurrent_vocab_builds: 16
    ";

    let parsed: BackpressureConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(parsed.max_concurrent_vocab_builds, 16);
    assert!(parsed.enabled);
    assert_eq!(parsed.upsert_queue_high_water, 256);
    assert_eq!(parsed.upsert_queue_hard_limit, 1024);
    assert_eq!(parsed.retry_after_seconds, 2);
    assert!(parsed.read_path_isolated_runtime);
    assert_eq!(parsed.log_rate_limit_per_5s, 1);
}

#[test]
fn embedded_in_vectorizer_config() {
    let config = VectorizerConfig::default();

    assert!(config.backpressure.enabled);
    assert_eq!(config.backpressure.upsert_queue_hard_limit, 1024);

    config
        .validate()
        .expect("default VectorizerConfig must validate end-to-end");
}

#[test]
fn vectorizer_config_validate_propagates_backpressure_error() {
    let mut config = VectorizerConfig::default();
    config.backpressure.upsert_queue_high_water = 2000;
    config.backpressure.upsert_queue_hard_limit = 100;

    let err = config
        .validate()
        .expect_err("inverted watermarks should fail at the top level");
    assert!(err.contains("must be <"));
}

#[test]
fn env_override_max_concurrent() {
    // SAFETY: env mutation is only safe in tests if no other test in
    // the same binary races on the same variable. The variable name
    // is unique to this test, so this remains isolated even under
    // cargo's default parallel test runner.
    let key = "CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS";
    // SAFETY: see above — single-writer for this key under test.
    unsafe {
        std::env::set_var(key, "12");
    }
    let config = VectorizerConfig::from_env();
    // SAFETY: see above.
    unsafe {
        std::env::remove_var(key);
    }

    assert_eq!(config.backpressure.max_concurrent_vocab_builds, 12);
    assert_eq!(
        config.backpressure.resolved_max_concurrent_vocab_builds(),
        12
    );
}

#[test]
fn env_override_enabled_falsey() {
    let key = "CORTEX_VECTORIZER_BACKPRESSURE_ENABLED";
    // SAFETY: single-writer for this key under test.
    unsafe {
        std::env::set_var(key, "false");
    }
    let config = VectorizerConfig::from_env();
    // SAFETY: see above.
    unsafe {
        std::env::remove_var(key);
    }

    assert!(!config.backpressure.enabled);
}

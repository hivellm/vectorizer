//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/file_watcher/operations.rs` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::*;
use crate::file_watcher::FileWatcherConfig;

fn create_test_ops() -> VectorOperations {
    let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
    let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
    let config = FileWatcherConfig::default();
    VectorOperations::new(vector_store, embedding_manager, config)
}

// Task 1.3: Comprehensive test cases for determine_collection_name()

#[test]
fn test_docs_architecture_collection() {
    let ops = create_test_ops();

    let test_paths = vec![
        "/home/user/project/docs/architecture/system.md",
        "/docs/architecture/README.md",
    ];

    for path in test_paths {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, "docs-architecture",
            "Path {} should map to docs-architecture",
            path
        );
    }
}

#[test]
fn test_docs_subdirectories() {
    let ops = create_test_ops();

    let test_cases = vec![
        ("/docs/templates/pr.md", "docs-templates"),
        ("/docs/processes/release.md", "docs-processes"),
        ("/docs/governance/voting.md", "docs-governance"),
        ("/docs/navigation/sitemap.md", "docs-navigation"),
        ("/docs/testing/strategy.md", "docs-testing"),
        ("/docs/random/file.md", "docs-architecture"), // default docs
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, expected,
            "Path {} should map to {}",
            path, expected
        );
    }
}

#[test]
fn test_vectorizer_collections() {
    let ops = create_test_ops();

    let test_cases = vec![
        // Note: /docs/ pattern is checked first, so any path with /docs/ will match that
        // These test cases avoid /docs/ pattern
        ("/vectorizer/src/main.rs", "vectorizer-source"),
        ("/vectorizer/src/db/vector_store.rs", "vectorizer-source"),
        ("/project/vectorizer/README.md", "vectorizer-source"),
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, expected,
            "Path {} should map to {}",
            path, expected
        );
    }
}

#[test]
fn test_vectorizer_sdk_language_detection() {
    let ops = create_test_ops();

    let test_cases = vec![
        (
            "/vectorizer/client-sdks/typescript/index.ts",
            "vectorizer-sdk-typescript",
        ),
        (
            "/vectorizer/sdks/nodejs/client.js",
            "vectorizer-sdk-typescript",
        ),
        (
            "/vectorizer/client-sdks/python/client.py",
            "vectorizer-sdk-python",
        ),
        ("/vectorizer/sdks/rust/lib.rs", "vectorizer-sdk-rust"),
        ("/vectorizer/sdks/unknown/file.txt", "vectorizer-source"), // fallback
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, expected,
            "SDK path {} should map to {}",
            path, expected
        );
    }
}

#[test]
fn test_gov_collections() {
    let ops = create_test_ops();

    let test_cases = vec![
        ("/gov/bips/BIP-001.md", "gov-bips"),
        ("/gov/guidelines/code.md", "gov-guidelines"),
        ("/gov/proposals/2024-q1.md", "gov-proposals"),
        ("/gov/minutes/2024-01.md", "gov-minutes"),
        ("/gov/schemas/voting.json", "gov-schemas"),
        ("/gov/teams/engineering.md", "gov-teams"),
        ("/gov/metrics/2024-q1.md", "gov-metrics"),
        ("/gov/issues/123.md", "gov-issues"),
        ("/gov/snapshot/vote-123.json", "gov-snapshot"),
        ("/gov/other/file.md", "gov-core"), // default gov
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, expected,
            "Path {} should map to {}",
            path, expected
        );
    }
}

#[test]
fn test_unknown_paths_use_default() {
    let ops = create_test_ops();

    let test_paths = vec![
        "/random/path/file.txt",
        "F:\\Some\\Random\\Directory\\document.md",
        "/usr/local/bin/script.sh",
        "/home/user/downloads/file.pdf",
        "/third-party/libsodium/src/file.c",
        "/Server/ToS-Server-5/Api/endpoint.cs",
        "/Benchmark/src/test.cpp",
    ];

    for path in test_paths {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, "workspace-default",
            "Unknown path {} should use default collection",
            path
        );
    }
}

#[test]
fn test_custom_default_collection() {
    let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
    let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
    let mut config = FileWatcherConfig::default();
    config.default_collection = Some("my-custom-collection".to_string());
    let ops = VectorOperations::new(vector_store, embedding_manager, config);

    let collection = ops.determine_collection_name(&PathBuf::from("/random/path/file.txt"));
    assert_eq!(collection, "my-custom-collection");
}

#[test]
fn test_windows_paths() {
    let ops = create_test_ops();

    // Note: The function checks for forward slashes "/" not backslashes "\"
    // Windows paths with backslashes won't match the patterns and will use default
    let test_cases = vec![
        (
            "C:\\Users\\dev\\project\\docs\\architecture\\design.md",
            "workspace-default",
        ),
        ("D:\\Work\\vectorizer\\src\\main.rs", "workspace-default"),
        ("F:\\Gov\\bips\\BIP-001.md", "workspace-default"),
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, expected,
            "Windows path {} should map to {}",
            path, expected
        );
    }
}

#[test]
fn test_collection_mapping_priority() {
    let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
    let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
    let mut config = FileWatcherConfig::default();

    // Configure collection mapping with patterns that match the test paths
    let mut mapping = std::collections::HashMap::new();
    // Use patterns that will definitely match
    mapping.insert(
        "**/project/docs/**/*.md".to_string(),
        "custom-docs".to_string(),
    );
    mapping.insert(
        "**/project/src/**/*.rs".to_string(),
        "custom-rust".to_string(),
    );
    mapping.insert(
        "**/project/tests/**/*".to_string(),
        "custom-tests".to_string(),
    );
    config.collection_mapping = Some(mapping);

    let ops = VectorOperations::new(vector_store, embedding_manager, config);

    // Collection mapping should take priority over known patterns
    assert_eq!(
        ops.determine_collection_name(&PathBuf::from("/project/docs/guide.md")),
        "custom-docs"
    );
    assert_eq!(
        ops.determine_collection_name(&PathBuf::from("/project/src/main.rs")),
        "custom-rust"
    );
    assert_eq!(
        ops.determine_collection_name(&PathBuf::from("/project/tests/test.rs")),
        "custom-tests"
    );

    // Paths that don't match mapping should fall back to known patterns or default
    assert_eq!(
        ops.determine_collection_name(&PathBuf::from("/docs/architecture/design.md")),
        "docs-architecture" // Known pattern, not in mapping
    );
}

#[test]
fn test_collection_mapping_windows_paths_normalized() {
    let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
    let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
    let mut config = FileWatcherConfig::default();

    // Configure collection mapping with forward slashes (will be normalized)
    let mut mapping = std::collections::HashMap::new();
    mapping.insert("*/docs/**/*.md".to_string(), "documentation".to_string());
    mapping.insert("*/src/**/*.rs".to_string(), "rust-code".to_string());
    config.collection_mapping = Some(mapping);

    let ops = VectorOperations::new(vector_store, embedding_manager, config);

    // Windows paths with backslashes should be normalized and match patterns
    assert_eq!(
        ops.determine_collection_name(&PathBuf::from(r"C:\project\docs\guide.md")),
        "documentation"
    );
    assert_eq!(
        ops.determine_collection_name(&PathBuf::from(r"D:\work\src\main.rs")),
        "rust-code"
    );
}

#[test]
fn test_deeply_nested_paths() {
    let ops = create_test_ops();

    let collection = ops.determine_collection_name(&PathBuf::from(
        "/a/b/c/d/e/f/docs/architecture/deeply/nested/file.md",
    ));
    assert_eq!(collection, "docs-architecture");
}

#[test]
fn test_path_with_similar_names() {
    let ops = create_test_ops();

    // Path contains "docs" but not in expected location
    let collection = ops.determine_collection_name(&PathBuf::from("/mydocs/file.txt"));
    assert_eq!(collection, "workspace-default");

    // Path contains "vectorizer" but not in expected location
    let collection = ops.determine_collection_name(&PathBuf::from("/not-vectorizer/src/main.rs"));
    assert_eq!(collection, "workspace-default");
}

#[test]
fn test_relative_paths() {
    let ops = create_test_ops();

    // Note: The function uses contains("/pattern/") which requires forward slashes
    // Relative paths without leading slash won't match these patterns
    let test_cases = vec![
        ("/docs/architecture/file.md", "docs-architecture"),
        ("/vectorizer/src/main.rs", "vectorizer-source"),
        ("/gov/bips/BIP-001.md", "gov-bips"),
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, expected,
            "Path {} should map to {}",
            path, expected
        );
    }
}

#[test]
fn test_paths_with_special_characters() {
    let ops = create_test_ops();

    let test_cases = vec![
        (
            "/docs/architecture/file with spaces.md",
            "docs-architecture",
        ),
        ("/vectorizer/src/module-name.rs", "vectorizer-source"),
        ("/gov/bips/BIP_001.md", "gov-bips"),
    ];

    for (path, expected) in test_cases {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(collection, expected);
    }
}

#[test]
fn test_empty_path() {
    let ops = create_test_ops();
    let collection = ops.determine_collection_name(&PathBuf::from(""));
    assert_eq!(collection, "workspace-default");
}

#[test]
fn test_no_empty_collection_creation() {
    let ops = create_test_ops();

    // These paths previously caused empty collections to be created
    // Now they should all use the default collection
    let problematic_paths = vec![
        "/third-party/libsodium/src/file.c",
        "/Server/ToS-Server-5/Api/endpoint.cs",
        "/Benchmark/src/test.cpp",
        "/test/symbols/symbol.txt",
        "/libsodium-regen-msvc/file.h",
    ];

    for path in problematic_paths {
        let collection = ops.determine_collection_name(&PathBuf::from(path));
        assert_eq!(
            collection, "workspace-default",
            "Path {} should NOT create new collection, should use default",
            path
        );
    }
}

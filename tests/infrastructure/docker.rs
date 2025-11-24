/// Test to validate that Docker virtual paths work correctly
/// This ensures the fix for path traversal validation doesn't break Docker environments
use vectorizer::file_operations::{FileOperationError, FileOperations};

#[tokio::test]
async fn test_docker_virtual_paths_acceptance() {
    let ops = FileOperations::new();

    // These are typical virtual paths that Docker might use
    let docker_virtual_paths = vec![
        "/workspace/../etc/config.yml",              // Absolute with traversal
        "../app/src/main.rs",                        // Relative with traversal
        "/mnt/f/Node/hivellm/vectorizer/src/lib.rs", // WSL-style absolute
        "./../../shared/utils.rs",                   // Multiple traversals
        "/virtual/workspace/./src/../lib/mod.rs",    // Mixed
    ];

    for path in docker_virtual_paths {
        // All these paths should pass validation now
        // (they will fail to find content because there's no VectorStore,
        // but the validation should not reject them)
        let result = ops.get_file_content("test-collection", path, 100).await;

        // Should fail with VectorStoreError (not initialized) or FileNotFound,
        // NOT with InvalidPath
        match result {
            Err(FileOperationError::InvalidPath { .. }) => {
                panic!("Path '{path}' should be valid but was rejected as invalid");
            }
            Err(FileOperationError::VectorStoreError(_))
            | Err(FileOperationError::CollectionNotFound { .. })
            | Err(FileOperationError::FileNotFound { .. }) => {
                // Expected - VectorStore is not initialized in this test
                println!("✓ Path '{path}' passed validation (failed later as expected)");
            }
            Err(e) => {
                println!("✓ Path '{path}' passed validation (error: {e:?})");
            }
            Ok(_) => {
                println!("✓ Path '{path}' passed validation and returned content");
            }
        }
    }
}

#[tokio::test]
async fn test_only_empty_paths_rejected() {
    let ops = FileOperations::new();

    // Empty paths should still be rejected
    let empty_paths = vec!["", "   ", "\t", "\n"];

    for path in empty_paths {
        let result = ops.get_file_content("test-collection", path, 100).await;

        match result {
            Err(FileOperationError::InvalidPath { reason, .. }) => {
                assert!(
                    reason.contains("empty") || reason.contains("cannot be empty"),
                    "Expected 'empty' error for path '{}'",
                    path.replace('\n', "\\n").replace('\t', "\\t")
                );
                println!(
                    "✓ Empty path correctly rejected: '{}'",
                    path.replace('\n', "\\n").replace('\t', "\\t")
                );
            }
            _ => {
                panic!(
                    "Empty path '{}' should be rejected",
                    path.replace('\n', "\\n").replace('\t', "\\t")
                );
            }
        }
    }
}

#[tokio::test]
async fn test_normal_paths_still_work() {
    let ops = FileOperations::new();

    // Normal paths should work as before
    let normal_paths = vec![
        "src/main.rs",
        "docs/README.md",
        "lib/utils/helpers.rs",
        "config.yml",
        "path/to/deep/nested/file.txt",
    ];

    for path in normal_paths {
        let result = ops.get_file_content("test-collection", path, 100).await;

        // Should not fail with InvalidPath
        match result {
            Err(FileOperationError::InvalidPath { .. }) => {
                panic!("Normal path '{path}' should be valid");
            }
            _ => {
                println!("✓ Normal path '{path}' is valid");
            }
        }
    }
}

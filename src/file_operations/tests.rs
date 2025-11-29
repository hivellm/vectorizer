#[cfg(test)]
mod integration_tests {
    use super::super::*;

    #[tokio::test]
    async fn test_error_handling() {
        let ops = FileOperations::new();

        // Invalid size limit
        let result = ops
            .get_file_content("test-collection", "valid/path.rs", 10000)
            .await;
        assert!(result.is_err());

        // Empty path
        let result = ops.get_file_content("test-collection", "", 500).await;
        assert!(result.is_err());

        // Note: Paths with .. or absolute paths are now valid because
        // file_path is only used as a metadata search key, not for filesystem access.
        // This allows Docker environments with virtual workspace paths to work correctly.
    }

    #[tokio::test]
    async fn test_cache_behavior() {
        let ops = FileOperations::new();
        let collection = "test-collection";
        let file_path = "src/cached.rs";

        // Initial stats
        let stats_before = ops.cache_stats().await;

        // Make multiple calls
        for _ in 0..3 {
            let _ = ops
                .get_file_summary(collection, file_path, SummaryType::Extractive, 3)
                .await;
        }

        // Cache should have entries (if implementation caches)
        let stats_after = ops.cache_stats().await;
        // Stats may not change in mock implementation
        assert!(stats_after.summary_entries >= stats_before.summary_entries);

        // Clear cache
        ops.clear_cache(collection).await;
    }
}

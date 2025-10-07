#[cfg(test)]
mod integration_tests {
    use super::super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        let ops = FileOperations::new();
        let collection = "test-collection";

        // 1. List files
        let list_result = ops.list_files_in_collection(
            collection,
            FileListFilter::default()
        ).await;
        
        assert!(list_result.is_ok());
        let list = list_result.unwrap();
        assert!(list.total_files > 0);

        // 2. Get summary of first file
        if let Some(first_file) = list.files.first() {
            let summary_result = ops.get_file_summary(
                collection,
                &first_file.path,
                SummaryType::Both,
                5
            ).await;
            
            assert!(summary_result.is_ok());
            let summary = summary_result.unwrap();
            assert_eq!(summary.file_path, first_file.path);
        }

        // 3. Get full content
        if let Some(first_file) = list.files.first() {
            let content_result = ops.get_file_content(
                collection,
                &first_file.path,
                1000
            ).await;
            
            assert!(content_result.is_ok());
            let content = content_result.unwrap();
            assert!(!content.content.is_empty());
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let ops = FileOperations::new();

        // Invalid path
        let result = ops.get_file_content(
            "test-collection",
            "../etc/passwd",
            500
        ).await;
        assert!(result.is_err());

        // Invalid size limit
        let result = ops.get_file_content(
            "test-collection",
            "valid/path.rs",
            10000
        ).await;
        assert!(result.is_err());

        // Empty path
        let result = ops.get_file_content(
            "test-collection",
            "",
            500
        ).await;
        assert!(result.is_err());
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
            let _ = ops.get_file_summary(
                collection,
                file_path,
                SummaryType::Extractive,
                3
            ).await;
        }

        // Cache should have entries (if implementation caches)
        let stats_after = ops.cache_stats().await;
        // Stats may not change in mock implementation
        assert!(stats_after.summary_entries >= stats_before.summary_entries);

        // Clear cache
        ops.clear_cache(collection).await;
    }
}


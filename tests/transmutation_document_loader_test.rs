//! Integration tests for transmutation in DocumentLoader

#[cfg(feature = "transmutation")]
#[cfg(test)]
mod document_loader_tests {
    use vectorizer::{VectorStore, document_loader::{DocumentLoader, LoaderConfig}};
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_loader_config_with_transmutation_formats() {
        let config = LoaderConfig {
            collection_name: "test_collection".to_string(),
            embedding_type: "bm25".to_string(),
            include_patterns: vec![
                "*.pdf".to_string(),
                "*.docx".to_string(),
                "*.html".to_string(),
            ],
            ..Default::default()
        };

        assert_eq!(config.collection_name, "test_collection");
        assert_eq!(config.embedding_type, "bm25");
        assert!(config.include_patterns.contains(&"*.pdf".to_string()));
        assert!(config.include_patterns.contains(&"*.docx".to_string()));
        assert!(config.include_patterns.contains(&"*.html".to_string()));
    }

    #[tokio::test]
    async fn test_document_loader_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let store = VectorStore::new();

        let config = LoaderConfig {
            collection_name: "empty_test".to_string(),
            embedding_type: "bm25".to_string(),
            ..Default::default()
        };

        let mut loader = DocumentLoader::new(config);
        let result = loader.load_project_async(temp_dir.path().to_str().unwrap(), &store).await;

        // Should succeed but with 0 documents
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_document_loader_with_text_files() {
        let temp_dir = TempDir::new().unwrap();
        let store = VectorStore::new();

        // Create some text files
        fs::write(temp_dir.path().join("test1.txt"), "This is test content 1").unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "This is test content 2").unwrap();

        let config = LoaderConfig {
            collection_name: "text_test".to_string(),
            embedding_type: "bm25".to_string(),
            include_patterns: vec!["*.txt".to_string()],
            ..Default::default()
        };

        let mut loader = DocumentLoader::new(config);
        let result = loader.load_project_async(temp_dir.path().to_str().unwrap(), &store).await;

        assert!(result.is_ok());
        let count = result.unwrap();
        assert!(count > 0, "Should have indexed text files");
    }

    #[test]
    fn test_document_loader_format_detection() {
        use vectorizer::transmutation_integration::TransmutationProcessor;

        // Test that DocumentLoader would recognize these formats
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("document.pdf")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("presentation.pptx")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("spreadsheet.xlsx")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("page.html")));
    }

    #[tokio::test]
    async fn test_document_loader_with_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let store = VectorStore::new();

        // Create nested directory structure
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        fs::write(temp_dir.path().join("root.txt"), "Root content").unwrap();
        fs::write(sub_dir.join("nested.txt"), "Nested content").unwrap();

        let config = LoaderConfig {
            collection_name: "nested_test".to_string(),
            embedding_type: "bm25".to_string(),
            include_patterns: vec!["**/*.txt".to_string()],
            ..Default::default()
        };

        let mut loader = DocumentLoader::new(config);
        let result = loader.load_project_async(temp_dir.path().to_str().unwrap(), &store).await;

        assert!(result.is_ok());
        let count = result.unwrap();
        assert!(count > 0, "Should have indexed files in subdirectories");
    }

    #[test]
    fn test_document_loader_exclude_patterns() {
        let config = LoaderConfig {
            collection_name: "exclude_test".to_string(),
            embedding_type: "bm25".to_string(),
            include_patterns: vec!["*.txt".to_string()],
            exclude_patterns: vec![
                "**/data/**".to_string(),
                "**/*.bin".to_string(),
                "**/target/**".to_string(),
            ],
            ..Default::default()
        };

        // Verify exclude patterns are set
        assert!(config.exclude_patterns.contains(&"**/data/**".to_string()));
        assert!(config.exclude_patterns.contains(&"**/*.bin".to_string()));
    }

    #[test]
    fn test_document_loader_max_file_size() {
        let config = LoaderConfig {
            collection_name: "size_test".to_string(),
            embedding_type: "bm25".to_string(),
            max_file_size: 1024 * 1024, // 1MB
            ..Default::default()
        };

        assert_eq!(config.max_file_size, 1024 * 1024);
    }
}

#[cfg(not(feature = "transmutation"))]
#[cfg(test)]
mod without_transmutation_tests {
    use vectorizer::document_loader::{DocumentLoader, LoaderConfig};
    use vectorizer::VectorStore;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_document_loader_without_transmutation() {
        let temp_dir = TempDir::new().unwrap();
        let store = VectorStore::new();

        // Create text files that should work without transmutation
        fs::write(temp_dir.path().join("test.txt"), "Test content").unwrap();
        fs::write(temp_dir.path().join("test.md"), "# Markdown content").unwrap();

        let config = LoaderConfig {
            collection_name: "no_trans_test".to_string(),
            embedding_type: "bm25".to_string(),
            include_patterns: vec!["*.txt".to_string(), "*.md".to_string()],
            ..Default::default()
        };

        let mut loader = DocumentLoader::new(config);
        let result = loader.load_project_async(temp_dir.path().to_str().unwrap(), &store).await;

        // Should still work for text formats
        assert!(result.is_ok());
    }
}


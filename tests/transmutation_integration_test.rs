//! Integration tests for transmutation support

#[cfg(feature = "transmutation")]
#[cfg(test)]
mod transmutation_tests {
    use std::path::PathBuf;
    use vectorizer::transmutation_integration::{TransmutationProcessor, types::*};

    #[test]
    fn test_format_detection() {
        // PDF
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.pdf")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("Test.PDF")));
        
        // Office formats
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.docx")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.xlsx")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.pptx")));
        
        // Web formats
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.html")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.xml")));
        
        // Image formats
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.jpg")));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.png")));
        
        // Unsupported formats
        assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.txt")));
        assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_converted_document_creation() {
        let doc = ConvertedDocument::new("Test content".to_string());
        assert_eq!(doc.content, "Test content");
        assert!(doc.page_info.is_none());
        assert_eq!(doc.total_pages(), None);
    }

    #[test]
    fn test_converted_document_with_pages() {
        let pages = vec![
            PageInfo {
                page_number: 1,
                start_char: 0,
                end_char: 100,
            },
            PageInfo {
                page_number: 2,
                start_char: 100,
                end_char: 200,
            },
        ];
        
        let doc = ConvertedDocument::with_pages("Test content".to_string(), pages);
        assert_eq!(doc.total_pages(), Some(2));
        assert_eq!(doc.get_page_at_position(50), Some(1));
        assert_eq!(doc.get_page_at_position(150), Some(2));
        assert_eq!(doc.get_page_at_position(250), None);
    }

    #[test]
    fn test_converted_document_metadata() {
        let doc = ConvertedDocument::new("Test".to_string())
            .with_metadata("source_format".to_string(), "pdf".to_string())
            .with_metadata("page_count".to_string(), "5".to_string());
        
        assert_eq!(doc.metadata.get("source_format"), Some(&"pdf".to_string()));
        assert_eq!(doc.metadata.get("page_count"), Some(&"5".to_string()));
    }

    #[tokio::test]
    async fn test_processor_initialization() {
        let result = TransmutationProcessor::new();
        assert!(result.is_ok(), "Processor should initialize successfully");
    }
}

#[cfg(not(feature = "transmutation"))]
#[cfg(test)]
mod without_feature_tests {
    use std::path::PathBuf;
    
    #[test]
    fn test_transmutation_disabled() {
        // When transmutation feature is disabled, module should not be available
        // This test just verifies compilation without the feature
        assert!(true);
    }
}


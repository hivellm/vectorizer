//! Tests for transmutation integration

use super::*;
use std::path::PathBuf;

#[test]
fn test_is_supported_format() {
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
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.tiff")));
    
    // Unsupported formats
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.txt")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.rs")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.mp3")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test")));
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
    
    let doc = ConvertedDocument::with_pages("Test content with pages".to_string(), pages);
    assert_eq!(doc.total_pages(), Some(2));
    assert_eq!(doc.get_page_at_position(50), Some(1));
    assert_eq!(doc.get_page_at_position(150), Some(2));
    assert_eq!(doc.get_page_at_position(250), None);
}

#[test]
fn test_converted_document_with_metadata() {
    let doc = ConvertedDocument::new("Test".to_string())
        .with_metadata("key1".to_string(), "value1".to_string())
        .with_metadata("key2".to_string(), "value2".to_string());
    
    assert_eq!(doc.metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(doc.metadata.get("key2"), Some(&"value2".to_string()));
}

#[cfg(feature = "transmutation")]
#[tokio::test]
async fn test_processor_creation() {
    let processor = TransmutationProcessor::new();
    assert!(processor.is_ok());
}

#[cfg(not(feature = "transmutation"))]
#[tokio::test]
async fn test_processor_without_feature() {
    let processor = TransmutationProcessor::new();
    assert!(processor.is_ok());
    
    // Conversion should fail when feature is disabled
    let result = processor.unwrap().convert_to_markdown(&PathBuf::from("test.pdf")).await;
    assert!(result.is_err());
}


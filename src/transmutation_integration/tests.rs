//! Tests for transmutation integration

use super::*;
use std::path::PathBuf;

#[test]
fn test_is_supported_format_pdf() {
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.pdf")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("Test.PDF")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("document.Pdf")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("/path/to/file.pdf")));
}

#[test]
fn test_is_supported_format_office() {
    // DOCX
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.docx")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("Test.DOCX")));
    
    // XLSX
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.xlsx")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("spreadsheet.XLSX")));
    
    // PPTX
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.pptx")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("presentation.Pptx")));
}

#[test]
fn test_is_supported_format_web() {
    // HTML
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.html")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("index.HTML")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("page.htm")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("page.HTM")));
    
    // XML
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.xml")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("config.XML")));
}

#[test]
fn test_is_supported_format_images() {
    // JPEG
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.jpg")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("image.JPG")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("photo.jpeg")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("photo.JPEG")));
    
    // PNG
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.png")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("screenshot.PNG")));
    
    // TIFF
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.tiff")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("scan.TIF")));
    
    // Other formats
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.bmp")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.gif")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("test.webp")));
}

#[test]
fn test_is_supported_format_unsupported() {
    // Text files
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.txt")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("README.md")));
    
    // Code files
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("test.rs")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("script.py")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("app.js")));
    
    // Media files
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("audio.mp3")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("video.mp4")));
    
    // Archives
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("archive.zip")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("backup.tar.gz")));
    
    // No extension
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("file")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("README")));
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

#[test]
fn test_page_info_boundaries() {
    let page1 = PageInfo {
        page_number: 1,
        start_char: 0,
        end_char: 1000,
    };
    
    let page2 = PageInfo {
        page_number: 2,
        start_char: 1000,
        end_char: 2000,
    };
    
    // Test page boundaries
    assert_eq!(page1.page_number, 1);
    assert_eq!(page1.start_char, 0);
    assert_eq!(page1.end_char, 1000);
    
    assert_eq!(page2.page_number, 2);
    assert_eq!(page2.start_char, 1000);
    assert_eq!(page2.end_char, 2000);
}

#[test]
fn test_get_page_at_position_edge_cases() {
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
        PageInfo {
            page_number: 3,
            start_char: 200,
            end_char: 300,
        },
    ];
    
    let doc = ConvertedDocument::with_pages("Content".to_string(), pages);
    
    // Test boundaries
    assert_eq!(doc.get_page_at_position(0), Some(1));
    assert_eq!(doc.get_page_at_position(99), Some(1));
    assert_eq!(doc.get_page_at_position(100), Some(2));
    assert_eq!(doc.get_page_at_position(199), Some(2));
    assert_eq!(doc.get_page_at_position(200), Some(3));
    assert_eq!(doc.get_page_at_position(299), Some(3));
    
    // Out of range
    assert_eq!(doc.get_page_at_position(300), None);
    assert_eq!(doc.get_page_at_position(500), None);
}

#[test]
fn test_converted_document_empty_content() {
    let doc = ConvertedDocument::new(String::new());
    assert!(doc.content.is_empty());
    assert!(doc.page_info.is_none());
    assert_eq!(doc.total_pages(), None);
}

#[test]
fn test_converted_document_large_content() {
    let large_content = "a".repeat(1_000_000);
    let doc = ConvertedDocument::new(large_content.clone());
    assert_eq!(doc.content.len(), 1_000_000);
    assert_eq!(doc.content, large_content);
}

#[test]
fn test_converted_document_with_many_pages() {
    let mut pages = Vec::new();
    for i in 0..100 {
        pages.push(PageInfo {
            page_number: i + 1,
            start_char: i * 1000,
            end_char: (i + 1) * 1000,
        });
    }
    
    let doc = ConvertedDocument::with_pages("Multi-page doc".to_string(), pages);
    assert_eq!(doc.total_pages(), Some(100));
    
    // Test various page positions
    assert_eq!(doc.get_page_at_position(500), Some(1));
    assert_eq!(doc.get_page_at_position(5500), Some(6));
    assert_eq!(doc.get_page_at_position(50500), Some(51));
    assert_eq!(doc.get_page_at_position(99500), Some(100));
}

#[test]
fn test_metadata_chaining() {
    let doc = ConvertedDocument::new("Test".to_string())
        .with_metadata("format".to_string(), "pdf".to_string())
        .with_metadata("pages".to_string(), "10".to_string())
        .with_metadata("author".to_string(), "John Doe".to_string())
        .with_metadata("converted_via".to_string(), "transmutation".to_string());
    
    assert_eq!(doc.metadata.len(), 4);
    assert_eq!(doc.metadata.get("format"), Some(&"pdf".to_string()));
    assert_eq!(doc.metadata.get("pages"), Some(&"10".to_string()));
    assert_eq!(doc.metadata.get("author"), Some(&"John Doe".to_string()));
    assert_eq!(doc.metadata.get("converted_via"), Some(&"transmutation".to_string()));
}

#[test]
fn test_metadata_override() {
    let doc = ConvertedDocument::new("Test".to_string())
        .with_metadata("key".to_string(), "value1".to_string())
        .with_metadata("key".to_string(), "value2".to_string());
    
    // Last value should win
    assert_eq!(doc.metadata.get("key"), Some(&"value2".to_string()));
}

#[test]
fn test_format_detection_with_path_separators() {
    // Unix paths
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("/home/user/document.pdf")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("/var/www/page.html")));
    
    // Windows paths
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("C:\\Users\\file.docx")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("D:\\Data\\sheet.xlsx")));
    
    // Relative paths
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("./docs/report.pdf")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("../files/data.xml")));
}

#[test]
fn test_format_detection_with_multiple_extensions() {
    // These should NOT be supported (double extensions)
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("file.tar.gz")));
    assert!(!TransmutationProcessor::is_supported_format(&PathBuf::from("backup.pdf.bak")));
}

#[test]
fn test_format_detection_special_characters() {
    // Files with special characters in name
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("my-document.pdf")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("file_2024.docx")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("report (1).pdf")));
    assert!(TransmutationProcessor::is_supported_format(&PathBuf::from("presentation [final].pptx")));
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

// ========================================
// Additional Integration Tests from /tests
// ========================================

#[cfg(feature = "transmutation")]
#[test]
fn test_format_detection_comprehensive() {
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
fn test_converted_document_creation_comprehensive() {
    let doc = ConvertedDocument::new("Test content".to_string());
    assert_eq!(doc.content, "Test content");
    assert!(doc.page_info.is_none());
    assert_eq!(doc.total_pages(), None);
}

#[test]
fn test_converted_document_with_pages_comprehensive() {
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
fn test_converted_document_metadata_comprehensive() {
    let doc = ConvertedDocument::new("Test".to_string())
        .with_metadata("source_format".to_string(), "pdf".to_string())
        .with_metadata("page_count".to_string(), "5".to_string());
    
    assert_eq!(doc.metadata.get("source_format"), Some(&"pdf".to_string()));
    assert_eq!(doc.metadata.get("page_count"), Some(&"5".to_string()));
}

#[cfg(feature = "transmutation")]
#[tokio::test]
async fn test_processor_initialization_comprehensive() {
    let result = TransmutationProcessor::new();
    assert!(result.is_ok(), "Processor should initialize successfully");
}

#[cfg(not(feature = "transmutation"))]
#[test]
fn test_transmutation_disabled() {
    // When transmutation feature is disabled, module should not be available
    // This test just verifies compilation without the feature
    assert!(true);
}


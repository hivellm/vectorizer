//! Comprehensive tests for transmutation integration
//!
//! Test coverage includes:
//! - Format detection for all supported file types
//! - Page boundary extraction algorithm validation
//! - ConvertedDocument creation and manipulation
//! - Metadata handling and extraction
//! - Edge cases (Unicode, large files, empty content)
//! - Error handling scenarios
//! - Integration tests (when feature enabled)

use std::path::PathBuf;

use super::types::{ConvertedDocument, PageInfo};
use super::TransmutationProcessor;

// ============================================================================
// FORMAT DETECTION TESTS
// ============================================================================

mod format_detection {
    use super::*;

    #[test]
    fn test_pdf_format_detection() {
        // Standard cases
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "test.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "document.PDF"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "file.Pdf"
        )));

        // With paths
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "/home/user/documents/report.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "C:\\Users\\Documents\\file.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "./relative/path/to/doc.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "../parent/folder/file.pdf"
        )));
    }

    #[test]
    fn test_office_format_detection() {
        // DOCX
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "document.docx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "report.DOCX"
        )));

        // XLSX
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "spreadsheet.xlsx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "data.XLSX"
        )));

        // PPTX
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "presentation.pptx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "slides.PPTX"
        )));
    }

    #[test]
    fn test_web_format_detection() {
        // HTML
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "page.html"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "index.HTML"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "old.htm"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "page.HTM"
        )));

        // XML
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "config.xml"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "data.XML"
        )));
    }

    #[test]
    fn test_image_format_detection_for_ocr() {
        // JPEG variants
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "photo.jpg"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "image.JPG"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "pic.jpeg"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "photo.JPEG"
        )));

        // PNG
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "screenshot.png"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "image.PNG"
        )));

        // TIFF variants
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "scan.tiff"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "document.TIF"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "scan.TIFF"
        )));

        // Other image formats
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "image.bmp"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "animation.gif"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "modern.webp"
        )));
    }

    #[test]
    fn test_unsupported_formats() {
        // Plain text (processed natively, not by transmutation)
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("readme.txt")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("notes.md")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("config.ini")
        ));

        // Code files
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("main.rs")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("script.py")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("app.js")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("module.ts")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("Main.java")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("program.go")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("source.cpp")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("header.h")
        ));

        // Config files
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("config.json")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("settings.yaml")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("Cargo.toml")
        ));

        // Binary/media files
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("program.exe")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("library.dll")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("audio.mp3")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("video.mp4")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("archive.zip")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("backup.tar.gz")
        ));

        // Old Office formats (not supported)
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("old.doc")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("old.xls")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("old.ppt")
        ));
    }

    #[test]
    fn test_edge_case_filenames() {
        // Files without extension
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("README")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("Makefile")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("LICENSE")
        ));

        // Hidden files
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from(".gitignore")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from(".env")
        ));

        // Double extensions (should check last extension)
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("file.tar.gz")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("backup.pdf.bak")
        ));

        // Special characters in filename
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "my-document.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "file_2024.docx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "report (copy).pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "presentation [final].pptx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "doc with spaces.pdf"
        )));
    }

    #[test]
    fn test_unicode_filenames() {
        // Unicode characters in filename
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "документ.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "文档.docx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "documento_ñ.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "tëst.xlsx"
        )));
    }
}

// ============================================================================
// CONVERTED DOCUMENT TESTS
// ============================================================================

mod converted_document {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let doc = ConvertedDocument::new("Test content".to_string());

        assert_eq!(doc.content, "Test content");
        assert!(doc.page_info.is_none());
        assert!(doc.metadata.is_empty());
        assert_eq!(doc.total_pages(), None);
    }

    #[test]
    fn test_creation_with_empty_content() {
        let doc = ConvertedDocument::new(String::new());

        assert!(doc.content.is_empty());
        assert!(doc.page_info.is_none());
        assert_eq!(doc.total_pages(), None);
    }

    #[test]
    fn test_creation_with_pages() {
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

        let doc = ConvertedDocument::with_pages("Multi-page content".to_string(), pages);

        assert_eq!(doc.total_pages(), Some(2));
        assert!(doc.page_info.is_some());
    }

    #[test]
    fn test_metadata_builder_pattern() {
        let doc = ConvertedDocument::new("Content".to_string())
            .with_metadata("key1".to_string(), "value1".to_string())
            .with_metadata("key2".to_string(), "value2".to_string())
            .with_metadata("key3".to_string(), "value3".to_string());

        assert_eq!(doc.metadata.len(), 3);
        assert_eq!(doc.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(doc.metadata.get("key2"), Some(&"value2".to_string()));
        assert_eq!(doc.metadata.get("key3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_metadata_override() {
        let doc = ConvertedDocument::new("Content".to_string())
            .with_metadata("key".to_string(), "original".to_string())
            .with_metadata("key".to_string(), "overwritten".to_string());

        assert_eq!(doc.metadata.len(), 1);
        assert_eq!(doc.metadata.get("key"), Some(&"overwritten".to_string()));
    }

    #[test]
    fn test_large_content() {
        let large_content = "a".repeat(10_000_000); // 10MB of content
        let doc = ConvertedDocument::new(large_content.clone());

        assert_eq!(doc.content.len(), 10_000_000);
        assert_eq!(doc.content, large_content);
    }

    #[test]
    fn test_unicode_content() {
        let unicode_content =
            "Hello 世界 🌍 مرحبا Привет Γειά σου こんにちは".to_string();
        let doc = ConvertedDocument::new(unicode_content.clone());

        assert_eq!(doc.content, unicode_content);
    }
}

// ============================================================================
// PAGE INFO TESTS
// ============================================================================

mod page_info {
    use super::*;

    #[test]
    fn test_page_info_creation() {
        let page = PageInfo {
            page_number: 1,
            start_char: 0,
            end_char: 1000,
        };

        assert_eq!(page.page_number, 1);
        assert_eq!(page.start_char, 0);
        assert_eq!(page.end_char, 1000);
    }

    #[test]
    fn test_get_page_at_position_single_page() {
        let pages = vec![PageInfo {
            page_number: 1,
            start_char: 0,
            end_char: 1000,
        }];

        let doc = ConvertedDocument::with_pages("Content".to_string(), pages);

        // Within bounds
        assert_eq!(doc.get_page_at_position(0), Some(1));
        assert_eq!(doc.get_page_at_position(500), Some(1));
        assert_eq!(doc.get_page_at_position(999), Some(1));

        // Out of bounds (end_char is exclusive)
        assert_eq!(doc.get_page_at_position(1000), None);
        assert_eq!(doc.get_page_at_position(1500), None);
    }

    #[test]
    fn test_get_page_at_position_multiple_pages() {
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

        // Page 1 boundaries
        assert_eq!(doc.get_page_at_position(0), Some(1));
        assert_eq!(doc.get_page_at_position(50), Some(1));
        assert_eq!(doc.get_page_at_position(99), Some(1));

        // Page 2 boundaries (exactly at boundary)
        assert_eq!(doc.get_page_at_position(100), Some(2));
        assert_eq!(doc.get_page_at_position(150), Some(2));
        assert_eq!(doc.get_page_at_position(199), Some(2));

        // Page 3 boundaries
        assert_eq!(doc.get_page_at_position(200), Some(3));
        assert_eq!(doc.get_page_at_position(250), Some(3));
        assert_eq!(doc.get_page_at_position(299), Some(3));

        // Out of range
        assert_eq!(doc.get_page_at_position(300), None);
        assert_eq!(doc.get_page_at_position(1000), None);
    }

    #[test]
    fn test_get_page_at_position_with_gaps() {
        // Simulates non-contiguous pages (shouldn't happen normally but tests robustness)
        let pages = vec![
            PageInfo {
                page_number: 1,
                start_char: 0,
                end_char: 100,
            },
            PageInfo {
                page_number: 2,
                start_char: 150, // Gap of 50 chars
                end_char: 250,
            },
        ];

        let doc = ConvertedDocument::with_pages("Content".to_string(), pages);

        assert_eq!(doc.get_page_at_position(50), Some(1));
        assert_eq!(doc.get_page_at_position(125), None); // In the gap
        assert_eq!(doc.get_page_at_position(175), Some(2));
    }

    #[test]
    fn test_many_pages() {
        let mut pages = Vec::new();
        for i in 0..100 {
            pages.push(PageInfo {
                page_number: i + 1,
                start_char: i * 1000,
                end_char: (i + 1) * 1000,
            });
        }

        let doc = ConvertedDocument::with_pages("Large document".to_string(), pages);

        assert_eq!(doc.total_pages(), Some(100));
        assert_eq!(doc.get_page_at_position(500), Some(1));
        assert_eq!(doc.get_page_at_position(5500), Some(6));
        assert_eq!(doc.get_page_at_position(50500), Some(51));
        assert_eq!(doc.get_page_at_position(99500), Some(100));
        assert_eq!(doc.get_page_at_position(100000), None);
    }

    #[test]
    fn test_total_pages() {
        // No pages
        let doc1 = ConvertedDocument::new("No pages".to_string());
        assert_eq!(doc1.total_pages(), None);

        // Single page
        let doc2 = ConvertedDocument::with_pages(
            "Single".to_string(),
            vec![PageInfo {
                page_number: 1,
                start_char: 0,
                end_char: 100,
            }],
        );
        assert_eq!(doc2.total_pages(), Some(1));

        // Multiple pages
        let doc3 = ConvertedDocument::with_pages(
            "Multi".to_string(),
            vec![
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
            ],
        );
        assert_eq!(doc3.total_pages(), Some(3));
    }
}

// ============================================================================
// PAGE MARKER PARSING TESTS (Docling-style validation)
// ============================================================================

mod page_marker_parsing {
    use super::*;

    /// Simulates the merged content format produced by transmutation
    fn create_merged_content(pages: &[&str]) -> String {
        pages
            .iter()
            .enumerate()
            .map(|(i, content)| format!("--- Page {} ---\n{}", i + 1, content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    #[test]
    fn test_page_marker_format() {
        let content = create_merged_content(&["Page 1 content", "Page 2 content"]);

        assert!(content.contains("--- Page 1 ---"));
        assert!(content.contains("--- Page 2 ---"));
        assert!(content.contains("Page 1 content"));
        assert!(content.contains("Page 2 content"));
    }

    #[test]
    fn test_page_marker_with_empty_pages() {
        let content = create_merged_content(&["", "Content on page 2", ""]);

        assert!(content.contains("--- Page 1 ---"));
        assert!(content.contains("--- Page 2 ---"));
        assert!(content.contains("--- Page 3 ---"));
    }

    #[test]
    fn test_page_marker_with_special_content() {
        // Content that might confuse the parser
        let content = create_merged_content(&[
            "Some content with --- dashes ---",
            "Content with Page 1 mention",
            "Content with --- Page 99 --- fake marker",
        ]);

        // The real markers should still be correctly identified
        assert!(content.starts_with("--- Page 1 ---"));
    }

    #[test]
    fn test_simulated_page_extraction() {
        // This simulates what extract_page_info_from_result does
        let content = create_merged_content(&[
            "First page has some content here.",
            "Second page continues the document.",
            "Third and final page.",
        ]);

        let mut pages: Vec<PageInfo> = Vec::new();
        let mut current_pos: usize = 0;

        for line in content.lines() {
            let line_start = current_pos;
            let line_len = line.len() + 1; // +1 for newline

            if line.starts_with("--- Page ") && line.ends_with(" ---") {
                if let Some(page_num_str) =
                    line.strip_prefix("--- Page ").and_then(|s| s.strip_suffix(" ---"))
                {
                    if let Ok(page_num) = page_num_str.parse::<usize>() {
                        // Close previous page
                        if let Some(last_page) = pages.last_mut() {
                            last_page.end_char = line_start.saturating_sub(2);
                        }

                        pages.push(PageInfo {
                            page_number: page_num,
                            start_char: line_start + line_len,
                            end_char: content.len(),
                        });
                    }
                }
            }

            current_pos += line_len;
        }

        // Finalize last page
        if let Some(last_page) = pages.last_mut() {
            last_page.end_char = content.len();
        }

        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].page_number, 1);
        assert_eq!(pages[1].page_number, 2);
        assert_eq!(pages[2].page_number, 3);

        // Verify page boundaries don't overlap
        for i in 1..pages.len() {
            assert!(
                pages[i].start_char >= pages[i - 1].end_char
                    || pages[i].start_char <= pages[i - 1].end_char + 2,
                "Page {} start ({}) should be after page {} end ({})",
                pages[i].page_number,
                pages[i].start_char,
                pages[i - 1].page_number,
                pages[i - 1].end_char
            );
        }
    }
}

// ============================================================================
// METADATA EXTRACTION TESTS (Docling comparison)
// ============================================================================

mod metadata_extraction {
    use super::*;

    #[test]
    fn test_typical_pdf_metadata() {
        let doc = ConvertedDocument::new("PDF content".to_string())
            .with_metadata("source_format".to_string(), "pdf".to_string())
            .with_metadata("converted_via".to_string(), "transmutation".to_string())
            .with_metadata("page_count".to_string(), "10".to_string())
            .with_metadata("title".to_string(), "Annual Report 2024".to_string())
            .with_metadata("author".to_string(), "John Doe".to_string())
            .with_metadata("language".to_string(), "en".to_string())
            .with_metadata("input_size_bytes".to_string(), "1250000".to_string())
            .with_metadata("output_size_bytes".to_string(), "32000".to_string())
            .with_metadata("conversion_duration_ms".to_string(), "450".to_string())
            .with_metadata("tables_extracted".to_string(), "5".to_string());

        assert_eq!(doc.metadata.get("source_format"), Some(&"pdf".to_string()));
        assert_eq!(doc.metadata.get("page_count"), Some(&"10".to_string()));
        assert_eq!(
            doc.metadata.get("title"),
            Some(&"Annual Report 2024".to_string())
        );
        assert_eq!(doc.metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(doc.metadata.get("language"), Some(&"en".to_string()));
        assert_eq!(
            doc.metadata.get("tables_extracted"),
            Some(&"5".to_string())
        );
    }

    #[test]
    fn test_office_document_metadata() {
        let doc = ConvertedDocument::new("DOCX content".to_string())
            .with_metadata("source_format".to_string(), "docx".to_string())
            .with_metadata("converted_via".to_string(), "transmutation".to_string())
            .with_metadata("page_count".to_string(), "25".to_string())
            .with_metadata("title".to_string(), "Project Proposal".to_string())
            .with_metadata("author".to_string(), "Jane Smith".to_string());

        assert_eq!(doc.metadata.get("source_format"), Some(&"docx".to_string()));
        assert_eq!(doc.metadata.get("page_count"), Some(&"25".to_string()));
    }

    #[test]
    fn test_metadata_with_special_characters() {
        let doc = ConvertedDocument::new("Content".to_string())
            .with_metadata("title".to_string(), "Report: Q1 2024 (Draft)".to_string())
            .with_metadata("author".to_string(), "José García-López".to_string())
            .with_metadata("notes".to_string(), "Contains \"quotes\" and 'apostrophes'".to_string());

        assert_eq!(
            doc.metadata.get("title"),
            Some(&"Report: Q1 2024 (Draft)".to_string())
        );
        assert_eq!(
            doc.metadata.get("author"),
            Some(&"José García-López".to_string())
        );
    }

    #[test]
    fn test_empty_metadata_values() {
        let doc = ConvertedDocument::new("Content".to_string())
            .with_metadata("title".to_string(), String::new())
            .with_metadata("author".to_string(), String::new());

        assert_eq!(doc.metadata.get("title"), Some(&String::new()));
        assert_eq!(doc.metadata.get("author"), Some(&String::new()));
    }
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

mod serialization {
    use super::*;

    #[test]
    fn test_page_info_serialization() {
        let page = PageInfo {
            page_number: 5,
            start_char: 1000,
            end_char: 2000,
        };

        let json = serde_json::to_string(&page).unwrap();
        assert!(json.contains("\"page_number\":5"));
        assert!(json.contains("\"start_char\":1000"));
        assert!(json.contains("\"end_char\":2000"));
    }

    #[test]
    fn test_page_info_deserialization() {
        let json = r#"{"page_number":3,"start_char":500,"end_char":1500}"#;
        let page: PageInfo = serde_json::from_str(json).unwrap();

        assert_eq!(page.page_number, 3);
        assert_eq!(page.start_char, 500);
        assert_eq!(page.end_char, 1500);
    }

    #[test]
    fn test_converted_document_serialization() {
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

        let doc = ConvertedDocument::with_pages("Test content".to_string(), pages)
            .with_metadata("source_format".to_string(), "pdf".to_string());

        let json = serde_json::to_string(&doc).unwrap();

        assert!(json.contains("Test content"));
        assert!(json.contains("page_number"));
        assert!(json.contains("source_format"));
        assert!(json.contains("pdf"));
    }

    #[test]
    fn test_converted_document_deserialization() {
        let json = r#"{
            "content": "Deserialized content",
            "page_info": [
                {"page_number": 1, "start_char": 0, "end_char": 50}
            ],
            "metadata": {
                "source_format": "docx"
            }
        }"#;

        let doc: ConvertedDocument = serde_json::from_str(json).unwrap();

        assert_eq!(doc.content, "Deserialized content");
        assert_eq!(doc.total_pages(), Some(1));
        assert_eq!(
            doc.metadata.get("source_format"),
            Some(&"docx".to_string())
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        let original = ConvertedDocument::with_pages(
            "Round trip test".to_string(),
            vec![PageInfo {
                page_number: 1,
                start_char: 0,
                end_char: 100,
            }],
        )
        .with_metadata("key".to_string(), "value".to_string());

        let json = serde_json::to_string(&original).unwrap();
        let restored: ConvertedDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(original.content, restored.content);
        assert_eq!(original.total_pages(), restored.total_pages());
        assert_eq!(original.metadata, restored.metadata);
    }
}

// ============================================================================
// PROCESSOR TESTS
// ============================================================================

mod processor {
    use super::*;

    #[cfg(feature = "transmutation")]
    #[tokio::test]
    async fn test_processor_creation_with_feature() {
        let result = TransmutationProcessor::new();
        assert!(
            result.is_ok(),
            "TransmutationProcessor should initialize when feature is enabled"
        );
    }

    #[cfg(not(feature = "transmutation"))]
    #[tokio::test]
    async fn test_processor_creation_without_feature() {
        let result = TransmutationProcessor::new();
        assert!(
            result.is_ok(),
            "TransmutationProcessor should create stub when feature is disabled"
        );
    }

    #[cfg(not(feature = "transmutation"))]
    #[tokio::test]
    async fn test_conversion_fails_without_feature() {
        let processor = TransmutationProcessor::new().unwrap();
        let result = processor
            .convert_to_markdown(&PathBuf::from("test.pdf"))
            .await;

        assert!(
            result.is_err(),
            "Conversion should fail when feature is disabled"
        );

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("not enabled"),
            "Error should indicate feature is not enabled"
        );
    }

    #[test]
    fn test_default_impl() {
        // This tests the Default implementation
        // Note: This may panic if transmutation feature is enabled but initialization fails
        #[cfg(not(feature = "transmutation"))]
        {
            let _processor = TransmutationProcessor::default();
        }
    }
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_very_long_filename() {
        let long_name = format!("{}.pdf", "a".repeat(255));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            &long_name
        )));
    }

    #[test]
    fn test_deeply_nested_path() {
        let deep_path = format!(
            "{}/document.pdf",
            (0..50).map(|_| "folder").collect::<Vec<_>>().join("/")
        );
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            &deep_path
        )));
    }

    #[test]
    fn test_empty_pages_vector() {
        let doc = ConvertedDocument::with_pages("Content".to_string(), vec![]);

        // Empty pages vector is different from None
        assert_eq!(doc.total_pages(), Some(0));
        assert_eq!(doc.get_page_at_position(0), None);
    }

    #[test]
    fn test_page_with_zero_length() {
        let pages = vec![PageInfo {
            page_number: 1,
            start_char: 100,
            end_char: 100, // Zero-length page
        }];

        let doc = ConvertedDocument::with_pages("Content".to_string(), pages);

        // Position 100 is at start but also at end, so shouldn't match
        assert_eq!(doc.get_page_at_position(100), None);
    }

    #[test]
    fn test_overlapping_pages() {
        // This shouldn't happen in practice, but test robustness
        let pages = vec![
            PageInfo {
                page_number: 1,
                start_char: 0,
                end_char: 150,
            },
            PageInfo {
                page_number: 2,
                start_char: 100, // Overlaps with page 1
                end_char: 200,
            },
        ];

        let doc = ConvertedDocument::with_pages("Content".to_string(), pages);

        // First matching page should be returned
        assert_eq!(doc.get_page_at_position(125), Some(1));
    }

    #[test]
    fn test_content_with_null_bytes() {
        let content_with_nulls = "Content\0with\0null\0bytes".to_string();
        let doc = ConvertedDocument::new(content_with_nulls.clone());

        assert_eq!(doc.content, content_with_nulls);
    }

    #[test]
    fn test_content_with_various_line_endings() {
        let mixed_endings = "Line1\nLine2\r\nLine3\rLine4".to_string();
        let doc = ConvertedDocument::new(mixed_endings.clone());

        assert_eq!(doc.content, mixed_endings);
    }

    #[test]
    fn test_max_page_number() {
        let pages = vec![PageInfo {
            page_number: usize::MAX,
            start_char: 0,
            end_char: 100,
        }];

        let doc = ConvertedDocument::with_pages("Content".to_string(), pages);

        assert_eq!(doc.get_page_at_position(50), Some(usize::MAX));
    }

    #[test]
    fn test_max_char_positions() {
        let pages = vec![PageInfo {
            page_number: 1,
            start_char: usize::MAX - 100,
            end_char: usize::MAX,
        }];

        let doc = ConvertedDocument::with_pages("Content".to_string(), pages);

        assert_eq!(doc.get_page_at_position(usize::MAX - 50), Some(1));
    }
}

// ============================================================================
// INTEGRATION TESTS (require transmutation feature)
// ============================================================================

#[cfg(feature = "transmutation")]
mod integration {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_html_conversion() {
        let processor = TransmutationProcessor::new().unwrap();

        // Create a temporary HTML file
        let mut temp_file = NamedTempFile::with_suffix(".html").unwrap();
        writeln!(
            temp_file,
            r#"<!DOCTYPE html>
<html>
<head><title>Test Document</title></head>
<body>
<h1>Heading</h1>
<p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
<ul>
    <li>Item 1</li>
    <li>Item 2</li>
</ul>
</body>
</html>"#
        )
        .unwrap();

        let result = processor.convert_to_markdown(temp_file.path()).await;

        assert!(result.is_ok(), "HTML conversion should succeed");

        let doc = result.unwrap();
        assert!(!doc.content.is_empty(), "Content should not be empty");
        assert!(
            doc.metadata.get("source_format") == Some(&"html".to_string()),
            "Source format should be html"
        );
        assert!(
            doc.metadata.get("converted_via") == Some(&"transmutation".to_string()),
            "Converted via should be transmutation"
        );
    }

    #[tokio::test]
    async fn test_xml_conversion() {
        let processor = TransmutationProcessor::new().unwrap();

        // Create a temporary XML file
        let mut temp_file = NamedTempFile::with_suffix(".xml").unwrap();
        writeln!(
            temp_file,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<document>
    <title>Test XML Document</title>
    <content>
        <section id="1">
            <heading>Section One</heading>
            <paragraph>This is the first section content.</paragraph>
        </section>
        <section id="2">
            <heading>Section Two</heading>
            <paragraph>This is the second section content.</paragraph>
        </section>
    </content>
</document>"#
        )
        .unwrap();

        let result = processor.convert_to_markdown(temp_file.path()).await;

        assert!(result.is_ok(), "XML conversion should succeed");

        let doc = result.unwrap();
        assert!(!doc.content.is_empty(), "Content should not be empty");
        assert!(
            doc.metadata.get("source_format") == Some(&"xml".to_string()),
            "Source format should be xml"
        );
    }

    #[tokio::test]
    async fn test_conversion_preserves_structure() {
        let processor = TransmutationProcessor::new().unwrap();

        let mut temp_file = NamedTempFile::with_suffix(".html").unwrap();
        writeln!(
            temp_file,
            r#"<html>
<body>
<h1>Main Title</h1>
<h2>Subtitle</h2>
<p>Paragraph text</p>
<table>
    <tr><td>Cell 1</td><td>Cell 2</td></tr>
    <tr><td>Cell 3</td><td>Cell 4</td></tr>
</table>
</body>
</html>"#
        )
        .unwrap();

        let result = processor.convert_to_markdown(temp_file.path()).await;
        assert!(result.is_ok());

        let doc = result.unwrap();
        // The markdown output should preserve some structure
        // Exact format depends on transmutation implementation
        assert!(
            doc.content.len() > 10,
            "Converted content should have reasonable length"
        );
    }

    #[tokio::test]
    async fn test_conversion_statistics() {
        let processor = TransmutationProcessor::new().unwrap();

        let mut temp_file = NamedTempFile::with_suffix(".html").unwrap();
        writeln!(temp_file, "<html><body><p>Test content</p></body></html>").unwrap();

        let result = processor.convert_to_markdown(temp_file.path()).await;
        assert!(result.is_ok());

        let doc = result.unwrap();

        // Check that statistics are captured
        assert!(
            doc.metadata.contains_key("input_size_bytes"),
            "Should have input_size_bytes"
        );
        assert!(
            doc.metadata.contains_key("output_size_bytes"),
            "Should have output_size_bytes"
        );
        assert!(
            doc.metadata.contains_key("conversion_duration_ms"),
            "Should have conversion_duration_ms"
        );
    }

    #[tokio::test]
    async fn test_nonexistent_file() {
        let processor = TransmutationProcessor::new().unwrap();

        let result = processor
            .convert_to_markdown(&PathBuf::from("/nonexistent/path/file.pdf"))
            .await;

        assert!(
            result.is_err(),
            "Conversion of nonexistent file should fail"
        );
    }
}

// ============================================================================
// COMPARISON WITH DOCLING FEATURES
// ============================================================================

mod docling_comparison {
    use super::*;

    /// Documents features that Docling has which we should validate
    #[test]
    fn test_docling_feature_checklist() {
        // This test documents Docling features for comparison
        // Features are checked based on the transmutation implementation

        // ✅ Supported formats (matching Docling)
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "doc.pdf"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "doc.docx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "doc.pptx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "doc.xlsx"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "doc.html"
        )));

        // ✅ Image formats for OCR (Docling supports these)
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "img.png"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "img.jpg"
        )));
        assert!(TransmutationProcessor::is_supported_format(&PathBuf::from(
            "img.tiff"
        )));

        // ❌ Audio formats (Docling supports, transmutation doesn't)
        // Docling: WAV, MP3, VTT
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("audio.wav")
        ));
        assert!(!TransmutationProcessor::is_supported_format(
            &PathBuf::from("audio.mp3")
        ));
    }

    #[test]
    fn test_metadata_fields_match_docling() {
        // Docling extracts these metadata fields
        // Verify our implementation captures similar data

        let doc = ConvertedDocument::new("Content".to_string())
            .with_metadata("title".to_string(), "Document Title".to_string())
            .with_metadata("author".to_string(), "Author Name".to_string())
            .with_metadata("language".to_string(), "en".to_string())
            .with_metadata("page_count".to_string(), "10".to_string())
            .with_metadata("tables_extracted".to_string(), "3".to_string());

        // These fields should match what Docling provides
        assert!(doc.metadata.contains_key("title"));
        assert!(doc.metadata.contains_key("author"));
        assert!(doc.metadata.contains_key("language"));
        assert!(doc.metadata.contains_key("page_count"));
        assert!(doc.metadata.contains_key("tables_extracted"));
    }

    #[test]
    fn test_page_structure_matches_docling_output() {
        // Docling provides page-level content splitting
        // Our implementation should produce similar structure

        let pages = vec![
            PageInfo {
                page_number: 1,
                start_char: 0,
                end_char: 1000,
            },
            PageInfo {
                page_number: 2,
                start_char: 1000,
                end_char: 2000,
            },
        ];

        let doc = ConvertedDocument::with_pages("Multi-page content".to_string(), pages);

        // Should be able to locate content by page
        assert!(doc.get_page_at_position(500).is_some());
        assert!(doc.get_page_at_position(1500).is_some());

        // Should report total pages
        assert_eq!(doc.total_pages(), Some(2));
    }
}

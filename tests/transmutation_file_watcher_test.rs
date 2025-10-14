//! Integration tests for transmutation with file watcher

#[cfg(feature = "transmutation")]
#[cfg(test)]
mod file_watcher_tests {
    use vectorizer::file_watcher::FileWatcherConfig;
    use std::path::PathBuf;

    #[test]
    fn test_file_watcher_recognizes_transmutation_formats() {
        let config = FileWatcherConfig::default();

        // Verify transmutation formats are in include patterns when feature is enabled
        assert!(config.include_patterns.iter().any(|p| p.contains("pdf")),
                "PDF should be in include patterns");
        assert!(config.include_patterns.iter().any(|p| p.contains("docx")),
                "DOCX should be in include patterns");
        assert!(config.include_patterns.iter().any(|p| p.contains("xlsx")),
                "XLSX should be in include patterns");
        assert!(config.include_patterns.iter().any(|p| p.contains("pptx")),
                "PPTX should be in include patterns");
        assert!(config.include_patterns.iter().any(|p| p.contains("html")),
                "HTML should be in include patterns");
    }

    #[test]
    fn test_file_watcher_should_process_pdf() {
        let config = FileWatcherConfig::default();
        
        assert!(config.should_process_file(&PathBuf::from("document.pdf")));
        assert!(config.should_process_file(&PathBuf::from("report.PDF")));
        assert!(config.should_process_file(&PathBuf::from("/path/to/file.pdf")));
    }

    #[test]
    fn test_file_watcher_should_process_office_formats() {
        let config = FileWatcherConfig::default();
        
        assert!(config.should_process_file(&PathBuf::from("document.docx")));
        assert!(config.should_process_file(&PathBuf::from("spreadsheet.xlsx")));
        assert!(config.should_process_file(&PathBuf::from("presentation.pptx")));
    }

    #[test]
    fn test_file_watcher_should_process_web_formats() {
        let config = FileWatcherConfig::default();
        
        assert!(config.should_process_file(&PathBuf::from("page.html")));
        assert!(config.should_process_file(&PathBuf::from("index.htm")));
        assert!(config.should_process_file(&PathBuf::from("config.xml")));
    }

    #[test]
    fn test_file_watcher_should_process_images() {
        let config = FileWatcherConfig::default();
        
        assert!(config.should_process_file(&PathBuf::from("image.jpg")));
        assert!(config.should_process_file(&PathBuf::from("photo.jpeg")));
        assert!(config.should_process_file(&PathBuf::from("screenshot.png")));
    }

    #[test]
    fn test_file_watcher_exclude_data_directory() {
        let config = FileWatcherConfig::default();
        
        // Should NOT process files in data directory
        assert!(!config.should_process_file(&PathBuf::from("data/file.pdf")));
        assert!(!config.should_process_file(&PathBuf::from("/project/data/document.docx")));
    }

    #[test]
    fn test_file_watcher_exclude_binary_files() {
        let config = FileWatcherConfig::default();
        
        // Should NOT process .bin files
        assert!(!config.should_process_file(&PathBuf::from("file.bin")));
        assert!(!config.should_process_file(&PathBuf::from("data.BIN")));
    }

    #[test]
    fn test_file_watcher_exclude_build_artifacts() {
        let config = FileWatcherConfig::default();
        
        // Should NOT process files in target directory
        assert!(!config.should_process_file(&PathBuf::from("target/debug/file.pdf")));
        assert!(!config.should_process_file(&PathBuf::from("node_modules/package/file.html")));
    }

    #[test]
    fn test_file_watcher_custom_patterns() {
        let mut config = FileWatcherConfig::default();
        config.include_patterns = vec!["*.pdf".to_string(), "*.docx".to_string()];
        config.exclude_patterns = vec!["**/temp/**".to_string()];

        assert!(config.should_process_file(&PathBuf::from("document.pdf")));
        assert!(config.should_process_file(&PathBuf::from("report.docx")));
        assert!(!config.should_process_file(&PathBuf::from("temp/file.pdf")));
    }

    #[test]
    fn test_file_watcher_silent_check() {
        let config = FileWatcherConfig::default();
        
        // Test silent version (no logging)
        assert!(config.should_process_file_silent(&PathBuf::from("document.pdf")));
        assert!(config.should_process_file_silent(&PathBuf::from("page.html")));
        assert!(!config.should_process_file_silent(&PathBuf::from("data/file.pdf")));
    }

    #[test]
    fn test_file_watcher_max_file_size() {
        let config = FileWatcherConfig::default();
        
        assert_eq!(config.max_file_size, 10 * 1024 * 1024); // 10MB default
    }

    #[test]
    fn test_file_watcher_debounce_delay() {
        let config = FileWatcherConfig::default();
        
        assert_eq!(config.debounce_delay_ms, 1000); // 1 second default
        assert_eq!(config.debounce_duration().as_millis(), 1000);
    }
}

#[cfg(not(feature = "transmutation"))]
#[cfg(test)]
mod without_transmutation_file_watcher_tests {
    use vectorizer::file_watcher::FileWatcherConfig;
    use std::path::PathBuf;

    #[test]
    fn test_file_watcher_without_transmutation() {
        let config = FileWatcherConfig::default();

        // When transmutation is disabled, PDF/DOCX patterns should not be present by default
        let has_pdf = config.include_patterns.iter().any(|p| p.contains("pdf"));
        let has_docx = config.include_patterns.iter().any(|p| p.contains("docx"));

        // Without transmutation feature, these formats are not in default patterns
        assert!(!has_pdf, "PDF should not be in default patterns without transmutation");
        assert!(!has_docx, "DOCX should not be in default patterns without transmutation");

        // But text formats should still be there
        assert!(config.include_patterns.iter().any(|p| p.contains("txt")));
        assert!(config.include_patterns.iter().any(|p| p.contains("md")));
    }
}


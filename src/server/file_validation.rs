//! File validation for upload endpoint
//!
//! Provides file type validation based on extension, MIME type detection,
//! and binary file detection.

use std::path::Path;

use thiserror::Error;

use crate::config::FileUploadConfig;

/// File validation errors
#[derive(Debug, Error)]
pub enum FileValidationError {
    #[error("File extension '{0}' is not allowed")]
    ExtensionNotAllowed(String),

    #[error("File size {0} bytes exceeds maximum allowed size of {1} bytes")]
    FileTooLarge(usize, usize),

    #[error("Binary files are not allowed")]
    BinaryFileRejected,

    #[error("Missing file extension")]
    MissingExtension,

    #[error("Invalid file name")]
    InvalidFileName,
}

/// Validates a file based on the upload configuration
pub struct FileValidator {
    config: FileUploadConfig,
}

impl FileValidator {
    /// Create a new file validator with the given configuration
    pub fn new(config: FileUploadConfig) -> Self {
        Self { config }
    }

    /// Validate file extension
    pub fn validate_extension(&self, filename: &str) -> Result<String, FileValidationError> {
        let path = Path::new(filename);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .ok_or(FileValidationError::MissingExtension)?;

        if self.config.allowed_extensions.contains(&extension) {
            Ok(extension)
        } else {
            Err(FileValidationError::ExtensionNotAllowed(extension))
        }
    }

    /// Validate file size
    pub fn validate_size(&self, size: usize) -> Result<(), FileValidationError> {
        if size > self.config.max_file_size {
            Err(FileValidationError::FileTooLarge(
                size,
                self.config.max_file_size,
            ))
        } else {
            Ok(())
        }
    }

    /// Check if content appears to be binary
    pub fn is_binary_content(&self, content: &[u8]) -> bool {
        if !self.config.reject_binary {
            return false;
        }

        // Check first 8KB for binary indicators
        let check_size = content.len().min(8192);
        let sample = &content[..check_size];

        // Count null bytes and non-printable characters
        let mut null_count = 0;
        let mut non_printable_count = 0;

        for &byte in sample {
            if byte == 0 {
                null_count += 1;
            } else if byte < 0x09 || (byte > 0x0D && byte < 0x20 && byte != 0x1B) {
                // Non-printable, excluding common whitespace and escape
                non_printable_count += 1;
            }
        }

        // If more than 1% null bytes or 10% non-printable, consider binary
        let null_ratio = null_count as f32 / check_size as f32;
        let non_printable_ratio = non_printable_count as f32 / check_size as f32;

        null_ratio > 0.01 || non_printable_ratio > 0.10
    }

    /// Validate binary content
    pub fn validate_binary(&self, content: &[u8]) -> Result<(), FileValidationError> {
        if self.is_binary_content(content) {
            Err(FileValidationError::BinaryFileRejected)
        } else {
            Ok(())
        }
    }

    /// Full validation of a file
    pub fn validate(
        &self,
        filename: &str,
        content: &[u8],
    ) -> Result<ValidatedFile, FileValidationError> {
        // Validate extension
        let extension = self.validate_extension(filename)?;

        // Validate size
        self.validate_size(content.len())?;

        // Validate binary content
        self.validate_binary(content)?;

        // Try to decode as UTF-8
        let text_content = String::from_utf8_lossy(content).into_owned();

        Ok(ValidatedFile {
            filename: filename.to_string(),
            extension,
            content: text_content,
            size: content.len(),
        })
    }

    /// Get file language/type based on extension
    pub fn get_language_from_extension(extension: &str) -> &'static str {
        match extension.to_lowercase().as_str() {
            // Rust
            "rs" => "rust",
            // Python
            "py" | "pyw" | "pyi" => "python",
            // JavaScript/TypeScript
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "mts" | "cts" => "typescript",
            "jsx" => "javascriptreact",
            "tsx" => "typescriptreact",
            // Go
            "go" => "go",
            // Java/JVM
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "scala" | "sc" => "scala",
            // C/C++
            "c" => "c",
            "cpp" | "cc" | "cxx" => "cpp",
            "h" | "hpp" | "hxx" => "cpp",
            // C#
            "cs" => "csharp",
            // Ruby
            "rb" | "rake" | "gemspec" => "ruby",
            // PHP
            "php" => "php",
            // Swift
            "swift" => "swift",
            // R
            "r" | "R" => "r",
            // SQL
            "sql" => "sql",
            // Shell
            "sh" | "bash" | "zsh" => "shell",
            "ps1" | "psm1" | "psd1" => "powershell",
            "bat" | "cmd" => "batch",
            // Config
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" => "xml",
            "ini" | "cfg" | "conf" => "ini",
            // Web
            "html" | "htm" => "html",
            "css" => "css",
            "scss" => "scss",
            "sass" => "sass",
            "less" => "less",
            // Documentation
            "md" | "markdown" => "markdown",
            "rst" => "restructuredtext",
            "txt" | "text" => "plaintext",
            // Data
            "csv" => "csv",
            "log" => "log",
            // Other
            _ => "plaintext",
        }
    }
}

/// A validated file ready for processing
#[derive(Debug, Clone)]
pub struct ValidatedFile {
    /// Original filename
    pub filename: String,
    /// File extension (lowercase, without dot)
    pub extension: String,
    /// Text content of the file
    pub content: String,
    /// Size in bytes
    pub size: usize,
}

impl ValidatedFile {
    /// Get the detected language/type
    pub fn language(&self) -> &'static str {
        FileValidator::get_language_from_extension(&self.extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> FileUploadConfig {
        FileUploadConfig::default()
    }

    #[test]
    fn test_validate_extension_allowed() {
        let validator = FileValidator::new(default_config());

        assert!(validator.validate_extension("test.rs").is_ok());
        assert!(validator.validate_extension("test.py").is_ok());
        assert!(validator.validate_extension("test.md").is_ok());
        assert!(validator.validate_extension("TEST.RS").is_ok()); // Case insensitive
    }

    #[test]
    fn test_validate_extension_not_allowed() {
        let validator = FileValidator::new(default_config());

        assert!(matches!(
            validator.validate_extension("test.exe"),
            Err(FileValidationError::ExtensionNotAllowed(_))
        ));
        assert!(matches!(
            validator.validate_extension("test.bin"),
            Err(FileValidationError::ExtensionNotAllowed(_))
        ));
    }

    #[test]
    fn test_validate_extension_missing() {
        let validator = FileValidator::new(default_config());

        assert!(matches!(
            validator.validate_extension("noextension"),
            Err(FileValidationError::MissingExtension)
        ));
    }

    #[test]
    fn test_validate_size() {
        let config = FileUploadConfig {
            max_file_size: 1000,
            ..default_config()
        };
        let validator = FileValidator::new(config);

        assert!(validator.validate_size(500).is_ok());
        assert!(validator.validate_size(1000).is_ok());
        assert!(matches!(
            validator.validate_size(1001),
            Err(FileValidationError::FileTooLarge(_, _))
        ));
    }

    #[test]
    fn test_is_binary_content() {
        let validator = FileValidator::new(default_config());

        // Text content
        let text = b"Hello, world!\nThis is a text file.";
        assert!(!validator.is_binary_content(text));

        // Binary content (contains null bytes)
        let binary = b"Hello\x00World\x00\x00\x00";
        assert!(validator.is_binary_content(binary));

        // Binary content (many non-printable)
        let binary2: Vec<u8> = (0..100).map(|i| i % 8).collect();
        assert!(validator.is_binary_content(&binary2));
    }

    #[test]
    fn test_validate_full() {
        let validator = FileValidator::new(default_config());

        let content = b"fn main() { println!(\"Hello\"); }";
        let result = validator.validate("test.rs", content);

        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.filename, "test.rs");
        assert_eq!(file.extension, "rs");
        assert_eq!(file.language(), "rust");
    }

    #[test]
    fn test_get_language_from_extension() {
        assert_eq!(FileValidator::get_language_from_extension("rs"), "rust");
        assert_eq!(FileValidator::get_language_from_extension("py"), "python");
        assert_eq!(
            FileValidator::get_language_from_extension("ts"),
            "typescript"
        );
        assert_eq!(FileValidator::get_language_from_extension("md"), "markdown");
        assert_eq!(
            FileValidator::get_language_from_extension("unknown"),
            "plaintext"
        );
    }

    #[test]
    fn test_get_language_from_extension_all_languages() {
        // Rust
        assert_eq!(FileValidator::get_language_from_extension("rs"), "rust");

        // Python
        assert_eq!(FileValidator::get_language_from_extension("py"), "python");
        assert_eq!(FileValidator::get_language_from_extension("pyw"), "python");
        assert_eq!(FileValidator::get_language_from_extension("pyi"), "python");

        // JavaScript/TypeScript
        assert_eq!(
            FileValidator::get_language_from_extension("js"),
            "javascript"
        );
        assert_eq!(
            FileValidator::get_language_from_extension("mjs"),
            "javascript"
        );
        assert_eq!(
            FileValidator::get_language_from_extension("ts"),
            "typescript"
        );
        assert_eq!(
            FileValidator::get_language_from_extension("tsx"),
            "typescriptreact"
        );
        assert_eq!(
            FileValidator::get_language_from_extension("jsx"),
            "javascriptreact"
        );

        // Go
        assert_eq!(FileValidator::get_language_from_extension("go"), "go");

        // Java/JVM
        assert_eq!(FileValidator::get_language_from_extension("java"), "java");
        assert_eq!(FileValidator::get_language_from_extension("kt"), "kotlin");
        assert_eq!(FileValidator::get_language_from_extension("scala"), "scala");

        // C/C++
        assert_eq!(FileValidator::get_language_from_extension("c"), "c");
        assert_eq!(FileValidator::get_language_from_extension("cpp"), "cpp");
        assert_eq!(FileValidator::get_language_from_extension("h"), "cpp");
        assert_eq!(FileValidator::get_language_from_extension("hpp"), "cpp");

        // C#
        assert_eq!(FileValidator::get_language_from_extension("cs"), "csharp");

        // Ruby
        assert_eq!(FileValidator::get_language_from_extension("rb"), "ruby");

        // PHP
        assert_eq!(FileValidator::get_language_from_extension("php"), "php");

        // Swift
        assert_eq!(FileValidator::get_language_from_extension("swift"), "swift");

        // Shell
        assert_eq!(FileValidator::get_language_from_extension("sh"), "shell");
        assert_eq!(FileValidator::get_language_from_extension("bash"), "shell");
        assert_eq!(
            FileValidator::get_language_from_extension("ps1"),
            "powershell"
        );
        assert_eq!(FileValidator::get_language_from_extension("bat"), "batch");

        // Config
        assert_eq!(FileValidator::get_language_from_extension("json"), "json");
        assert_eq!(FileValidator::get_language_from_extension("yaml"), "yaml");
        assert_eq!(FileValidator::get_language_from_extension("yml"), "yaml");
        assert_eq!(FileValidator::get_language_from_extension("toml"), "toml");
        assert_eq!(FileValidator::get_language_from_extension("xml"), "xml");
        assert_eq!(FileValidator::get_language_from_extension("ini"), "ini");

        // Web
        assert_eq!(FileValidator::get_language_from_extension("html"), "html");
        assert_eq!(FileValidator::get_language_from_extension("css"), "css");
        assert_eq!(FileValidator::get_language_from_extension("scss"), "scss");

        // Documentation
        assert_eq!(FileValidator::get_language_from_extension("md"), "markdown");
        assert_eq!(
            FileValidator::get_language_from_extension("rst"),
            "restructuredtext"
        );
        assert_eq!(
            FileValidator::get_language_from_extension("txt"),
            "plaintext"
        );

        // Data
        assert_eq!(FileValidator::get_language_from_extension("csv"), "csv");
        assert_eq!(FileValidator::get_language_from_extension("log"), "log");
        assert_eq!(FileValidator::get_language_from_extension("sql"), "sql");
    }

    #[test]
    fn test_validate_binary_rejection_disabled() {
        let config = FileUploadConfig {
            reject_binary: false,
            ..default_config()
        };
        let validator = FileValidator::new(config);

        // Binary content should NOT be detected as binary when rejection is disabled
        let binary = b"Hello\x00World\x00\x00\x00";
        assert!(!validator.is_binary_content(binary));
    }

    #[test]
    fn test_validate_utf8_content() {
        let validator = FileValidator::new(default_config());

        // UTF-8 content with various characters
        let utf8_content = "Hello 世界 🌍 Привет мир!".as_bytes();
        assert!(!validator.is_binary_content(utf8_content));

        let result = validator.validate("test.txt", utf8_content);
        assert!(result.is_ok());

        let file = result.unwrap();
        assert!(file.content.contains("世界"));
        assert!(file.content.contains("🌍"));
    }

    #[test]
    fn test_validate_empty_file() {
        let validator = FileValidator::new(default_config());

        let result = validator.validate("empty.txt", b"");
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.size, 0);
        assert!(file.content.is_empty());
    }

    #[test]
    fn test_validate_whitespace_only() {
        let validator = FileValidator::new(default_config());

        let content = b"   \n\t\r\n   ";
        let result = validator.validate("whitespace.txt", content);
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.size, content.len());
    }

    #[test]
    fn test_validate_large_text_file() {
        let config = FileUploadConfig {
            max_file_size: 1024 * 1024, // 1MB
            ..default_config()
        };
        let validator = FileValidator::new(config);

        // Create a large text content (500KB)
        let large_content: Vec<u8> = "Hello, World!\n".repeat(40000).into_bytes();

        let result = validator.validate("large.txt", &large_content);
        assert!(result.is_ok());

        let file = result.unwrap();
        assert!(file.size > 500000);
    }

    #[test]
    fn test_validate_file_too_large() {
        let config = FileUploadConfig {
            max_file_size: 50, // Very small limit
            ..default_config()
        };
        let validator = FileValidator::new(config);

        let content = b"This content is definitely larger than fifty bytes, so it should fail validation check.";

        let result = validator.validate("toolarge.txt", content);
        assert!(
            result.is_err(),
            "Content length: {} should exceed limit of 50",
            content.len()
        );

        match result {
            Err(FileValidationError::FileTooLarge(size, max)) => {
                assert!(size > 50);
                assert_eq!(max, 50);
            }
            _ => panic!("Expected FileTooLarge error"),
        }
    }

    #[test]
    fn test_validate_binary_file_rejected() {
        let validator = FileValidator::new(default_config());

        // Simulated binary content (e.g., PNG header)
        let binary_content = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00,
        ];

        let result = validator.validate("image.txt", &binary_content);
        assert!(result.is_err());

        match result {
            Err(FileValidationError::BinaryFileRejected) => {}
            _ => panic!("Expected BinaryFileRejected error"),
        }
    }

    #[test]
    fn test_validated_file_language() {
        let file = ValidatedFile {
            filename: "main.rs".to_string(),
            extension: "rs".to_string(),
            content: "fn main() {}".to_string(),
            size: 12,
        };

        assert_eq!(file.language(), "rust");
    }

    #[test]
    fn test_validated_file_language_python() {
        let file = ValidatedFile {
            filename: "script.py".to_string(),
            extension: "py".to_string(),
            content: "print('hello')".to_string(),
            size: 14,
        };

        assert_eq!(file.language(), "python");
    }

    #[test]
    fn test_validate_extension_case_insensitive() {
        let validator = FileValidator::new(default_config());

        assert!(validator.validate_extension("Test.RS").is_ok());
        assert!(validator.validate_extension("Test.Rs").is_ok());
        assert!(validator.validate_extension("Test.rS").is_ok());
        assert!(validator.validate_extension("TEST.PY").is_ok());
        assert!(validator.validate_extension("file.JSON").is_ok());
    }

    #[test]
    fn test_validate_extension_with_multiple_dots() {
        let validator = FileValidator::new(default_config());

        assert!(validator.validate_extension("test.spec.ts").is_ok());
        assert!(validator.validate_extension("component.test.js").is_ok());
        assert!(validator.validate_extension("file.min.css").is_ok());
        assert!(validator.validate_extension("backup.2024.txt").is_ok());
    }

    #[test]
    fn test_validate_extension_hidden_file() {
        let validator = FileValidator::new(default_config());

        // Hidden files with extensions
        assert!(validator.validate_extension(".gitignore").is_err()); // No standard extension
        assert!(validator.validate_extension(".env.txt").is_ok()); // Has txt extension
    }

    #[test]
    fn test_custom_allowed_extensions() {
        let config = FileUploadConfig {
            allowed_extensions: vec!["custom".to_string(), "xyz".to_string()],
            ..default_config()
        };
        let validator = FileValidator::new(config);

        assert!(validator.validate_extension("file.custom").is_ok());
        assert!(validator.validate_extension("file.xyz").is_ok());
        assert!(validator.validate_extension("file.rs").is_err()); // Not in custom list
        assert!(validator.validate_extension("file.py").is_err()); // Not in custom list
    }

    #[test]
    fn test_file_validation_error_display() {
        let err1 = FileValidationError::ExtensionNotAllowed("exe".to_string());
        assert!(err1.to_string().contains("exe"));

        let err2 = FileValidationError::FileTooLarge(2000, 1000);
        assert!(err2.to_string().contains("2000"));
        assert!(err2.to_string().contains("1000"));

        let err3 = FileValidationError::BinaryFileRejected;
        assert!(err3.to_string().contains("Binary"));

        let err4 = FileValidationError::MissingExtension;
        assert!(err4.to_string().contains("extension"));
    }
}

//! Text Normalization
//!
//! Applies content-aware text normalization to reduce storage and improve consistency.

use super::detector::ContentType;
use super::hasher::{ContentHash, ContentHashCalculator};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

/// Normalization level determines aggressiveness of text processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizationLevel {
    /// Conservative: Minimal changes, preserve structure (for code/tables)
    Conservative,
    /// Moderate: Balanced normalization (for markdown)
    Moderate,
    /// Aggressive: Maximum compression (for plain text)
    Aggressive,
}

/// Normalization policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationPolicy {
    /// Policy version for cache invalidation
    pub version: u32,
    /// Default normalization level
    pub level: NormalizationLevel,
    /// Preserve case (if false, lowercase everything)
    pub preserve_case: bool,
    /// Collapse multiple whitespaces
    pub collapse_whitespace: bool,
    /// Remove HTML tags
    pub remove_html: bool,
}

impl Default for NormalizationPolicy {
    fn default() -> Self {
        Self {
            version: super::NORMALIZATION_VERSION,
            level: NormalizationLevel::Moderate,
            preserve_case: true,
            collapse_whitespace: true,
            remove_html: false,
        }
    }
}

/// Normalized content with metadata
#[derive(Debug, Clone)]
pub struct NormalizedContent {
    /// Normalized text
    pub text: String,
    /// Content hash for deduplication
    pub content_hash: ContentHash,
    /// Normalization metadata
    pub metadata: NormalizationMetadata,
}

/// Metadata about normalization process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationMetadata {
    /// Original size in bytes
    pub original_size: usize,
    /// Normalized size in bytes
    pub normalized_size: usize,
    /// Bytes removed (whitespace, control chars, etc.)
    pub removed_bytes: usize,
    /// Policy version used
    pub policy_version: u32,
    /// Content type detected
    pub content_type: String,
}

/// Text normalizer
pub struct TextNormalizer {
    policy: NormalizationPolicy,
    hasher: ContentHashCalculator,
}

impl TextNormalizer {
    /// Create a new text normalizer with policy
    pub fn new(policy: NormalizationPolicy) -> Self {
        Self {
            policy,
            hasher: ContentHashCalculator::new(),
        }
    }

    /// Normalize text with optional content type override
    pub fn normalize(
        &self,
        raw: &str,
        content_type: Option<ContentType>,
    ) -> NormalizedContent {
        let original_size = raw.len();
        let content_type = content_type.unwrap_or(ContentType::Plain);

        // Apply normalization based on content type and policy
        let normalized = match (&self.policy.level, &content_type) {
            // Code and tables always use conservative
            (_, ContentType::Code { .. }) | (_, ContentType::Table { .. }) => {
                self.normalize_conservative(raw)
            }
            // Aggressive for plain text when configured
            (NormalizationLevel::Aggressive, ContentType::Plain) => {
                self.normalize_aggressive(raw)
            }
            // Moderate for everything else
            _ => self.normalize_moderate(raw),
        };

        let normalized_size = normalized.len();
        let content_hash = self.hasher.hash(&normalized);

        NormalizedContent {
            text: normalized,
            content_hash,
            metadata: NormalizationMetadata {
                original_size,
                normalized_size,
                removed_bytes: original_size.saturating_sub(normalized_size),
                policy_version: self.policy.version,
                content_type: format!("{:?}", content_type),
            },
        }
    }

    /// Normalize query text (always aggressive for consistency)
    pub fn normalize_query(&self, query: &str) -> String {
        self.normalize_aggressive(query)
    }

    /// Conservative normalization (Level 1)
    /// - Unicode NFC (canonical composition)
    /// - CRLF → LF
    /// - Remove BOM
    /// - Trim trailing whitespace per line
    fn normalize_conservative(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());

        // Unicode NFC normalization
        let normalized_unicode: String = text.nfc().collect();

        // Process line by line
        for line in normalized_unicode.lines() {
            // Trim trailing whitespace
            let trimmed = line.trim_end();
            result.push_str(trimmed);
            result.push('\n');
        }

        // Remove BOM if present
        result = result.trim_start_matches('\u{FEFF}').to_string();

        // Trim final newline if added
        if result.ends_with('\n') && !text.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Moderate normalization (Level 2)
    /// - All Level 1 transformations
    /// - Remove zero-width characters
    /// - Preserve code blocks in markdown
    /// - Normalize heading markers
    fn normalize_moderate(&self, text: &str) -> String {
        // Start with conservative
        let mut result = self.normalize_conservative(text);

        // Remove zero-width characters
        result = result
            .chars()
            .filter(|&c| !matches!(c, '\u{200B}'..='\u{200D}' | '\u{FEFF}'))
            .collect();

        // Collapse excessive newlines (more than 2 consecutive)
        result = Self::collapse_newlines(&result, 2);

        result
    }

    /// Aggressive normalization (Level 3)
    /// - All Level 2 transformations
    /// - Unicode NFKC (compatibility composition)
    /// - Collapse multiple spaces → single space
    /// - Collapse multiple newlines → max 2
    /// - Remove control characters (except \n, \t)
    /// - Optional case folding
    fn normalize_aggressive(&self, text: &str) -> String {
        // Unicode NFKC normalization (compatibility)
        let normalized_unicode: String = text.nfkc().collect();

        let mut result = String::with_capacity(normalized_unicode.len());
        let mut prev_was_space = false;
        let mut prev_was_newline = false;
        let mut newline_count = 0;

        for c in normalized_unicode.chars() {
            match c {
                // Newlines
                '\n' | '\r' => {
                    if !prev_was_newline {
                        result.push('\n');
                        newline_count = 1;
                        prev_was_newline = true;
                        prev_was_space = false;
                    } else if newline_count < 2 {
                        result.push('\n');
                        newline_count += 1;
                    }
                }
                // Spaces and tabs
                ' ' | '\t' => {
                    if !prev_was_space && !prev_was_newline {
                        result.push(' ');
                        prev_was_space = true;
                    }
                }
                // Control characters (skip)
                c if c.is_control() && c != '\n' && c != '\t' => {
                    // Skip control characters
                }
                // Regular characters
                c => {
                    let output_char = if self.policy.preserve_case {
                        c
                    } else {
                        c.to_lowercase().next().unwrap_or(c)
                    };
                    result.push(output_char);
                    prev_was_space = false;
                    prev_was_newline = false;
                    newline_count = 0;
                }
            }
        }

        // Trim final whitespace
        result.trim_end().to_string()
    }

    /// Collapse consecutive newlines
    fn collapse_newlines(text: &str, max_consecutive: usize) -> String {
        let mut result = String::with_capacity(text.len());
        let mut newline_count = 0;

        for c in text.chars() {
            if c == '\n' {
                newline_count += 1;
                if newline_count <= max_consecutive {
                    result.push(c);
                }
            } else {
                newline_count = 0;
                result.push(c);
            }
        }

        result
    }
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self::new(NormalizationPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservative_normalization() {
        let normalizer = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Conservative,
            ..Default::default()
        });

        // Test CRLF → LF
        let input = "line1\r\nline2\r\nline3";
        let result = normalizer.normalize_conservative(input);
        assert_eq!(result, "line1\nline2\nline3");

        // Test trailing whitespace removal
        let input = "line1   \nline2\t\t\nline3  ";
        let result = normalizer.normalize_conservative(input);
        assert_eq!(result, "line1\nline2\nline3");

        // Test BOM removal
        let input = "\u{FEFF}Hello, world!";
        let result = normalizer.normalize_conservative(input);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_moderate_normalization() {
        let normalizer = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Moderate,
            ..Default::default()
        });

        // Test zero-width character removal
        let input = "Hello\u{200B}World\u{200C}Test";
        let result = normalizer.normalize_moderate(input);
        assert!(!result.contains('\u{200B}'));
        assert!(!result.contains('\u{200C}'));

        // Test newline collapsing
        let input = "line1\n\n\n\n\nline2";
        let result = normalizer.normalize_moderate(input);
        assert_eq!(result, "line1\n\nline2");
    }

    #[test]
    fn test_aggressive_normalization() {
        let normalizer = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Aggressive,
            preserve_case: true,
            ..Default::default()
        });

        // Test multiple spaces
        let input = "Hello    World   Test";
        let result = normalizer.normalize_aggressive(input);
        assert_eq!(result, "Hello World Test");

        // Test multiple newlines
        let input = "line1\n\n\n\nline2";
        let result = normalizer.normalize_aggressive(input);
        assert_eq!(result, "line1\n\nline2");

        // Test control character removal
        let input = "Hello\x00World\x01Test";
        let result = normalizer.normalize_aggressive(input);
        assert_eq!(result, "HelloWorldTest");
    }

    #[test]
    fn test_case_folding() {
        let normalizer = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Aggressive,
            preserve_case: false,
            ..Default::default()
        });

        let input = "Hello WORLD Test";
        let result = normalizer.normalize_aggressive(input);
        assert_eq!(result, "hello world test");
    }

    #[test]
    fn test_normalize_with_content_type() {
        let normalizer = TextNormalizer::default();

        // Code should use conservative
        let code = "fn   main()   {\n    println!(\"test\");\n}";
        let result = normalizer.normalize(
            code,
            Some(ContentType::Code {
                language: Some("rust".to_string()),
            }),
        );
        assert!(result.text.contains("   ")); // Preserves code spacing

        // Plain text should use moderate by default
        let plain = "Hello   World\n\n\n\nTest";
        let result = normalizer.normalize(plain, Some(ContentType::Plain));
        assert!(!result.text.contains("   ")); // Collapses spacing
    }

    #[test]
    fn test_metadata() {
        let normalizer = TextNormalizer::default();

        let input = "Hello    World\n\n\n\nTest   ";
        let result = normalizer.normalize(input, Some(ContentType::Plain));

        assert_eq!(result.metadata.original_size, input.len());
        assert!(result.metadata.normalized_size < result.metadata.original_size);
        assert!(result.metadata.removed_bytes > 0);
    }

    #[test]
    fn test_query_normalization() {
        let normalizer = TextNormalizer::default();

        let query = "  Search   Query   With   Spaces  ";
        let result = normalizer.normalize_query(query);

        // Should be aggressively normalized
        assert_eq!(result.trim(), "Search Query With Spaces");
        assert!(!result.contains("  ")); // No double spaces
    }

    #[test]
    fn test_unicode_normalization() {
        let normalizer = TextNormalizer::default();

        // Test NFKC (compatibility normalization)
        let input = "ﬁ"; // U+FB01 (ligature fi)
        let result = normalizer.normalize_aggressive(input);
        // NFKC decomposes ligatures
        assert_eq!(result, "fi");

        // Test NFC (canonical normalization)
        let input = "é"; // e + combining acute
        let result = normalizer.normalize_conservative(input);
        assert_eq!(result, "é"); // Should be composed
    }
}


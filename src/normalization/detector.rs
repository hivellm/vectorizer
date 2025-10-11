//! Content Type Detection
//!
//! Detects content type to apply appropriate normalization strategies.

use regex::Regex;
use std::path::Path;

/// Content type classification for normalization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// Programming language code (preserve whitespace)
    Code { language: Option<String> },
    /// Markdown with possible code blocks
    Markdown,
    /// Tabular data (CSV, TSV)
    Table { format: TableFormat },
    /// HTML markup
    Html,
    /// Plain text (aggressive normalization)
    Plain,
    /// JSON/YAML structured data
    Json,
}

/// Table format variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableFormat {
    Csv,
    Tsv,
    Psv, // Pipe-separated
}

/// Content type detector using heuristics and file extensions
pub struct ContentTypeDetector {
    // Cached regex patterns
    code_patterns: Vec<Regex>,
    html_pattern: Regex,
    json_pattern: Regex,
    markdown_pattern: Regex,
}

impl ContentTypeDetector {
    /// Create a new content type detector
    pub fn new() -> Self {
        Self {
            code_patterns: vec![
                // Shebang
                Regex::new(r"^#!/").unwrap(),
                // Function definitions
                Regex::new(r"(fn|def|function|func)\s+\w+\s*\(").unwrap(),
                // Class definitions
                Regex::new(r"(class|struct|interface|trait)\s+\w+").unwrap(),
                // Import/include statements
                Regex::new(r"^(import|use|require|include|from)\s+").unwrap(),
            ],
            html_pattern: Regex::new(r"<[a-zA-Z][^>]*>.*</[a-zA-Z]+>").unwrap(),
            json_pattern: Regex::new(r#"^\s*[{\[].*[}\]]\s*$"#).unwrap(),
            markdown_pattern: Regex::new(r"^#{1,6}\s+.+|```|^\*\*|^-\s|^\d+\.\s").unwrap(),
        }
    }

    /// Detect content type from file path and content
    pub fn detect(&self, content: &str, file_path: Option<&Path>) -> ContentType {
        // First, try file extension
        if let Some(path) = file_path {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Some(content_type) = self.detect_by_extension(ext) {
                    return content_type;
                }
            }
        }

        // Fallback to content heuristics
        self.detect_by_heuristics(content)
    }

    /// Detect by file extension
    fn detect_by_extension(&self, ext: &str) -> Option<ContentType> {
        match ext.to_lowercase().as_str() {
            // Programming languages
            "rs" => Some(ContentType::Code {
                language: Some("rust".to_string()),
            }),
            "py" => Some(ContentType::Code {
                language: Some("python".to_string()),
            }),
            "js" | "jsx" | "mjs" => Some(ContentType::Code {
                language: Some("javascript".to_string()),
            }),
            "ts" | "tsx" => Some(ContentType::Code {
                language: Some("typescript".to_string()),
            }),
            "java" => Some(ContentType::Code {
                language: Some("java".to_string()),
            }),
            "c" | "h" => Some(ContentType::Code {
                language: Some("c".to_string()),
            }),
            "cpp" | "cc" | "cxx" | "hpp" => Some(ContentType::Code {
                language: Some("cpp".to_string()),
            }),
            "go" => Some(ContentType::Code {
                language: Some("go".to_string()),
            }),
            "rb" => Some(ContentType::Code {
                language: Some("ruby".to_string()),
            }),
            "php" => Some(ContentType::Code {
                language: Some("php".to_string()),
            }),
            "cs" => Some(ContentType::Code {
                language: Some("csharp".to_string()),
            }),
            "swift" => Some(ContentType::Code {
                language: Some("swift".to_string()),
            }),
            "kt" | "kts" => Some(ContentType::Code {
                language: Some("kotlin".to_string()),
            }),

            // Markup and data
            "md" | "markdown" => Some(ContentType::Markdown),
            "html" | "htm" => Some(ContentType::Html),
            "json" | "jsonl" => Some(ContentType::Json),
            "yaml" | "yml" => Some(ContentType::Json),

            // Tables
            "csv" => Some(ContentType::Table {
                format: TableFormat::Csv,
            }),
            "tsv" => Some(ContentType::Table {
                format: TableFormat::Tsv,
            }),

            // Plain text
            "txt" | "text" => Some(ContentType::Plain),

            _ => None,
        }
    }

    /// Detect by content heuristics
    fn detect_by_heuristics(&self, content: &str) -> ContentType {
        let lines: Vec<&str> = content.lines().take(50).collect(); // Sample first 50 lines
        let sample = lines.join("\n");

        // Check for HTML
        if self.html_pattern.is_match(&sample) {
            return ContentType::Html;
        }

        // Check for JSON
        if self.json_pattern.is_match(content.trim()) {
            return ContentType::Json;
        }

        // Check for Markdown
        let markdown_indicators = lines
            .iter()
            .filter(|line| self.markdown_pattern.is_match(line))
            .count();
        if markdown_indicators > 2 {
            return ContentType::Markdown;
        }

        // Check for code patterns
        let code_indicators = self
            .code_patterns
            .iter()
            .filter(|pattern| pattern.is_match(&sample))
            .count();
        if code_indicators > 0 {
            return ContentType::Code { language: None };
        }

        // Check for table (heuristic: consistent delimiters)
        if self.is_likely_table(&lines) {
            return ContentType::Table {
                format: self.detect_table_format(&lines),
            };
        }

        // Default to plain text
        ContentType::Plain
    }

    /// Check if content looks like a table
    fn is_likely_table(&self, lines: &[&str]) -> bool {
        if lines.len() < 3 {
            return false;
        }

        let non_empty_lines: Vec<&str> = lines.iter().filter(|l| !l.trim().is_empty()).copied().collect();
        if non_empty_lines.len() < 3 {
            return false;
        }

        // Check for consistent delimiter count
        let delimiters = [',', '\t', '|'];
        for &delimiter in &delimiters {
            let counts: Vec<usize> = non_empty_lines
                .iter()
                .map(|line| line.matches(delimiter).count())
                .filter(|&c| c > 0)
                .collect();

            if counts.len() >= 3 {
                // Check consistency (at least 70% of lines have same delimiter count)
                let most_common_count = Self::most_common(&counts);
                let consistent_count = counts
                    .iter()
                    .filter(|&&c| c == most_common_count)
                    .count();

                if consistent_count as f64 / counts.len() as f64 >= 0.7 {
                    return true;
                }
            }
        }

        false
    }

    /// Detect table format
    fn detect_table_format(&self, lines: &[&str]) -> TableFormat {
        let non_empty: Vec<&str> = lines.iter().filter(|l| !l.trim().is_empty()).copied().collect();
        if non_empty.is_empty() {
            return TableFormat::Csv;
        }

        let sample = non_empty.join("\n");
        let comma_count = sample.matches(',').count();
        let tab_count = sample.matches('\t').count();
        let pipe_count = sample.matches('|').count();

        if tab_count > comma_count && tab_count > pipe_count {
            TableFormat::Tsv
        } else if pipe_count > comma_count {
            TableFormat::Psv
        } else {
            TableFormat::Csv
        }
    }

    /// Find most common value in a slice
    fn most_common(values: &[usize]) -> usize {
        if values.is_empty() {
            return 0;
        }

        let mut counts = std::collections::HashMap::new();
        for &val in values {
            *counts.entry(val).or_insert(0) += 1;
        }

        *counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&val, _)| val)
            .unwrap_or(&0)
    }
}

impl Default for ContentTypeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_by_extension() {
        let detector = ContentTypeDetector::new();

        assert!(matches!(
            detector.detect("", Some(Path::new("test.rs"))),
            ContentType::Code { .. }
        ));

        assert_eq!(
            detector.detect("", Some(Path::new("test.md"))),
            ContentType::Markdown
        );

        assert_eq!(
            detector.detect("", Some(Path::new("test.json"))),
            ContentType::Json
        );

        assert!(matches!(
            detector.detect("", Some(Path::new("test.csv"))),
            ContentType::Table { .. }
        ));
    }

    #[test]
    fn test_detect_code_heuristics() {
        let detector = ContentTypeDetector::new();

        let rust_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        assert!(matches!(
            detector.detect(rust_code, None),
            ContentType::Code { .. }
        ));

        let python_code = r#"
def hello():
    print("Hello, world!")
"#;
        assert!(matches!(
            detector.detect(python_code, None),
            ContentType::Code { .. }
        ));
    }

    #[test]
    fn test_detect_markdown() {
        let detector = ContentTypeDetector::new();

        let markdown = r#"
# Heading

This is a paragraph.

## Subheading

- List item 1
- List item 2

```rust
fn main() {}
```
"#;
        assert_eq!(detector.detect(markdown, None), ContentType::Markdown);
    }

    #[test]
    fn test_detect_json() {
        let detector = ContentTypeDetector::new();

        let json = r#"{"key": "value", "nested": {"foo": "bar"}}"#;
        assert_eq!(detector.detect(json, None), ContentType::Json);
    }

    #[test]
    fn test_detect_table() {
        let detector = ContentTypeDetector::new();

        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        assert!(matches!(
            detector.detect(csv, None),
            ContentType::Table { .. }
        ));

        let tsv = "name\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA";
        assert!(matches!(
            detector.detect(tsv, None),
            ContentType::Table { .. }
        ));
    }

    #[test]
    fn test_detect_html() {
        let detector = ContentTypeDetector::new();

        let html = r#"
<html>
    <body>
        <h1>Hello</h1>
        <p>World</p>
    </body>
</html>
"#;
        assert_eq!(detector.detect(html, None), ContentType::Html);
    }

    #[test]
    fn test_plain_text_fallback() {
        let detector = ContentTypeDetector::new();

        let plain = "This is just some plain text without any special structure.";
        assert_eq!(detector.detect(plain, None), ContentType::Plain);
    }
}


//! Text chunking utilities

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::config::{DocumentChunk, LoaderConfig};

pub struct Chunker {
    config: LoaderConfig,
}

impl Chunker {
    pub fn new(config: LoaderConfig) -> Self {
        Self { config }
    }

    /// Split documents into chunks
    pub fn chunk_documents(&self, documents: &[(PathBuf, String)]) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();

        for (path, content) in documents {
            let file_chunks = self.chunk_text(content, path)?;
            chunks.extend(file_chunks);
        }

        Ok(chunks)
    }

    /// Split a single document into chunks
    pub fn chunk_text(&self, text: &str, file_path: &Path) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < text.len() {
            // Calculate the end position for this chunk
            let mut end = std::cmp::min(start + self.config.max_chunk_size, text.len());

            // If we're not at the end of the text, try to find a good break point
            if end < text.len() {
                // Ensure we're at a UTF-8 character boundary
                while end > start && !text.is_char_boundary(end) {
                    end -= 1;
                }

                // Try to break at a word boundary (whitespace, punctuation)
                if let Some(pos) = text[start..end].rfind(|c: char| {
                    c.is_whitespace() || c == '.' || c == '!' || c == '?' || c == '\n'
                }) {
                    end = start + pos + 1;

                    // Ensure the new end is still at a UTF-8 character boundary
                    while end > start && !text.is_char_boundary(end) {
                        end -= 1;
                    }
                }
            }

            // Extract the chunk text
            let chunk_text = text[start..end].trim();

            // Only create a chunk if it has content
            if !chunk_text.is_empty() {
                let chunk_id = format!("{}#{}", file_path.to_string_lossy(), chunk_index);

                let mut metadata = HashMap::new();
                metadata.insert(
                    "file_path".to_string(),
                    serde_json::Value::String(file_path.to_string_lossy().to_string()),
                );
                metadata.insert(
                    "chunk_index".to_string(),
                    serde_json::Value::Number(chunk_index.into()),
                );
                metadata.insert(
                    "file_extension".to_string(),
                    serde_json::Value::String(
                        file_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    ),
                );
                metadata.insert(
                    "chunk_size".to_string(),
                    serde_json::Value::Number(chunk_text.len().into()),
                );

                chunks.push(DocumentChunk {
                    id: chunk_id,
                    content: chunk_text.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    chunk_index,
                    metadata,
                });

                chunk_index += 1;
            }

            // Calculate the next start position with overlap
            let next_start = if end >= self.config.chunk_overlap {
                end - self.config.chunk_overlap
            } else {
                end
            };

            // Ensure we make progress (prevent infinite loop)
            if next_start <= start {
                start = end;
            } else {
                start = next_start;
            }

            // Ensure start is at a UTF-8 character boundary
            while start < text.len() && !text.is_char_boundary(start) {
                start += 1;
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn create_test_config() -> LoaderConfig {
        LoaderConfig {
            max_chunk_size: 100,
            chunk_overlap: 20,
            include_patterns: vec!["**/*.txt".to_string()],
            exclude_patterns: vec![],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: "test".to_string(),
            max_file_size: 1024 * 1024,
        }
    }

    #[test]
    fn test_chunker_creation() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        // Chunker should be created successfully
        assert!(true);
    }

    #[test]
    fn test_chunk_short_text() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let text = "This is a short text.";
        let path = PathBuf::from("/test.txt");

        let result = chunker.chunk_text(text, &path);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("This is a short text"));
        assert_eq!(chunks[0].chunk_index, 0);
    }

    #[test]
    fn test_chunk_long_text_with_overlap() {
        let config = LoaderConfig {
            max_chunk_size: 50,
            chunk_overlap: 10,
            ..create_test_config()
        };
        let chunker = Chunker::new(config);

        let text = "word ".repeat(30); // 150 chars (5 * 30)
        let path = PathBuf::from("/test.txt");

        let result = chunker.chunk_text(&text, &path);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert!(chunks.len() > 1);

        // Each chunk should be <= max_chunk_size
        for chunk in &chunks {
            assert!(chunk.content.len() <= 50);
        }
    }

    #[test]
    fn test_chunk_documents_empty() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let documents: Vec<(PathBuf, String)> = vec![];
        let result = chunker.chunk_documents(&documents);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_chunk_documents_multiple() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let documents = vec![
            (PathBuf::from("/file1.txt"), "Content of file 1".to_string()),
            (PathBuf::from("/file2.txt"), "Content of file 2".to_string()),
            (PathBuf::from("/file3.txt"), "Content of file 3".to_string()),
        ];

        let result = chunker.chunk_documents(&documents);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_chunk_metadata() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let text = "Test content";
        let path = PathBuf::from("/test.rs");

        let chunks = chunker.chunk_text(text, &path).unwrap();
        assert_eq!(chunks.len(), 1);

        let chunk = &chunks[0];

        // Check metadata
        assert_eq!(chunk.metadata["file_path"], "/test.rs");
        assert_eq!(chunk.metadata["chunk_index"], 0);
        assert_eq!(chunk.metadata["file_extension"], "rs");
        assert!(chunk.metadata.contains_key("chunk_size"));
    }

    #[test]
    fn test_chunk_empty_text() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let text = "";
        let path = PathBuf::from("/empty.txt");

        let result = chunker.chunk_text(text, &path);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_whitespace_only() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let text = "   \n\t   \n   ";
        let path = PathBuf::from("/whitespace.txt");

        let result = chunker.chunk_text(text, &path);
        assert!(result.is_ok());

        // Should produce no chunks (only whitespace)
        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_utf8_characters() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let text = "Hello 世界 🌍 émojis and special chars!";
        let path = PathBuf::from("/utf8.txt");

        let result = chunker.chunk_text(text, &path);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("世界"));
        assert!(chunks[0].content.contains("🌍"));
    }

    #[test]
    fn test_chunk_boundary_handling() {
        let config = LoaderConfig {
            max_chunk_size: 30,
            chunk_overlap: 5,
            ..create_test_config()
        };
        let chunker = Chunker::new(config);

        // Text with clear sentence boundaries
        let text = "First sentence here. Second sentence follows. Third one too.";
        let path = PathBuf::from("/test.txt");

        let result = chunker.chunk_text(text, &path);
        assert!(result.is_ok());

        let chunks = result.unwrap();

        // Should break at sentence boundaries where possible
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
        }
    }

    #[test]
    fn test_chunk_id_format() {
        let config = create_test_config();
        let chunker = Chunker::new(config);

        let text = "Chunk content";
        let path = PathBuf::from("/path/to/document.md");

        let chunks = chunker.chunk_text(text, &path).unwrap();
        assert_eq!(chunks.len(), 1);

        // ID should be in format "file_path#chunk_index"
        assert!(chunks[0].id.contains("/path/to/document.md"));
        assert!(chunks[0].id.contains("#0"));
    }
}

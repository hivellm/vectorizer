//! Text chunking utilities

use super::config::{DocumentChunk, LoaderConfig};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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


//! Transmutation integration module for document conversion
//!
//! This module provides integration with the transmutation document conversion engine,
//! enabling automatic conversion of various document formats (PDF, DOCX, XLSX, PPTX, etc.)
//! to Markdown for chunking and embedding.

pub mod types;

#[cfg(test)]
mod tests;

use std::path::Path;

use tracing::{debug, info, warn};
#[cfg(feature = "transmutation")]
use transmutation::{Converter, OutputFormat};
use types::{ConvertedDocument, PageInfo};

use crate::error::{Result, VectorizerError};

/// Transmutation processor for document conversion
pub struct TransmutationProcessor {
    #[cfg(feature = "transmutation")]
    converter: Converter,
}

impl TransmutationProcessor {
    /// Create a new transmutation processor
    pub fn new() -> Result<Self> {
        #[cfg(feature = "transmutation")]
        {
            let converter =
                Converter::new().map_err(|e| VectorizerError::TransmutationError(e.to_string()))?;
            info!("✅ Transmutation processor initialized");
            Ok(Self { converter })
        }

        #[cfg(not(feature = "transmutation"))]
        {
            Ok(Self {})
        }
    }

    /// Check if a file format is supported by transmutation
    pub fn is_supported_format(file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_lower.as_str(),
                // Document formats
                "pdf" | "docx" | "xlsx" | "pptx" |
                // Web formats
                "html" | "htm" | "xml" |
                // Image formats (with OCR)
                "jpg" | "jpeg" | "png" | "tiff" | "tif" | "bmp" | "gif" | "webp"
            )
        } else {
            false
        }
    }

    /// Convert a document to markdown
    ///
    /// Uses the transmutation crate to convert various document formats (PDF, DOCX, XLSX, etc.)
    /// to Markdown format optimized for LLM processing.
    #[cfg(feature = "transmutation")]
    pub async fn convert_to_markdown(&self, file_path: &Path) -> Result<ConvertedDocument> {
        info!("🔄 Converting document: {:?}", file_path);

        // Determine if this is a paginated format
        let is_paginated = if let Some(ext) = file_path.extension() {
            matches!(
                ext.to_string_lossy().to_lowercase().as_str(),
                "pdf" | "docx" | "pptx"
            )
        } else {
            false
        };

        // Set output format with page splitting for paginated documents
        let output_format = OutputFormat::Markdown {
            split_pages: is_paginated,
            optimize_for_llm: true,
        };

        // Perform the conversion
        let result = self
            .converter
            .convert(file_path)
            .to(output_format)
            .execute()
            .await
            .map_err(|e| VectorizerError::TransmutationError(e.to_string()))?;

        // Extract content from ConversionResult
        // Each ConversionOutput has data: Vec<u8> which is the converted content
        let content = if result.content.is_empty() {
            String::new()
        } else if result.content.len() == 1 {
            // Single output - convert bytes to string
            String::from_utf8_lossy(&result.content[0].data).to_string()
        } else {
            // Multiple outputs (split by pages) - join with page markers
            result
                .content
                .iter()
                .enumerate()
                .map(|(i, output)| {
                    let page_content = String::from_utf8_lossy(&output.data);
                    format!("--- Page {} ---\n{}", i + 1, page_content)
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        // Extract page information for paginated documents
        let page_info = if is_paginated && result.content.len() > 1 {
            Self::extract_page_info_from_result(&result, &content)
        } else {
            None
        };

        // Build metadata from ConversionResult
        let mut metadata = std::collections::HashMap::new();

        if let Some(ext) = file_path.extension() {
            metadata.insert(
                "source_format".to_string(),
                ext.to_string_lossy().to_string(),
            );
        }
        metadata.insert("converted_via".to_string(), "transmutation".to_string());

        // Add page count from result metadata
        let page_count = result.metadata.page_count;
        if page_count > 0 {
            metadata.insert("page_count".to_string(), page_count.to_string());
        }

        // Add document metadata if available
        if let Some(ref title) = result.metadata.title {
            metadata.insert("title".to_string(), title.clone());
        }
        if let Some(ref author) = result.metadata.author {
            metadata.insert("author".to_string(), author.clone());
        }
        if let Some(ref language) = result.metadata.language {
            metadata.insert("language".to_string(), language.clone());
        }

        // Add conversion statistics
        metadata.insert(
            "input_size_bytes".to_string(),
            result.statistics.input_size_bytes.to_string(),
        );
        metadata.insert(
            "output_size_bytes".to_string(),
            result.statistics.output_size_bytes.to_string(),
        );
        metadata.insert(
            "conversion_duration_ms".to_string(),
            result.statistics.duration.as_millis().to_string(),
        );
        if result.statistics.tables_extracted > 0 {
            metadata.insert(
                "tables_extracted".to_string(),
                result.statistics.tables_extracted.to_string(),
            );
        }

        let mut converted_doc = if let Some(pages) = page_info {
            ConvertedDocument::with_pages(content, pages)
        } else {
            ConvertedDocument::new(content)
        };

        // Add metadata
        for (key, value) in metadata {
            converted_doc = converted_doc.with_metadata(key, value);
        }

        info!(
            "✅ Conversion complete: {} characters, {} pages",
            converted_doc.content.len(),
            page_count
        );
        Ok(converted_doc)
    }

    /// Convert a document to markdown (feature disabled fallback)
    #[cfg(not(feature = "transmutation"))]
    pub async fn convert_to_markdown(&self, file_path: &Path) -> Result<ConvertedDocument> {
        warn!(
            "Transmutation feature is disabled, cannot convert: {:?}",
            file_path
        );
        Err(VectorizerError::TransmutationError(
            "Transmutation feature is not enabled".to_string(),
        ))
    }

    /// Extract page information from conversion result
    ///
    /// Uses the ConversionResult's content array and metadata to build PageInfo entries
    /// with accurate character positions based on the merged content string.
    #[cfg(feature = "transmutation")]
    fn extract_page_info_from_result(
        result: &transmutation::ConversionResult,
        merged_content: &str,
    ) -> Option<Vec<PageInfo>> {
        let page_count = result.content.len();
        if page_count <= 1 {
            return None;
        }

        let mut pages: Vec<PageInfo> = Vec::new();
        let mut current_pos: usize = 0;

        // Parse page markers in the merged content to get accurate positions
        for line in merged_content.lines() {
            let line_start: usize = current_pos;
            let line_len: usize = line.len() + 1; // +1 for newline

            // Check if this is a page marker we inserted
            if line.starts_with("--- Page ") {
                // Extract page number from marker
                if let Some(page_num_str) = line
                    .strip_prefix("--- Page ")
                    .and_then(|s| s.strip_suffix(" ---"))
                {
                    if let Ok(page_num) = page_num_str.parse::<usize>() {
                        // Close the previous page if exists
                        if let Some(last_page) = pages.last_mut() {
                            last_page.end_char = line_start.saturating_sub(2); // -2 for \n\n between pages
                        }

                        // Start a new page (content starts after the marker line)
                        pages.push(PageInfo {
                            page_number: page_num,
                            start_char: line_start + line_len,
                            end_char: merged_content.len(), // Will be updated for non-last pages
                        });
                    }
                }
            }

            current_pos += line_len;
        }

        // Finalize the last page's end position
        if let Some(last_page) = pages.last_mut() {
            last_page.end_char = merged_content.len();
        }

        if pages.is_empty() {
            None
        } else {
            debug!(
                "Extracted {} page boundaries from converted document",
                pages.len()
            );
            Some(pages)
        }
    }
}

impl Default for TransmutationProcessor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!("Failed to initialize transmutation processor: {}", e);
            #[cfg(feature = "transmutation")]
            panic!("Failed to initialize transmutation processor: {}", e);

            #[cfg(not(feature = "transmutation"))]
            Self {}
        })
    }
}

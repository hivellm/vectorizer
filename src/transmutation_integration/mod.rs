//! Transmutation integration module for document conversion
//!
//! This module provides integration with the transmutation document conversion engine,
//! enabling automatic conversion of various document formats (PDF, DOCX, XLSX, PPTX, etc.)
//! to Markdown for chunking and embedding.

pub mod types;

#[cfg(test)]
mod tests;

use crate::error::{Result, VectorizerError};
use std::path::Path;
use tracing::{debug, info, warn};
use types::{ConvertedDocument, PageInfo};

#[cfg(feature = "transmutation")]
use transmutation::{Converter, OutputFormat, ConversionOptions};

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
            let converter = Converter::new()
                .map_err(|e| VectorizerError::TransmutationError(e.to_string()))?;
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
    #[cfg(feature = "transmutation")]
    pub async fn convert_to_markdown(&self, file_path: &Path) -> Result<ConvertedDocument> {
        info!("🔄 Converting document: {:?}", file_path);

        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Determine if this is a paginated format
        let is_paginated = if let Some(ext) = file_path.extension() {
            matches!(
                ext.to_string_lossy().to_lowercase().as_str(),
                "pdf" | "docx" | "pptx"
            )
        } else {
            false
        };

        // Set conversion options
        let options = ConversionOptions {
            split_pages: is_paginated,
            optimize_for_llm: true,
            preserve_layout: false,
            extract_tables: true,
            extract_images: false,
            include_metadata: true,
            normalize_whitespace: true,
            ..Default::default()
        };

        // Perform the conversion
        let result = self.converter
            .convert(&file_path_str)
            .to(OutputFormat::Markdown)
            .with_options(options)
            .execute()
            .await
            .map_err(|e| VectorizerError::TransmutationError(e.to_string()))?;

        // Extract page information for paginated documents
        let page_info = if is_paginated {
            Self::extract_page_info(&result)
        } else {
            None
        };

        // Get the converted markdown content
        let content = result.text()
            .map_err(|e| VectorizerError::TransmutationError(e.to_string()))?;

        // Build metadata
        let mut metadata = std::collections::HashMap::new();
        
        if let Some(ext) = file_path.extension() {
            metadata.insert("source_format".to_string(), ext.to_string_lossy().to_string());
        }
        metadata.insert("converted_via".to_string(), "transmutation".to_string());
        
        if let Some(page_count) = result.page_count() {
            metadata.insert("page_count".to_string(), page_count.to_string());
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

        info!("✅ Conversion complete: {} characters", converted_doc.content.len());
        Ok(converted_doc)
    }

    /// Convert a document to markdown (feature disabled fallback)
    #[cfg(not(feature = "transmutation"))]
    pub async fn convert_to_markdown(&self, file_path: &Path) -> Result<ConvertedDocument> {
        warn!("Transmutation feature is disabled, cannot convert: {:?}", file_path);
        Err(VectorizerError::TransmutationError(
            "Transmutation feature is not enabled".to_string()
        ))
    }

    /// Extract page information from conversion result
    #[cfg(feature = "transmutation")]
    fn extract_page_info(result: &transmutation::ConversionResult) -> Option<Vec<PageInfo>> {
        // If the result contains page boundaries
        if let Some(page_count) = result.page_count() {
            let mut pages = Vec::new();
            let content = result.text().ok()?;
            
            // Try to detect page breaks in the markdown
            // Transmutation typically uses "--- Page N ---" markers
            let mut current_pos = 0;
            let mut page_num = 1;
            
            for (idx, line) in content.lines().enumerate() {
                let line_start = current_pos;
                let line_end = current_pos + line.len() + 1; // +1 for newline
                
                // Check if this is a page marker
                if line.starts_with("--- Page") || line.starts_with("# Page") {
                    if page_num > 1 {
                        // Close the previous page
                        if let Some(last_page) = pages.last_mut() {
                            last_page.end_char = line_start;
                        }
                    }
                    
                    // Start a new page
                    pages.push(PageInfo {
                        page_number: page_num,
                        start_char: line_start,
                        end_char: content.len(),
                    });
                    
                    page_num += 1;
                }
                
                current_pos = line_end;
            }
            
            // If we found page markers, return them
            if !pages.is_empty() {
                return Some(pages);
            }
            
            // Fallback: estimate equal page distribution
            let chars_per_page = content.len() / page_count;
            let mut pages = Vec::new();
            
            for i in 0..page_count {
                pages.push(PageInfo {
                    page_number: i + 1,
                    start_char: i * chars_per_page,
                    end_char: if i == page_count - 1 {
                        content.len()
                    } else {
                        (i + 1) * chars_per_page
                    },
                });
            }
            
            Some(pages)
        } else {
            None
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


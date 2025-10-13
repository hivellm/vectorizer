//! Type definitions for transmutation integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a page in a paginated document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    /// Page number (1-indexed)
    pub page_number: usize,
    /// Start character position in the full document
    pub start_char: usize,
    /// End character position in the full document
    pub end_char: usize,
}

/// Result of document conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedDocument {
    /// Converted markdown content
    pub content: String,
    /// Page information for paginated documents (PDF, DOCX, PPTX)
    pub page_info: Option<Vec<PageInfo>>,
    /// Document-level metadata
    pub metadata: HashMap<String, String>,
}

impl ConvertedDocument {
    /// Create a new converted document
    pub fn new(content: String) -> Self {
        Self {
            content,
            page_info: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new converted document with page information
    pub fn with_pages(content: String, page_info: Vec<PageInfo>) -> Self {
        Self {
            content,
            page_info: Some(page_info),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the document
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the page number for a given character position
    pub fn get_page_at_position(&self, char_pos: usize) -> Option<usize> {
        if let Some(pages) = &self.page_info {
            for page in pages {
                if char_pos >= page.start_char && char_pos < page.end_char {
                    return Some(page.page_number);
                }
            }
        }
        None
    }

    /// Get the total number of pages
    pub fn total_pages(&self) -> Option<usize> {
        self.page_info.as_ref().map(|pages| pages.len())
    }
}


//! File index for tracking files and their collections

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Mapping between file and collection with vector information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionVectorMapping {
    pub collection_name: String,
    pub vector_ids: Vec<String>,
    pub last_hash: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// File index for tracking files and their collections
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FileIndex {
    /// file_path -> Vec<(collection_name, vector_ids)>
    file_to_collections: HashMap<PathBuf, Vec<CollectionVectorMapping>>,
    /// collection_name -> Vec<file_path>
    collection_to_files: HashMap<String, HashSet<PathBuf>>,
}

impl FileIndex {
    /// Create a new file index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mapping between file and collection
    pub fn add_mapping(
        &mut self,
        file_path: PathBuf,
        collection_name: String,
        vector_ids: Vec<String>,
        last_hash: String,
    ) {
        let mapping = CollectionVectorMapping {
            collection_name: collection_name.clone(),
            vector_ids,
            last_hash,
            last_modified: chrono::Utc::now(),
        };

        // Add to file -> collections mapping
        self.file_to_collections
            .entry(file_path.clone())
            .or_insert_with(Vec::new)
            .push(mapping);

        // Add to collection -> files mapping
        self.collection_to_files
            .entry(collection_name)
            .or_insert_with(HashSet::new)
            .insert(file_path);
    }

    /// Remove a mapping between file and collection
    pub fn remove_mapping(&mut self, file_path: &PathBuf, collection_name: &str) {
        // Remove from file -> collections mapping
        if let Some(mappings) = self.file_to_collections.get_mut(file_path) {
            mappings.retain(|m| m.collection_name != collection_name);
            if mappings.is_empty() {
                self.file_to_collections.remove(file_path);
            }
        }

        // Remove from collection -> files mapping
        if let Some(files) = self.collection_to_files.get_mut(collection_name) {
            files.remove(file_path);
            if files.is_empty() {
                self.collection_to_files.remove(collection_name);
            }
        }
    }

    /// Remove all mappings for a file
    pub fn remove_file(&mut self, file_path: &PathBuf) -> Vec<(String, Vec<String>)> {
        let mut removed_collections = Vec::new();

        if let Some(mappings) = self.file_to_collections.remove(file_path) {
            for mapping in mappings {
                // Remove from collection -> files mapping
                if let Some(files) = self.collection_to_files.get_mut(&mapping.collection_name) {
                    files.remove(file_path);
                    if files.is_empty() {
                        self.collection_to_files.remove(&mapping.collection_name);
                    }
                }

                removed_collections.push((mapping.collection_name, mapping.vector_ids));
            }
        }

        removed_collections
    }

    /// Get all collections for a file
    pub fn get_collections_for_file(&self, file_path: &PathBuf) -> Vec<String> {
        self.file_to_collections
            .get(file_path)
            .map(|mappings| mappings.iter().map(|m| m.collection_name.clone()).collect())
            .unwrap_or_default()
    }

    /// Get all files for a collection
    pub fn get_files_for_collection(&self, collection_name: &str) -> Vec<PathBuf> {
        self.collection_to_files
            .get(collection_name)
            .map(|files| files.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get vector IDs for a file in a specific collection
    pub fn get_vector_ids(&self, file_path: &PathBuf, collection_name: &str) -> Option<Vec<String>> {
        self.file_to_collections
            .get(file_path)
            .and_then(|mappings| {
                mappings
                    .iter()
                    .find(|m| m.collection_name == collection_name)
                    .map(|m| m.vector_ids.clone())
            })
    }

    /// Update hash for a file
    pub fn update_hash(&mut self, file_path: &PathBuf, collection_name: &str, new_hash: String) {
        if let Some(mappings) = self.file_to_collections.get_mut(file_path) {
            for mapping in mappings {
                if mapping.collection_name == collection_name {
                    mapping.last_hash = new_hash;
                    mapping.last_modified = chrono::Utc::now();
                    break;
                }
            }
        }
    }

    /// Check if file exists in index
    pub fn contains_file(&self, file_path: &PathBuf) -> bool {
        self.file_to_collections.contains_key(file_path)
    }

    /// Get all files in index
    pub fn get_all_files(&self) -> Vec<PathBuf> {
        self.file_to_collections.keys().cloned().collect()
    }

    /// Get all collections in index
    pub fn get_all_collections(&self) -> Vec<String> {
        self.collection_to_files.keys().cloned().collect()
    }

    /// Get index statistics
    pub fn get_stats(&self) -> FileIndexStats {
        FileIndexStats {
            total_files: self.file_to_collections.len(),
            total_collections: self.collection_to_files.len(),
            total_mappings: self.file_to_collections.values().map(|v| v.len()).sum(),
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.file_to_collections.clear();
        self.collection_to_files.clear();
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// File index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndexStats {
    pub total_files: usize,
    pub total_collections: usize,
    pub total_mappings: usize,
}

/// Thread-safe file index
pub type FileIndexArc = Arc<RwLock<FileIndex>>;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_index_operations() {
        let mut index = FileIndex::new();
        let file_path = PathBuf::from("test.rs");
        let collection_name = "test-collection".to_string();

        // Add mapping
        index.add_mapping(
            file_path.clone(),
            collection_name.clone(),
            vec!["vec1".to_string(), "vec2".to_string()],
            "hash123".to_string(),
        );

        // Check file exists
        assert!(index.contains_file(&file_path));

        // Check collections for file
        let collections = index.get_collections_for_file(&file_path);
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], collection_name);

        // Check vector IDs
        let vector_ids = index.get_vector_ids(&file_path, &collection_name).unwrap();
        assert_eq!(vector_ids.len(), 2);
        assert_eq!(vector_ids[0], "vec1");
        assert_eq!(vector_ids[1], "vec2");

        // Remove mapping
        index.remove_mapping(&file_path, &collection_name);
        assert!(!index.contains_file(&file_path));

        // Check stats
        let stats = index.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_collections, 0);
        assert_eq!(stats.total_mappings, 0);
    }

    #[test]
    fn test_file_removal() {
        let mut index = FileIndex::new();
        let file_path = PathBuf::from("test.rs");
        let collection_name = "test-collection".to_string();

        // Add mapping
        index.add_mapping(
            file_path.clone(),
            collection_name.clone(),
            vec!["vec1".to_string()],
            "hash123".to_string(),
        );

        // Remove file
        let removed = index.remove_file(&file_path);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].0, collection_name);
        assert_eq!(removed[0].1, vec!["vec1".to_string()]);

        assert!(!index.contains_file(&file_path));
    }

    #[test]
    fn test_json_serialization() {
        let mut index = FileIndex::new();
        index.add_mapping(
            PathBuf::from("test.rs"),
            "test-collection".to_string(),
            vec!["vec1".to_string()],
            "hash123".to_string(),
        );

        let json = index.to_json().unwrap();
        let deserialized = FileIndex::from_json(&json).unwrap();

        assert_eq!(index.get_stats().total_files, deserialized.get_stats().total_files);
    }
}

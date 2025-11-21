//! Payload indexing for efficient payload field filtering
//!
//! This module provides indexing capabilities for payload fields to enable
//! fast filtering operations without scanning all vectors.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::Payload;

/// Payload index type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PayloadIndexType {
    /// Keyword index (exact match)
    Keyword,
    /// Text index (full-text search)
    Text,
    /// Integer index (range queries)
    Integer,
    /// Float index (range queries)
    Float,
    /// Geo index (geo-location queries)
    Geo,
}

/// Payload index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadIndexConfig {
    /// Field name to index
    pub field_name: String,
    /// Index type
    pub index_type: PayloadIndexType,
    /// Enable indexing (can be disabled for rarely queried fields)
    pub enabled: bool,
}

impl PayloadIndexConfig {
    /// Create a new payload index config
    pub fn new(field_name: String, index_type: PayloadIndexType) -> Self {
        Self {
            field_name,
            index_type,
            enabled: true,
        }
    }
}

/// Payload index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadIndexStats {
    /// Number of indexed vectors
    pub indexed_count: usize,
    /// Number of unique values
    pub unique_values: usize,
    /// Memory usage in bytes
    pub memory_bytes: usize,
}

/// Keyword index for exact match queries
#[derive(Debug, Clone)]
struct KeywordIndex {
    /// Value -> Set of vector IDs
    value_to_ids: HashMap<String, HashSet<String>>,
    /// Vector ID -> Value
    id_to_value: HashMap<String, String>,
}

impl KeywordIndex {
    fn new() -> Self {
        Self {
            value_to_ids: HashMap::new(),
            id_to_value: HashMap::new(),
        }
    }

    fn insert(&mut self, vector_id: String, value: String) {
        // Remove old value if exists
        if let Some(old_value) = self.id_to_value.get(&vector_id) {
            if let Some(ids) = self.value_to_ids.get_mut(old_value) {
                ids.remove(&vector_id);
            }
        }

        // Insert new value
        self.value_to_ids
            .entry(value.clone())
            .or_insert_with(HashSet::new)
            .insert(vector_id.clone());
        self.id_to_value.insert(vector_id, value);
    }

    fn remove(&mut self, vector_id: &str) {
        if let Some(value) = self.id_to_value.remove(vector_id) {
            if let Some(ids) = self.value_to_ids.get_mut(&value) {
                ids.remove(vector_id);
                if ids.is_empty() {
                    self.value_to_ids.remove(&value);
                }
            }
        }
    }

    fn get_ids_for_value(&self, value: &str) -> Option<&HashSet<String>> {
        self.value_to_ids.get(value)
    }

    fn stats(&self) -> PayloadIndexStats {
        PayloadIndexStats {
            indexed_count: self.id_to_value.len(),
            unique_values: self.value_to_ids.len(),
            memory_bytes: self.estimate_memory(),
        }
    }

    fn estimate_memory(&self) -> usize {
        let value_size: usize = self.value_to_ids.keys().map(|k| k.len()).sum();
        let ids_size = self.id_to_ids().len() * std::mem::size_of::<String>();
        value_size + ids_size + self.id_to_value.len() * std::mem::size_of::<String>() * 2
    }

    fn id_to_ids(&self) -> &HashMap<String, HashSet<String>> {
        &self.value_to_ids
    }
}

/// Integer index for range queries
#[derive(Debug, Clone)]
struct IntegerIndex {
    /// Vector ID -> Value
    id_to_value: HashMap<String, i64>,
}

impl IntegerIndex {
    fn new() -> Self {
        Self {
            id_to_value: HashMap::new(),
        }
    }

    fn insert(&mut self, vector_id: String, value: i64) {
        self.id_to_value.insert(vector_id, value);
    }

    fn remove(&mut self, vector_id: &str) {
        self.id_to_value.remove(vector_id);
    }

    fn get_ids_in_range(&self, min: Option<i64>, max: Option<i64>) -> HashSet<String> {
        self.id_to_value
            .iter()
            .filter_map(|(id, &value)| {
                let in_range = match (min, max) {
                    (Some(min_val), Some(max_val)) => value >= min_val && value <= max_val,
                    (Some(min_val), None) => value >= min_val,
                    (None, Some(max_val)) => value <= max_val,
                    (None, None) => true,
                };
                if in_range { Some(id.clone()) } else { None }
            })
            .collect()
    }

    fn stats(&self) -> PayloadIndexStats {
        PayloadIndexStats {
            indexed_count: self.id_to_value.len(),
            unique_values: self.id_to_value.values().collect::<HashSet<_>>().len(),
            memory_bytes: self.id_to_value.len()
                * (std::mem::size_of::<String>() + std::mem::size_of::<i64>()),
        }
    }
}

/// Float index for range queries
#[derive(Debug, Clone)]
struct FloatIndex {
    /// Vector ID -> Value
    id_to_value: HashMap<String, f64>,
}

impl FloatIndex {
    fn new() -> Self {
        Self {
            id_to_value: HashMap::new(),
        }
    }

    fn insert(&mut self, vector_id: String, value: f64) {
        self.id_to_value.insert(vector_id, value);
    }

    fn remove(&mut self, vector_id: &str) {
        self.id_to_value.remove(vector_id);
    }

    fn get_ids_in_range(&self, min: Option<f64>, max: Option<f64>) -> HashSet<String> {
        self.id_to_value
            .iter()
            .filter_map(|(id, &value)| {
                let in_range = match (min, max) {
                    (Some(min_val), Some(max_val)) => value >= min_val && value <= max_val,
                    (Some(min_val), None) => value >= min_val,
                    (None, Some(max_val)) => value <= max_val,
                    (None, None) => true,
                };
                if in_range { Some(id.clone()) } else { None }
            })
            .collect()
    }

    fn stats(&self) -> PayloadIndexStats {
        // Count unique values by rounding to avoid floating point precision issues
        let mut unique_rounded: HashSet<i64> = HashSet::new();
        for &value in self.id_to_value.values() {
            // Round to 6 decimal places for uniqueness check
            unique_rounded.insert((value * 1_000_000.0).round() as i64);
        }

        PayloadIndexStats {
            indexed_count: self.id_to_value.len(),
            unique_values: unique_rounded.len(),
            memory_bytes: self.id_to_value.len()
                * (std::mem::size_of::<String>() + std::mem::size_of::<f64>()),
        }
    }
}

/// Text index for full-text search (simple token-based)
#[derive(Debug, Clone)]
struct TextIndex {
    /// Term -> Set of vector IDs containing this term
    term_to_ids: HashMap<String, HashSet<String>>,
    /// Vector ID -> Full text
    id_to_text: HashMap<String, String>,
}

impl TextIndex {
    fn new() -> Self {
        Self {
            term_to_ids: HashMap::new(),
            id_to_text: HashMap::new(),
        }
    }

    fn insert(&mut self, vector_id: String, text: String) {
        // Remove old text if exists
        if let Some(old_text) = self.id_to_text.get(&vector_id).cloned() {
            self.remove_terms(vector_id.clone(), &old_text);
        }

        // Store full text
        self.id_to_text.insert(vector_id.clone(), text.clone());

        // Tokenize and index terms
        let terms = Self::tokenize(&text);
        for term in terms {
            self.term_to_ids
                .entry(term)
                .or_insert_with(HashSet::new)
                .insert(vector_id.clone());
        }
    }

    fn remove(&mut self, vector_id: &str) {
        if let Some(text) = self.id_to_text.remove(vector_id) {
            self.remove_terms(vector_id.to_string(), &text);
        }
    }

    fn remove_terms(&mut self, vector_id: String, text: &str) {
        let terms = Self::tokenize(text);
        for term in terms {
            if let Some(ids) = self.term_to_ids.get_mut(&term) {
                ids.remove(&vector_id);
                if ids.is_empty() {
                    self.term_to_ids.remove(&term);
                }
            }
        }
    }

    fn search(&self, query: &str) -> HashSet<String> {
        let query_terms = Self::tokenize(query);
        let mut result_ids = HashSet::new();

        for term in query_terms {
            if let Some(ids) = self.term_to_ids.get(&term) {
                if result_ids.is_empty() {
                    result_ids = ids.clone();
                } else {
                    // Intersection for AND semantics
                    result_ids = result_ids.intersection(ids).cloned().collect();
                }
            } else {
                // If any term not found, return empty (AND semantics)
                return HashSet::new();
            }
        }

        result_ids
    }

    fn tokenize(text: &str) -> Vec<String> {
        // Simple tokenization: lowercase, split on whitespace and punctuation
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    fn stats(&self) -> PayloadIndexStats {
        PayloadIndexStats {
            indexed_count: self.id_to_text.len(),
            unique_values: self.term_to_ids.len(),
            memory_bytes: self.estimate_memory(),
        }
    }

    fn estimate_memory(&self) -> usize {
        let text_size: usize = self.id_to_text.values().map(|t| t.len()).sum();
        let term_size: usize = self.term_to_ids.keys().map(|k| k.len()).sum();
        let ids_size = self
            .term_to_ids
            .values()
            .map(|ids| ids.len())
            .sum::<usize>()
            * std::mem::size_of::<String>();
        text_size + term_size + ids_size + self.id_to_text.len() * std::mem::size_of::<String>() * 2
    }
}

/// Geo index for geo-location queries
#[derive(Debug, Clone)]
struct GeoIndex {
    /// Vector ID -> (latitude, longitude)
    id_to_coords: HashMap<String, (f64, f64)>,
}

impl GeoIndex {
    fn new() -> Self {
        Self {
            id_to_coords: HashMap::new(),
        }
    }

    fn insert(&mut self, vector_id: String, lat: f64, lon: f64) {
        self.id_to_coords.insert(vector_id, (lat, lon));
    }

    fn remove(&mut self, vector_id: &str) {
        self.id_to_coords.remove(vector_id);
    }

    fn get_ids_in_bounding_box(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> HashSet<String> {
        self.id_to_coords
            .iter()
            .filter_map(|(id, &(lat, lon))| {
                if lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_ids_in_radius(
        &self,
        center_lat: f64,
        center_lon: f64,
        radius_km: f64,
    ) -> HashSet<String> {
        self.id_to_coords
            .iter()
            .filter_map(|(id, &(lat, lon))| {
                let distance_km = Self::haversine_distance(center_lat, center_lon, lat, lon);
                if distance_km <= radius_km {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        EARTH_RADIUS_KM * c
    }

    fn stats(&self) -> PayloadIndexStats {
        PayloadIndexStats {
            indexed_count: self.id_to_coords.len(),
            unique_values: self.id_to_coords.len(), // Each coordinate pair is unique
            memory_bytes: self.id_to_coords.len()
                * (std::mem::size_of::<String>() + std::mem::size_of::<(f64, f64)>()),
        }
    }
}

/// Main payload index manager
#[derive(Debug, Clone)]
pub struct PayloadIndex {
    /// Index configurations
    configs: Arc<DashMap<String, PayloadIndexConfig>>,
    /// Keyword indexes (field_name -> index)
    keyword_indexes: Arc<DashMap<String, KeywordIndex>>,
    /// Integer indexes (field_name -> index)
    integer_indexes: Arc<DashMap<String, IntegerIndex>>,
    /// Float indexes (field_name -> index)
    float_indexes: Arc<DashMap<String, FloatIndex>>,
    /// Text indexes (field_name -> index)
    text_indexes: Arc<DashMap<String, TextIndex>>,
    /// Geo indexes (field_name -> index)
    geo_indexes: Arc<DashMap<String, GeoIndex>>,
}

impl PayloadIndex {
    /// Create a new payload index
    pub fn new() -> Self {
        Self {
            configs: Arc::new(DashMap::new()),
            keyword_indexes: Arc::new(DashMap::new()),
            integer_indexes: Arc::new(DashMap::new()),
            float_indexes: Arc::new(DashMap::new()),
            text_indexes: Arc::new(DashMap::new()),
            geo_indexes: Arc::new(DashMap::new()),
        }
    }

    /// Add index configuration
    pub fn add_index_config(&self, config: PayloadIndexConfig) {
        let field_name = config.field_name.clone();
        let index_type = config.index_type;
        self.configs.insert(field_name.clone(), config);

        // Initialize index based on type
        match index_type {
            PayloadIndexType::Keyword => {
                self.keyword_indexes
                    .entry(field_name.clone())
                    .or_insert_with(KeywordIndex::new);
            }
            PayloadIndexType::Integer => {
                self.integer_indexes
                    .entry(field_name.clone())
                    .or_insert_with(IntegerIndex::new);
            }
            PayloadIndexType::Float => {
                self.float_indexes
                    .entry(field_name.clone())
                    .or_insert_with(FloatIndex::new);
            }
            PayloadIndexType::Text => {
                self.text_indexes
                    .entry(field_name.clone())
                    .or_insert_with(TextIndex::new);
            }
            PayloadIndexType::Geo => {
                self.geo_indexes
                    .entry(field_name.clone())
                    .or_insert_with(GeoIndex::new);
            }
        }
    }

    /// Get nested value from payload using dot notation (e.g., "user.age")
    fn get_nested_value<'a>(&self, payload: &'a Payload, key: &str) -> Option<&'a Value> {
        let keys: Vec<&str> = key.split('.').collect();
        let mut current = &payload.data;

        for k in keys {
            match current {
                Value::Object(map) => {
                    current = map.get(k)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Index a vector's payload
    pub fn index_vector(&self, vector_id: String, payload: &Payload) {
        // Index all configured fields (including nested fields)
        for config_entry in self.configs.iter() {
            let field_name = config_entry.key();
            let config = config_entry.value();

            if !config.enabled {
                continue;
            }

            // Get value (supports nested fields via dot notation)
            let value = if field_name.contains('.') {
                // Nested field
                self.get_nested_value(payload, field_name)
            } else {
                // Top-level field
                payload.data.as_object().and_then(|obj| obj.get(field_name))
            };

            let value = match value {
                Some(v) => v,
                None => continue,
            };

            match config.index_type {
                PayloadIndexType::Keyword => {
                    if let Some(mut keyword_index) = self.keyword_indexes.get_mut(field_name) {
                        if let Some(value_str) = value.as_str() {
                            keyword_index.insert(vector_id.clone(), value_str.to_string());
                        } else if let Some(value_num) = value.as_i64() {
                            keyword_index.insert(vector_id.clone(), value_num.to_string());
                        } else if let Some(value_bool) = value.as_bool() {
                            keyword_index.insert(vector_id.clone(), value_bool.to_string());
                        }
                    }
                }
                PayloadIndexType::Integer => {
                    if let Some(mut integer_index) = self.integer_indexes.get_mut(field_name) {
                        if let Some(value_num) = value.as_i64() {
                            integer_index.insert(vector_id.clone(), value_num);
                        } else if let Some(value_float) = value.as_f64() {
                            integer_index.insert(vector_id.clone(), value_float as i64);
                        }
                    }
                }
                PayloadIndexType::Float => {
                    if let Some(mut float_index) = self.float_indexes.get_mut(field_name) {
                        if let Some(value_float) = value.as_f64() {
                            float_index.insert(vector_id.clone(), value_float);
                        } else if let Some(value_num) = value.as_i64() {
                            float_index.insert(vector_id.clone(), value_num as f64);
                        }
                    }
                }
                PayloadIndexType::Text => {
                    if let Some(mut text_index) = self.text_indexes.get_mut(field_name) {
                        if let Some(value_str) = value.as_str() {
                            text_index.insert(vector_id.clone(), value_str.to_string());
                        }
                    }
                }
                PayloadIndexType::Geo => {
                    if let Some(mut geo_index) = self.geo_indexes.get_mut(field_name) {
                        // Support both object format {"lat": x, "lon": y} and array format [lat, lon]
                        if let Some(obj) = value.as_object() {
                            if let (Some(lat), Some(lon)) = (
                                obj.get("lat").and_then(|v| v.as_f64()),
                                obj.get("lon").and_then(|v| v.as_f64()),
                            ) {
                                geo_index.insert(vector_id.clone(), lat, lon);
                            }
                        } else if let Some(arr) = value.as_array() {
                            if arr.len() == 2 {
                                if let (Some(lat), Some(lon)) = (arr[0].as_f64(), arr[1].as_f64()) {
                                    geo_index.insert(vector_id.clone(), lat, lon);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Remove vector from indexes
    pub fn remove_vector(&self, vector_id: &str) {
        // Remove from all keyword indexes
        for mut keyword_index in self.keyword_indexes.iter_mut() {
            keyword_index.remove(vector_id);
        }

        // Remove from all integer indexes
        for mut integer_index in self.integer_indexes.iter_mut() {
            integer_index.remove(vector_id);
        }

        // Remove from all float indexes
        for mut float_index in self.float_indexes.iter_mut() {
            float_index.remove(vector_id);
        }

        // Remove from all text indexes
        for mut text_index in self.text_indexes.iter_mut() {
            text_index.remove(vector_id);
        }

        // Remove from all geo indexes
        for mut geo_index in self.geo_indexes.iter_mut() {
            geo_index.remove(vector_id);
        }
    }

    /// Get vector IDs matching a keyword value
    pub fn get_ids_for_keyword(&self, field_name: &str, value: &str) -> Option<HashSet<String>> {
        self.keyword_indexes
            .get(field_name)
            .and_then(|index| index.get_ids_for_value(value).cloned())
    }

    /// Get vector IDs in integer range
    pub fn get_ids_in_range(
        &self,
        field_name: &str,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Option<HashSet<String>> {
        self.integer_indexes
            .get(field_name)
            .map(|index| index.get_ids_in_range(min, max))
    }

    /// Get vector IDs in float range
    pub fn get_ids_in_float_range(
        &self,
        field_name: &str,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Option<HashSet<String>> {
        self.float_indexes
            .get(field_name)
            .map(|index| index.get_ids_in_range(min, max))
    }

    /// Search text index for matching vectors
    pub fn search_text(&self, field_name: &str, query: &str) -> Option<HashSet<String>> {
        self.text_indexes
            .get(field_name)
            .map(|index| index.search(query))
    }

    /// Get vector IDs in geo bounding box
    pub fn get_ids_in_geo_bounding_box(
        &self,
        field_name: &str,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> Option<HashSet<String>> {
        self.geo_indexes
            .get(field_name)
            .map(|index| index.get_ids_in_bounding_box(min_lat, max_lat, min_lon, max_lon))
    }

    /// Get vector IDs in geo radius
    pub fn get_ids_in_geo_radius(
        &self,
        field_name: &str,
        center_lat: f64,
        center_lon: f64,
        radius_km: f64,
    ) -> Option<HashSet<String>> {
        self.geo_indexes
            .get(field_name)
            .map(|index| index.get_ids_in_radius(center_lat, center_lon, radius_km))
    }

    /// Get statistics for all indexes
    pub fn get_stats(&self) -> HashMap<String, PayloadIndexStats> {
        let mut stats = HashMap::new();

        for entry in self.keyword_indexes.iter() {
            stats.insert(entry.key().clone(), entry.value().stats());
        }

        for entry in self.integer_indexes.iter() {
            stats.insert(entry.key().clone(), entry.value().stats());
        }

        for entry in self.float_indexes.iter() {
            stats.insert(entry.key().clone(), entry.value().stats());
        }

        for entry in self.text_indexes.iter() {
            stats.insert(entry.key().clone(), entry.value().stats());
        }

        for entry in self.geo_indexes.iter() {
            stats.insert(entry.key().clone(), entry.value().stats());
        }

        stats
    }

    /// Get index configuration
    pub fn get_config(&self, field_name: &str) -> Option<PayloadIndexConfig> {
        self.configs.get(field_name).map(|e| e.value().clone())
    }

    /// List all indexed fields
    pub fn list_indexed_fields(&self) -> Vec<String> {
        self.configs.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for PayloadIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::models::Payload;

    #[test]
    fn test_keyword_index() {
        let index = PayloadIndex::new();

        // Add index config
        let config = PayloadIndexConfig::new("status".to_string(), PayloadIndexType::Keyword);
        index.add_index_config(config);

        // Index vectors
        let payload1 = Payload {
            data: json!({"status": "active", "name": "test1"}),
        };
        index.index_vector("v1".to_string(), &payload1);

        let payload2 = Payload {
            data: json!({"status": "inactive", "name": "test2"}),
        };
        index.index_vector("v2".to_string(), &payload2);

        // Query
        let ids = index.get_ids_for_keyword("status", "active").unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("v1"));

        // Remove vector
        index.remove_vector("v1");
        let ids = index.get_ids_for_keyword("status", "active");
        assert!(ids.is_none() || ids.unwrap().is_empty());
    }

    #[test]
    fn test_integer_index() {
        let index = PayloadIndex::new();

        // Add index config
        let config = PayloadIndexConfig::new("age".to_string(), PayloadIndexType::Integer);
        index.add_index_config(config);

        // Index vectors
        let payload1 = Payload {
            data: json!({"age": 25, "name": "test1"}),
        };
        index.index_vector("v1".to_string(), &payload1);

        let payload2 = Payload {
            data: json!({"age": 30, "name": "test2"}),
        };
        index.index_vector("v2".to_string(), &payload2);

        let payload3 = Payload {
            data: json!({"age": 35, "name": "test3"}),
        };
        index.index_vector("v3".to_string(), &payload3);

        // Query range
        let ids = index.get_ids_in_range("age", Some(25), Some(30)).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("v1"));
        assert!(ids.contains("v2"));
    }

    #[test]
    fn test_index_stats() {
        let index = PayloadIndex::new();

        let config = PayloadIndexConfig::new("status".to_string(), PayloadIndexType::Keyword);
        index.add_index_config(config);

        let payload = Payload {
            data: json!({"status": "active"}),
        };
        index.index_vector("v1".to_string(), &payload);

        let stats = index.get_stats();
        assert!(stats.contains_key("status"));
        assert_eq!(stats["status"].indexed_count, 1);
    }

    #[test]
    fn test_float_index() {
        let index = PayloadIndex::new();

        let config = PayloadIndexConfig::new("price".to_string(), PayloadIndexType::Float);
        index.add_index_config(config);

        let payload1 = Payload {
            data: json!({"price": 19.99, "name": "item1"}),
        };
        index.index_vector("v1".to_string(), &payload1);

        let payload2 = Payload {
            data: json!({"price": 29.99, "name": "item2"}),
        };
        index.index_vector("v2".to_string(), &payload2);

        let payload3 = Payload {
            data: json!({"price": 39.99, "name": "item3"}),
        };
        index.index_vector("v3".to_string(), &payload3);

        // Query range
        let ids = index
            .get_ids_in_float_range("price", Some(20.0), Some(35.0))
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("v2"));
    }

    #[test]
    fn test_text_index() {
        let index = PayloadIndex::new();

        let config = PayloadIndexConfig::new("description".to_string(), PayloadIndexType::Text);
        index.add_index_config(config);

        let payload1 = Payload {
            data: json!({"description": "rust programming language tutorial"}),
        };
        index.index_vector("v1".to_string(), &payload1);

        let payload2 = Payload {
            data: json!({"description": "python programming tutorial"}),
        };
        index.index_vector("v2".to_string(), &payload2);

        let payload3 = Payload {
            data: json!({"description": "rust and python comparison"}),
        };
        index.index_vector("v3".to_string(), &payload3);

        // Search for "rust"
        let ids = index.search_text("description", "rust").unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("v1"));
        assert!(ids.contains("v3"));

        // Search for "rust tutorial"
        let ids = index.search_text("description", "rust tutorial").unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("v1"));
    }

    #[test]
    fn test_geo_index() {
        let index = PayloadIndex::new();

        let config = PayloadIndexConfig::new("location".to_string(), PayloadIndexType::Geo);
        index.add_index_config(config);

        // São Paulo, Brazil
        let payload1 = Payload {
            data: json!({"location": {"lat": -23.5505, "lon": -46.6333}}),
        };
        index.index_vector("v1".to_string(), &payload1);

        // Rio de Janeiro, Brazil
        let payload2 = Payload {
            data: json!({"location": {"lat": -22.9068, "lon": -43.1729}}),
        };
        index.index_vector("v2".to_string(), &payload2);

        // New York, USA
        let payload3 = Payload {
            data: json!({"location": [-74.0060, 40.7128]}), // Array format
        };
        index.index_vector("v3".to_string(), &payload3);

        // Query bounding box (Brazil region)
        let ids = index
            .get_ids_in_geo_bounding_box(
                "location", -25.0, // min_lat
                -20.0, // max_lat
                -50.0, // min_lon
                -40.0, // max_lon
            )
            .unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("v1"));
        assert!(ids.contains("v2"));

        // Query radius (1000km from São Paulo)
        let ids = index
            .get_ids_in_geo_radius(
                "location", -23.5505, // center_lat (São Paulo)
                -46.6333, // center_lon
                1000.0,   // radius_km
            )
            .unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("v1"));
        assert!(ids.contains("v2"));
    }

    #[test]
    fn test_nested_field_indexing() {
        let index = PayloadIndex::new();

        // Index nested field "user.age"
        let config = PayloadIndexConfig::new("user.age".to_string(), PayloadIndexType::Integer);
        index.add_index_config(config);

        // Index nested field "metadata.price"
        let config = PayloadIndexConfig::new("metadata.price".to_string(), PayloadIndexType::Float);
        index.add_index_config(config);

        // Index nested field "user.name"
        let config = PayloadIndexConfig::new("user.name".to_string(), PayloadIndexType::Keyword);
        index.add_index_config(config);

        // Index vectors with nested payloads
        let payload1 = Payload {
            data: json!({
                "user": {
                    "name": "Alice",
                    "age": 25
                },
                "metadata": {
                    "price": 19.99
                }
            }),
        };
        index.index_vector("v1".to_string(), &payload1);

        let payload2 = Payload {
            data: json!({
                "user": {
                    "name": "Bob",
                    "age": 30
                },
                "metadata": {
                    "price": 29.99
                }
            }),
        };
        index.index_vector("v2".to_string(), &payload2);

        // Query nested integer field
        let ids = index
            .get_ids_in_range("user.age", Some(25), Some(30))
            .unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("v1"));
        assert!(ids.contains("v2"));

        // Query nested float field
        let ids = index
            .get_ids_in_float_range("metadata.price", Some(20.0), Some(30.0))
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("v2"));

        // Query nested keyword field
        let ids = index.get_ids_for_keyword("user.name", "Alice").unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("v1"));
    }
}

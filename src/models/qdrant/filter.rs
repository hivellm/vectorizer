//! Qdrant filter models

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Qdrant filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantFilter {
    /// Must conditions
    pub must: Option<Vec<QdrantCondition>>,
    /// Should conditions
    pub should: Option<Vec<QdrantCondition>>,
    /// Must not conditions
    pub must_not: Option<Vec<QdrantCondition>>,
}

/// Qdrant condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QdrantCondition {
    /// Match condition
    Match {
        /// Key to match
        key: String,
        /// Match value
        match_value: QdrantMatchValue,
    },
    /// Range condition
    Range {
        /// Key to range
        key: String,
        /// Range parameters
        range: QdrantRange,
    },
    /// Geo bounding box condition
    GeoBoundingBox {
        /// Key to geo
        key: String,
        /// Bounding box
        geo_bounding_box: QdrantGeoBoundingBox,
    },
    /// Geo radius condition
    GeoRadius {
        /// Key to geo
        key: String,
        /// Geo radius
        geo_radius: QdrantGeoRadius,
    },
    /// Values count condition
    ValuesCount {
        /// Key to count
        key: String,
        /// Values count
        values_count: QdrantValuesCount,
    },
    /// Nested filter
    Nested {
        /// Nested filter
        filter: Box<QdrantFilter>,
    },
}

/// Qdrant match value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantMatchValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Bool(bool),
    /// Any value
    Any,
    /// Text match
    Text(QdrantTextMatch),
}

/// Qdrant text match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantTextMatch {
    /// Text to match
    pub text: String,
    /// Type of text match
    #[serde(rename = "type")]
    pub match_type: QdrantTextMatchType,
}

/// Qdrant text match types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantTextMatchType {
    /// Exact match
    Exact,
    /// Prefix match
    Prefix,
    /// Suffix match
    Suffix,
    /// Contains match
    Contains,
}

/// Qdrant range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRange {
    /// Greater than
    pub gt: Option<f64>,
    /// Greater than or equal
    pub gte: Option<f64>,
    /// Less than
    pub lt: Option<f64>,
    /// Less than or equal
    pub lte: Option<f64>,
}

/// Qdrant geo bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGeoBoundingBox {
    /// Top right corner
    pub top_right: QdrantGeoPoint,
    /// Bottom left corner
    pub bottom_left: QdrantGeoPoint,
}

/// Qdrant geo radius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGeoRadius {
    /// Center point
    pub center: QdrantGeoPoint,
    /// Radius in meters
    pub radius: f64,
}

/// Qdrant geo point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGeoPoint {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
}

/// Qdrant values count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantValuesCount {
    /// Greater than
    pub gt: Option<u32>,
    /// Greater than or equal
    pub gte: Option<u32>,
    /// Less than
    pub lt: Option<u32>,
    /// Less than or equal
    pub lte: Option<u32>,
}

/// Qdrant filter builder
#[derive(Debug, Clone)]
pub struct QdrantFilterBuilder {
    must: Vec<QdrantCondition>,
    should: Vec<QdrantCondition>,
    must_not: Vec<QdrantCondition>,
}

impl QdrantFilterBuilder {
    /// Create a new filter builder
    pub fn new() -> Self {
        Self {
            must: Vec::new(),
            should: Vec::new(),
            must_not: Vec::new(),
        }
    }

    /// Add a must condition
    pub fn must(mut self, condition: QdrantCondition) -> Self {
        self.must.push(condition);
        self
    }

    /// Add a should condition
    pub fn should(mut self, condition: QdrantCondition) -> Self {
        self.should.push(condition);
        self
    }

    /// Add a must not condition
    pub fn must_not(mut self, condition: QdrantCondition) -> Self {
        self.must_not.push(condition);
        self
    }

    /// Build the filter
    pub fn build(self) -> QdrantFilter {
        QdrantFilter {
            must: if self.must.is_empty() {
                None
            } else {
                Some(self.must)
            },
            should: if self.should.is_empty() {
                None
            } else {
                Some(self.should)
            },
            must_not: if self.must_not.is_empty() {
                None
            } else {
                Some(self.must_not)
            },
        }
    }
}

impl Default for QdrantFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common conditions
impl QdrantCondition {
    /// Create a string match condition
    pub fn match_string(key: &str, value: &str) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::String(value.to_string()),
        }
    }

    /// Create an integer match condition
    pub fn match_integer(key: &str, value: i64) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::Integer(value),
        }
    }

    /// Create a boolean match condition
    pub fn match_bool(key: &str, value: bool) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::Bool(value),
        }
    }

    /// Create an any match condition
    pub fn match_any(key: &str) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::Any,
        }
    }

    /// Create a text match condition
    pub fn match_text(key: &str, text: &str, match_type: QdrantTextMatchType) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::Text(QdrantTextMatch {
                text: text.to_string(),
                match_type,
            }),
        }
    }

    /// Create a range condition
    pub fn range(key: &str, range: QdrantRange) -> Self {
        Self::Range {
            key: key.to_string(),
            range,
        }
    }

    /// Create a geo bounding box condition
    pub fn geo_bounding_box(
        key: &str,
        top_right: QdrantGeoPoint,
        bottom_left: QdrantGeoPoint,
    ) -> Self {
        Self::GeoBoundingBox {
            key: key.to_string(),
            geo_bounding_box: QdrantGeoBoundingBox {
                top_right,
                bottom_left,
            },
        }
    }

    /// Create a geo radius condition
    pub fn geo_radius(key: &str, center: QdrantGeoPoint, radius: f64) -> Self {
        Self::GeoRadius {
            key: key.to_string(),
            geo_radius: QdrantGeoRadius { center, radius },
        }
    }

    /// Create a values count condition
    pub fn values_count(key: &str, values_count: QdrantValuesCount) -> Self {
        Self::ValuesCount {
            key: key.to_string(),
            values_count,
        }
    }

    /// Create a nested filter condition
    pub fn nested(filter: QdrantFilter) -> Self {
        Self::Nested {
            filter: Box::new(filter),
        }
    }
}

/// Helper functions for creating common ranges
impl QdrantRange {
    /// Create a range with greater than
    pub fn gt(value: f64) -> Self {
        Self {
            gt: Some(value),
            gte: None,
            lt: None,
            lte: None,
        }
    }

    /// Create a range with greater than or equal
    pub fn gte(value: f64) -> Self {
        Self {
            gt: None,
            gte: Some(value),
            lt: None,
            lte: None,
        }
    }

    /// Create a range with less than
    pub fn lt(value: f64) -> Self {
        Self {
            gt: None,
            gte: None,
            lt: Some(value),
            lte: None,
        }
    }

    /// Create a range with less than or equal
    pub fn lte(value: f64) -> Self {
        Self {
            gt: None,
            gte: None,
            lt: None,
            lte: Some(value),
        }
    }

    /// Create a range between two values
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            gt: None,
            gte: Some(min),
            lt: Some(max),
            lte: None,
        }
    }

    /// Create a range between two values (inclusive)
    pub fn between_inclusive(min: f64, max: f64) -> Self {
        Self {
            gt: None,
            gte: Some(min),
            lt: None,
            lte: Some(max),
        }
    }
}

/// Helper functions for creating common values counts
impl QdrantValuesCount {
    /// Create a values count with greater than
    pub fn gt(value: u32) -> Self {
        Self {
            gt: Some(value),
            gte: None,
            lt: None,
            lte: None,
        }
    }

    /// Create a values count with greater than or equal
    pub fn gte(value: u32) -> Self {
        Self {
            gt: None,
            gte: Some(value),
            lt: None,
            lte: None,
        }
    }

    /// Create a values count with less than
    pub fn lt(value: u32) -> Self {
        Self {
            gt: None,
            gte: None,
            lt: Some(value),
            lte: None,
        }
    }

    /// Create a values count with less than or equal
    pub fn lte(value: u32) -> Self {
        Self {
            gt: None,
            gte: None,
            lt: None,
            lte: Some(value),
        }
    }

    /// Create a values count between two values
    pub fn between(min: u32, max: u32) -> Self {
        Self {
            gt: None,
            gte: Some(min),
            lt: Some(max),
            lte: None,
        }
    }

    /// Create a values count between two values (inclusive)
    pub fn between_inclusive(min: u32, max: u32) -> Self {
        Self {
            gt: None,
            gte: Some(min),
            lt: None,
            lte: Some(max),
        }
    }
}

/// Helper functions for creating geo points
impl QdrantGeoPoint {
    /// Create a geo point
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

//! Typed Qdrant-compatible filter builder for `delete_by_filter` and
//! `bulk_update_metadata`.
//!
//! Both tier-control endpoints accept a `filter` body field whose wire shape
//! is a **Qdrant-style** filter with three optional boolean clauses:
//!
//! ```json
//! {
//!   "must":     [ <condition>, ... ],
//!   "should":   [ <condition>, ... ],
//!   "must_not": [ <condition>, ... ]
//! }
//! ```
//!
//! See `docs/users/api/API_REFERENCE.md § Filter shape` for the full
//! reference with all condition types, error responses, and common mistakes.
//!
//! # Quick start
//!
//! ```rust
//! use vectorizer_sdk::models::filter::{QdrantFilter, QdrantCondition};
//!
//! // Match all vectors where `topic == "index"`:
//! let filter = QdrantFilter::must(vec![
//!     QdrantCondition::match_string("topic", "index"),
//! ]);
//!
//! // Match where `tier == "hot"` AND `score >= 0.8`:
//! let filter = QdrantFilter::must(vec![
//!     QdrantCondition::match_string("tier", "hot"),
//!     QdrantCondition::range("score", QdrantRange { gte: Some(0.8), ..Default::default() }),
//! ]);
//!
//! // Serialises to the wire shape the server expects:
//! let json = serde_json::to_value(&filter).unwrap();
//! ```

use serde::{Deserialize, Serialize};

// ───────────────────────────────────────── top-level filter ──────────────────

/// Top-level Qdrant-style filter accepted by `delete_by_filter` and
/// `bulk_update_metadata`.
///
/// All three clause arrays are optional; omit any you don't need. At least
/// one clause with at least one condition must be present — the server rejects
/// an all-absent filter with `400 validation_error` (message: "filter has no
/// conditions").
///
/// # Wire shape
///
/// ```json
/// { "must": [...], "should": [...], "must_not": [...] }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantFilter {
    /// All conditions must be true (AND semantics).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must: Option<Vec<QdrantCondition>>,
    /// At least one condition must be true (OR semantics).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub should: Option<Vec<QdrantCondition>>,
    /// All conditions must be false (NOT semantics).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_not: Option<Vec<QdrantCondition>>,
}

impl QdrantFilter {
    /// Build a filter that requires all `conditions` to be true.
    pub fn must(conditions: Vec<QdrantCondition>) -> Self {
        Self {
            must: Some(conditions),
            should: None,
            must_not: None,
        }
    }

    /// Build a filter that requires at least one of `conditions` to be true.
    pub fn should(conditions: Vec<QdrantCondition>) -> Self {
        Self {
            must: None,
            should: Some(conditions),
            must_not: None,
        }
    }

    /// Build a filter that requires all `conditions` to be false.
    pub fn must_not(conditions: Vec<QdrantCondition>) -> Self {
        Self {
            must: None,
            should: None,
            must_not: Some(conditions),
        }
    }
}

// ───────────────────────────────────────── conditions ────────────────────────

/// A single filter condition. The `"type"` JSON discriminant is required by
/// the server; the `#[serde(tag = "type")]` attribute serialises it
/// automatically so callers never need to set it manually.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QdrantCondition {
    /// Exact value match on a payload field.
    ///
    /// ```json
    /// { "type": "match", "key": "topic", "match_value": "index" }
    /// ```
    Match {
        /// Payload field path (dot-separated for nested fields).
        key: String,
        /// Value to match. Accepts string, integer, or boolean.
        match_value: QdrantMatchValue,
    },
    /// Numeric range check on a payload field.
    ///
    /// ```json
    /// { "type": "range", "key": "score", "range": { "gte": 0.5, "lt": 0.9 } }
    /// ```
    Range {
        /// Payload field path.
        key: String,
        /// Range bounds (all optional, combined with AND).
        range: QdrantRange,
    },
    /// Cardinality check on an array-valued payload field.
    ///
    /// ```json
    /// { "type": "values_count", "key": "tags", "values_count": { "gte": 1 } }
    /// ```
    ValuesCount {
        /// Payload field path.
        key: String,
        /// Element-count bounds.
        values_count: QdrantValuesCount,
    },
    /// Geospatial bounding-box check.
    GeoBoundingBox {
        /// Payload field path (must hold `{lat, lon}`).
        key: String,
        /// Bounding box corners.
        geo_bounding_box: QdrantGeoBoundingBox,
    },
    /// Geospatial radius check.
    GeoRadius {
        /// Payload field path (must hold `{lat, lon}`).
        key: String,
        /// Centre + radius in metres.
        geo_radius: QdrantGeoRadius,
    },
    /// Apply a sub-filter to a nested object payload field.
    Nested {
        /// Nested Qdrant filter applied to the value at the target path.
        filter: Box<QdrantFilter>,
    },
}

impl QdrantCondition {
    /// Create an exact string-match condition.
    pub fn match_string(key: &str, value: &str) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::String(value.to_string()),
        }
    }

    /// Create an exact integer-match condition.
    pub fn match_integer(key: &str, value: i64) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::Integer(value),
        }
    }

    /// Create an exact boolean-match condition.
    pub fn match_bool(key: &str, value: bool) -> Self {
        Self::Match {
            key: key.to_string(),
            match_value: QdrantMatchValue::Bool(value),
        }
    }

    /// Create a numeric range condition.
    pub fn range(key: &str, range: QdrantRange) -> Self {
        Self::Range {
            key: key.to_string(),
            range,
        }
    }

    /// Create a values-count condition.
    pub fn values_count(key: &str, values_count: QdrantValuesCount) -> Self {
        Self::ValuesCount {
            key: key.to_string(),
            values_count,
        }
    }

    /// Create a nested sub-filter condition.
    pub fn nested(filter: QdrantFilter) -> Self {
        Self::Nested {
            filter: Box::new(filter),
        }
    }
}

// ───────────────────────────────────────── value types ───────────────────────

/// Match value — string, integer, or boolean.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantMatchValue {
    /// String value.
    String(String),
    /// Integer value.
    Integer(i64),
    /// Boolean value.
    Bool(bool),
}

/// Numeric range bounds (all optional, combined with AND).
///
/// Example — `score` between 0.5 (inclusive) and 0.9 (exclusive):
/// ```rust
/// use vectorizer_sdk::models::filter::QdrantRange;
/// let r = QdrantRange { gte: Some(0.5), lt: Some(0.9), ..Default::default() };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantRange {
    /// Greater than.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<f64>,
    /// Greater than or equal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<f64>,
    /// Less than.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<f64>,
    /// Less than or equal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<f64>,
}

impl QdrantRange {
    /// Greater-than-or-equal lower bound only.
    pub fn gte(value: f64) -> Self {
        Self {
            gte: Some(value),
            ..Default::default()
        }
    }

    /// Less-than upper bound only.
    pub fn lt(value: f64) -> Self {
        Self {
            lt: Some(value),
            ..Default::default()
        }
    }

    /// Closed `[min, max]` interval.
    pub fn between_inclusive(min: f64, max: f64) -> Self {
        Self {
            gte: Some(min),
            lte: Some(max),
            ..Default::default()
        }
    }
}

/// Array element-count bounds (unsigned integers).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantValuesCount {
    /// Greater than.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<u32>,
    /// Greater than or equal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<u32>,
    /// Less than.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<u32>,
    /// Less than or equal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<u32>,
}

/// Geospatial bounding box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGeoBoundingBox {
    /// Top-right corner.
    pub top_right: QdrantGeoPoint,
    /// Bottom-left corner.
    pub bottom_left: QdrantGeoPoint,
}

/// Geospatial radius.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGeoRadius {
    /// Centre point.
    pub center: QdrantGeoPoint,
    /// Radius in metres.
    pub radius: f64,
}

/// Geographic coordinate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGeoPoint {
    /// Latitude.
    pub lat: f64,
    /// Longitude.
    pub lon: f64,
}

impl QdrantGeoPoint {
    /// Construct a geo point.
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

// ───────────────────────────────────────── unit tests ────────────────────────

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    // Verify the basic string-match condition serialises to the shape the
    // server expects: `{"type":"match","key":"...","match_value":"..."}`.
    #[test]
    fn match_string_wire_shape() {
        let cond = QdrantCondition::match_string("topic", "index");
        let v = serde_json::to_value(&cond).unwrap();
        assert_eq!(v["type"], "match");
        assert_eq!(v["key"], "topic");
        assert_eq!(v["match_value"], "index");
    }

    // Verify the integer-match condition serialises correctly.
    #[test]
    fn match_integer_wire_shape() {
        let cond = QdrantCondition::match_integer("count", 42);
        let v = serde_json::to_value(&cond).unwrap();
        assert_eq!(v["type"], "match");
        assert_eq!(v["key"], "count");
        assert_eq!(v["match_value"], 42);
    }

    // Verify a `range` condition serialises with `"type":"range"` and a
    // `"range"` sub-object containing only the supplied bounds.
    #[test]
    fn range_condition_wire_shape() {
        let cond = QdrantCondition::range(
            "score",
            QdrantRange {
                gte: Some(0.5),
                lt: Some(0.9),
                ..Default::default()
            },
        );
        let v = serde_json::to_value(&cond).unwrap();
        assert_eq!(v["type"], "range");
        assert_eq!(v["key"], "score");
        assert_eq!(v["range"]["gte"], 0.5);
        assert_eq!(v["range"]["lt"], 0.9);
        // Fields not set must be absent from the serialised object
        assert!(v["range"].get("gt").is_none() || v["range"]["gt"].is_null());
        assert!(v["range"].get("lte").is_none() || v["range"]["lte"].is_null());
    }

    // Verify the top-level `QdrantFilter::must` shorthand omits the absent
    // `should` and `must_not` keys (they're skipped when None).
    #[test]
    fn filter_must_omits_absent_clauses() {
        let filter = QdrantFilter::must(vec![QdrantCondition::match_string("topic", "index")]);
        let v = serde_json::to_value(&filter).unwrap();
        assert!(v.get("must").is_some());
        assert!(v.get("should").is_none());
        assert!(v.get("must_not").is_none());
    }

    // Verify a compound AND filter with two conditions round-trips through serde.
    #[test]
    fn compound_and_filter_round_trips() {
        let filter = QdrantFilter::must(vec![
            QdrantCondition::match_string("tier", "hot"),
            QdrantCondition::range("score", QdrantRange::gte(0.8)),
        ]);
        let serialised = serde_json::to_string(&filter).unwrap();
        let deserialised: QdrantFilter = serde_json::from_str(&serialised).unwrap();

        let must = deserialised.must.unwrap();
        assert_eq!(must.len(), 2);
        match &must[0] {
            QdrantCondition::Match {
                key,
                match_value: QdrantMatchValue::String(v),
            } => {
                assert_eq!(key, "tier");
                assert_eq!(v, "hot");
            }
            other => panic!("unexpected condition: {other:?}"),
        }
        match &must[1] {
            QdrantCondition::Range { key, range } => {
                assert_eq!(key, "score");
                assert_eq!(range.gte, Some(0.8));
            }
            other => panic!("unexpected condition: {other:?}"),
        }
    }

    // Verify nested filter serialises with `"type":"nested"` and a `"filter"`
    // sub-object.
    #[test]
    fn nested_filter_wire_shape() {
        let inner = QdrantFilter::must(vec![QdrantCondition::match_string("inner_key", "value")]);
        let outer = QdrantFilter::must(vec![QdrantCondition::nested(inner)]);
        let v = serde_json::to_value(&outer).unwrap();
        let nested_cond = &v["must"][0];
        assert_eq!(nested_cond["type"], "nested");
        assert!(nested_cond.get("filter").is_some());
        assert_eq!(nested_cond["filter"]["must"][0]["key"], "inner_key");
    }

    // Verify that `QdrantFilter` serialised as the `"filter"` field of a
    // request body produces the correct top-level shape — this is the actual
    // wire format sent to the server by `delete_by_filter`.
    #[test]
    fn filter_wrapped_in_request_body() {
        let filter = QdrantFilter::must(vec![QdrantCondition::match_string("topic", "index")]);
        let body = json!({ "filter": filter });
        let filter_obj = &body["filter"];
        assert!(filter_obj.get("must").is_some());
        assert_eq!(filter_obj["must"][0]["type"], "match");
        assert_eq!(filter_obj["must"][0]["key"], "topic");
        assert_eq!(filter_obj["must"][0]["match_value"], "index");
    }
}

//! Filter processor for applying Qdrant filters to search results

use std::collections::HashMap;

use serde_json::Value;

use super::filter::{
    QdrantCondition, QdrantFilter, QdrantGeoPoint, QdrantMatchValue, QdrantRange, QdrantValuesCount,
};
use crate::models::Payload;

/// Result of filter evaluation
pub type FilterResult = bool;

/// Filter processor for applying Qdrant filters
pub struct FilterProcessor;

impl FilterProcessor {
    /// Apply filter to a payload
    pub fn apply_filter(filter: &QdrantFilter, payload: &Payload) -> FilterResult {
        let mut result = true;

        // Apply MUST conditions (AND logic)
        if let Some(must_conditions) = &filter.must {
            result &= must_conditions
                .iter()
                .all(|condition| Self::evaluate_condition(condition, payload));
        }

        // Apply MUST NOT conditions (NOT logic)
        if let Some(must_not_conditions) = &filter.must_not {
            result &= must_not_conditions
                .iter()
                .all(|condition| !Self::evaluate_condition(condition, payload));
        }

        // Apply SHOULD conditions (OR logic)
        if let Some(should_conditions) = &filter.should {
            if !should_conditions.is_empty() {
                result &= should_conditions
                    .iter()
                    .any(|condition| Self::evaluate_condition(condition, payload));
            }
        }

        result
    }

    /// Evaluate a single condition
    fn evaluate_condition(condition: &QdrantCondition, payload: &Payload) -> bool {
        match condition {
            QdrantCondition::Match { key, match_value } => {
                Self::evaluate_match(key, match_value, payload)
            }
            QdrantCondition::Range { key, range } => Self::evaluate_range(key, range, payload),
            QdrantCondition::GeoBoundingBox {
                key,
                geo_bounding_box,
            } => Self::evaluate_geo_bounding_box(key, geo_bounding_box, payload),
            QdrantCondition::GeoRadius { key, geo_radius } => {
                Self::evaluate_geo_radius(key, geo_radius, payload)
            }
            QdrantCondition::ValuesCount { key, values_count } => {
                Self::evaluate_values_count(key, values_count, payload)
            }
            QdrantCondition::Nested { filter } => Self::apply_filter(filter, payload),
        }
    }

    /// Evaluate match condition
    fn evaluate_match(key: &str, match_value: &QdrantMatchValue, payload: &Payload) -> bool {
        let value = match Self::get_nested_value(key, payload) {
            Some(v) => v,
            None => return false,
        };

        match match_value {
            QdrantMatchValue::String(s) => match value {
                Value::String(v) => v == s,
                _ => false,
            },
            QdrantMatchValue::Integer(i) => match value {
                Value::Number(n) => n.as_i64() == Some(*i),
                _ => false,
            },
            QdrantMatchValue::Bool(b) => match value {
                Value::Bool(v) => v == b,
                _ => false,
            },
            QdrantMatchValue::Any => !value.is_null(),
            QdrantMatchValue::Text(text_match) => match value {
                Value::String(v) => {
                    use super::filter::QdrantTextMatchType;
                    match text_match.match_type {
                        QdrantTextMatchType::Exact => v == &text_match.text,
                        QdrantTextMatchType::Prefix => v.starts_with(&text_match.text),
                        QdrantTextMatchType::Suffix => v.ends_with(&text_match.text),
                        QdrantTextMatchType::Contains => v.contains(&text_match.text),
                    }
                }
                _ => false,
            },
        }
    }

    /// Evaluate range condition
    fn evaluate_range(key: &str, range: &QdrantRange, payload: &Payload) -> bool {
        let value = match Self::get_nested_value(key, payload) {
            Some(v) => v,
            None => return false,
        };

        let number = match value {
            Value::Number(n) => n.as_f64().unwrap_or(0.0),
            _ => return false,
        };

        let mut result = true;

        if let Some(gt) = range.gt {
            result &= number > gt;
        }

        if let Some(gte) = range.gte {
            result &= number >= gte;
        }

        if let Some(lt) = range.lt {
            result &= number < lt;
        }

        if let Some(lte) = range.lte {
            result &= number <= lte;
        }

        result
    }

    /// Evaluate geo bounding box condition
    fn evaluate_geo_bounding_box(
        key: &str,
        geo_bounding_box: &super::filter::QdrantGeoBoundingBox,
        payload: &Payload,
    ) -> bool {
        let value = match Self::get_nested_value(key, payload) {
            Some(v) => v,
            None => return false,
        };

        let geo_point = match Self::parse_geo_point(value) {
            Some(p) => p,
            None => return false,
        };

        // Check if point is within bounding box
        let top_right = &geo_bounding_box.top_right;
        let bottom_left = &geo_bounding_box.bottom_left;

        geo_point.lat >= bottom_left.lat
            && geo_point.lat <= top_right.lat
            && geo_point.lon >= bottom_left.lon
            && geo_point.lon <= top_right.lon
    }

    /// Evaluate geo radius condition
    fn evaluate_geo_radius(
        key: &str,
        geo_radius: &super::filter::QdrantGeoRadius,
        payload: &Payload,
    ) -> bool {
        let value = match Self::get_nested_value(key, payload) {
            Some(v) => v,
            None => return false,
        };

        let geo_point = match Self::parse_geo_point(value) {
            Some(p) => p,
            None => return false,
        };

        // Calculate distance using Haversine formula
        let distance = Self::haversine_distance(&geo_point, &geo_radius.center);

        distance <= geo_radius.radius
    }

    /// Evaluate values count condition
    fn evaluate_values_count(
        key: &str,
        values_count: &QdrantValuesCount,
        payload: &Payload,
    ) -> bool {
        let value = match Self::get_nested_value(key, payload) {
            Some(v) => v,
            None => return false,
        };

        let count = match value {
            Value::Array(arr) => arr.len() as u32,
            Value::Object(obj) => obj.len() as u32,
            Value::String(s) => s.len() as u32,
            _ => return false,
        };

        let mut result = true;

        if let Some(gt) = values_count.gt {
            result &= count > gt;
        }

        if let Some(gte) = values_count.gte {
            result &= count >= gte;
        }

        if let Some(lt) = values_count.lt {
            result &= count < lt;
        }

        if let Some(lte) = values_count.lte {
            result &= count <= lte;
        }

        result
    }

    /// Get nested value from payload using dot notation
    fn get_nested_value<'a>(key: &str, payload: &'a Payload) -> Option<&'a Value> {
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

    /// Parse geo point from JSON value
    fn parse_geo_point(value: &Value) -> Option<QdrantGeoPoint> {
        match value {
            Value::Object(obj) => {
                let lat = obj.get("lat")?.as_f64()?;
                let lon = obj.get("lon")?.as_f64()?;
                Some(QdrantGeoPoint { lat, lon })
            }
            Value::Array(arr) if arr.len() == 2 => {
                let lat = arr[0].as_f64()?;
                let lon = arr[1].as_f64()?;
                Some(QdrantGeoPoint { lat, lon })
            }
            _ => None,
        }
    }

    /// Calculate distance between two geo points using Haversine formula
    /// Returns distance in meters
    fn haversine_distance(point1: &QdrantGeoPoint, point2: &QdrantGeoPoint) -> f64 {
        const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

        let lat1 = point1.lat.to_radians();
        let lat2 = point2.lat.to_radians();
        let delta_lat = (point2.lat - point1.lat).to_radians();
        let delta_lon = (point2.lon - point1.lon).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_METERS * c
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn create_test_payload(data: Value) -> Payload {
        Payload::new(data)
    }

    #[test]
    fn test_match_string() {
        let payload = create_test_payload(json!({
            "name": "John",
            "city": "New York"
        }));

        let condition = QdrantCondition::match_string("name", "John");
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        let condition = QdrantCondition::match_string("name", "Jane");
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_match_integer() {
        let payload = create_test_payload(json!({
            "age": 30,
            "score": 95
        }));

        let condition = QdrantCondition::match_integer("age", 30);
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        let condition = QdrantCondition::match_integer("age", 25);
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_range_filter() {
        let payload = create_test_payload(json!({
            "price": 50.0,
            "rating": 4.5
        }));

        // Test gt (greater than)
        let condition = QdrantCondition::range("price", QdrantRange::gt(40.0));
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        let condition = QdrantCondition::range("price", QdrantRange::gt(60.0));
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));

        // Test between
        let condition = QdrantCondition::range("price", QdrantRange::between(30.0, 70.0));
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        let condition = QdrantCondition::range("price", QdrantRange::between(60.0, 80.0));
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_geo_bounding_box() {
        let payload = create_test_payload(json!({
            "location": {
                "lat": 40.7128,
                "lon": -74.0060
            }
        }));

        // New York is within this bounding box
        let condition = QdrantCondition::geo_bounding_box(
            "location",
            QdrantGeoPoint::new(41.0, -73.0), // top-right
            QdrantGeoPoint::new(40.0, -75.0), // bottom-left
        );
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        // New York is NOT within this bounding box
        let condition = QdrantCondition::geo_bounding_box(
            "location",
            QdrantGeoPoint::new(35.0, -70.0),
            QdrantGeoPoint::new(30.0, -75.0),
        );
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_geo_radius() {
        let payload = create_test_payload(json!({
            "location": {
                "lat": 40.7128,
                "lon": -74.0060
            }
        }));

        // Point within 10km radius
        let condition = QdrantCondition::geo_radius(
            "location",
            QdrantGeoPoint::new(40.7128, -74.0060),
            10000.0, // 10km in meters
        );
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        // Point NOT within 1m radius of different location
        let condition = QdrantCondition::geo_radius(
            "location",
            QdrantGeoPoint::new(41.0, -73.0),
            1.0, // 1 meter
        );
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_values_count() {
        let payload = create_test_payload(json!({
            "tags": ["rust", "vector", "search"],
            "categories": ["tech", "ai"]
        }));

        // Test gte (greater than or equal)
        let condition = QdrantCondition::values_count("tags", QdrantValuesCount::gte(3));
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        let condition = QdrantCondition::values_count("tags", QdrantValuesCount::gte(4));
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));

        // Test between
        let condition =
            QdrantCondition::values_count("categories", QdrantValuesCount::between(1, 5));
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_nested_keys() {
        let payload = create_test_payload(json!({
            "user": {
                "profile": {
                    "age": 30
                }
            }
        }));

        let condition = QdrantCondition::match_integer("user.profile.age", 30);
        assert!(FilterProcessor::evaluate_condition(&condition, &payload));

        let condition = QdrantCondition::match_integer("user.profile.age", 25);
        assert!(!FilterProcessor::evaluate_condition(&condition, &payload));
    }

    #[test]
    fn test_combined_filters() {
        let payload = create_test_payload(json!({
            "name": "Product A",
            "price": 50.0,
            "category": "electronics",
            "in_stock": true
        }));

        use super::super::filter::QdrantFilterBuilder;

        let filter = QdrantFilterBuilder::new()
            .must(QdrantCondition::range(
                "price",
                QdrantRange::between(30.0, 70.0),
            ))
            .must(QdrantCondition::match_bool("in_stock", true))
            .must(QdrantCondition::match_string("category", "electronics"))
            .build();

        assert!(FilterProcessor::apply_filter(&filter, &payload));

        // Should fail with must_not
        let filter = QdrantFilterBuilder::new()
            .must(QdrantCondition::range(
                "price",
                QdrantRange::between(30.0, 70.0),
            ))
            .must_not(QdrantCondition::match_string("category", "electronics"))
            .build();

        assert!(!FilterProcessor::apply_filter(&filter, &payload));
    }
}

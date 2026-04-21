//! Qdrant-compatible gRPC service implementations
//!
//! This module implements the Qdrant gRPC API on top of Vectorizer,
//! enabling drop-in replacement for Qdrant clients using gRPC.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::sync::Arc;
use std::time::Instant;

use ::vectorizer::models::qdrant::filter::{
    QdrantCondition, QdrantFilter, QdrantMatchValue, QdrantRange,
};
use ::vectorizer::models::qdrant::filter_processor::FilterProcessor;
use ::vectorizer::models::{Payload, Vector};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};
use vectorizer::VectorStore;

use crate::grpc::qdrant_proto::collections_server::Collections;
use crate::grpc::qdrant_proto::r#match::MatchValue;
use crate::grpc::qdrant_proto::points_server::Points;
use crate::grpc::qdrant_proto::snapshots_server::Snapshots;
use crate::grpc::qdrant_proto::*;

/// Qdrant-compatible gRPC service
#[derive(Clone)]
pub struct QdrantGrpcService {
    store: Arc<VectorStore>,
    snapshot_manager: Option<Arc<vectorizer::storage::SnapshotManager>>,
}

impl QdrantGrpcService {
    pub fn new(store: Arc<VectorStore>) -> Self {
        Self {
            store,
            snapshot_manager: None,
        }
    }

    pub fn with_snapshot_manager(
        store: Arc<VectorStore>,
        snapshot_manager: Arc<vectorizer::storage::SnapshotManager>,
    ) -> Self {
        Self {
            store,
            snapshot_manager: Some(snapshot_manager),
        }
    }
}

// ============================================================================
// Filter Conversion Functions
// ============================================================================

/// Convert gRPC Filter to internal QdrantFilter
fn convert_grpc_filter(filter: &Filter) -> QdrantFilter {
    let must = if filter.must.is_empty() {
        None
    } else {
        Some(
            filter
                .must
                .iter()
                .filter_map(convert_grpc_condition)
                .collect(),
        )
    };

    let must_not = if filter.must_not.is_empty() {
        None
    } else {
        Some(
            filter
                .must_not
                .iter()
                .filter_map(convert_grpc_condition)
                .collect(),
        )
    };

    let should = if filter.should.is_empty() {
        None
    } else {
        Some(
            filter
                .should
                .iter()
                .filter_map(convert_grpc_condition)
                .collect(),
        )
    };

    QdrantFilter {
        must,
        must_not,
        should,
    }
}

/// Convert gRPC Condition to internal QdrantCondition
fn convert_grpc_condition(condition: &Condition) -> Option<QdrantCondition> {
    use condition::ConditionOneOf;

    match &condition.condition_one_of {
        Some(ConditionOneOf::Field(field)) => convert_field_condition(field),
        Some(ConditionOneOf::Filter(nested_filter)) => Some(QdrantCondition::Nested {
            filter: Box::new(convert_grpc_filter(nested_filter)),
        }),
        Some(ConditionOneOf::IsEmpty(is_empty)) => {
            // IsEmpty checks if field is empty/doesn't exist
            Some(QdrantCondition::Match {
                key: is_empty.key.clone(),
                match_value: QdrantMatchValue::Any,
            })
        }
        Some(ConditionOneOf::IsNull(is_null)) => {
            // IsNull checks if field is null
            Some(QdrantCondition::Match {
                key: is_null.key.clone(),
                match_value: QdrantMatchValue::Any,
            })
        }
        Some(ConditionOneOf::HasId(_has_id)) => {
            // HasId is handled separately in the calling code
            None
        }
        Some(ConditionOneOf::Nested(nested)) => {
            nested.filter.as_ref().map(|f| QdrantCondition::Nested {
                filter: Box::new(convert_grpc_filter(f)),
            })
        }
        Some(ConditionOneOf::HasVector(_)) => {
            // HasVector condition - not applicable for filter-based operations
            None
        }
        None => None,
    }
}

/// Convert gRPC FieldCondition to internal QdrantCondition
fn convert_field_condition(field: &FieldCondition) -> Option<QdrantCondition> {
    let key = field.key.clone();

    // Check match field first
    if let Some(m) = &field.r#match {
        if let Some(match_value) = &m.match_value {
            return match match_value {
                MatchValue::Keyword(s) => Some(QdrantCondition::Match {
                    key,
                    match_value: QdrantMatchValue::String(s.clone()),
                }),
                MatchValue::Integer(i) => Some(QdrantCondition::Match {
                    key,
                    match_value: QdrantMatchValue::Integer(*i),
                }),
                MatchValue::Boolean(b) => Some(QdrantCondition::Match {
                    key,
                    match_value: QdrantMatchValue::Bool(*b),
                }),
                MatchValue::Text(t) => Some(QdrantCondition::Match {
                    key,
                    match_value: QdrantMatchValue::String(t.clone()),
                }),
                MatchValue::Keywords(kw) => {
                    // Match any of the keywords
                    kw.strings.first().map(|first| QdrantCondition::Match {
                        key,
                        match_value: QdrantMatchValue::String(first.clone()),
                    })
                }
                MatchValue::Integers(ints) => {
                    ints.integers.first().map(|first| QdrantCondition::Match {
                        key,
                        match_value: QdrantMatchValue::Integer(*first),
                    })
                }
                MatchValue::ExceptIntegers(_) => None,
                MatchValue::ExceptKeywords(_) => None,
                MatchValue::Phrase(p) => Some(QdrantCondition::Match {
                    key,
                    match_value: QdrantMatchValue::String(p.clone()),
                }),
                MatchValue::TextAny(t) => Some(QdrantCondition::Match {
                    key,
                    match_value: QdrantMatchValue::String(t.clone()),
                }),
            };
        }
    }

    // Check range field
    if let Some(r) = &field.range {
        return Some(QdrantCondition::Range {
            key,
            range: QdrantRange {
                lt: r.lt,
                lte: r.lte,
                gt: r.gt,
                gte: r.gte,
            },
        });
    }

    // Geo conditions not yet fully supported
    if field.geo_bounding_box.is_some() || field.geo_radius.is_some() || field.geo_polygon.is_some()
    {
        return None;
    }

    // Values count not yet fully supported
    if field.values_count.is_some() {
        return None;
    }

    // Datetime range not yet fully supported
    if field.datetime_range.is_some() {
        return None;
    }

    None
}

/// Get vector IDs that match a filter from collection
fn get_matching_vector_ids(
    collection: &vectorizer::db::vector_store::CollectionType,
    filter: &QdrantFilter,
) -> Vec<String> {
    let all_vectors = collection.get_all_vectors();
    let total_count = all_vectors.len();
    let mut matching_ids = Vec::new();

    for vector in all_vectors {
        let payload = vector.payload.as_ref().cloned().unwrap_or_else(|| Payload {
            data: serde_json::json!({}),
        });

        if FilterProcessor::apply_filter(filter, &payload) {
            matching_ids.push(vector.id.clone());
        }
    }

    debug!(
        "Filter matched {} vectors out of {}",
        matching_ids.len(),
        total_count
    );
    matching_ids
}

// ============================================================================
// Sub-modules — one per gRPC trait (phase3_split-qdrant-grpc).
// The trait impls live in collections.rs / points.rs / snapshots.rs; this
// file keeps the struct, its constructor, and the shared helper fns that
// every trait impl imports via 'use super::...'.
// ============================================================================

pub mod collections;
pub mod points;
pub mod snapshots;

// ============================================================================
// Helper Functions
// ============================================================================

fn convert_payload_to_json(
    payload: &std::collections::HashMap<String, Value>,
) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = payload
        .iter()
        .map(|(k, v)| (k.clone(), convert_value_to_json(v)))
        .collect();
    serde_json::Value::Object(map)
}

fn convert_value_to_json(value: &Value) -> serde_json::Value {
    match &value.kind {
        Some(value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(value::Kind::DoubleValue(d)) => serde_json::json!(*d),
        Some(value::Kind::IntegerValue(i)) => serde_json::json!(*i),
        Some(value::Kind::StringValue(s)) => serde_json::json!(s),
        Some(value::Kind::BoolValue(b)) => serde_json::json!(*b),
        Some(value::Kind::StructValue(s)) => {
            let map: serde_json::Map<String, serde_json::Value> = s
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), convert_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Some(value::Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.iter().map(convert_value_to_json).collect())
        }
        None => serde_json::Value::Null,
    }
}

fn convert_json_to_payload(json: &serde_json::Value) -> std::collections::HashMap<String, Value> {
    match json {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), convert_json_to_value(v)))
            .collect(),
        _ => std::collections::HashMap::new(),
    }
}

fn convert_json_to_value(json: &serde_json::Value) -> Value {
    Value {
        kind: Some(match json {
            serde_json::Value::Null => value::Kind::NullValue(0),
            serde_json::Value::Bool(b) => value::Kind::BoolValue(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    value::Kind::IntegerValue(i)
                } else if let Some(f) = n.as_f64() {
                    value::Kind::DoubleValue(f)
                } else {
                    value::Kind::NullValue(0)
                }
            }
            serde_json::Value::String(s) => value::Kind::StringValue(s.clone()),
            serde_json::Value::Array(arr) => value::Kind::ListValue(ListValue {
                values: arr.iter().map(convert_json_to_value).collect(),
            }),
            serde_json::Value::Object(map) => value::Kind::StructValue(Struct {
                fields: map
                    .iter()
                    .map(|(k, v)| (k.clone(), convert_json_to_value(v)))
                    .collect(),
            }),
        }),
    }
}

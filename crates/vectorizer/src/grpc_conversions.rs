//! Conversion utilities between Protobuf types and internal types

use vectorizer_protocol::grpc_gen::vectorizer as proto;

use crate::db::hybrid_search::{HybridScoringAlgorithm, HybridSearchConfig};
use crate::error::{Result, VectorizerError};
use crate::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, SearchResult,
    SparseVector, StorageType, Vector,
};

impl TryFrom<&proto::CollectionConfig> for crate::models::CollectionConfig {
    type Error = VectorizerError;

    fn try_from(proto: &proto::CollectionConfig) -> Result<Self> {
        let metric_enum = proto::DistanceMetric::from_i32(proto.metric).ok_or_else(|| {
            VectorizerError::InvalidConfiguration {
                message: "Invalid distance metric".to_string(),
            }
        })?;

        Ok(CollectionConfig {
            dimension: proto.dimension as usize,
            metric: DistanceMetric::from(metric_enum),
            hnsw_config: proto
                .hnsw_config
                .as_ref()
                .map(|h| HnswConfig {
                    m: h.m as usize,
                    ef_construction: h.ef_construction as usize,
                    ef_search: h.ef as usize, // proto uses 'ef', model uses 'ef_search'
                    seed: Some(h.seed),
                })
                .unwrap_or_default(),
            quantization: proto
                .quantization
                .as_ref()
                .map(|q| match q.config.as_ref() {
                    Some(proto::quantization_config::Config::Scalar(s)) => QuantizationConfig::SQ {
                        bits: s.bits as usize,
                    },
                    Some(proto::quantization_config::Config::Product(p)) => {
                        QuantizationConfig::PQ {
                            n_subquantizers: p.subvectors as usize,
                            n_centroids: p.centroids as usize,
                        }
                    }
                    Some(proto::quantization_config::Config::Binary(_)) => {
                        QuantizationConfig::Binary
                    }
                    None => QuantizationConfig::None,
                })
                .unwrap_or(QuantizationConfig::None),
            compression: Default::default(),
            normalization: None,
            sharding: None,
            graph: None,
            storage_type: {
                let storage_enum = proto::StorageType::from_i32(proto.storage_type)
                    .unwrap_or(proto::StorageType::Memory);
                Some(StorageType::from(storage_enum))
            },
            encryption: None,
        })
    }
}

impl From<proto::DistanceMetric> for DistanceMetric {
    fn from(proto: proto::DistanceMetric) -> Self {
        match proto {
            proto::DistanceMetric::Cosine => DistanceMetric::Cosine,
            proto::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            proto::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
        }
    }
}

impl From<proto::StorageType> for StorageType {
    fn from(proto: proto::StorageType) -> Self {
        match proto {
            proto::StorageType::Memory => StorageType::Memory,
            proto::StorageType::Mmap => StorageType::Mmap,
        }
    }
}

impl From<&SearchResult> for proto::SearchResult {
    fn from(result: &SearchResult) -> Self {
        use std::collections::HashMap;
        proto::SearchResult {
            id: result.id.clone(),
            // proto `score` is now `float` (f32). See phase2_unify-search-result-type.
            score: result.score,
            vector: result.vector.clone().unwrap_or_default(),
            payload: result
                .payload
                .as_ref()
                .and_then(|p| {
                    // Payload is a wrapper around serde_json::Value
                    // Convert to HashMap<String, String> for protobuf
                    if let serde_json::Value::Object(map) = &p.data {
                        Some(
                            map.iter()
                                .map(|(k, v)| (k.clone(), v.to_string()))
                                .collect::<HashMap<String, String>>(),
                        )
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
        }
    }
}

impl From<proto::SearchResult> for SearchResult {
    /// Inverse of `From<&SearchResult> for proto::SearchResult`. Builds
    /// a canonical `models::SearchResult` from a proto payload, so callers
    /// on the gRPC receiving side never have to hand-roll the conversion
    /// (and never silently lose precision via an `as f32` cast).
    fn from(proto: proto::SearchResult) -> Self {
        let payload = if proto.payload.is_empty() {
            None
        } else {
            let map: serde_json::Map<String, serde_json::Value> = proto
                .payload
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            Some(Payload {
                data: serde_json::Value::Object(map),
            })
        };
        SearchResult {
            id: proto.id,
            score: proto.score,
            dense_score: None,
            sparse_score: None,
            vector: if proto.vector.is_empty() {
                None
            } else {
                Some(proto.vector)
            },
            payload,
        }
    }
}

impl From<proto::HybridSearchResult> for SearchResult {
    /// Canonicalise a proto `HybridSearchResult` back into the single
    /// `models::SearchResult` shape. The fused `hybrid_score` becomes the
    /// main `score`; individual dense/sparse scores are preserved in the
    /// optional fields.
    fn from(proto: proto::HybridSearchResult) -> Self {
        let payload = if proto.payload.is_empty() {
            None
        } else {
            let map: serde_json::Map<String, serde_json::Value> = proto
                .payload
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            Some(Payload {
                data: serde_json::Value::Object(map),
            })
        };
        SearchResult {
            id: proto.id,
            score: proto.hybrid_score,
            dense_score: Some(proto.dense_score),
            sparse_score: Some(proto.sparse_score),
            vector: if proto.vector.is_empty() {
                None
            } else {
                Some(proto.vector)
            },
            payload,
        }
    }
}

impl TryFrom<&proto::InsertVectorRequest> for Vector {
    type Error = VectorizerError;

    fn try_from(req: &proto::InsertVectorRequest) -> Result<Self> {
        use std::collections::HashMap;
        Ok(Vector {
            id: req.vector_id.clone(),
            data: req.data.clone(),
            sparse: None, // gRPC doesn't support sparse vectors directly yet
            payload: if req.payload.is_empty() {
                None
            } else {
                // Convert HashMap<String, String> to Payload (which wraps serde_json::Value)
                let json_map: serde_json::Map<String, serde_json::Value> = req
                    .payload
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();
                Some(Payload::new(serde_json::Value::Object(json_map)))
            },
            document_id: None,
        })
    }
}

impl From<proto::HybridScoringAlgorithm> for HybridScoringAlgorithm {
    fn from(proto: proto::HybridScoringAlgorithm) -> Self {
        match proto {
            proto::HybridScoringAlgorithm::Rrf => HybridScoringAlgorithm::ReciprocalRankFusion,
            proto::HybridScoringAlgorithm::Weighted => HybridScoringAlgorithm::WeightedCombination,
            proto::HybridScoringAlgorithm::AlphaBlend => HybridScoringAlgorithm::AlphaBlending,
        }
    }
}

/// Convert a hybrid-search proto request into the
/// `(dense_query, sparse_query, config)` triple the engine expects.
///
/// This was a `TryFrom` impl before phase4_split-vectorizer-workspace
/// sub-phase 2 moved `HybridSearchRequest` into `vectorizer-protocol` —
/// the orphan rule then rejected the impl because every non-tuple
/// component was foreign to this crate. A free function carries the
/// same conversion without invoking the orphan rule.
pub fn hybrid_search_request_to_engine_args(
    req: &proto::HybridSearchRequest,
) -> Result<(Vec<f32>, Option<SparseVector>, HybridSearchConfig)> {
    let dense_query = req.dense_query.clone();
    let sparse_query = req.sparse_query.as_ref().map(|sv| SparseVector {
        indices: sv.indices.iter().map(|&i| i as usize).collect(),
        values: sv.values.clone(),
    });
    let config = req.config.as_ref().map(|c| HybridSearchConfig {
        dense_k: c.dense_k as usize,
        sparse_k: c.sparse_k as usize,
        final_k: c.final_k as usize,
        alpha: c.alpha as f32,
        algorithm: HybridScoringAlgorithm::from(c.algorithm()),
    });

    Ok((
        dense_query,
        sparse_query,
        config.unwrap_or_else(|| HybridSearchConfig {
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            alpha: 0.5,
            algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
        }),
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_payload() -> Payload {
        Payload {
            data: serde_json::json!({
                "title": "hello",
                "tag": "rust",
            }),
        }
    }

    #[test]
    fn search_result_round_trip_preserves_score_exactly() {
        // With proto `float` the score is f32 on both sides — no `as f32`
        // narrowing, no reordering of near-tie results.
        let original = SearchResult {
            id: "vec-1".to_string(),
            score: 0.987_654_32_f32,
            dense_score: None,
            sparse_score: None,
            vector: Some(vec![0.1, 0.2, 0.3]),
            payload: Some(sample_payload()),
        };

        let proto: proto::SearchResult = (&original).into();
        assert_eq!(
            proto.score, original.score,
            "f32→f32 round-trip must be bit-exact"
        );

        let back: SearchResult = proto.into();
        assert_eq!(back.id, original.id);
        assert_eq!(back.score, original.score);
        assert_eq!(back.vector, original.vector);
    }

    #[test]
    fn search_result_ordering_preserved_through_round_trip() {
        // Regression for the f64→f32 narrowing bug: three nearly-tied scores
        // must come back in the same order after a proto round-trip.
        let scores = [0.950_000_1_f32, 0.950_000_2_f32, 0.950_000_3_f32];
        let originals: Vec<SearchResult> = scores
            .iter()
            .enumerate()
            .map(|(i, &s)| SearchResult {
                id: format!("v{}", i),
                score: s,
                dense_score: None,
                sparse_score: None,
                vector: None,
                payload: None,
            })
            .collect();

        let mut round_tripped: Vec<SearchResult> = originals
            .iter()
            .map(|r| {
                let proto: proto::SearchResult = r.into();
                proto.into()
            })
            .collect();
        round_tripped.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        assert_eq!(round_tripped[0].id, "v2");
        assert_eq!(round_tripped[1].id, "v1");
        assert_eq!(round_tripped[2].id, "v0");
    }

    #[test]
    fn hybrid_search_result_canonicalises_into_models_search_result() {
        let proto = proto::HybridSearchResult {
            id: "doc-42".to_string(),
            hybrid_score: 0.75,
            dense_score: 0.60,
            sparse_score: 0.90,
            vector: vec![],
            payload: Default::default(),
        };
        let canonical: SearchResult = proto.into();
        assert_eq!(canonical.id, "doc-42");
        assert_eq!(canonical.score, 0.75);
        assert_eq!(canonical.dense_score, Some(0.60));
        assert_eq!(canonical.sparse_score, Some(0.90));
    }

    #[test]
    fn search_result_empty_vector_round_trip_preserves_none() {
        let original = SearchResult {
            id: "no-vec".to_string(),
            score: 0.5,
            dense_score: None,
            sparse_score: None,
            vector: None,
            payload: None,
        };
        let proto: proto::SearchResult = (&original).into();
        let back: SearchResult = proto.into();
        assert!(back.vector.is_none(), "empty proto vector must map to None");
        assert!(
            back.payload.is_none(),
            "empty proto payload must map to None"
        );
    }
}

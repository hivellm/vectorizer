//! Conversion utilities between Protobuf types and internal types

use super::vectorizer;
use crate::db::hybrid_search::{HybridScoringAlgorithm, HybridSearchConfig};
use crate::error::{Result, VectorizerError};
use crate::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, SearchResult,
    SparseVector, StorageType, Vector,
};

impl TryFrom<&vectorizer::CollectionConfig> for crate::models::CollectionConfig {
    type Error = VectorizerError;

    fn try_from(proto: &vectorizer::CollectionConfig) -> Result<Self> {
        let metric_enum = vectorizer::DistanceMetric::from_i32(proto.metric).ok_or_else(|| {
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
                    Some(vectorizer::quantization_config::Config::Scalar(s)) => {
                        QuantizationConfig::SQ {
                            bits: s.bits as usize,
                        }
                    }
                    Some(vectorizer::quantization_config::Config::Product(p)) => {
                        QuantizationConfig::PQ {
                            n_subquantizers: p.subvectors as usize,
                            n_centroids: p.centroids as usize,
                        }
                    }
                    Some(vectorizer::quantization_config::Config::Binary(_)) => {
                        QuantizationConfig::Binary
                    }
                    None => QuantizationConfig::None,
                })
                .unwrap_or(QuantizationConfig::None),
            compression: Default::default(),
            normalization: None,
            sharding: None,
            storage_type: {
                let storage_enum = vectorizer::StorageType::from_i32(proto.storage_type)
                    .unwrap_or(vectorizer::StorageType::Memory);
                Some(StorageType::from(storage_enum))
            },
        })
    }
}

impl From<vectorizer::DistanceMetric> for DistanceMetric {
    fn from(proto: vectorizer::DistanceMetric) -> Self {
        match proto {
            vectorizer::DistanceMetric::Cosine => DistanceMetric::Cosine,
            vectorizer::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            vectorizer::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
        }
    }
}

impl From<vectorizer::StorageType> for StorageType {
    fn from(proto: vectorizer::StorageType) -> Self {
        match proto {
            vectorizer::StorageType::Memory => StorageType::Memory,
            vectorizer::StorageType::Mmap => StorageType::Mmap,
        }
    }
}

impl From<&SearchResult> for vectorizer::SearchResult {
    fn from(result: &SearchResult) -> Self {
        use std::collections::HashMap;
        vectorizer::SearchResult {
            id: result.id.clone(),
            score: result.score as f64,
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

impl TryFrom<&vectorizer::InsertVectorRequest> for Vector {
    type Error = VectorizerError;

    fn try_from(req: &vectorizer::InsertVectorRequest) -> Result<Self> {
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
        })
    }
}

impl From<vectorizer::HybridScoringAlgorithm> for HybridScoringAlgorithm {
    fn from(proto: vectorizer::HybridScoringAlgorithm) -> Self {
        match proto {
            vectorizer::HybridScoringAlgorithm::Rrf => HybridScoringAlgorithm::ReciprocalRankFusion,
            vectorizer::HybridScoringAlgorithm::Weighted => {
                HybridScoringAlgorithm::WeightedCombination
            }
            vectorizer::HybridScoringAlgorithm::AlphaBlend => HybridScoringAlgorithm::AlphaBlending,
        }
    }
}

impl TryFrom<&vectorizer::HybridSearchRequest>
    for (Vec<f32>, Option<SparseVector>, HybridSearchConfig)
{
    type Error = VectorizerError;

    fn try_from(req: &vectorizer::HybridSearchRequest) -> Result<Self> {
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
}

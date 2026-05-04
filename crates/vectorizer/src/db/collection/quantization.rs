//! Quantization — SQ/Binary migration of existing vectors, PQ
//! training, and small scalar helpers.
//!
//! Wire format details live in [`crate::models::QuantizedVector`] and
//! [`crate::quantization::product`]; this module just orchestrates
//! when quantization runs against a [`Collection`]'s vectors.

use tracing::{debug, info, warn};

use super::Collection;
use crate::error::Result;

impl Collection {
    /// Requantize existing vectors if quantization is enabled (parallel processing)
    /// Migrates vectors from full precision to quantized storage
    pub fn requantize_existing_vectors(&self) -> Result<()> {
        use rayon::prelude::*;

        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            debug!(
                "Migrating existing vectors to quantized storage in collection '{}'",
                self.name
            );

            // Use vector_order to iterate over all vectors
            let vector_order = self.vector_order.read();
            let vector_count = vector_order.len();

            if vector_count == 0 {
                return Ok(());
            }

            // Convert all vectors to quantized format in parallel
            let quantization_config = self.config.quantization.clone();
            let quantized: Vec<(String, crate::models::QuantizedVector)> = vector_order
                .par_iter()
                .filter_map(|id| {
                    if let Ok(Some(vector)) = self.vectors.get(id) {
                        let qv = crate::models::QuantizedVector::from_vector(
                            vector,
                            &quantization_config,
                        );
                        Some((id.clone(), qv))
                    } else {
                        None
                    }
                })
                .collect();

            // Move to quantized storage
            let mut quantized_storage = self.quantized_vectors.lock();
            for (id, qv) in quantized {
                quantized_storage.insert(id, qv);
            }

            info!(
                "✅ Migrated {} vectors to quantized storage (~75% memory reduction)",
                vector_count
            );
        }

        Ok(())
    }

    /// Quantize a vector using scalar quantization (8-bit)
    #[allow(dead_code)]
    fn quantize_vector(&self, vector: &[f32], bits: u8) -> Result<Vec<u8>> {
        let max_val = 2_u32.pow(bits as u32) - 1;
        let mut quantized = Vec::with_capacity(vector.len());

        for &val in vector {
            // Normalize to [0, 1] range (assuming vectors are normalized to [-1, 1])
            let normalized = (val + 1.0) / 2.0;
            let normalized_clamped = normalized.clamp(0.0, 1.0);
            let quantized_val = (normalized_clamped * max_val as f32) as u8;
            quantized.push(quantized_val);
        }

        Ok(quantized)
    }

    /// Dequantize a vector from scalar quantization (8-bit)
    #[allow(dead_code)]
    fn dequantize_vector(&self, quantized: &[u8], bits: u8) -> Result<Vec<f32>> {
        let max_val = 2_u32.pow(bits as u32) - 1;
        let mut dequantized = Vec::with_capacity(quantized.len());

        for &val in quantized {
            // Denormalize from [0, 1] range back to [-1, 1]
            let normalized = val as f32 / max_val as f32;
            let denormalized = normalized * 2.0 - 1.0;
            dequantized.push(denormalized);
        }

        Ok(dequantized)
    }

    /// Train Product Quantization if enabled and not yet trained
    pub fn train_pq_if_needed(&self) -> Result<()> {
        use crate::models::QuantizationConfig;
        use crate::quantization::product::{ProductQuantization, ProductQuantizationConfig};

        // Check if PQ is enabled
        let (n_centroids, n_subquantizers) = match &self.config.quantization {
            QuantizationConfig::PQ {
                n_centroids,
                n_subquantizers,
            } => (*n_centroids, *n_subquantizers),
            _ => return Ok(()), // PQ not enabled
        };

        // Check if already trained
        {
            let pq = self.pq_quantizer.read();
            if pq.is_some() {
                return Ok(()); // Already trained
            }
        }

        // Collect training vectors (up to 10k)
        let training_limit = 10000;
        let vector_order = self.vector_order.read();
        let mut training_vectors = Vec::new();

        for id in vector_order.iter().take(training_limit) {
            if let Ok(Some(vector)) = self.vectors.get(id) {
                training_vectors.push(vector.data);
            }
        }

        if training_vectors.is_empty() {
            warn!("Cannot train PQ: no vectors available");
            return Ok(());
        }

        info!(
            "Training PQ with {} vectors (subvectors={}, centroids={})",
            training_vectors.len(),
            n_subquantizers,
            n_centroids
        );

        // Create and train PQ
        let pq_config = ProductQuantizationConfig {
            subvectors: n_subquantizers,
            centroids_per_subvector: n_centroids,
            training_samples: training_limit,
            adaptive_assignment: true,
        };

        let mut pq = ProductQuantization::new(pq_config, self.config.dimension);

        if let Err(e) = pq.train(&training_vectors) {
            warn!("Failed to train PQ: {}", e);
            return Ok(());
        }

        // Store trained quantizer
        *self.pq_quantizer.write() = Some(pq);
        info!("✅ PQ trained successfully");

        Ok(())
    }

    /// Get PQ-quantized representation of a vector
    pub fn pq_quantize_vector(&self, vector: &[f32]) -> Result<Option<Vec<u8>>> {
        let pq = self.pq_quantizer.read();
        if let Some(ref quantizer) = *pq {
            match quantizer.quantize(vector) {
                Ok(codes) => Ok(Some(codes)),
                Err(e) => {
                    warn!("PQ quantization failed: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Re-encode (re-quantize) all vectors in-place to `target_encoding`.
    ///
    /// # Correctness invariant
    ///
    /// Writes are write-locked for the duration of the swap: the collection
    /// acquires an exclusive lock on `vector_order` (which serialises new
    /// inserts, which also take that lock), builds the new quantized store
    /// off-line, then atomically replaces `quantized_vectors` under the
    /// same hold. This is the write-lock fallback described in the phase13
    /// spec. An atomic double-buffer swap without locking writes would
    /// require a secondary index that mirrors concurrent inserts; given the
    /// non-trivial complexity, the safer correct version is shipped here
    /// and documented.
    ///
    /// **Concurrent writes are blocked for the duration of the reencode.**
    /// For typical collections (< 1M vectors) this is sub-second. For
    /// larger collections operators should schedule the reencode during a
    /// maintenance window.
    pub fn reencode_inplace(&self, target_encoding: &str) -> Result<()> {
        use crate::models::QuantizationConfig;

        let new_config = match target_encoding {
            "sq8" | "SQ8" | "scalar" => QuantizationConfig::SQ { bits: 8 },
            "binary" => QuantizationConfig::Binary,
            "none" | "fp32" => QuantizationConfig::None,
            other => {
                return Err(crate::error::VectorizerError::Storage(format!(
                    "unsupported target_encoding '{}'; valid values: sq8, binary, fp32/none",
                    other
                )));
            }
        };

        // Hold vector_order write lock for the duration to serialise
        // concurrent inserts (they also take this lock).
        let vector_order = self.vector_order.write();
        let count = vector_order.len();

        info!(
            "reencode_inplace: collection '{}', {} vectors, target='{}'",
            self.name, count, target_encoding
        );

        if count == 0 {
            return Ok(());
        }

        match &new_config {
            QuantizationConfig::None => {
                // Migrate quantized → full-precision: dequantize into vectors map.
                let mut qvecs = self.quantized_vectors.lock();
                for id in vector_order.iter() {
                    if let Some(qv) = qvecs.remove(id) {
                        let vec = qv.to_vector();
                        self.vectors.insert(id.clone(), vec)?;
                    }
                }
                info!(
                    "reencode_inplace: '{}' converted to fp32 ({} vectors)",
                    self.name, count
                );
            }
            QuantizationConfig::SQ { .. } | QuantizationConfig::Binary => {
                // Build new quantized map from whatever is currently in storage.
                let mut new_qvecs = std::collections::HashMap::with_capacity(count);

                for id in vector_order.iter() {
                    // Try full-precision first, then existing quantized storage.
                    let vector = if let Ok(Some(v)) = self.vectors.get(id) {
                        v
                    } else if let Some(qv) = self.quantized_vectors.lock().get(id) {
                        qv.to_vector()
                    } else {
                        warn!(
                            "reencode_inplace: vector '{}' not found in collection '{}'",
                            id, self.name
                        );
                        continue;
                    };

                    let new_qv = crate::models::QuantizedVector::from_vector(vector, &new_config);
                    new_qvecs.insert(id.clone(), new_qv);
                }

                // Atomically swap the quantized store.
                *self.quantized_vectors.lock() = new_qvecs;

                info!(
                    "reencode_inplace: '{}' converted to '{}' ({} vectors)",
                    self.name, target_encoding, count
                );
            }
            QuantizationConfig::PQ { .. } => {
                // PQ requires training first; reuse the existing train path.
                self.train_pq_if_needed()?;
                info!(
                    "reencode_inplace: '{}' PQ training completed ({} vectors)",
                    self.name, count
                );
            }
        }

        Ok(())
    }
}

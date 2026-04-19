//! SVD embedding provider — TF-IDF + truncated singular value decomposition.
//!
//! Extracted from the monolithic `embedding/mod.rs` by
//! phase4_split-interleaved-embedding-providers. Depends on
//! [`super::tfidf::TfIdfEmbedding`] for the base transformation.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use crate::embedding::EmbeddingProvider;
use crate::embedding::providers::tfidf::TfIdfEmbedding;
use crate::error::{Result, VectorizerError};

#[derive(Debug)]
pub struct SvdEmbedding {
    /// The target reduced dimension
    reduced_dimension: usize,
    /// TF-IDF embedding for base transformation
    tfidf: TfIdfEmbedding,
    /// SVD transformation matrix (V^T truncated to reduced_dimension)
    transformation_matrix: Option<ndarray::Array2<f32>>,
    /// Whether SVD has been fitted
    fitted: bool,
}
impl SvdEmbedding {
    /// Create a new SVD embedding provider
    pub fn new(reduced_dimension: usize, vocabulary_size: usize) -> Self {
        Self {
            reduced_dimension,
            tfidf: TfIdfEmbedding::new(vocabulary_size),
            transformation_matrix: None,
            fitted: false,
        }
    }

    /// Fit a simple linear transformation (simplified SVD approximation)
    pub fn fit_svd(&mut self, texts: &[&str]) -> Result<()> {
        // First, build TF-IDF vocabulary
        self.tfidf.build_vocabulary(texts);

        // Create a simple transformation matrix using hash-based pseudo-random orthogonal vectors
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let vocab_size = self.tfidf.dimension();
        let mut transformation_matrix =
            ndarray::Array2::<f32>::zeros((self.reduced_dimension, vocab_size));

        // Generate transformation matrix using seeded random values
        let mut hasher = DefaultHasher::new();
        texts.hash(&mut hasher);
        let base_seed = hasher.finish();

        for i in 0..self.reduced_dimension {
            // Create a vector for this dimension
            let mut vector = Vec::with_capacity(vocab_size);

            for j in 0..vocab_size {
                // Generate pseudo-random value seeded by dimension and position
                let seed = base_seed.wrapping_add((i as u64 * 1000) + j as u64);
                let value = ((seed.wrapping_mul(1103515245) % 65536) as f32 / 32768.0) - 1.0;
                vector.push(value);
            }

            // Orthogonalize with previous vectors (simplified Gram-Schmidt)
            for k in 0..i {
                let prev_row = transformation_matrix.row(k);
                let dot_product: f32 = vector.iter().zip(prev_row.iter()).map(|(a, b)| a * b).sum();
                let norm_sq: f32 = prev_row.iter().map(|x| x * x).sum();

                if norm_sq > 0.0 {
                    let projection = dot_product / norm_sq;
                    for j in 0..vocab_size {
                        vector[j] -= projection * prev_row[j];
                    }
                }
            }

            // Normalize the vector
            let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for j in 0..vocab_size {
                    vector[j] /= norm;
                }
            }

            // Store in matrix
            for j in 0..vocab_size {
                transformation_matrix[[i, j]] = vector[j];
            }
        }

        self.transformation_matrix = Some(transformation_matrix);
        self.fitted = true;

        Ok(())
    }
}

impl EmbeddingProvider for SvdEmbedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if !self.fitted {
            return Err(VectorizerError::Other(
                "SVD embedding not fitted. Call fit_svd first.".to_string(),
            ));
        }

        // Get TF-IDF embedding
        let tfidf_embedding = self.tfidf.embed(text)?;

        // Apply transformation: result = tfidf_vector * V^T_reduced.
        // `fit_svd` always populates `transformation_matrix`; surface the
        // missing-matrix case as an error rather than panic in case the
        // invariant gets broken by a future refactor.
        let vt = self.transformation_matrix.as_ref().ok_or_else(|| {
            VectorizerError::Other("SVD transformation matrix missing after fit".to_string())
        })?;
        let mut result = vec![0.0f32; self.reduced_dimension];

        // Manual matrix multiplication for simplicity
        for i in 0..self.reduced_dimension {
            for j in 0..tfidf_embedding.len() {
                result[i] += tfidf_embedding[j] * vt[[i, j]];
            }
        }

        Ok(result)
    }

    fn dimension(&self) -> usize {
        self.reduced_dimension
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

//! Top-K Selection on Device
//! 
//! This module implements optimized Top-K selection that can run on GPU
//! to avoid expensive host-device transfers.

use crate::error::{Result, VectorizerError};
use tracing::{debug, info, warn};
use rayon::prelude::*;

/// Top-K selector for device operations
pub struct DeviceTopKSelector {
    device_available: bool,
    device_id: usize,
}

impl DeviceTopKSelector {
    /// Create new device Top-K selector
    pub fn new(device_id: usize) -> Result<Self> {
        debug!("Initializing device Top-K selector on device {}", device_id);
        
        // For now, always use CPU simulation until CUDA is properly set up
        let device_available = false;
        
        info!("Device Top-K selector initialized successfully on device {} (CPU simulation with optimized Top-K selection)", device_id);
        
        Ok(Self {
            device_available,
            device_id,
        })
    }

    /// Select Top-K elements from similarity scores
    pub fn select_top_k(
        &self,
        similarities: &[Vec<f32>],
        k: usize,
    ) -> Result<Vec<Vec<(usize, f32)>>> {
        debug!("Selecting Top-{} from {} similarity vectors", k, similarities.len());
        
        if similarities.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::with_capacity(similarities.len());
        
        // Use parallel processing for Top-K selection
        let top_k_results: Vec<Vec<(usize, f32)>> = similarities
            .par_iter()
            .map(|similarities| {
                self.select_top_k_single(similarities, k)
            })
            .collect();
        
        results.extend(top_k_results);
        
        debug!("Device Top-K selection completed for {} vectors", similarities.len());
        Ok(results)
    }

    /// Select Top-K from a single similarity vector
    fn select_top_k_single(&self, similarities: &[f32], k: usize) -> Vec<(usize, f32)> {
        if similarities.is_empty() {
            return vec![];
        }

        let k = k.min(similarities.len());
        
        // Create indexed pairs (index, similarity)
        let mut indexed: Vec<(usize, f32)> = similarities
            .iter()
            .enumerate()
            .map(|(idx, &sim)| (idx, sim))
            .collect();
        
        // Use partial sort for better performance when k << n
        if k < similarities.len() / 2 {
            // Partial sort - more efficient for small k
            indexed.select_nth_unstable_by(k - 1, |a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            indexed.truncate(k);
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            // Full sort - more efficient for large k
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            indexed.truncate(k);
        }
        
        indexed
    }

    /// Select Top-K with indices and return only indices
    pub fn select_top_k_indices(
        &self,
        similarities: &[Vec<f32>],
        k: usize,
    ) -> Result<Vec<Vec<usize>>> {
        let top_k_results = self.select_top_k(similarities, k)?;
        
        Ok(top_k_results
            .into_iter()
            .map(|results| results.into_iter().map(|(idx, _)| idx).collect())
            .collect())
    }

    /// Select Top-K with indices and return only scores
    pub fn select_top_k_scores(
        &self,
        similarities: &[Vec<f32>],
        k: usize,
    ) -> Result<Vec<Vec<f32>>> {
        let top_k_results = self.select_top_k(similarities, k)?;
        
        Ok(top_k_results
            .into_iter()
            .map(|results| results.into_iter().map(|(_, score)| score).collect())
            .collect())
    }

    /// Batch Top-K selection with custom comparison function
    pub fn select_top_k_custom<F>(
        &self,
        similarities: &[Vec<f32>],
        k: usize,
        compare_fn: F,
    ) -> Result<Vec<Vec<(usize, f32)>>>
    where
        F: Fn(f32, f32) -> std::cmp::Ordering + Sync + Send,
    {
        debug!("Selecting Top-{} with custom comparison from {} similarity vectors", k, similarities.len());
        
        if similarities.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::with_capacity(similarities.len());
        
        // Use parallel processing for Top-K selection
        let top_k_results: Vec<Vec<(usize, f32)>> = similarities
            .par_iter()
            .map(|similarities| {
                self.select_top_k_single_custom(similarities, k, &compare_fn)
            })
            .collect();
        
        results.extend(top_k_results);
        
        debug!("Device Top-K selection with custom comparison completed for {} vectors", similarities.len());
        Ok(results)
    }

    /// Select Top-K from a single similarity vector with custom comparison
    fn select_top_k_single_custom<F>(
        &self,
        similarities: &[f32],
        k: usize,
        compare_fn: &F,
    ) -> Vec<(usize, f32)>
    where
        F: Fn(f32, f32) -> std::cmp::Ordering,
    {
        if similarities.is_empty() {
            return vec![];
        }

        let k = k.min(similarities.len());
        
        // Create indexed pairs (index, similarity)
        let mut indexed: Vec<(usize, f32)> = similarities
            .iter()
            .enumerate()
            .map(|(idx, &sim)| (idx, sim))
            .collect();
        
        // Sort with custom comparison function
        indexed.sort_by(|a, b| compare_fn(a.1, b.1));
        indexed.truncate(k);
        
        indexed
    }

    /// Get device information
    pub fn get_device_info(&self) -> String {
        if self.device_available {
            format!("Top-K Device {} (GPU)", self.device_id)
        } else {
            format!("Top-K Device {} (CPU Optimized)", self.device_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_top_k_selector_creation() {
        let selector = DeviceTopKSelector::new(0).unwrap();
        assert_eq!(selector.device_id, 0);
    }

    #[test]
    fn test_select_top_k() {
        let selector = DeviceTopKSelector::new(0).unwrap();
        let similarities = vec![
            vec![0.1, 0.9, 0.3, 0.8, 0.2],
            vec![0.5, 0.1, 0.7, 0.4, 0.6],
        ];
        
        let results = selector.select_top_k(&similarities, 3).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 3);
        assert_eq!(results[1].len(), 3);
        
        // Check that results are sorted by similarity (descending)
        assert!(results[0][0].1 >= results[0][1].1);
        assert!(results[0][1].1 >= results[0][2].1);
        assert!(results[1][0].1 >= results[1][1].1);
        assert!(results[1][1].1 >= results[1][2].1);
        
        // Check specific values
        assert_eq!(results[0][0].1, 0.9); // Highest similarity in first vector
        assert_eq!(results[1][0].1, 0.7); // Highest similarity in second vector
    }

    #[test]
    fn test_select_top_k_indices() {
        let selector = DeviceTopKSelector::new(0).unwrap();
        let similarities = vec![vec![0.1, 0.9, 0.3, 0.8, 0.2]];
        
        let results = selector.select_top_k_indices(&similarities, 3).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 3);
        
        // Check that indices correspond to top similarities
        assert_eq!(results[0][0], 1); // Index of 0.9
        assert_eq!(results[0][1], 3); // Index of 0.8
        assert_eq!(results[0][2], 2); // Index of 0.3
    }

    #[test]
    fn test_select_top_k_scores() {
        let selector = DeviceTopKSelector::new(0).unwrap();
        let similarities = vec![vec![0.1, 0.9, 0.3, 0.8, 0.2]];
        
        let results = selector.select_top_k_scores(&similarities, 3).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 3);
        
        // Check that scores are sorted (descending)
        assert_eq!(results[0][0], 0.9);
        assert_eq!(results[0][1], 0.8);
        assert_eq!(results[0][2], 0.3);
    }

    #[test]
    fn test_select_top_k_custom() {
        let selector = DeviceTopKSelector::new(0).unwrap();
        let similarities = vec![vec![0.1, 0.9, 0.3, 0.8, 0.2]];
        
        // Use ascending order (smallest first)
        let results = selector.select_top_k_custom(&similarities, 3, |a, b| a.partial_cmp(&b).unwrap()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 3);
        
        // Check that results are sorted by similarity (ascending)
        assert!(results[0][0].1 <= results[0][1].1);
        assert!(results[0][1].1 <= results[0][2].1);
        
        // Check specific values
        assert_eq!(results[0][0].1, 0.1); // Smallest similarity
        assert_eq!(results[0][1].1, 0.2); // Second smallest
        assert_eq!(results[0][2].1, 0.3); // Third smallest
    }
}

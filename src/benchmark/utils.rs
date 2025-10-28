//! Benchmark Utility Functions
//!
//! Provides common utility functions used across benchmark modules.

use std::time::{Duration, Instant};

/// Utility functions for benchmarks
pub mod utils {
    use super::*;

    /// Format duration as human-readable string
    pub fn format_duration(duration: Duration) -> String {
        if duration.as_secs() > 0 {
            format!("{:.2}s", duration.as_secs_f64())
        } else if duration.as_millis() > 0 {
            format!("{:.2}ms", duration.as_millis() as f64)
        } else if duration.as_micros() > 0 {
            format!("{:.2}μs", duration.as_micros() as f64)
        } else {
            format!("{:.2}ns", duration.as_nanos() as f64)
        }
    }

    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Format throughput as human-readable string
    pub fn format_throughput(ops_per_sec: f64) -> String {
        if ops_per_sec >= 1_000_000.0 {
            format!("{:.2}M ops/s", ops_per_sec / 1_000_000.0)
        } else if ops_per_sec >= 1_000.0 {
            format!("{:.2}K ops/s", ops_per_sec / 1_000.0)
        } else {
            format!("{:.2} ops/s", ops_per_sec)
        }
    }

    /// Calculate percentage change
    pub fn percentage_change(old_value: f64, new_value: f64) -> f64 {
        if old_value == 0.0 {
            if new_value == 0.0 { 0.0 } else { 100.0 }
        } else {
            ((new_value - old_value) / old_value) * 100.0
        }
    }

    /// Calculate speedup ratio
    pub fn speedup_ratio(old_time: f64, new_time: f64) -> f64 {
        if new_time == 0.0 {
            f64::INFINITY
        } else {
            old_time / new_time
        }
    }

    /// Calculate efficiency (speedup / number of threads)
    pub fn efficiency(speedup: f64, num_threads: usize) -> f64 {
        if num_threads == 0 {
            0.0
        } else {
            speedup / num_threads as f64
        }
    }

    /// Calculate coefficient of variation (standard deviation / mean)
    pub fn coefficient_of_variation(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if mean == 0.0 { 0.0 } else { std_dev / mean }
    }

    /// Calculate confidence interval for a set of values
    pub fn confidence_interval(values: &[f64], confidence: f64) -> (f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0);
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
        let std_dev = variance.sqrt();

        // Simplified confidence interval calculation
        // In practice, you might want to use t-distribution for small samples
        let margin = 1.96 * std_dev / (values.len() as f64).sqrt(); // 95% confidence

        (mean - margin, mean + margin)
    }

    /// Calculate percentile rank
    pub fn percentile_rank(values: &[f64], value: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let count_below = values.iter().filter(|&&x| x < value).count();
        (count_below as f64 / values.len() as f64) * 100.0
    }

    /// Calculate interquartile range (IQR)
    pub fn interquartile_range(values: &[f64]) -> f64 {
        if values.len() < 4 {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_idx = sorted.len() / 4;
        let q3_idx = 3 * sorted.len() / 4;

        sorted[q3_idx] - sorted[q1_idx]
    }

    /// Detect outliers using IQR method
    pub fn detect_outliers(values: &[f64]) -> Vec<usize> {
        if values.len() < 4 {
            return Vec::new();
        }

        let mut sorted_values: Vec<(usize, f64)> =
            values.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        sorted_values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let q1_idx = sorted_values.len() / 4;
        let q3_idx = 3 * sorted_values.len() / 4;
        let q1 = sorted_values[q1_idx].1;
        let q3 = sorted_values[q3_idx].1;
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        sorted_values
            .iter()
            .filter(|(_, value)| *value < lower_bound || *value > upper_bound)
            .map(|(original_idx, _)| *original_idx)
            .collect()
    }

    /// Calculate moving average
    pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
        if values.is_empty() || window_size == 0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        let window_size = window_size.min(values.len());

        for i in 0..=values.len().saturating_sub(window_size) {
            let window = &values[i..i + window_size];
            let avg = window.iter().sum::<f64>() / window.len() as f64;
            result.push(avg);
        }

        result
    }

    /// Calculate exponential moving average
    pub fn exponential_moving_average(values: &[f64], alpha: f64) -> Vec<f64> {
        if values.is_empty() || alpha <= 0.0 || alpha >= 1.0 {
            return values.to_vec();
        }

        let mut result = Vec::with_capacity(values.len());
        result.push(values[0]);

        for i in 1..values.len() {
            let ema = alpha * values[i] + (1.0 - alpha) * result[i - 1];
            result.push(ema);
        }

        result
    }

    /// Calculate trend line using linear regression
    pub fn linear_trend(values: &[f64]) -> (f64, f64) {
        if values.len() < 2 {
            return (0.0, 0.0);
        }

        let n = values.len() as f64;
        let x_sum: f64 = (0..values.len()).map(|i| i as f64).sum();
        let y_sum: f64 = values.iter().sum();
        let xy_sum: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x2_sum: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum * x_sum);
        let intercept = (y_sum - slope * x_sum) / n;

        (slope, intercept)
    }

    /// Calculate correlation coefficient
    pub fn correlation_coefficient(x_values: &[f64], y_values: &[f64]) -> f64 {
        if x_values.len() != y_values.len() || x_values.len() < 2 {
            return 0.0;
        }

        let n = x_values.len() as f64;
        let x_mean = x_values.iter().sum::<f64>() / n;
        let y_mean = y_values.iter().sum::<f64>() / n;

        let numerator: f64 = x_values
            .iter()
            .zip(y_values.iter())
            .map(|(&x, &y)| (x - x_mean) * (y - y_mean))
            .sum();

        let x_variance: f64 = x_values.iter().map(|&x| (x - x_mean).powi(2)).sum();
        let y_variance: f64 = y_values.iter().map(|&y| (y - y_mean).powi(2)).sum();

        let denominator = (x_variance * y_variance).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// Benchmark timing utilities
pub mod timing {
    use super::*;

    /// Timer for measuring operation duration
    pub struct Timer {
        start_time: Option<Instant>,
    }

    impl Timer {
        /// Create new timer
        pub fn new() -> Self {
            Self { start_time: None }
        }

        /// Start timing
        pub fn start(&mut self) {
            self.start_time = Some(Instant::now());
        }

        /// Stop timing and return duration
        pub fn stop(&mut self) -> Option<Duration> {
            self.start_time.take().map(|start| start.elapsed())
        }

        /// Get elapsed time without stopping
        pub fn elapsed(&self) -> Option<Duration> {
            self.start_time.map(|start| start.elapsed())
        }

        /// Check if timer is running
        pub fn is_running(&self) -> bool {
            self.start_time.is_some()
        }
    }

    impl Default for Timer {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Measure execution time of a closure
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure execution time of an async closure
    pub async fn measure_time_async<F, Fut, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }
}

/// Memory utilities
pub mod memory {
    /// Get current process memory usage in bytes
    pub fn get_process_memory() -> u64 {
        // This is a simplified implementation
        // In practice, you might want to use platform-specific APIs
        0
    }

    /// Get peak memory usage in bytes
    pub fn get_peak_memory() -> u64 {
        // This is a simplified implementation
        0
    }

    /// Force garbage collection (if applicable)
    pub fn force_gc() {
        // This is a placeholder for languages that support explicit GC
        // Rust doesn't have explicit GC, but we can suggest the allocator to return memory
    }

    /// Allocate memory for testing
    pub fn allocate_memory(size: usize) -> Vec<u8> {
        vec![0u8; size]
    }

    /// Deallocate memory
    pub fn deallocate_memory(_memory: Vec<u8>) {
        // Memory is automatically deallocated when the Vec goes out of scope
    }
}

/// File utilities
pub mod file {
    use std::fs;
    use std::path::Path;

    /// Ensure directory exists
    pub fn ensure_dir_exists(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Get file size in bytes
    pub fn get_file_size(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// Check if file exists
    pub fn file_exists(path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    /// Get file extension
    pub fn get_file_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
    }
}

/// Math utilities
pub mod math {
    /// Calculate mean of values
    pub fn mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    /// Calculate median of values
    pub fn median(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Calculate standard deviation
    pub fn standard_deviation(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = mean(values);
        let variance =
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
        variance.sqrt()
    }

    /// Calculate variance
    pub fn variance(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = mean(values);
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64
    }

    /// Calculate skewness
    pub fn skewness(values: &[f64]) -> f64 {
        if values.len() < 3 {
            return 0.0;
        }

        let mean = mean(values);
        let std_dev = standard_deviation(values);

        if std_dev == 0.0 {
            return 0.0;
        }

        let n = values.len() as f64;
        let skewness = values
            .iter()
            .map(|x| ((x - mean) / std_dev).powi(3))
            .sum::<f64>()
            / n;

        skewness
    }

    /// Calculate kurtosis
    pub fn kurtosis(values: &[f64]) -> f64 {
        if values.len() < 4 {
            return 0.0;
        }

        let mean = mean(values);
        let std_dev = standard_deviation(values);

        if std_dev == 0.0 {
            return 0.0;
        }

        let n = values.len() as f64;
        let kurtosis = values
            .iter()
            .map(|x| ((x - mean) / std_dev).powi(4))
            .sum::<f64>()
            / n
            - 3.0;

        kurtosis
    }
}

// Re-export commonly used utilities
pub use file::*;
pub use math::*;
pub use memory::*;
pub use timing::*;
pub use utils::*;

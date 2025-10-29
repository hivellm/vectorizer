//! Benchmark Configuration
//!
//! Provides configuration management for benchmarks including test parameters,
//! measurement settings, and output options.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Benchmark configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Vector dimensions to test
    pub dimensions: Vec<usize>,
    /// Vector counts to test
    pub vector_counts: Vec<usize>,
    /// Measurement time per benchmark
    pub measurement_time: Duration,
    /// Number of samples to collect
    pub sample_size: usize,
    /// Warm-up time before measurement
    pub warm_up_time: Duration,
    /// Number of measurement iterations
    pub measurement_iterations: usize,
    /// Enable memory tracking
    pub track_memory: bool,
    /// Memory sampling interval in milliseconds
    pub memory_sampling_interval: u64,
    /// Enable HTML reports
    pub html_reports: bool,
    /// Enable JSON reports
    pub json_reports: bool,
    /// Enable CSV reports
    pub csv_reports: bool,
    /// Output directory for reports
    pub output_directory: String,
    /// HNSW configuration parameters
    pub hnsw_config: HnswBenchmarkConfig,
    /// Quantization settings
    pub quantization_config: QuantizationBenchmarkConfig,
    /// GPU settings
    pub gpu_config: GpuBenchmarkConfig,
}

/// HNSW-specific benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswBenchmarkConfig {
    /// Maximum connections per node
    pub max_connections: usize,
    /// Maximum connections for level 0
    pub max_connections_0: usize,
    /// Construction parameter
    pub ef_construction: usize,
    /// Search parameter
    pub ef_search: usize,
    /// Enable parallel processing
    pub parallel: bool,
    /// Initial capacity
    pub initial_capacity: usize,
    /// Batch size for operations
    pub batch_size: usize,
}

/// Quantization benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationBenchmarkConfig {
    /// Enable scalar quantization tests
    pub test_scalar_quantization: bool,
    /// Scalar quantization bit depths to test
    pub scalar_bits: Vec<usize>,
    /// Enable product quantization tests
    pub test_product_quantization: bool,
    /// Product quantization subquantizer counts
    pub pq_subquantizers: Vec<usize>,
    /// Product quantization centroid counts
    pub pq_centroids: Vec<usize>,
    /// Enable binary quantization tests
    pub test_binary_quantization: bool,
}

/// GPU benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuBenchmarkConfig {
    /// Enable Metal GPU tests (macOS)
    pub enable_metal: bool,
    /// Enable CUDA GPU tests (NVIDIA)
    pub enable_cuda: bool,
    /// Warm-up iterations for GPU
    pub gpu_warm_up_iterations: usize,
    /// Measurement iterations for GPU
    pub gpu_measurement_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            dimensions: vec![64, 128, 256, 512, 768, 1024],
            vector_counts: vec![100, 1000, 10000, 100000],
            measurement_time: Duration::from_secs(10),
            sample_size: 100,
            warm_up_time: Duration::from_secs(2),
            measurement_iterations: 10,
            track_memory: true,
            memory_sampling_interval: 100,
            html_reports: true,
            json_reports: true,
            csv_reports: false,
            output_directory: "target/criterion".to_string(),
            hnsw_config: HnswBenchmarkConfig::default(),
            quantization_config: QuantizationBenchmarkConfig::default(),
            gpu_config: GpuBenchmarkConfig::default(),
        }
    }
}

impl Default for HnswBenchmarkConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            max_connections_0: 32,
            ef_construction: 200,
            ef_search: 50,
            parallel: true,
            initial_capacity: 10000,
            batch_size: 1000,
        }
    }
}

impl Default for QuantizationBenchmarkConfig {
    fn default() -> Self {
        Self {
            test_scalar_quantization: true,
            scalar_bits: vec![4, 8],
            test_product_quantization: true,
            pq_subquantizers: vec![8, 16],
            pq_centroids: vec![256, 512],
            test_binary_quantization: true,
        }
    }
}

impl Default for GpuBenchmarkConfig {
    fn default() -> Self {
        Self {
            enable_metal: true,
            enable_cuda: true,
            gpu_warm_up_iterations: 5,
            gpu_measurement_iterations: 20,
        }
    }
}

impl BenchmarkConfig {
    /// Create a new benchmark configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set vector dimensions to test
    pub fn with_dimensions(mut self, dimensions: Vec<usize>) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// Set vector counts to test
    pub fn with_vector_counts(mut self, vector_counts: Vec<usize>) -> Self {
        self.vector_counts = vector_counts;
        self
    }

    /// Set measurement time
    pub fn with_measurement_time(mut self, measurement_time: Duration) -> Self {
        self.measurement_time = measurement_time;
        self
    }

    /// Set sample size
    pub fn with_sample_size(mut self, sample_size: usize) -> Self {
        self.sample_size = sample_size;
        self
    }

    /// Set warm-up time
    pub fn with_warm_up_time(mut self, warm_up_time: Duration) -> Self {
        self.warm_up_time = warm_up_time;
        self
    }

    /// Set measurement iterations
    pub fn with_measurement_iterations(mut self, measurement_iterations: usize) -> Self {
        self.measurement_iterations = measurement_iterations;
        self
    }

    /// Enable or disable memory tracking
    pub fn with_memory_tracking(mut self, track_memory: bool) -> Self {
        self.track_memory = track_memory;
        self
    }

    /// Set memory sampling interval
    pub fn with_memory_sampling_interval(mut self, interval_ms: u64) -> Self {
        self.memory_sampling_interval = interval_ms;
        self
    }

    /// Set output directory
    pub fn with_output_directory(mut self, output_directory: String) -> Self {
        self.output_directory = output_directory;
        self
    }

    /// Configure HNSW parameters
    pub fn with_hnsw_config(mut self, hnsw_config: HnswBenchmarkConfig) -> Self {
        self.hnsw_config = hnsw_config;
        self
    }

    /// Configure quantization settings
    pub fn with_quantization_config(
        mut self,
        quantization_config: QuantizationBenchmarkConfig,
    ) -> Self {
        self.quantization_config = quantization_config;
        self
    }

    /// Configure GPU settings
    pub fn with_gpu_config(mut self, gpu_config: GpuBenchmarkConfig) -> Self {
        self.gpu_config = gpu_config;
        self
    }

    /// Create a quick benchmark configuration for development
    pub fn quick() -> Self {
        Self::default()
            .with_measurement_time(Duration::from_secs(3))
            .with_sample_size(10)
            .with_warm_up_time(Duration::from_secs(1))
            .with_vector_counts(vec![100, 1000])
            .with_dimensions(vec![128, 256])
    }

    /// Create a comprehensive benchmark configuration for CI/CD
    pub fn comprehensive() -> Self {
        Self::default()
            .with_measurement_time(Duration::from_secs(30))
            .with_sample_size(200)
            .with_warm_up_time(Duration::from_secs(5))
            .with_vector_counts(vec![100, 1000, 10000, 100000])
            .with_dimensions(vec![64, 128, 256, 512, 768, 1024])
    }

    /// Create a regression test configuration
    pub fn regression() -> Self {
        Self::default()
            .with_measurement_time(Duration::from_secs(60))
            .with_sample_size(500)
            .with_warm_up_time(Duration::from_secs(10))
            .with_vector_counts(vec![1000, 10000, 100000, 1000000])
            .with_dimensions(vec![128, 256, 512, 768, 1024, 1536])
    }

    /// Load configuration from TOML file
    pub fn from_toml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_toml_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.dimensions.is_empty() {
            return Err("At least one dimension must be specified".to_string());
        }

        if self.vector_counts.is_empty() {
            return Err("At least one vector count must be specified".to_string());
        }

        if self.sample_size == 0 {
            return Err("Sample size must be greater than 0".to_string());
        }

        if self.measurement_iterations == 0 {
            return Err("Measurement iterations must be greater than 0".to_string());
        }

        if self.hnsw_config.max_connections == 0 {
            return Err("HNSW max_connections must be greater than 0".to_string());
        }

        if self.hnsw_config.ef_construction == 0 {
            return Err("HNSW ef_construction must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Benchmark profile types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkProfile {
    /// Quick benchmarks for development
    Quick,
    /// Comprehensive benchmarks for CI/CD
    Comprehensive,
    /// Regression testing benchmarks
    Regression,
}

impl BenchmarkProfile {
    /// Get configuration for this profile
    pub fn config(&self) -> BenchmarkConfig {
        match self {
            BenchmarkProfile::Quick => BenchmarkConfig::quick(),
            BenchmarkProfile::Comprehensive => BenchmarkConfig::comprehensive(),
            BenchmarkProfile::Regression => BenchmarkConfig::regression(),
        }
    }

    /// Parse profile from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "quick" => Ok(BenchmarkProfile::Quick),
            "comprehensive" => Ok(BenchmarkProfile::Comprehensive),
            "regression" => Ok(BenchmarkProfile::Regression),
            _ => Err(format!("Unknown benchmark profile: {}", s)),
        }
    }
}

impl std::fmt::Display for BenchmarkProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchmarkProfile::Quick => write!(f, "quick"),
            BenchmarkProfile::Comprehensive => write!(f, "comprehensive"),
            BenchmarkProfile::Regression => write!(f, "regression"),
        }
    }
}

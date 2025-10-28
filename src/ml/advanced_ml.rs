//! Advanced machine learning integration
//!
//! Provides sophisticated ML capabilities including:
//! - Multiple embedding model support
//! - Model fine-tuning and adaptation
//! - Transfer learning and domain adaptation
//! - Model versioning and management
//! - A/B testing for models
//! - Model performance monitoring
//! - Automated model selection

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};

use crate::error::VectorizerError;

/// Advanced ML manager
#[derive(Debug, Clone)]
pub struct AdvancedMlManager {
    /// ML configuration
    config: MlConfig,

    /// Model registry
    model_registry: Arc<RwLock<ModelRegistry>>,

    /// Model trainer
    model_trainer: Arc<ModelTrainer>,

    /// Model evaluator
    model_evaluator: Arc<ModelEvaluator>,

    /// Model monitor
    model_monitor: Arc<ModelMonitor>,

    /// A/B testing manager
    ab_testing_manager: Arc<AbTestingManager>,
}

/// ML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    /// Model configuration
    pub models: ModelConfig,

    /// Training configuration
    pub training: TrainingConfig,

    /// Evaluation configuration
    pub evaluation: EvaluationConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,

    /// A/B testing configuration
    pub ab_testing: AbTestingConfig,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Default model
    pub default_model: String,

    /// Available models
    pub available_models: Vec<ModelInfo>,

    /// Model selection strategy
    pub selection_strategy: ModelSelectionStrategy,

    /// Model caching
    pub caching: ModelCachingConfig,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,

    /// Model name
    pub name: String,

    /// Model type
    pub model_type: ModelType,

    /// Model version
    pub version: String,

    /// Model description
    pub description: String,

    /// Model parameters
    pub parameters: ModelParameters,

    /// Model performance
    pub performance: ModelPerformance,

    /// Model status
    pub status: ModelStatus,
}

/// Model types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Embedding model
    Embedding,

    /// Classification model
    Classification,

    /// Clustering model
    Clustering,

    /// Anomaly detection model
    AnomalyDetection,

    /// Recommendation model
    Recommendation,
}

/// Model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// Model architecture
    pub architecture: String,

    /// Model size
    pub size_mb: f64,

    /// Input dimensions
    pub input_dimensions: usize,

    /// Output dimensions
    pub output_dimensions: usize,

    /// Model hyperparameters
    pub hyperparameters: HashMap<String, serde_json::Value>,
}

/// Model performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Accuracy
    pub accuracy: f64,

    /// Precision
    pub precision: f64,

    /// Recall
    pub recall: f64,

    /// F1 score
    pub f1_score: f64,

    /// Inference time
    pub inference_time_ms: f64,

    /// Memory usage
    pub memory_usage_mb: f64,
}

/// Model status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model is ready
    Ready,

    /// Model is training
    Training,

    /// Model is evaluating
    Evaluating,

    /// Model is deployed
    Deployed,

    /// Model is deprecated
    Deprecated,

    /// Model has error
    Error,
}

/// Model selection strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelSelectionStrategy {
    /// Select best performing model
    BestPerformance,

    /// Select fastest model
    Fastest,

    /// Select smallest model
    Smallest,

    /// Random selection
    Random,

    /// Custom selection
    Custom(String),
}

/// Model caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCachingConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache size
    pub cache_size: usize,

    /// Cache TTL
    pub cache_ttl_seconds: u64,

    /// Cache eviction policy
    pub eviction_policy: CacheEvictionPolicy,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    /// Least recently used
    Lru,

    /// Least frequently used
    Lfu,

    /// First in first out
    Fifo,
}

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Enable training
    pub enabled: bool,

    /// Training data source
    pub data_source: DataSource,

    /// Training parameters
    pub parameters: TrainingParameters,

    /// Training schedule
    pub schedule: TrainingSchedule,

    /// Training monitoring
    pub monitoring: TrainingMonitoringConfig,
}

/// Data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Source type
    pub source_type: DataSourceType,

    /// Source configuration
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Data source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceType {
    /// File system
    FileSystem,

    /// Database
    Database,

    /// API
    Api,

    /// Message queue
    MessageQueue,
}

/// Training parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingParameters {
    /// Learning rate
    pub learning_rate: f64,

    /// Batch size
    pub batch_size: usize,

    /// Number of epochs
    pub epochs: u32,

    /// Optimizer
    pub optimizer: Optimizer,

    /// Loss function
    pub loss_function: LossFunction,

    /// Regularization
    pub regularization: RegularizationConfig,
}

/// Optimizers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Optimizer {
    /// Adam optimizer
    Adam,

    /// SGD optimizer
    Sgd,

    /// RMSprop optimizer
    Rmsprop,

    /// AdaGrad optimizer
    Adagrad,
}

/// Loss functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LossFunction {
    /// Mean squared error
    Mse,

    /// Cross entropy
    CrossEntropy,

    /// Hinge loss
    Hinge,

    /// Custom loss
    Custom(String),
}

/// Regularization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularizationConfig {
    /// L1 regularization
    pub l1: f64,

    /// L2 regularization
    pub l2: f64,

    /// Dropout rate
    pub dropout: f64,
}

/// Training schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSchedule {
    /// Schedule type
    pub schedule_type: ScheduleType,

    /// Schedule configuration
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// Manual training
    Manual,

    /// Scheduled training
    Scheduled,

    /// Continuous training
    Continuous,
}

/// Training monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,

    /// Monitoring metrics
    pub metrics: Vec<TrainingMetric>,

    /// Monitoring interval
    pub interval_seconds: u64,
}

/// Training metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingMetric {
    /// Loss
    Loss,

    /// Accuracy
    Accuracy,

    /// Learning rate
    LearningRate,

    /// Gradient norm
    GradientNorm,
}

/// Evaluation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// Enable evaluation
    pub enabled: bool,

    /// Evaluation dataset
    pub dataset: EvaluationDataset,

    /// Evaluation metrics
    pub metrics: Vec<EvaluationMetric>,

    /// Evaluation schedule
    pub schedule: EvaluationSchedule,
}

/// Evaluation dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationDataset {
    /// Dataset name
    pub name: String,

    /// Dataset source
    pub source: DataSource,

    /// Dataset split
    pub split: DatasetSplit,
}

/// Dataset split
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSplit {
    /// Training split
    pub training: f64,

    /// Validation split
    pub validation: f64,

    /// Test split
    pub test: f64,
}

/// Evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationMetric {
    /// Accuracy
    Accuracy,

    /// Precision
    Precision,

    /// Recall
    Recall,

    /// F1 score
    F1Score,

    /// AUC
    Auc,

    /// Custom metric
    Custom(String),
}

/// Evaluation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSchedule {
    /// Schedule type
    pub schedule_type: EvaluationScheduleType,

    /// Schedule configuration
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Evaluation schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationScheduleType {
    /// Manual evaluation
    Manual,

    /// Scheduled evaluation
    Scheduled,

    /// Continuous evaluation
    Continuous,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,

    /// Monitoring metrics
    pub metrics: Vec<MonitoringMetric>,

    /// Monitoring alerts
    pub alerts: Vec<MonitoringAlert>,
}

/// Monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringMetric {
    /// Model accuracy
    ModelAccuracy,

    /// Model latency
    ModelLatency,

    /// Model throughput
    ModelThroughput,

    /// Model memory usage
    ModelMemoryUsage,

    /// Model CPU usage
    ModelCpuUsage,
}

/// Monitoring alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringAlert {
    /// Alert name
    pub name: String,

    /// Alert condition
    pub condition: AlertCondition,

    /// Alert threshold
    pub threshold: f64,

    /// Alert actions
    pub actions: Vec<AlertAction>,
}

/// Alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Greater than
    GreaterThan,

    /// Less than
    LessThan,

    /// Equal to
    EqualTo,

    /// Not equal to
    NotEqualTo,
}

/// Alert actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    /// Send email
    SendEmail,

    /// Send notification
    SendNotification,

    /// Log alert
    LogAlert,

    /// Execute script
    ExecuteScript,
}

/// A/B testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestingConfig {
    /// Enable A/B testing
    pub enabled: bool,

    /// A/B tests
    pub tests: Vec<AbTest>,

    /// A/B testing strategy
    pub strategy: AbTestingStrategy,
}

/// A/B test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTest {
    /// Test ID
    pub id: String,

    /// Test name
    pub name: String,

    /// Test description
    pub description: String,

    /// Test variants
    pub variants: Vec<TestVariant>,

    /// Test traffic split
    pub traffic_split: Vec<f64>,

    /// Test duration
    pub duration_days: u32,

    /// Test status
    pub status: AbTestStatus,
}

/// Test variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVariant {
    /// Variant name
    pub name: String,

    /// Variant model
    pub model_id: String,

    /// Variant configuration
    pub configuration: HashMap<String, serde_json::Value>,
}

/// A/B test status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AbTestStatus {
    /// Test is running
    Running,

    /// Test is paused
    Paused,

    /// Test is completed
    Completed,

    /// Test is cancelled
    Cancelled,
}

/// A/B testing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbTestingStrategy {
    /// Random assignment
    Random,

    /// Weighted assignment
    Weighted,

    /// User-based assignment
    UserBased,
}

/// Model registry
#[derive(Debug)]
pub struct ModelRegistry {
    /// Registered models
    models: HashMap<String, ModelInfo>,

    /// Model versions
    versions: HashMap<String, Vec<String>>,
}

/// Model trainer
#[derive(Debug)]
pub struct ModelTrainer {
    /// Training configuration
    config: TrainingConfig,

    /// Training jobs
    jobs: Arc<RwLock<Vec<TrainingJob>>>,
}

/// Training job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    /// Job ID
    pub id: String,

    /// Job name
    pub name: String,

    /// Job status
    pub status: TrainingJobStatus,

    /// Job start time
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Job end time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Job progress
    pub progress: f64,

    /// Job metrics
    pub metrics: HashMap<String, f64>,
}

/// Training job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingJobStatus {
    /// Job is queued
    Queued,

    /// Job is running
    Running,

    /// Job is completed
    Completed,

    /// Job has failed
    Failed,

    /// Job is cancelled
    Cancelled,
}

/// Model evaluator
#[derive(Debug)]
pub struct ModelEvaluator {
    /// Evaluation configuration
    config: EvaluationConfig,

    /// Evaluation results
    results: Arc<RwLock<Vec<EvaluationResult>>>,
}

/// Evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Result ID
    pub id: String,

    /// Model ID
    pub model_id: String,

    /// Evaluation timestamp
    pub timestamp: u64,

    /// Evaluation metrics
    pub metrics: HashMap<String, f64>,

    /// Evaluation dataset
    pub dataset: String,
}

/// Model monitor
#[derive(Debug)]
pub struct ModelMonitor {
    /// Monitoring configuration
    config: MonitoringConfig,

    /// Monitoring data
    data: Arc<RwLock<Vec<MonitoringData>>>,
}

/// Monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    /// Data timestamp
    pub timestamp: u64,

    /// Model ID
    pub model_id: String,

    /// Metric name
    pub metric_name: String,

    /// Metric value
    pub metric_value: f64,
}

/// A/B testing manager
#[derive(Debug)]
pub struct AbTestingManager {
    /// A/B testing configuration
    config: AbTestingConfig,

    /// Active tests
    active_tests: Arc<RwLock<Vec<AbTest>>>,

    /// Test results
    test_results: Arc<RwLock<Vec<AbTestResult>>>,
}

/// A/B test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestResult {
    /// Test ID
    pub test_id: String,

    /// Variant name
    pub variant_name: String,

    /// Result metrics
    pub metrics: HashMap<String, f64>,

    /// Result timestamp
    pub timestamp: u64,
}

impl AdvancedMlManager {
    /// Create a new advanced ML manager
    pub fn new(config: MlConfig) -> Self {
        Self {
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            model_trainer: Arc::new(ModelTrainer::new(config.training.clone())),
            model_evaluator: Arc::new(ModelEvaluator::new(config.evaluation.clone())),
            model_monitor: Arc::new(ModelMonitor::new(config.monitoring.clone())),
            ab_testing_manager: Arc::new(AbTestingManager::new(config.ab_testing.clone())),
            config,
        }
    }

    /// Register a model
    pub async fn register_model(&self, model: ModelInfo) -> Result<()> {
        let mut registry = self.model_registry.write().unwrap();
        registry.register_model(model).await
    }

    /// Get model by ID
    pub async fn get_model(&self, model_id: &str) -> Result<Option<ModelInfo>> {
        let registry = self.model_registry.read().unwrap();
        Ok(registry.get_model(model_id).await)
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let registry = self.model_registry.read().unwrap();
        Ok(registry.list_models().await)
    }

    /// Train a model
    pub async fn train_model(&self, training_request: TrainingRequest) -> Result<TrainingJob> {
        self.model_trainer.train_model(training_request).await
    }

    /// Evaluate a model
    pub async fn evaluate_model(
        &self,
        evaluation_request: EvaluationRequest,
    ) -> Result<EvaluationResult> {
        self.model_evaluator
            .evaluate_model(evaluation_request)
            .await
    }

    /// Start A/B test
    pub async fn start_ab_test(&self, test: AbTest) -> Result<()> {
        self.ab_testing_manager.start_test(test).await
    }

    /// Get A/B test results
    pub async fn get_ab_test_results(&self, test_id: &str) -> Result<Vec<AbTestResult>> {
        self.ab_testing_manager.get_test_results(test_id).await
    }

    /// Get model recommendations
    pub async fn get_model_recommendations(
        &self,
        criteria: ModelSelectionCriteria,
    ) -> Result<Vec<ModelInfo>> {
        let registry = self.model_registry.read().unwrap();
        registry.get_recommendations(criteria).await
    }
}

impl ModelRegistry {
    /// Create new model registry
    fn new() -> Self {
        Self {
            models: HashMap::new(),
            versions: HashMap::new(),
        }
    }

    /// Register a model
    async fn register_model(&mut self, model: ModelInfo) -> Result<()> {
        let model_id = model.id.clone();
        let version = model.version.clone();

        self.models.insert(model_id.clone(), model);

        self.versions
            .entry(model_id)
            .or_insert_with(Vec::new)
            .push(version);

        Ok(())
    }

    /// Get model by ID
    async fn get_model(&self, model_id: &str) -> Option<ModelInfo> {
        self.models.get(model_id).cloned()
    }

    /// List all models
    async fn list_models(&self) -> Vec<ModelInfo> {
        self.models.values().cloned().collect()
    }

    /// Get model recommendations
    async fn get_recommendations(
        &self,
        criteria: ModelSelectionCriteria,
    ) -> Result<Vec<ModelInfo>> {
        let mut models: Vec<ModelInfo> = self.models.values().cloned().collect();

        // Apply selection criteria
        match criteria.strategy {
            ModelSelectionStrategy::BestPerformance => {
                models.sort_by(|a, b| {
                    b.performance
                        .accuracy
                        .partial_cmp(&a.performance.accuracy)
                        .unwrap()
                });
            }
            ModelSelectionStrategy::Fastest => {
                models.sort_by(|a, b| {
                    a.performance
                        .inference_time_ms
                        .partial_cmp(&b.performance.inference_time_ms)
                        .unwrap()
                });
            }
            ModelSelectionStrategy::Smallest => {
                models.sort_by(|a, b| {
                    a.parameters
                        .size_mb
                        .partial_cmp(&b.parameters.size_mb)
                        .unwrap()
                });
            }
            ModelSelectionStrategy::Random => {
                // Random selection would be implemented here
            }
            ModelSelectionStrategy::Custom(_) => {
                // Custom selection logic would be implemented here
            }
        }

        // Apply filters
        if let Some(model_type) = criteria.model_type {
            models.retain(|m| m.model_type == model_type);
        }

        if let Some(min_accuracy) = criteria.min_accuracy {
            models.retain(|m| m.performance.accuracy >= min_accuracy);
        }

        if let Some(max_latency) = criteria.max_latency {
            models.retain(|m| m.performance.inference_time_ms <= max_latency);
        }

        Ok(models)
    }
}

impl ModelTrainer {
    /// Create new model trainer
    fn new(config: TrainingConfig) -> Self {
        Self {
            config,
            jobs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Train a model
    async fn train_model(&self, request: TrainingRequest) -> Result<TrainingJob> {
        let job = TrainingJob {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            status: TrainingJobStatus::Queued,
            start_time: None,
            end_time: None,
            progress: 0.0,
            metrics: HashMap::new(),
        };

        // Add job to registry
        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.push(job.clone());
        }

        // Start training in background
        let jobs = self.jobs.clone();
        let job_id = job.id.clone();
        tokio::spawn(async move {
            // Training implementation would go here
            {
                let mut jobs = jobs.write().unwrap();
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                    job.status = TrainingJobStatus::Running;
                    job.start_time = Some(chrono::Utc::now());
                }
            } // Drop the lock here

            // Simulate training progress
            for i in 0..100 {
                {
                    let mut jobs = jobs.write().unwrap();
                    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                        job.progress = i as f64 / 100.0;
                    }
                } // Drop the lock before await
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            {
                let mut jobs = jobs.write().unwrap();
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                    job.status = TrainingJobStatus::Completed;
                    job.end_time = Some(chrono::Utc::now());
                }
            }
        });

        Ok(job)
    }
}

impl ModelEvaluator {
    /// Create new model evaluator
    fn new(config: EvaluationConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Evaluate a model
    async fn evaluate_model(&self, request: EvaluationRequest) -> Result<EvaluationResult> {
        let result = EvaluationResult {
            id: uuid::Uuid::new_v4().to_string(),
            model_id: request.model_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics: HashMap::new(),
            dataset: request.dataset,
        };

        // Add result to registry
        {
            let mut results = self.results.write().unwrap();
            results.push(result.clone());
        }

        Ok(result)
    }
}

impl ModelMonitor {
    /// Create new model monitor
    fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start monitoring
    async fn start_monitoring(&self) -> Result<()> {
        if self.config.enabled {
            // Start monitoring implementation
        }
        Ok(())
    }
}

impl AbTestingManager {
    /// Create new A/B testing manager
    fn new(config: AbTestingConfig) -> Self {
        Self {
            config,
            active_tests: Arc::new(RwLock::new(Vec::new())),
            test_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start A/B test
    async fn start_test(&self, test: AbTest) -> Result<()> {
        let mut active_tests = self.active_tests.write().unwrap();
        active_tests.push(test);
        Ok(())
    }

    /// Get test results
    async fn get_test_results(&self, test_id: &str) -> Result<Vec<AbTestResult>> {
        let results = self.test_results.read().unwrap();
        Ok(results
            .iter()
            .filter(|r| r.test_id == test_id)
            .cloned()
            .collect())
    }
}

/// Training request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRequest {
    /// Request name
    pub name: String,

    /// Model type
    pub model_type: ModelType,

    /// Training data
    pub training_data: DataSource,

    /// Training parameters
    pub parameters: TrainingParameters,
}

/// Evaluation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRequest {
    /// Model ID
    pub model_id: String,

    /// Evaluation dataset
    pub dataset: String,

    /// Evaluation metrics
    pub metrics: Vec<EvaluationMetric>,
}

/// Model selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionCriteria {
    /// Selection strategy
    pub strategy: ModelSelectionStrategy,

    /// Model type filter
    pub model_type: Option<ModelType>,

    /// Minimum accuracy
    pub min_accuracy: Option<f64>,

    /// Maximum latency
    pub max_latency: Option<f64>,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            models: ModelConfig {
                default_model: "default".to_string(),
                available_models: vec![],
                selection_strategy: ModelSelectionStrategy::BestPerformance,
                caching: ModelCachingConfig {
                    enabled: true,
                    cache_size: 10,
                    cache_ttl_seconds: 3600,
                    eviction_policy: CacheEvictionPolicy::Lru,
                },
            },
            training: TrainingConfig {
                enabled: true,
                data_source: DataSource {
                    source_type: DataSourceType::FileSystem,
                    configuration: HashMap::new(),
                },
                parameters: TrainingParameters {
                    learning_rate: 0.001,
                    batch_size: 32,
                    epochs: 10,
                    optimizer: Optimizer::Adam,
                    loss_function: LossFunction::Mse,
                    regularization: RegularizationConfig {
                        l1: 0.0,
                        l2: 0.01,
                        dropout: 0.1,
                    },
                },
                schedule: TrainingSchedule {
                    schedule_type: ScheduleType::Manual,
                    configuration: HashMap::new(),
                },
                monitoring: TrainingMonitoringConfig {
                    enabled: true,
                    metrics: vec![TrainingMetric::Loss, TrainingMetric::Accuracy],
                    interval_seconds: 60,
                },
            },
            evaluation: EvaluationConfig {
                enabled: true,
                dataset: EvaluationDataset {
                    name: "default".to_string(),
                    source: DataSource {
                        source_type: DataSourceType::FileSystem,
                        configuration: HashMap::new(),
                    },
                    split: DatasetSplit {
                        training: 0.7,
                        validation: 0.2,
                        test: 0.1,
                    },
                },
                metrics: vec![
                    EvaluationMetric::Accuracy,
                    EvaluationMetric::Precision,
                    EvaluationMetric::Recall,
                    EvaluationMetric::F1Score,
                ],
                schedule: EvaluationSchedule {
                    schedule_type: EvaluationScheduleType::Manual,
                    configuration: HashMap::new(),
                },
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics: vec![
                    MonitoringMetric::ModelAccuracy,
                    MonitoringMetric::ModelLatency,
                    MonitoringMetric::ModelThroughput,
                ],
                alerts: vec![],
            },
            ab_testing: AbTestingConfig {
                enabled: true,
                tests: vec![],
                strategy: AbTestingStrategy::Random,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_config_default() {
        let config = MlConfig::default();
        assert!(config.training.enabled);
        assert!(config.evaluation.enabled);
        assert!(config.monitoring.enabled);
        assert!(config.ab_testing.enabled);
    }

    #[test]
    fn test_model_info_creation() {
        let model = ModelInfo {
            id: "model1".to_string(),
            name: "Test Model".to_string(),
            model_type: ModelType::Embedding,
            version: "1.0.0".to_string(),
            description: "Test model".to_string(),
            parameters: ModelParameters {
                architecture: "transformer".to_string(),
                size_mb: 100.0,
                input_dimensions: 512,
                output_dimensions: 384,
                hyperparameters: HashMap::new(),
            },
            performance: ModelPerformance {
                accuracy: 0.95,
                precision: 0.94,
                recall: 0.96,
                f1_score: 0.95,
                inference_time_ms: 10.0,
                memory_usage_mb: 50.0,
            },
            status: ModelStatus::Ready,
        };

        assert_eq!(model.id, "model1");
        assert_eq!(model.name, "Test Model");
        assert_eq!(model.model_type, ModelType::Embedding);
        assert_eq!(model.performance.accuracy, 0.95);
    }

    #[test]
    fn test_training_parameters() {
        let params = TrainingParameters {
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 10,
            optimizer: Optimizer::Adam,
            loss_function: LossFunction::Mse,
            regularization: RegularizationConfig {
                l1: 0.0,
                l2: 0.01,
                dropout: 0.1,
            },
        };

        assert_eq!(params.learning_rate, 0.001);
        assert_eq!(params.batch_size, 32);
        assert_eq!(params.epochs, 10);
        assert_eq!(params.optimizer, Optimizer::Adam);
        assert_eq!(params.loss_function, LossFunction::Mse);
    }

    #[test]
    fn test_ab_test_creation() {
        let test = AbTest {
            id: "test1".to_string(),
            name: "Test A/B Test".to_string(),
            description: "Test A/B test".to_string(),
            variants: vec![],
            traffic_split: vec![0.5, 0.5],
            duration_days: 7,
            status: AbTestStatus::Running,
        };

        assert_eq!(test.id, "test1");
        assert_eq!(test.name, "Test A/B Test");
        assert_eq!(test.traffic_split, vec![0.5, 0.5]);
        assert_eq!(test.duration_days, 7);
        assert_eq!(test.status, AbTestStatus::Running);
    }

    #[test]
    fn test_model_selection_criteria() {
        let criteria = ModelSelectionCriteria {
            strategy: ModelSelectionStrategy::BestPerformance,
            model_type: Some(ModelType::Embedding),
            min_accuracy: Some(0.9),
            max_latency: Some(100.0),
        };

        assert_eq!(criteria.strategy, ModelSelectionStrategy::BestPerformance);
        assert_eq!(criteria.model_type, Some(ModelType::Embedding));
        assert_eq!(criteria.min_accuracy, Some(0.9));
        assert_eq!(criteria.max_latency, Some(100.0));
    }
}

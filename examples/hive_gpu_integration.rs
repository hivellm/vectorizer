//! Example of integrating hive-gpu with vectorizer
//!
//! This example shows how to use hive-gpu for GPU-accelerated vector operations
//! in the vectorizer project.

use vectorizer::error::Result;
use vectorizer::gpu_adapter::GpuAdapter;
use vectorizer::models::{DistanceMetric, HnswConfig, Payload, Vector};

#[cfg(feature = "hive-gpu")]
async fn example_hive_gpu_integration() -> Result<()> {
    println!("🚀 Hive-GPU Integration Example");

    // Create a sample vector
    let vector = Vector {
        id: "test_vector_1".to_string(),
        data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        payload: Some(Payload::new(serde_json::json!({
            "category": "example",
            "source": "hive-gpu"
        }))),
    };

    // Convert to hive-gpu format
    let gpu_vector = GpuAdapter::vector_to_gpu_vector(&vector);
    println!(
        "✅ Converted vector to hive-gpu format: {:?}",
        gpu_vector.id
    );

    // Convert distance metric
    let gpu_metric = GpuAdapter::distance_metric_to_gpu_metric(DistanceMetric::Cosine);
    println!("✅ Converted distance metric: {gpu_metric:?}");

    // Convert HNSW config
    let hnsw_config = HnswConfig {
        m: 16,
        ef_construction: 200,
        ef_search: 50,
        seed: Some(42),
    };

    let gpu_config = GpuAdapter::hnsw_config_to_gpu_config(&hnsw_config);
    println!("✅ Converted HNSW config: {gpu_config:?}");

    // Note: In a real implementation, you would:
    // 1. Create a GPU context (Metal, CUDA, or wgpu)
    // 2. Create a GPU vector storage
    // 3. Add vectors to the storage
    // 4. Perform GPU-accelerated searches

    println!("🎯 Hive-GPU integration example completed successfully!");
    Ok(())
}

#[cfg(not(feature = "hive-gpu"))]
async fn example_hive_gpu_integration() -> Result<()> {
    println!("⚠️ Hive-GPU feature not enabled. Run with --features hive-gpu to see the example.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    example_hive_gpu_integration().await
}

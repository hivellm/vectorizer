//! Simple test to verify benchmark utilities work

use tracing::{debug, error, info, warn};
use vectorizer::benchmark::{BenchmarkConfig, TestDataGenerator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🧪 Simple Test of Benchmark Utilities");
    tracing::info!("====================================\n");

    // Create simple configuration
    let config = BenchmarkConfig::quick()
        .with_dimensions(vec![128])
        .with_vector_counts(vec![100]);

    tracing::info!("📊 Configuration:");
    tracing::info!("  - Dimensions: {:?}", config.dimensions);
    tracing::info!("  - Vector counts: {:?}", config.vector_counts);
    tracing::info!();

    // Generate small test data
    tracing::info!("🔧 Generating test data...");
    let mut generator = TestDataGenerator::new(config);
    let test_data = generator.generate_vectors(100, 128)?;

    tracing::info!(
        "  ✅ Generated {} vectors (dimension: {})",
        test_data.vector_count(),
        test_data.dimension()
    );
    tracing::info!("  ✅ Generated {} documents", test_data.documents().len());
    tracing::info!("  ✅ Generated {} queries", test_data.queries().len());
    tracing::info!();

    // Test data access
    tracing::info!("🔍 Testing data access...");
    if let Some((id, vector)) = test_data.vectors().first() {
        tracing::info!("  ✅ First vector ID: {id}");
        tracing::info!("  ✅ First vector dimension: {}", vector.len());
        tracing::info!(
            "  ✅ First vector sample: {:?}",
            &vector[..5.min(vector.len())]
        );
    }

    if let Some(query) = test_data.queries().first() {
        tracing::info!("  ✅ First query: {query}");
    }

    tracing::info!("\n✅ Simple test completed successfully!");

    Ok(())
}

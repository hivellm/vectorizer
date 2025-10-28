//! Simple test to verify benchmark utilities work

use vectorizer::benchmark::{BenchmarkConfig, TestDataGenerator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Simple Test of Benchmark Utilities");
    println!("====================================\n");

    // Create simple configuration
    let config = BenchmarkConfig::quick()
        .with_dimensions(vec![128])
        .with_vector_counts(vec![100]);

    println!("📊 Configuration:");
    println!("  - Dimensions: {:?}", config.dimensions);
    println!("  - Vector counts: {:?}", config.vector_counts);
    println!();

    // Generate small test data
    println!("🔧 Generating test data...");
    let mut generator = TestDataGenerator::new(config);
    let test_data = generator.generate_vectors(100, 128)?;

    println!(
        "  ✅ Generated {} vectors (dimension: {})",
        test_data.vector_count(),
        test_data.dimension()
    );
    println!("  ✅ Generated {} documents", test_data.documents().len());
    println!("  ✅ Generated {} queries", test_data.queries().len());
    println!();

    // Test data access
    println!("🔍 Testing data access...");
    if let Some((id, vector)) = test_data.vectors().first() {
        println!("  ✅ First vector ID: {id}");
        println!("  ✅ First vector dimension: {}", vector.len());
        println!(
            "  ✅ First vector sample: {:?}",
            &vector[..5.min(vector.len())]
        );
    }

    if let Some(query) = test_data.queries().first() {
        println!("  ✅ First query: {query}");
    }

    println!("\n✅ Simple test completed successfully!");

    Ok(())
}

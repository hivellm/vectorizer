//! Minimal Benchmark Example
//!
//! This example demonstrates the benchmark helper utilities without using HNSW
//! to avoid the empty index issue.

use std::time::Instant;

use vectorizer::benchmark::{
    BenchmarkConfig, OperationMetrics, PerformanceMetrics, ReportGenerator, TestDataGenerator,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Minimal Benchmark Example");
    println!("============================\n");

    // Create benchmark configuration
    let config = BenchmarkConfig::quick()
        .with_dimensions(vec![128])
        .with_vector_counts(vec![1000]);

    println!("📊 Configuration:");
    println!("  - Dimensions: {:?}", config.dimensions);
    println!("  - Vector counts: {:?}", config.vector_counts);
    println!();

    // Generate test data
    println!("🔧 Generating test data...");
    let mut generator = TestDataGenerator::new(config.clone());
    let test_data = generator.generate_vectors(1000, 128)?;

    println!(
        "  ✅ Generated {} vectors (dimension: {})",
        test_data.vector_count(),
        test_data.dimension()
    );
    println!("  ✅ Generated {} documents", test_data.documents().len());
    println!("  ✅ Generated {} queries", test_data.queries().len());
    println!();

    // Run simple benchmark
    println!("🏃 Running simple benchmark...");
    let start = Instant::now();

    // Simulate some work
    let mut sum = 0.0;
    for (_, vector) in test_data.vectors() {
        sum += vector.iter().sum::<f32>();
    }

    let duration = start.elapsed();
    println!("  ✅ Benchmark completed");
    println!("    - Duration: {duration:?}");
    println!("    - Sum of all vectors: {sum:.2}");
    println!("    - Vectors processed: {}", test_data.vector_count());

    // Create performance metrics
    let mut metrics = PerformanceMetrics::new(
        "minimal_benchmark".to_string(),
        test_data.vector_count(),
        test_data.dimension(),
    );

    // Add a simple operation metric
    let operation_metrics = OperationMetrics::new(
        "vector_sum".to_string(),
        "Sum all vector elements".to_string(),
    );

    metrics.add_operation("vector_sum".to_string(), operation_metrics);
    metrics.total_duration_ms = duration.as_millis() as f64;

    // Generate reports
    println!("\n📊 Generating reports...");
    let reporter = ReportGenerator::new()
        .with_output_directory("benchmark/reports".to_string())
        .with_system_info(true)
        .with_detailed_metrics(true);

    // Generate Markdown report
    let md_report = reporter.generate_markdown(&metrics)?;
    reporter.save_report(&md_report, "minimal_benchmark.md")?;
    println!("  ✅ Markdown report saved");

    // Generate JSON report
    let json_report = reporter.generate_json(&metrics)?;
    reporter.save_report(&json_report, "minimal_benchmark.json")?;
    println!("  ✅ JSON report saved");

    // Generate CSV report
    let csv_report = reporter.generate_csv(&metrics)?;
    reporter.save_report(&csv_report, "minimal_benchmark.csv")?;
    println!("  ✅ CSV report saved");

    // Generate HTML report
    let html_report = reporter.generate_html(&metrics)?;
    reporter.save_report(&html_report, "minimal_benchmark.html")?;
    println!("  ✅ HTML report saved");

    println!("\n✅ Minimal benchmark completed successfully!");
    println!("📄 Reports saved to: benchmark/reports/");

    Ok(())
}

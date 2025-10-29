//! Example Benchmark using Benchmark Helper Utilities
//!
//! This example demonstrates how to use the new benchmark helper utilities
//! to create comprehensive benchmarks.

// Removed unused imports: BenchmarkScenario, VectorPattern
use vectorizer::benchmark::{BenchmarkConfig, BenchmarkRunner, ReportGenerator, TestDataGenerator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Vectorizer Benchmark Helper Utilities Example");
    println!("===============================================\n");

    // Create benchmark configuration
    let config = BenchmarkConfig::quick()
        .with_dimensions(vec![128, 256])
        .with_vector_counts(vec![1000, 5000])
        .with_measurement_time(std::time::Duration::from_secs(5));

    println!("📊 Configuration:");
    println!("  - Dimensions: {:?}", config.dimensions);
    println!("  - Vector counts: {:?}", config.vector_counts);
    println!("  - Measurement time: {:?}", config.measurement_time);
    println!();

    // Generate test data
    println!("🔧 Generating test data...");
    let mut generator = TestDataGenerator::new(config.clone());
    let test_data = generator.generate_vectors(5000, 256)?;

    println!(
        "  ✅ Generated {} vectors (dimension: {})",
        test_data.vector_count(),
        test_data.dimension()
    );
    println!("  ✅ Generated {} documents", test_data.documents().len());
    println!("  ✅ Generated {} queries", test_data.queries().len());
    println!();

    // Run benchmarks
    println!("🏃 Running benchmarks...");
    let mut runner = BenchmarkRunner::new(config.clone()).with_system_monitoring();

    // Only run benchmarks if we have data
    if test_data.vector_count() > 0 {
        // Search benchmark
        let search_metrics = runner.benchmark_search(&test_data, &[1, 10, 100])?;
        println!("  ✅ Search benchmark completed");
        println!(
            "    - Operations: {}",
            search_metrics.summary.total_operations
        );
        println!(
            "    - Throughput: {:.2} ops/sec",
            search_metrics.summary.overall_throughput
        );
        println!(
            "    - Avg latency: {:.0} μs",
            search_metrics.summary.avg_latency_us
        );

        // Insert benchmark
        let insert_metrics = runner.benchmark_insert(&test_data, &[1, 100, 1000])?;
        println!("  ✅ Insert benchmark completed");
        println!(
            "    - Operations: {}",
            insert_metrics.summary.total_operations
        );
        println!(
            "    - Throughput: {:.2} ops/sec",
            insert_metrics.summary.overall_throughput
        );
    } else {
        println!("  ⚠️  No test data available, skipping benchmarks");
    }

    // Generate reports
    if test_data.vector_count() > 0 {
        println!("\n📊 Generating reports...");
        let reporter = ReportGenerator::new()
            .with_output_directory("benchmark/reports".to_string())
            .with_system_info(true)
            .with_detailed_metrics(true);

        // Create a simple metrics object for demonstration
        let metrics = vectorizer::benchmark::PerformanceMetrics::new(
            "example_benchmark".to_string(),
            test_data.vector_count(),
            test_data.dimension(),
        );

        // Generate Markdown report
        let md_report = reporter.generate_markdown(&metrics)?;
        reporter.save_report(&md_report, "example_benchmark.md")?;
        println!("  ✅ Markdown report saved");

        // Generate JSON report
        let json_report = reporter.generate_json(&metrics)?;
        reporter.save_report(&json_report, "example_benchmark.json")?;
        println!("  ✅ JSON report saved");

        // Generate CSV report
        let csv_report = reporter.generate_csv(&metrics)?;
        reporter.save_report(&csv_report, "example_benchmark.csv")?;
        println!("  ✅ CSV report saved");

        // Generate HTML report
        let html_report = reporter.generate_html(&metrics)?;
        reporter.save_report(&html_report, "example_benchmark.html")?;
        println!("  ✅ HTML report saved");
    } else {
        println!("\n⚠️  No test data available, skipping report generation");
    }

    println!("\n✅ Example benchmark completed successfully!");
    println!("📄 Reports saved to: benchmark/reports/");

    Ok(())
}

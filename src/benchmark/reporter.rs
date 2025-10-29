//! Benchmark Report Generation
//!
//! Provides utilities for generating various types of benchmark reports
//! including Markdown, JSON, CSV, and HTML formats.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::benchmark::{BenchmarkResult, OperationMetrics, PerformanceMetrics};

/// Report generator for benchmark results
pub struct ReportGenerator {
    output_directory: String,
    include_system_info: bool,
    include_detailed_metrics: bool,
    custom_template: Option<String>,
}

impl ReportGenerator {
    /// Create new report generator
    pub fn new() -> Self {
        Self {
            output_directory: "target/criterion".to_string(),
            include_system_info: true,
            include_detailed_metrics: true,
            custom_template: None,
        }
    }

    /// Set output directory
    pub fn with_output_directory(mut self, directory: String) -> Self {
        self.output_directory = directory;
        self
    }

    /// Enable or disable system information
    pub fn with_system_info(mut self, include: bool) -> Self {
        self.include_system_info = include;
        self
    }

    /// Enable or disable detailed metrics
    pub fn with_detailed_metrics(mut self, include: bool) -> Self {
        self.include_detailed_metrics = include;
        self
    }

    /// Set custom template
    pub fn with_custom_template(mut self, template: String) -> Self {
        self.custom_template = Some(template);
        self
    }

    /// Generate Markdown report
    pub fn generate_markdown(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut report = String::new();

        // Header
        report.push_str("# Vectorizer Benchmark Report\n\n");
        report.push_str(&format!("**Generated**: {}\n\n", metrics.timestamp));

        // System information
        if self.include_system_info {
            report.push_str("## System Information\n\n");
            report.push_str(&format!(
                "- **CPU**: {} ({} cores)\n",
                metrics.system_info.cpu_model, metrics.system_info.cpu_cores
            ));
            report.push_str(&format!(
                "- **Memory**: {:.1} MB total, {:.1} MB available\n",
                metrics.system_info.total_memory_mb, metrics.system_info.available_memory_mb
            ));
            report.push_str(&format!("- **OS**: {}\n", metrics.system_info.os));
            report.push_str(&format!(
                "- **Rust Version**: {}\n",
                metrics.system_info.rust_version
            ));
            report.push_str(&format!(
                "- **Vectorizer Version**: {}\n\n",
                metrics.system_info.vectorizer_version
            ));
        }

        // Test configuration
        report.push_str("## Test Configuration\n\n");
        report.push_str(&format!(
            "- **Dataset Size**: {} vectors\n",
            metrics.dataset_size
        ));
        report.push_str(&format!("- **Vector Dimension**: {}\n", metrics.dimension));
        report.push_str(&format!("- **Configuration**: {}\n\n", metrics.config));

        // Executive summary
        report.push_str("## Executive Summary\n\n");
        report.push_str("| Operation | Throughput (ops/s) | Avg Latency (μs) | P95 Latency (μs) | P99 Latency (μs) |\n");
        report.push_str("|-----------|-------------------|------------------|------------------|------------------|\n");

        for (name, op_metrics) in &metrics.operations {
            report.push_str(&format!(
                "| {} | {:.2} | {:.0} | {:.0} | {:.0} |\n",
                name,
                op_metrics.throughput_ops_per_sec,
                op_metrics.avg_latency_us,
                op_metrics.p95_latency_us,
                op_metrics.p99_latency_us
            ));
        }

        report.push_str("\n");

        // Overall summary
        report.push_str("## Overall Performance\n\n");
        report.push_str(&format!(
            "- **Total Operations**: {}\n",
            metrics.summary.total_operations
        ));
        report.push_str(&format!(
            "- **Overall Throughput**: {:.2} ops/sec\n",
            metrics.summary.overall_throughput
        ));
        report.push_str(&format!(
            "- **Average Latency**: {:.0} μs\n",
            metrics.summary.avg_latency_us
        ));
        report.push_str(&format!(
            "- **P95 Latency**: {:.0} μs\n",
            metrics.summary.p95_latency_us
        ));
        report.push_str(&format!(
            "- **P99 Latency**: {:.0} μs\n",
            metrics.summary.p99_latency_us
        ));
        report.push_str(&format!(
            "- **Peak Memory**: {:.2} MB\n",
            metrics.summary.peak_memory_mb
        ));
        report.push_str(&format!(
            "- **Average CPU Usage**: {:.1}%\n",
            metrics.summary.avg_cpu_usage
        ));
        report.push_str(&format!(
            "- **Success Rate**: {:.1}%\n\n",
            metrics.summary.success_rate * 100.0
        ));

        // Detailed results
        if self.include_detailed_metrics {
            report.push_str("## Detailed Results\n\n");

            for (name, op_metrics) in &metrics.operations {
                report.push_str(&format!("### {}\n\n", name));
                report.push_str(&format!("**Configuration**: {}\n\n", op_metrics.config));

                report.push_str("#### Performance Metrics\n\n");
                report.push_str(&format!(
                    "- **Total Operations**: {}\n",
                    op_metrics.total_operations
                ));
                report.push_str(&format!(
                    "- **Total Time**: {:.2} ms ({:.2} s)\n",
                    op_metrics.total_time_ms,
                    op_metrics.total_time_ms / 1000.0
                ));
                report.push_str(&format!(
                    "- **Throughput**: {:.2} ops/sec\n",
                    op_metrics.throughput_ops_per_sec
                ));
                report.push_str(&format!(
                    "- **Throughput**: {:.2} ops/min\n\n",
                    op_metrics.throughput_ops_per_sec * 60.0
                ));

                report.push_str("#### Latency Distribution\n\n");
                report.push_str(&format!(
                    "- **Average**: {:.0} μs ({:.2} ms)\n",
                    op_metrics.avg_latency_us,
                    op_metrics.avg_latency_us / 1000.0
                ));
                report.push_str(&format!(
                    "- **P50 (Median)**: {:.0} μs\n",
                    op_metrics.p50_latency_us
                ));
                report.push_str(&format!("- **P95**: {:.0} μs\n", op_metrics.p95_latency_us));
                report.push_str(&format!("- **P99**: {:.0} μs\n", op_metrics.p99_latency_us));
                report.push_str(&format!("- **Min**: {:.0} μs\n", op_metrics.min_latency_us));
                report.push_str(&format!(
                    "- **Max**: {:.0} μs\n\n",
                    op_metrics.max_latency_us
                ));

                report.push_str("#### Memory Impact\n\n");
                report.push_str(&format!(
                    "- **Before**: {:.2} MB\n",
                    op_metrics.memory_before_mb
                ));
                report.push_str(&format!(
                    "- **After**: {:.2} MB\n",
                    op_metrics.memory_after_mb
                ));
                report.push_str(&format!(
                    "- **Delta**: {:.2} MB\n\n",
                    op_metrics.memory_delta_mb
                ));

                if !op_metrics.custom_metrics.is_empty() {
                    report.push_str("#### Custom Metrics\n\n");
                    for (key, value) in &op_metrics.custom_metrics {
                        report.push_str(&format!("- **{}**: {:.4}\n", key, value));
                    }
                    report.push_str("\n");
                }

                report.push_str("---\n\n");
            }
        }

        // Analysis and recommendations
        report.push_str("## Analysis & Recommendations\n\n");
        self.add_analysis(&mut report, metrics);

        // Footer
        report.push_str("---\n\n");
        report.push_str("*Report generated by Vectorizer Benchmark Suite*\n");

        Ok(report)
    }

    /// Generate JSON report
    pub fn generate_json(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(metrics)?;
        Ok(json)
    }

    /// Generate CSV report
    pub fn generate_csv(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut csv = String::new();

        // Header
        csv.push_str("Operation,Config,Total Operations,Total Time (ms),Throughput (ops/s),");
        csv.push_str("Avg Latency (μs),P50 Latency (μs),P95 Latency (μs),P99 Latency (μs),");
        csv.push_str("Min Latency (μs),Max Latency (μs),Memory Before (MB),Memory After (MB),");
        csv.push_str("Memory Delta (MB),CPU Usage (%)\n");

        // Data rows
        for (name, op_metrics) in &metrics.operations {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                name,
                op_metrics.config,
                op_metrics.total_operations,
                op_metrics.total_time_ms,
                op_metrics.throughput_ops_per_sec,
                op_metrics.avg_latency_us,
                op_metrics.p50_latency_us,
                op_metrics.p95_latency_us,
                op_metrics.p99_latency_us,
                op_metrics.min_latency_us,
                op_metrics.max_latency_us,
                op_metrics.memory_before_mb,
                op_metrics.memory_after_mb,
                op_metrics.memory_delta_mb,
                op_metrics.cpu_usage_percent
            ));
        }

        Ok(csv)
    }

    /// Generate HTML report
    pub fn generate_html(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n<head>\n");
        html.push_str("<title>Vectorizer Benchmark Report</title>\n");
        html.push_str("<style>\n");
        html.push_str(REPORT_STYLES);
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Header
        html.push_str("<div class=\"header\">\n");
        html.push_str("<h1>Vectorizer Benchmark Report</h1>\n");
        html.push_str(&format!(
            "<p class=\"timestamp\">Generated: {}</p>\n",
            metrics.timestamp
        ));
        html.push_str("</div>\n");

        // Summary table
        html.push_str("<div class=\"section\">\n");
        html.push_str("<h2>Performance Summary</h2>\n");
        html.push_str("<table class=\"summary-table\">\n");
        html.push_str("<thead><tr><th>Operation</th><th>Throughput (ops/s)</th><th>Avg Latency (μs)</th><th>P95 Latency (μs)</th><th>P99 Latency (μs)</th></tr></thead>\n");
        html.push_str("<tbody>\n");

        for (name, op_metrics) in &metrics.operations {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{:.2}</td><td>{:.0}</td><td>{:.0}</td><td>{:.0}</td></tr>\n",
                name,
                op_metrics.throughput_ops_per_sec,
                op_metrics.avg_latency_us,
                op_metrics.p95_latency_us,
                op_metrics.p99_latency_us
            ));
        }

        html.push_str("</tbody></table>\n");
        html.push_str("</div>\n");

        // Detailed results
        if self.include_detailed_metrics {
            html.push_str("<div class=\"section\">\n");
            html.push_str("<h2>Detailed Results</h2>\n");

            for (name, op_metrics) in &metrics.operations {
                html.push_str(&format!("<div class=\"operation\">\n"));
                html.push_str(&format!("<h3>{}</h3>\n", name));
                html.push_str(&format!(
                    "<p class=\"config\">Configuration: {}</p>\n",
                    op_metrics.config
                ));

                // Performance metrics
                html.push_str("<div class=\"metrics\">\n");
                html.push_str("<h4>Performance Metrics</h4>\n");
                html.push_str("<ul>\n");
                html.push_str(&format!(
                    "<li>Total Operations: {}</li>\n",
                    op_metrics.total_operations
                ));
                html.push_str(&format!(
                    "<li>Total Time: {:.2} ms</li>\n",
                    op_metrics.total_time_ms
                ));
                html.push_str(&format!(
                    "<li>Throughput: {:.2} ops/sec</li>\n",
                    op_metrics.throughput_ops_per_sec
                ));
                html.push_str("</ul>\n");
                html.push_str("</div>\n");

                // Latency distribution
                html.push_str("<div class=\"metrics\">\n");
                html.push_str("<h4>Latency Distribution</h4>\n");
                html.push_str("<ul>\n");
                html.push_str(&format!(
                    "<li>Average: {:.0} μs</li>\n",
                    op_metrics.avg_latency_us
                ));
                html.push_str(&format!(
                    "<li>P50: {:.0} μs</li>\n",
                    op_metrics.p50_latency_us
                ));
                html.push_str(&format!(
                    "<li>P95: {:.0} μs</li>\n",
                    op_metrics.p95_latency_us
                ));
                html.push_str(&format!(
                    "<li>P99: {:.0} μs</li>\n",
                    op_metrics.p99_latency_us
                ));
                html.push_str("</ul>\n");
                html.push_str("</div>\n");

                html.push_str("</div>\n");
            }

            html.push_str("</div>\n");
        }

        html.push_str("</body>\n</html>\n");

        Ok(html)
    }

    /// Save report to file
    pub fn save_report(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = Path::new(&self.output_directory);

        if !output_path.exists() {
            fs::create_dir_all(output_path)?;
        }

        let file_path = output_path.join(filename);
        fs::write(file_path, content)?;

        Ok(())
    }

    /// Generate and save all report formats
    pub fn generate_all_reports(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut saved_files = Vec::new();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        // Markdown report
        let md_content = self.generate_markdown(metrics)?;
        let md_filename = format!("benchmark_report_{}.md", timestamp);
        self.save_report(&md_content, &md_filename)?;
        saved_files.push(format!("{}/{}", self.output_directory, md_filename));

        // JSON report
        let json_content = self.generate_json(metrics)?;
        let json_filename = format!("benchmark_report_{}.json", timestamp);
        self.save_report(&json_content, &json_filename)?;
        saved_files.push(format!("{}/{}", self.output_directory, json_filename));

        // CSV report
        let csv_content = self.generate_csv(metrics)?;
        let csv_filename = format!("benchmark_report_{}.csv", timestamp);
        self.save_report(&csv_content, &csv_filename)?;
        saved_files.push(format!("{}/{}", self.output_directory, csv_filename));

        // HTML report
        let html_content = self.generate_html(metrics)?;
        let html_filename = format!("benchmark_report_{}.html", timestamp);
        self.save_report(&html_content, &html_filename)?;
        saved_files.push(format!("{}/{}", self.output_directory, html_filename));

        Ok(saved_files)
    }

    fn add_analysis(&self, report: &mut String, metrics: &PerformanceMetrics) {
        // Performance analysis
        report.push_str("### Performance Analysis\n\n");

        // Find best and worst performing operations
        let mut operations: Vec<_> = metrics.operations.iter().collect();
        operations.sort_by(|a, b| {
            b.1.throughput_ops_per_sec
                .partial_cmp(&a.1.throughput_ops_per_sec)
                .unwrap()
        });

        if let Some((best_name, best_metrics)) = operations.first() {
            report.push_str(&format!(
                "- **Best Throughput**: {} ({:.2} ops/sec)\n",
                best_name, best_metrics.throughput_ops_per_sec
            ));
        }

        if let Some((worst_name, worst_metrics)) = operations.last() {
            report.push_str(&format!(
                "- **Lowest Throughput**: {} ({:.2} ops/sec)\n",
                worst_name, worst_metrics.throughput_ops_per_sec
            ));
        }

        // Latency analysis
        operations.sort_by(|a, b| a.1.avg_latency_us.partial_cmp(&b.1.avg_latency_us).unwrap());

        if let Some((fastest_name, fastest_metrics)) = operations.first() {
            report.push_str(&format!(
                "- **Fastest Operation**: {} ({:.0} μs avg)\n",
                fastest_name, fastest_metrics.avg_latency_us
            ));
        }

        if let Some((slowest_name, slowest_metrics)) = operations.last() {
            report.push_str(&format!(
                "- **Slowest Operation**: {} ({:.0} μs avg)\n",
                slowest_name, slowest_metrics.avg_latency_us
            ));
        }

        report.push_str("\n");

        // Memory analysis
        report.push_str("### Memory Analysis\n\n");
        report.push_str(&format!(
            "- **Peak Memory Usage**: {:.2} MB\n",
            metrics.summary.peak_memory_mb
        ));
        report.push_str(&format!(
            "- **Memory per Vector**: {:.4} MB\n",
            metrics.summary.peak_memory_mb / metrics.dataset_size as f64
        ));

        // Recommendations
        report.push_str("\n### Recommendations\n\n");

        // Throughput recommendations
        if metrics.summary.overall_throughput < 1000.0 {
            report.push_str("- **Low Throughput**: Consider optimizing batch sizes or enabling parallel processing\n");
        }

        // Latency recommendations
        if metrics.summary.p95_latency_us > 10000.0 {
            report.push_str("- **High Latency**: Consider reducing vector dimensions or optimizing HNSW parameters\n");
        }

        // Memory recommendations
        if metrics.summary.peak_memory_mb > 1000.0 {
            report.push_str(
                "- **High Memory Usage**: Consider enabling quantization or reducing batch sizes\n",
            );
        }

        report.push_str("\n");
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS styles for HTML reports
const REPORT_STYLES: &str = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

.header {
    text-align: center;
    margin-bottom: 30px;
    padding: 20px;
    background: #f8f9fa;
    border-radius: 8px;
}

.timestamp {
    color: #666;
    font-size: 0.9em;
}

.section {
    margin-bottom: 30px;
}

.summary-table {
    width: 100%;
    border-collapse: collapse;
    margin: 20px 0;
}

.summary-table th,
.summary-table td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #ddd;
}

.summary-table th {
    background-color: #f8f9fa;
    font-weight: 600;
}

.operation {
    margin-bottom: 30px;
    padding: 20px;
    border: 1px solid #e9ecef;
    border-radius: 8px;
}

.config {
    color: #666;
    font-style: italic;
    margin-bottom: 15px;
}

.metrics {
    display: inline-block;
    vertical-align: top;
    margin-right: 30px;
    margin-bottom: 20px;
}

.metrics h4 {
    margin-top: 0;
    color: #495057;
}

.metrics ul {
    list-style-type: none;
    padding-left: 0;
}

.metrics li {
    padding: 4px 0;
    border-bottom: 1px solid #f8f9fa;
}
"#;

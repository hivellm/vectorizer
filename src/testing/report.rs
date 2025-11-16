//! Test reporting module for advanced features
//!
//! Provides structured test reporting with JSON and HTML output formats

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Test report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Timestamp when report was generated
    pub timestamp: String,
    /// Test suite name
    pub test_suite: String,
    /// Individual test results
    pub tests: Vec<TestResult>,
    /// Summary statistics
    pub summary: TestSummary,
    /// Test suite results
    pub suites: HashMap<String, SuiteResult>,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test status
    pub status: TestStatus,
    /// Duration in seconds
    pub duration_seconds: f64,
    /// Error message if failed
    pub error: Option<String>,
    /// Suite name
    pub suite: String,
}

/// Test status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Ignored,
    Skipped,
}

/// Test summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total number of tests
    pub total: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of ignored tests
    pub ignored: usize,
    /// Total duration in seconds
    pub duration_seconds: f64,
}

/// Suite result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    /// Duration in seconds
    pub duration_seconds: f64,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Suite status
    pub status: SuiteStatus,
    /// Test results in this suite
    pub tests: Vec<TestResult>,
}

/// Suite status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuiteStatus {
    Passed,
    Failed,
    Partial,
}

impl TestReport {
    /// Create a new test report
    pub fn new(test_suite: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            test_suite: test_suite.into(),
            tests: Vec::new(),
            summary: TestSummary {
                total: 0,
                passed: 0,
                failed: 0,
                ignored: 0,
                duration_seconds: 0.0,
            },
            suites: HashMap::new(),
        }
    }

    /// Add a test result
    pub fn add_test(&mut self, result: TestResult) {
        self.tests.push(result.clone());

        // Update summary
        self.summary.total += 1;
        match result.status {
            TestStatus::Passed => self.summary.passed += 1,
            TestStatus::Failed => self.summary.failed += 1,
            TestStatus::Ignored | TestStatus::Skipped => self.summary.ignored += 1,
        }
        self.summary.duration_seconds += result.duration_seconds;

        // Update suite
        let suite = self
            .suites
            .entry(result.suite.clone())
            .or_insert_with(|| SuiteResult {
                duration_seconds: 0.0,
                passed: 0,
                failed: 0,
                status: SuiteStatus::Passed,
                tests: Vec::new(),
            });

        suite.tests.push(result.clone());
        suite.duration_seconds += result.duration_seconds;

        match result.status {
            TestStatus::Passed => suite.passed += 1,
            TestStatus::Failed => {
                suite.failed += 1;
                suite.status = SuiteStatus::Failed;
            }
            TestStatus::Ignored | TestStatus::Skipped => {}
        }

        // Update suite status if partial
        if suite.passed > 0 && suite.failed > 0 {
            suite.status = SuiteStatus::Partial;
        }
    }

    /// Save report to JSON file
    pub fn save_json(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Generate HTML report
    pub fn generate_html(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let html = self.to_html();
        let mut file = File::create(path)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }

    /// Convert to HTML string
    fn to_html(&self) -> String {
        let pass_rate = if self.summary.total > 0 {
            (self.summary.passed as f64 / self.summary.total as f64) * 100.0
        } else {
            0.0
        };

        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{} Test Report</title>
    <meta charset="UTF-8">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; background: #f5f7fa; padding: 20px; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 10px; margin-bottom: 20px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header h1 {{ font-size: 2em; margin-bottom: 10px; }}
        .header p {{ opacity: 0.9; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .card {{ background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .card h3 {{ color: #666; font-size: 0.9em; text-transform: uppercase; margin-bottom: 10px; }}
        .card .value {{ font-size: 2em; font-weight: bold; }}
        .card.passed .value {{ color: #27ae60; }}
        .card.failed .value {{ color: #e74c3c; }}
        .card.total .value {{ color: #3498db; }}
        .suites {{ margin-bottom: 30px; }}
        .suite {{ background: white; padding: 20px; border-radius: 10px; margin-bottom: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); border-left: 4px solid #3498db; }}
        .suite.passed {{ border-left-color: #27ae60; }}
        .suite.failed {{ border-left-color: #e74c3c; }}
        .suite.partial {{ border-left-color: #f39c12; }}
        .suite h3 {{ margin-bottom: 10px; color: #2c3e50; }}
        .suite-stats {{ display: flex; gap: 20px; margin-top: 10px; }}
        .suite-stats span {{ padding: 5px 10px; border-radius: 5px; font-size: 0.9em; }}
        .suite-stats .passed {{ background: #d4edda; color: #155724; }}
        .suite-stats .failed {{ background: #f8d7da; color: #721c24; }}
        .tests {{ margin-top: 20px; }}
        .test {{ padding: 10px; margin: 5px 0; border-radius: 5px; font-family: 'Courier New', monospace; font-size: 0.9em; }}
        .test.passed {{ background: #d4edda; color: #155724; }}
        .test.failed {{ background: #f8d7da; color: #721c24; }}
        .test.ignored {{ background: #e2e3e5; color: #383d41; }}
        .test-name {{ font-weight: bold; }}
        .test-duration {{ color: #666; font-size: 0.8em; }}
        .test-error {{ margin-top: 5px; padding: 5px; background: rgba(0,0,0,0.05); border-radius: 3px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{} Test Report</h1>
            <p>Generated: {}</p>
            <p>Pass Rate: {:.1}%</p>
        </div>
        
        <div class="summary">
            <div class="card total">
                <h3>Total Tests</h3>
                <div class="value">{}</div>
            </div>
            <div class="card passed">
                <h3>Passed</h3>
                <div class="value">{}</div>
            </div>
            <div class="card failed">
                <h3>Failed</h3>
                <div class="value">{}</div>
            </div>
            <div class="card">
                <h3>Duration</h3>
                <div class="value">{:.2}s</div>
            </div>
        </div>
        
        <div class="suites">
            <h2 style="margin-bottom: 20px; color: #2c3e50;">Test Suites</h2>
"#,
            self.test_suite,
            self.test_suite,
            self.timestamp,
            pass_rate,
            self.summary.total,
            self.summary.passed,
            self.summary.failed,
            self.summary.duration_seconds
        );

        // Add suite results
        for (suite_name, suite) in &self.suites {
            let status_class = match suite.status {
                SuiteStatus::Passed => "passed",
                SuiteStatus::Failed => "failed",
                SuiteStatus::Partial => "partial",
            };

            html.push_str(&format!(
                r#"
            <div class="suite {}">
                <h3>{}</h3>
                <div class="suite-stats">
                    <span class="passed">✓ {} passed</span>
                    <span class="failed">✗ {} failed</span>
                    <span>Duration: {:.2}s</span>
                </div>
"#,
                status_class, suite_name, suite.passed, suite.failed, suite.duration_seconds
            ));

            // Add test results
            if !suite.tests.is_empty() {
                html.push_str(r#"                <div class="tests">"#);
                for test in &suite.tests {
                    let test_class = match test.status {
                        TestStatus::Passed => "passed",
                        TestStatus::Failed => "failed",
                        TestStatus::Ignored | TestStatus::Skipped => "ignored",
                    };

                    html.push_str(&format!(
                        r#"
                    <div class="test {}">
                        <div class="test-name">{}</div>
                        <div class="test-duration">{:.3}s</div>
"#,
                        test_class, test.name, test.duration_seconds
                    ));

                    if let Some(error) = &test.error {
                        html.push_str(&format!(
                            r#"                        <div class="test-error">{}</div>"#,
                            html_escape(error)
                        ));
                    }

                    html.push_str("                    </div>");
                }
                html.push_str("                </div>");
            }

            html.push_str("            </div>");
        }

        html.push_str(
            r#"
        </div>
    </div>
</body>
</html>"#,
        );

        html
    }
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_creation() {
        let mut report = TestReport::new("test-suite");

        report.add_test(TestResult {
            name: "test1".to_string(),
            status: TestStatus::Passed,
            duration_seconds: 0.5,
            error: None,
            suite: "suite1".to_string(),
        });

        assert_eq!(report.summary.total, 1);
        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.summary.failed, 0);
    }

    #[test]
    fn test_json_serialization() {
        let mut report = TestReport::new("test-suite");
        report.add_test(TestResult {
            name: "test1".to_string(),
            status: TestStatus::Passed,
            duration_seconds: 0.5,
            error: None,
            suite: "suite1".to_string(),
        });

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("test-suite"));
        assert!(json.contains("test1"));
    }
}

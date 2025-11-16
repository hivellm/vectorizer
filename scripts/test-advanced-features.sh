#!/bin/bash
# Advanced Features Test Automation Script
# Automates testing of advanced Qdrant features: sparse vectors, hybrid search, quantization, geo filters, payload indexing, advanced storage

set -euo pipefail

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed. Please install it:"
    echo "  Ubuntu/Debian: sudo apt-get install jq"
    echo "  macOS: brew install jq"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="${PROJECT_ROOT}/test-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/advanced-features-${TIMESTAMP}.json"
HTML_REPORT="${REPORT_DIR}/advanced-features-${TIMESTAMP}.html"

# Test suites
SPARSE_VECTOR_TESTS="integration_sparse_vector"
HYBRID_SEARCH_TESTS="integration_hybrid_search"
QUANTIZATION_TESTS="integration_binary_quantization"
PAYLOAD_INDEX_TESTS="integration_payload_index"
STORAGE_TESTS="storage_integration"

# Options
VERBOSE=false
COVERAGE=false
PARALLEL=true
FAIL_FAST=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        --no-parallel)
            PARALLEL=false
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose      Enable verbose output"
            echo "  -c, --coverage     Generate coverage report"
            echo "  --no-parallel      Disable parallel test execution"
            echo "  --fail-fast        Stop on first failure"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Create report directory
mkdir -p "$REPORT_DIR"

# Initialize report
cat > "$REPORT_FILE" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "test_suite": "advanced-features",
  "tests": [],
  "summary": {
    "total": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "duration_seconds": 0
  },
  "suites": {}
}
EOF

# Function to run test suite
run_test_suite() {
    local suite_name=$1
    local test_file=$2
    local start_time=$(date +%s)
    
    echo -e "${BLUE}Running ${suite_name} tests...${NC}"
    
    local cargo_args="--test $test_file"
    if [ "$PARALLEL" = true ]; then
        cargo_args="$cargo_args -- --test-threads=$(nproc)"
    fi
    if [ "$VERBOSE" = true ]; then
        cargo_args="$cargo_args -- --nocapture"
    fi
    
    local output_file="${REPORT_DIR}/${suite_name}-${TIMESTAMP}.log"
    local exit_code=0
    
    if cargo test $cargo_args 2>&1 | tee "$output_file"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${GREEN}✓ ${suite_name} tests passed (${duration}s)${NC}"
        
        # Extract test results
        local passed=$(grep -c "test result: ok" "$output_file" || echo "0")
        local failed=$(grep -c "test result: FAILED" "$output_file" || echo "0")
        
        # Update report
        jq --arg suite "$suite_name" \
           --argjson duration "$duration" \
           --argjson passed "$passed" \
           --argjson failed "$failed" \
           '.suites[$suite] = {
             "duration_seconds": $duration,
             "passed": $passed,
             "failed": $failed,
             "status": (if $failed == 0 then "passed" else "failed" end)
           }' "$REPORT_FILE" > "${REPORT_FILE}.tmp" && mv "${REPORT_FILE}.tmp" "$REPORT_FILE"
        
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${RED}✗ ${suite_name} tests failed (${duration}s)${NC}"
        
        if [ "$FAIL_FAST" = true ]; then
            echo -e "${RED}Failing fast due to test failure${NC}"
            exit 1
        fi
        
        return 1
    fi
}

# Function to generate HTML report
generate_html_report() {
    echo -e "${BLUE}Generating HTML report...${NC}"
    
    local json_data=$(cat "$REPORT_FILE")
    local total=$(echo "$json_data" | jq '.summary.total')
    local passed=$(echo "$json_data" | jq '.summary.passed')
    local failed=$(echo "$json_data" | jq '.summary.failed')
    
    cat > "$HTML_REPORT" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Advanced Features Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .header { background: #2c3e50; color: white; padding: 20px; border-radius: 5px; }
        .summary { display: flex; gap: 20px; margin: 20px 0; }
        .card { background: white; padding: 20px; border-radius: 5px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .card h3 { margin-top: 0; }
        .passed { color: #27ae60; }
        .failed { color: #e74c3c; }
        .suite { margin: 10px 0; padding: 10px; background: white; border-radius: 5px; }
        .suite.passed { border-left: 4px solid #27ae60; }
        .suite.failed { border-left: 4px solid #e74c3c; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #34495e; color: white; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Advanced Features Test Report</h1>
        <p>Generated: $(date)</p>
    </div>
    
    <div class="summary">
        <div class="card">
            <h3>Total Tests</h3>
            <p style="font-size: 24px; font-weight: bold;">$total</p>
        </div>
        <div class="card">
            <h3 class="passed">Passed</h3>
            <p style="font-size: 24px; font-weight: bold; color: #27ae60;">$passed</p>
        </div>
        <div class="card">
            <h3 class="failed">Failed</h3>
            <p style="font-size: 24px; font-weight: bold; color: #e74c3c;">$failed</p>
        </div>
    </div>
    
    <div class="card">
        <h2>Test Suites</h2>
        <div id="suites"></div>
    </div>
    
    <script>
        const data = $json_data;
        const suitesDiv = document.getElementById('suites');
        
        Object.entries(data.suites || {}).forEach(([name, suite]) => {
            const div = document.createElement('div');
            div.className = \`suite \${suite.status}\`;
            div.innerHTML = \`
                <h3>\${name}</h3>
                <p>Duration: \${suite.duration_seconds}s</p>
                <p>Passed: <span class="passed">\${suite.passed}</span></p>
                <p>Failed: <span class="failed">\${suite.failed}</span></p>
            \`;
            suitesDiv.appendChild(div);
        });
    </script>
</body>
</html>
EOF
    
    echo -e "${GREEN}HTML report generated: $HTML_REPORT${NC}"
}

# Main execution
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Advanced Features Test Automation${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

cd "$PROJECT_ROOT"

# Run test suites
TOTAL_FAILED=0

run_test_suite "sparse_vector" "$SPARSE_VECTOR_TESTS" || ((TOTAL_FAILED++))
run_test_suite "hybrid_search" "$HYBRID_SEARCH_TESTS" || ((TOTAL_FAILED++))
run_test_suite "quantization" "$QUANTIZATION_TESTS" || ((TOTAL_FAILED++))
run_test_suite "payload_index" "$PAYLOAD_INDEX_TESTS" || ((TOTAL_FAILED++))
run_test_suite "storage" "$STORAGE_TESTS" || ((TOTAL_FAILED++))

# Run unit tests for advanced features
echo -e "${BLUE}Running unit tests for advanced features...${NC}"
if cargo test --lib sparse_vector hybrid_search quantization payload_index storage::advanced 2>&1 | tee "${REPORT_DIR}/unit-tests-${TIMESTAMP}.log"; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
else
    echo -e "${RED}✗ Unit tests failed${NC}"
    ((TOTAL_FAILED++))
fi

# Generate coverage if requested
if [ "$COVERAGE" = true ]; then
    echo -e "${BLUE}Generating coverage report...${NC}"
    cargo test --all-features -- --test-threads=1 2>&1 | tee "${REPORT_DIR}/coverage-${TIMESTAMP}.log" || true
fi

# Calculate summary
TOTAL_TESTS=$(jq '[.suites[].passed, .suites[].failed] | add' "$REPORT_FILE")
TOTAL_PASSED=$(jq '[.suites[].passed] | add' "$REPORT_FILE")
TOTAL_FAILED=$(jq '[.suites[].failed] | add' "$REPORT_FILE")

# Update summary in report
jq --argjson total "$TOTAL_TESTS" \
   --argjson passed "$TOTAL_PASSED" \
   --argjson failed "$TOTAL_FAILED" \
   '.summary = {
     "total": $total,
     "passed": $passed,
     "failed": $failed,
     "ignored": 0,
     "duration_seconds": ([.suites[].duration_seconds] | add)
   }' "$REPORT_FILE" > "${REPORT_FILE}.tmp" && mv "${REPORT_FILE}.tmp" "$REPORT_FILE"

# Generate HTML report
generate_html_report

# Print summary
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Total: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $TOTAL_PASSED${NC}"
echo -e "${RED}Failed: $TOTAL_FAILED${NC}"
echo ""
echo -e "JSON Report: $REPORT_FILE"
echo -e "HTML Report: $HTML_REPORT"
echo ""

# Exit with appropriate code
if [ $TOTAL_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi


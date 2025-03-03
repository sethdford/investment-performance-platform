#!/bin/bash
set -e

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "cargo-tarpaulin is not installed. Installing now..."
    cargo install cargo-tarpaulin
fi

# Create a directory for coverage reports
mkdir -p coverage_reports

# Run tarpaulin to generate coverage data
echo "Generating code coverage report..."
cargo tarpaulin --workspace --timeout 120 --out Html --output-dir coverage_reports

# Generate a JSON report for CI integration
cargo tarpaulin --workspace --timeout 120 --out Json --output-dir coverage_reports

# Print summary
echo "Coverage report generated in coverage_reports/"
echo "Open coverage_reports/tarpaulin-report.html in your browser to view the report"

# Check if coverage is below threshold
COVERAGE=$(grep -o '"line_coverage": [0-9.]*' coverage_reports/tarpaulin-report.json | grep -o '[0-9.]*')
THRESHOLD=70.0

echo "Current coverage: $COVERAGE%"
echo "Threshold: $THRESHOLD%"

if (( $(echo "$COVERAGE < $THRESHOLD" | bc -l) )); then
    echo "Coverage is below threshold!"
    exit 1
else
    echo "Coverage meets or exceeds threshold."
fi 
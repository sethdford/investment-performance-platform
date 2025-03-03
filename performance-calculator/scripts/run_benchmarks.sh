#!/bin/bash
# Performance Calculator Benchmark Runner
# This script runs the performance benchmarks and generates reports

set -e

echo "🚀 Running Performance Calculator Benchmarks"
echo "============================================"

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Build the benchmarks in release mode
echo "📦 Building benchmarks in release mode..."
cargo build --release --features "benchmarking"

# Run the benchmarks
echo "🏃 Running benchmarks..."
cargo bench

# Generate HTML report if criterion-table is installed
if command -v criterion-table &> /dev/null; then
    echo "📊 Generating HTML report..."
    criterion-table --output benchmark_report.html
    echo "Report generated at benchmark_report.html"
fi

# Compare with previous run if specified
if [ "$1" = "--compare" ]; then
    echo "🔍 Comparing with previous benchmark run..."
    cargo bench -- --baseline previous
fi

echo "✅ Benchmarks completed"

# Print summary of results
echo "📋 Summary:"
echo "Check the target/criterion directory for detailed results"
echo "Key metrics to observe:"
echo "- Performance impact of caching"
echo "- Scaling with portfolio size"
echo "- Multi-currency conversion overhead" 
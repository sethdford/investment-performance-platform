# Performance Calculator Testing Suite

This directory contains the performance calculator lambda function and a comprehensive testing suite to ensure its reliability and performance.

## Testing Tools

The testing suite includes the following tools:

### 1. Unit Tests

Run the unit tests with:

```bash
cargo test
```

### 2. Property-Based Tests

Property-based tests use the `proptest` crate to generate random inputs and verify that our functions behave correctly under a wide range of inputs.

Run the property-based tests with:

```bash
cargo test -- --nocapture property_tests
```

### 3. Load Testing

The load testing tool sends a large number of messages to the SQS queue to test how the system performs under high load.

Build and run the load test:

```bash
# Build the load test binary
cargo build --release --bin load_test

# Run the load test
./target/release/load_test run-load-test <queue-url> <num-messages> <concurrency> <portfolio-prefix>

# Example
./target/release/load_test run-load-test https://sqs.us-east-1.amazonaws.com/123456789012/performance-calculator-queue 1000 50 test-portfolio
```

Monitor the SQS queue during the load test:

```bash
./target/release/load_test monitor-queue <queue-url> <interval-seconds> <duration-seconds>

# Example
./target/release/load_test monitor-queue https://sqs.us-east-1.amazonaws.com/123456789012/performance-calculator-queue 5 300
```

### 4. Chaos Testing

The chaos testing tool introduces random failures and latency to test the resilience of the system.

Build and run the chaos test:

```bash
# Set environment variables
export DYNAMODB_TABLE=your-dynamodb-table-name
export SQS_QUEUE_URL=your-sqs-queue-url

# Build and run the chaos test
cargo run --bin chaos_test

# Customize the test parameters
cargo run --bin chaos_test --dynamodb-failure-rate=0.2 --sqs-failure-rate=0.3 --min-latency=100 --max-latency=1000 --duration=300
```

### 5. Code Coverage

Generate a code coverage report using cargo-tarpaulin:

```bash
# Run the coverage script
./coverage.sh
```

The coverage report will be generated in the `coverage_reports` directory. Open `coverage_reports/tarpaulin-report.html` in your browser to view the report.

## CI/CD Integration

The testing tools are designed to be integrated into a CI/CD pipeline. Here's an example of how to use them in a GitHub Actions workflow:

```yaml
name: Test and Deploy

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run unit tests
        run: cargo test
      - name: Run property-based tests
        run: cargo test -- --nocapture property_tests
      - name: Generate coverage report
        run: ./performance-lambda/coverage.sh
      - name: Upload coverage report
        uses: actions/upload-artifact@v2
        with:
          name: coverage-report
          path: performance-lambda/coverage_reports/
```

## Best Practices

1. **Run tests locally before pushing**: Always run the tests locally before pushing changes to ensure they pass.
2. **Monitor code coverage**: Aim for at least 70% code coverage for critical components.
3. **Add tests for new features**: When adding new features, also add tests to ensure they work correctly.
4. **Use load testing for performance tuning**: Use the load testing tool to identify performance bottlenecks.
5. **Use chaos testing for resilience**: Use the chaos testing tool to ensure the system can handle failures gracefully. 
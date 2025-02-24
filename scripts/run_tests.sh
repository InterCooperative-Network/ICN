#!/bin/bash
set -e

# Source environment variables
source scripts/setup_test_env.sh

echo "Running all tests..."

# Run format check
echo "Checking code formatting..."
cargo fmt -- --check

# Run clippy
echo "Running clippy..."
cargo clippy -- -D warnings

# Run unit tests
echo "Running unit tests..."
cargo test --lib --all-features --workspace

# Run integration tests
echo "Running integration tests..."
cargo test --test '*' --all-features

# Run property tests
echo "Running property tests..."
cargo test --test property_tests --all-features

# Run end-to-end tests
echo "Running end-to-end tests..."
cargo test --test e2e_tests --all-features

# Generate coverage report
echo "Generating coverage report..."
cargo tarpaulin --verbose --workspace --timeout 120 --out Xml --out Html --output-dir test-results/coverage

# Run security audit
echo "Running security audit..."
cargo audit
cargo deny check licenses
cargo deny check bans

# Start test server for load tests
echo "Starting test server for load tests..."
cargo run &
SERVER_PID=$!
sleep 5  # Wait for server to start

# Run load tests
echo "Running load tests..."
k6 run tests/load_tests/federation_test.js -o json=test-results/k6/federation.json
k6 run tests/load_tests/governance_test.js -o json=test-results/k6/governance.json
k6 run tests/load_tests/resource_test.js -o json=test-results/k6/resource.json

# Stop test server
kill $SERVER_PID

# Run benchmarks
echo "Running benchmarks..."
cargo bench --all-features

echo "All tests completed!"

# Print summary
echo "Test Summary:"
echo "============"
echo "Coverage report: test-results/coverage/tarpaulin-report.html"
echo "Load test results: test-results/k6/"
echo "Benchmark results: target/criterion/" 
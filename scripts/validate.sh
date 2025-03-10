#!/bin/bash

# Script to validate code quality before submitting PRs

set -e

echo "Running validation checks..."

# Check for required tools
tools=("cargo" "rustfmt" "clippy")
for tool in "${tools[@]}"; do
    if ! command -v "$tool" &> /dev/null; then
        echo "Error: $tool is not installed or not in PATH"
        echo "Please run 'rustup component add $tool'"
        exit 1
    fi
done

# Run formatting checks
echo "-----------------------"
echo "Checking code formatting..."
cargo fmt -- --check
echo "✅ Formatting is valid"

# Run clippy
echo "-----------------------"
echo "Running Clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy checks passed"

# Run tests
echo "-----------------------"
echo "Running tests..."
cargo test --all
echo "✅ All tests passed"

# Documentation checks
echo "-----------------------"
echo "Checking documentation..."
cargo doc --no-deps --document-private-items --all-features
echo "✅ Documentation built successfully"

# Check for outdated dependencies (advisory only)
echo "-----------------------"
echo "Checking for outdated dependencies..."
cargo outdated || true

# Run audit for security vulnerabilities
echo "-----------------------"
echo "Checking for security vulnerabilities..."
if ! cargo audit --version &> /dev/null; then
    echo "⚠️ cargo-audit is not installed. Install with 'cargo install cargo-audit' for security checks"
else
    cargo audit || echo "⚠️ Security vulnerabilities found. Please review above output."
fi

echo "-----------------------"
echo "✅ All validation checks completed!"
echo "Your code looks good to go. Remember to update documentation if you've changed any public APIs." 
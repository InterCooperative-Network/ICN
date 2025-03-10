#!/bin/bash
cd /workspaces/ICN

# Build the federation crate
echo "Building federation crate..."
cargo build -p icn-federation

# Run the integration tests
echo "Running federation integration tests..."
RUST_BACKTRACE=1 cargo test -p icn-federation -- --nocapture

# Check the result
if [ $? -eq 0 ]; then
    echo "✅ Federation tests passed!"
else
    echo "❌ Federation tests failed!"
fi 
#!/bin/bash

# Set environment variables
export RUST_LOG=debug

# Run the consensus node
cd "$(dirname "$0")"
echo "Starting simplified consensus node..."
echo "Current directory: $(pwd)"
echo "Running command: ./target/debug/simple_node -t bootstrap -p 9000 -a 8091"
./target/debug/simple_node -t bootstrap -p 9000 -a 8091 
#!/bin/bash

# Set environment variables
export RUST_LOG=debug

# Run the consensus node
cd "$(dirname "$0")"
echo "Starting consensus node..."
echo "Current directory: $(pwd)"
echo "Running command: ./target/debug/icn_bin --node-type bootstrap --node-port 9000 --api-port 8091"
./target/debug/icn_bin --node-type bootstrap --node-port 9000 --api-port 8091 
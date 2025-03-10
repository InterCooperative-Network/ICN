#!/bin/bash

# Set environment variables
export RUST_LOG=info

# Run the consensus node
cd "$(dirname "$0")"
./target/debug/icn_bin --node-type bootstrap --node-port 9000 --api-port 8090 
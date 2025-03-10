#!/bin/bash
set -e

# Build the CLI
echo "Building ICN CLI..."
cargo build -p icn-cli

# Find and execute the CLI binary
CLI_PATH=$(find target/debug -type f -executable -name "icn-cli" | head -n 1)

if [ -z "$CLI_PATH" ]; then
    echo "Error: icn-cli binary not found after build"
    exit 1
fi

echo "Found CLI binary at: $CLI_PATH"
echo "Running CLI command: $CLI_PATH $@"
$CLI_PATH "$@"

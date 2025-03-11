#!/bin/bash
# Development script for running the backend

# Set error handling
set -e

# Load environment variables
if [ -f ".env" ]; then
    source .env
else
    echo "Error: .env file not found"
    exit 1
fi

# Ensure we're in the backend directory
cd backend || {
    echo "Error: backend directory not found"
    exit 1
}

# Set default port if not specified
BACKEND_PORT=${BACKEND_PORT:-8081}
BACKEND_HOST=${BACKEND_HOST:-0.0.0.0}

echo "Starting backend server on $BACKEND_HOST:$BACKEND_PORT..."

# Run the backend in development mode
RUST_LOG=${RUST_LOG:-debug} cargo run --bin icn-server 
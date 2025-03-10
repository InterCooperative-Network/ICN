#!/bin/bash
# Direct run script for the ICN backend using SQLite

set -e

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Setup environment variables for SQLite
export DATABASE_URL="sqlite:${PROJECT_ROOT}/data/icn.db"
export RUST_LOG=debug
export RUST_BACKTRACE=1
export API_PORT=8081
export API_HOST=0.0.0.0
export COOPERATIVE_ID=icn-primary
export ICN_NETWORK_MODE=development

# Create data directory if it doesn't exist
mkdir -p "${PROJECT_ROOT}/data"

# Build and run the backend
echo "Building and running ICN backend..."
cd backend
cargo run 
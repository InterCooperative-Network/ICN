#!/bin/bash

# Debug script to test server component directly

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========== ICN Debug Startup ==========${NC}"

# Kill any existing processes
echo -e "${YELLOW}Stopping existing processes...${NC}"
pkill -f "icn-server" >/dev/null 2>&1 || true

# Set environment variables for better debugging
export RUST_LOG=debug
export ICN_SERVER_PORT=8085
export RUST_BACKTRACE=1

echo -e "${YELLOW}Starting ICN server in debug mode...${NC}"
cd icn-server
cargo build && cargo run

echo -e "${RED}Server process exited.${NC}" 
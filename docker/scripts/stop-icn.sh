#!/bin/bash

# ICN System Stop Script
set -e

# Ensure we're in the right directory
cd "$(dirname "$0")/../.."
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Stopping ICN system...${NC}"

# Stop Docker containers
cd "${PROJECT_ROOT}/docker"
docker-compose -f docker-compose.dev.yml down

echo -e "${GREEN}ICN system stopped successfully${NC}" 
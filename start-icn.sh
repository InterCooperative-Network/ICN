#!/bin/bash

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to check if a command exists
check_requirement() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}Error: $2 is not installed${NC}"
        return 1
    fi
    return 0
}

# Function to check service health
check_health() {
    local service=$1
    local port=$2
    local max_attempts=30
    local attempt=1

    echo -n "Waiting for $service to be healthy"
    while [ $attempt -le $max_attempts ]; do
        if curl -s "http://localhost:${port}/api/v1/health" &> /dev/null; then
            echo -e "\n${GREEN}$service is healthy${NC}"
            return 0
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    echo -e "\n${RED}$service failed to become healthy${NC}"
    return 1
}

# Check prerequisites
echo "Checking prerequisites..."
check_requirement "docker" "Docker" || exit 1
check_requirement "docker-compose" "Docker Compose" || exit 1

# Create required directories
mkdir -p data/{db,bootstrap,validator1,validator2} logs config

# Load environment variables
if [ -f .env ]; then
    source .env
fi

# Set default environment variables
export POSTGRES_USER=${POSTGRES_USER:-icnuser}
export POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-icnpass}
export POSTGRES_DB=${POSTGRES_DB:-icndb}
export RUST_LOG=${RUST_LOG:-info}
export COOPERATIVE_ID=${COOPERATIVE_ID:-icn-primary}

echo -e "${BLUE}Starting ICN network...${NC}"

# Stop any existing containers
docker-compose -f docker/docker-compose.yml down

# Start database first
echo "Starting database..."
docker-compose -f docker/docker-compose.yml up -d db
sleep 5  # Give the database time to initialize

# Start the bootstrap node
echo "Starting bootstrap node..."
docker-compose -f docker/docker-compose.yml up -d bootstrap
check_health "bootstrap node" "8082"

# Start validator nodes
echo "Starting validator nodes..."
docker-compose -f docker/docker-compose.yml up -d validator1 validator2
check_health "validator1" "8083"
check_health "validator2" "8084"

# Start the backend
echo "Starting backend services..."
docker-compose -f docker/docker-compose.yml up -d backend
check_health "backend" "8081"

# Start the frontend
echo "Starting frontend..."
docker-compose -f docker/docker-compose.yml up -d frontend
check_health "frontend" "3000"

echo -e "${GREEN}ICN network has been started successfully!${NC}"
echo -e "${BLUE}Available Services:${NC}"
echo "- Dashboard: http://localhost:3000"
echo "- Backend API: http://localhost:8081"
echo "- Bootstrap Node: http://localhost:8082"
echo "- Validator 1: http://localhost:8083"
echo "- Validator 2: http://localhost:8084"
echo ""
echo -e "${BLUE}Useful commands:${NC}"
echo "- View logs: docker-compose -f docker/docker-compose.yml logs -f"
echo "- Stop network: docker-compose -f docker/docker-compose.yml down"
echo "- Check status: docker-compose -f docker/docker-compose.yml ps"
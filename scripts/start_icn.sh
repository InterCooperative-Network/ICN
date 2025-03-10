#!/bin/bash
set -e

# Function to check if postgres is ready
check_postgres() {
    local max_attempts=30
    local attempt=1

    echo "Waiting for database to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if docker exec docker_db_1 pg_isready -U icnuser -d icndb > /dev/null 2>&1; then
            echo "Database is ready"
            return 0
        fi
        echo "Attempt $attempt/$max_attempts: Database not ready yet..."
        sleep 5
        attempt=$((attempt + 1))
    done
    echo "Database failed to become ready"
    return 1
}

# Function to check if a service is healthy
check_health() {
    local service=$1
    local port=$2
    local max_attempts=30
    local attempt=1

    echo "Waiting for $service to be healthy..."
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "http://localhost:$port/health" > /dev/null 2>&1; then
            echo "$service is healthy"
            return 0
        fi
        echo "Attempt $attempt/$max_attempts: $service not ready yet..."
        sleep 5
        attempt=$((attempt + 1))
    done
    echo "$service failed to become healthy"
    return 1
}

# Create required directories
mkdir -p data/db data/bootstrap data/validator1 data/validator2 logs config

# Load environment variables
if [ -f .env ]; then
    source .env
fi

# Set default environment variables if not set
export POSTGRES_USER=${POSTGRES_USER:-icnuser}
export POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-icnpass}
export POSTGRES_DB=${POSTGRES_DB:-icndb}
export COOPERATIVE_ID=${COOPERATIVE_ID:-icn-primary}
export RUST_LOG=${RUST_LOG:-info}

# Start the ICN network
echo "Starting ICN network..."
docker-compose -f docker/docker-compose.yml up -d db

# Wait for database to be ready
check_postgres

# Start the bootstrap node first
docker-compose -f docker/docker-compose.yml up -d bootstrap
check_health "bootstrap node" "8082"

# Start validator nodes
docker-compose -f docker/docker-compose.yml up -d validator1 validator2
check_health "validator1" "8083"
check_health "validator2" "8084"

# Start the backend and frontend services
docker-compose -f docker/docker-compose.yml up -d backend frontend
check_health "backend" "8081"
check_health "frontend" "80"

echo "ICN network is now running!"
echo "Access the dashboard at http://localhost"
echo "Bootstrap node API: http://localhost:8082"
echo "Validator1 API: http://localhost:8083"
echo "Validator2 API: http://localhost:8084"
echo "Main backend API: http://localhost:8081"
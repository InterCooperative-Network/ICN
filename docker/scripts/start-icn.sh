#!/bin/bash

# ICN System Startup Script
set -e

# Ensure we're in the right directory
cd "$(dirname "$0")/../.."
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Setup logging
mkdir -p "${PROJECT_ROOT}/logs"
LOG_DATE=$(date +"%Y%m%d-%H%M%S")
MASTER_LOG="${PROJECT_ROOT}/logs/icn-master-${LOG_DATE}.log"
touch "$MASTER_LOG"
echo "--- ICN System Startup Log - $(date) ---" >> "$MASTER_LOG"

# Log message helper
log_message() {
    local SERVICE=$1
    local MESSAGE=$2
    local LEVEL=${3:-INFO}
    local TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    
    case "$LEVEL" in
        "ERROR")
            echo -e "${RED}[ERROR][$SERVICE] $MESSAGE${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}[WARNING][$SERVICE] $MESSAGE${NC}"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS][$SERVICE] $MESSAGE${NC}"
            ;;
        *)
            echo -e "[INFO][$SERVICE] $MESSAGE"
            ;;
    esac
    
    echo "[$TIMESTAMP][$LEVEL][$SERVICE] $MESSAGE" >> "$MASTER_LOG"
}

# Stop any existing containers
log_message "SYSTEM" "Stopping any existing ICN containers..." "INFO"
cd "${PROJECT_ROOT}/docker"
docker-compose -f docker-compose.dev.yml down 2>/dev/null || true

# Create required directories
log_message "SYSTEM" "Setting up directories..." "INFO"
mkdir -p "${PROJECT_ROOT}/data/db"
mkdir -p "${PROJECT_ROOT}/logs"

# Start services with Docker Compose
log_message "SYSTEM" "Starting Docker services..." "INFO"
cd "${PROJECT_ROOT}/docker"

# Load environment variables
if [ -f ".env" ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Start the database first
log_message "SYSTEM" "Starting database..." "INFO"
docker-compose -f docker-compose.dev.yml up -d db

# Wait for database to be ready
log_message "SYSTEM" "Waiting for database to be ready..." "INFO"
for i in {1..30}; do
    if docker-compose -f docker-compose.dev.yml exec db pg_isready -U ${POSTGRES_USER:-icn_user} -d ${POSTGRES_DB:-icn_db} > /dev/null 2>&1; then
        log_message "SYSTEM" "Database is ready" "SUCCESS"
        break
    fi
    sleep 2
    log_message "SYSTEM" "Still waiting for database... ($i of 30)" "INFO"
    if [ $i -eq 30 ]; then
        log_message "SYSTEM" "Database failed to start properly" "ERROR"
        exit 1
    fi
done

# Start the backend
log_message "SYSTEM" "Starting backend..." "INFO"
docker-compose -f docker-compose.dev.yml up -d backend

# Wait for backend to be ready
log_message "SYSTEM" "Waiting for backend to be ready..." "INFO"
for i in {1..30}; do
    if curl -s http://localhost:8081/api/v1/health > /dev/null 2>&1; then
        log_message "SYSTEM" "Backend is ready" "SUCCESS"
        break
    fi
    sleep 2
    log_message "SYSTEM" "Still waiting for backend... ($i of 30)" "INFO"
    if [ $i -eq 30 ]; then
        log_message "SYSTEM" "Backend failed to start properly" "ERROR"
        exit 1
    fi
done

# Start the frontend
log_message "SYSTEM" "Starting frontend..." "INFO"
docker-compose -f docker-compose.dev.yml up -d frontend

log_message "SYSTEM" "ICN system startup complete!" "SUCCESS"
echo
echo -e "${GREEN}ICN system is now running!${NC}"
echo -e "Backend API: ${BLUE}http://localhost:8081${NC}"
echo -e "Frontend:    ${BLUE}http://localhost:3000${NC}"
echo
echo -e "Check the logs with: ${YELLOW}docker-compose -f docker/docker-compose.dev.yml logs -f${NC}" 
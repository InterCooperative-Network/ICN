#!/bin/bash

# ICN System Startup Script
# Description: Starts all components of the ICN system with proper dependency management and error handling

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Initialize global variables
declare -a SERVICE_PIDS
export STARTUP_TIMESTAMP=$(date +"%Y%m%d-%H%M%S")

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Source the dependency management and utility functions
source "${PROJECT_ROOT}/scripts/startup-utils.sh" 2>/dev/null || {
    echo -e "${RED}Error: startup-utils.sh not found. Using local functions.${NC}"
    
    # Define basic service dependencies as fallback
    declare -A SERVICE_DEPENDENCIES
    SERVICE_DEPENDENCIES["db"]=""
    SERVICE_DEPENDENCIES["backend"]="db"
    SERVICE_DEPENDENCIES["frontend"]="backend"
    SERVICE_DEPENDENCIES["icn-cli"]="backend"
    
    # Define service health check ports
    declare -A SERVICE_PORTS
    SERVICE_PORTS["db"]="5432"
    SERVICE_PORTS["backend"]="8081"
    SERVICE_PORTS["frontend"]="3000"
}

# Setup logging infrastructure
setup_logging() {
    # Create log directory if it doesn't exist
    mkdir -p "${PROJECT_ROOT}/logs"
    
    # Create master log file with timestamp
    LOG_DATE=$(date +"%Y%m%d-%H%M%S")
    export MASTER_LOG="${PROJECT_ROOT}/logs/icn-master-${LOG_DATE}.log"
    touch "$MASTER_LOG"
    
    echo "--- ICN System Startup Log - $(date) ---" >> "$MASTER_LOG"
    echo "ICN startup initiated" >> "$MASTER_LOG"
}

# Log message to both console and log file
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

# Stop any existing ICN processes
stop_existing_processes() {
    log_message "SYSTEM" "Stopping any existing ICN processes..." "INFO"
    
    # Check if docker-compose is being used
    if [ -f "${PROJECT_ROOT}/docker/docker-compose.dev.yml" ]; then
        log_message "SYSTEM" "Found Docker Compose configuration. Stopping containers..." "INFO"
        (cd "${PROJECT_ROOT}/docker" && docker-compose -f docker-compose.dev.yml down) || true
    fi
    
    # Kill any running processes
    pkill -f "icn-server" >/dev/null 2>&1 || true
    pkill -f "react-scripts start" >/dev/null 2>&1 || true
    pkill -f "icn_bin" >/dev/null 2>&1 || true
    pkill -f "icn-cli" >/dev/null 2>&1 || true
    pkill -f "simple_node" >/dev/null 2>&1 || true
    pkill -f "icn-validator" >/dev/null 2>&1 || true
    pkill -f "icn-identity" >/dev/null 2>&1 || true
    
    # Remove old service registry if it exists
    if [ -f "${PROJECT_ROOT}/.icn_services" ]; then
        rm "${PROJECT_ROOT}/.icn_services"
    fi
    
    log_message "SYSTEM" "Existing processes stopped" "INFO"
}

# Start Docker services with proper dependency order
start_docker_services() {
    local COMPOSE_FILE="docker-compose.dev.yml"
    
    log_message "SYSTEM" "Starting Docker services using $COMPOSE_FILE..." "INFO"
    
    # Check if the compose file exists
    if [ ! -f "${PROJECT_ROOT}/docker/$COMPOSE_FILE" ]; then
        log_message "SYSTEM" "Docker Compose file not found: $COMPOSE_FILE" "ERROR"
        return 1
    fi
    
    # Start the containers
    (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" up -d) || {
        log_message "SYSTEM" "Failed to start Docker containers" "ERROR"
        return 1
    }
    
    log_message "SYSTEM" "Docker containers started successfully" "SUCCESS"
    
    # Wait for services to fully initialize
    log_message "SYSTEM" "Waiting for services to initialize..." "INFO"
    
    # Wait for database to become healthy (max 30 seconds)
    log_message "db" "Waiting for database to become ready..." "INFO"
    local DB_READY=false
    local TIMEOUT=30
    local ELAPSED=0
    
    while [ $ELAPSED -lt $TIMEOUT ]; do
        if docker ps | grep -q "db" && docker exec $(docker ps -q -f name=db) pg_isready -U ${POSTGRES_USER:-icnuser} -d ${POSTGRES_DB:-icndb} > /dev/null 2>&1; then
            DB_READY=true
            log_message "db" "Database is ready" "SUCCESS"
            break
        fi
        sleep 2
        ELAPSED=$((ELAPSED + 2))
        if [ $((ELAPSED % 10)) -eq 0 ]; then
            log_message "db" "Still waiting for database... ($ELAPSED/$TIMEOUT seconds)" "INFO"
        fi
    done
    
    if [ "$DB_READY" != "true" ]; then
        log_message "db" "Database failed to become ready within timeout period" "ERROR"
        log_message "db" "Checking container logs:" "INFO"
        docker logs $(docker ps -a -q -f name=db) | tail -n 10 > /tmp/db_error.log
        cat /tmp/db_error.log >> "$MASTER_LOG"
        log_message "db" "See logs for details" "INFO"
        return 1
    fi
    
    # Wait for backend to become healthy (max 60 seconds)
    log_message "backend" "Waiting for backend to become ready..." "INFO"
    local BACKEND_READY=false
    local TIMEOUT=60
    local ELAPSED=0
    
    while [ $ELAPSED -lt $TIMEOUT ]; do
        if docker ps | grep -q "backend" && curl -s --max-time 2 "http://localhost:8081/api/v1/health" > /dev/null 2>&1; then
            BACKEND_READY=true
            log_message "backend" "Backend API is ready" "SUCCESS"
            break
        fi
        sleep 3
        ELAPSED=$((ELAPSED + 3))
        if [ $((ELAPSED % 15)) -eq 0 ]; then
            log_message "backend" "Still waiting for backend... ($ELAPSED/$TIMEOUT seconds)" "INFO"
        fi
    done
    
    if [ "$BACKEND_READY" != "true" ]; then
        log_message "backend" "Backend failed to become ready within timeout period" "ERROR"
        log_message "backend" "Checking container logs:" "INFO"
        docker logs $(docker ps -a -q -f name=backend) | tail -n 15 > /tmp/backend_error.log
        cat /tmp/backend_error.log >> "$MASTER_LOG"
        log_message "backend" "See logs for details" "INFO"
        return 1
    fi
    
    # Register services in service registry
    echo -n > "${PROJECT_ROOT}/.icn_services"
    
    # Add database to service registry
    if docker ps | grep -q "db"; then
        log_message "db" "Database service registered" "SUCCESS"
        docker ps | grep "db" | awk '{print $1}' | xargs -I{} echo "db:{}:" >> "${PROJECT_ROOT}/.icn_services"
    else
        log_message "db" "Database service not found" "ERROR"
        return 1
    fi
    
    # Add backend to service registry
    if docker ps | grep -q "backend"; then
        log_message "backend" "Backend service registered" "SUCCESS"
        docker ps | grep "backend" | awk '{print $1}' | xargs -I{} echo "backend:{}:" >> "${PROJECT_ROOT}/.icn_services"
    else
        log_message "backend" "Backend service not found" "ERROR"
        return 1
    fi
    
    # Wait for frontend to start (more lenient, as it's less critical)
    log_message "frontend" "Waiting for frontend to initialize..." "INFO"
    local TIMEOUT=45
    local ELAPSED=0
    
    while [ $ELAPSED -lt $TIMEOUT ]; do
        if docker ps | grep -q "frontend" && curl -s --max-time 2 "http://localhost:3000" > /dev/null 2>&1; then
            log_message "frontend" "Frontend is ready" "SUCCESS"
            docker ps | grep "frontend" | awk '{print $1}' | xargs -I{} echo "frontend:{}:" >> "${PROJECT_ROOT}/.icn_services"
            break
        fi
        sleep 3
        ELAPSED=$((ELAPSED + 3))
        if [ $((ELAPSED % 15)) -eq 0 ]; then
            log_message "frontend" "Still waiting for frontend... ($ELAPSED/$TIMEOUT seconds)" "INFO"
        fi
    done
    
    # Even if frontend isn't fully ready, we don't fail the startup as long as the container is running
    if ! docker ps | grep -q "frontend"; then
        log_message "frontend" "Frontend container is not running" "ERROR"
        return 1
    elif ! curl -s --max-time 2 "http://localhost:3000" > /dev/null 2>&1; then
        log_message "frontend" "Frontend container is running but not yet serving content" "WARNING"
        docker ps | grep "frontend" | awk '{print $1}' | xargs -I{} echo "frontend:{}:" >> "${PROJECT_ROOT}/.icn_services"
    fi
    
    return 0
}

# Check health of services
check_service_health() {
    log_message "SYSTEM" "Checking service health..." "INFO"
    
    # Check database
    if docker exec $(docker ps -q -f name=db) pg_isready -U ${POSTGRES_USER:-icnuser} -d ${POSTGRES_DB:-icndb} > /dev/null 2>&1; then
        log_message "db" "Database service is healthy" "SUCCESS"
    else
        log_message "db" "Database service health check failed" "ERROR"
        return 1
    fi
    
    # Check backend API
    if curl -s --max-time 5 "http://localhost:8081/api/v1/health" > /dev/null 2>&1; then
        log_message "backend" "Backend API is responding" "SUCCESS"
    else
        log_message "backend" "Backend API health check failed" "ERROR"
        return 1
    fi
    
    # Check frontend (just check if the service is serving something)
    if curl -s --max-time 5 "http://localhost:3000" | grep -q "ICN" > /dev/null 2>&1; then
        log_message "frontend" "Frontend is responding" "SUCCESS"
    else
        log_message "frontend" "Frontend check failed - might still be initializing" "WARNING"
    fi
    
    return 0
}

# Start the ICN CLI tool for testing
start_icn_cli() {
    log_message "icn-cli" "Preparing ICN CLI..." "INFO"
    
    # Check if we need to build the CLI
    if [ ! -f "${PROJECT_ROOT}/target/debug/icn-cli" ] && [ ! -f "${PROJECT_ROOT}/target/release/icn-cli" ]; then
        log_message "icn-cli" "Building ICN CLI..." "INFO"
        (cd "${PROJECT_ROOT}" && cargo build --bin icn-cli) || {
            log_message "icn-cli" "Failed to build ICN CLI" "ERROR"
            return 1
        }
    fi
    
    # Add ICN CLI to service registry but don't start it automatically
    # It will be available for the user to use
    local CLI_PATH
    if [ -f "${PROJECT_ROOT}/target/debug/icn-cli" ]; then
        CLI_PATH="${PROJECT_ROOT}/target/debug/icn-cli"
    else
        CLI_PATH="${PROJECT_ROOT}/target/release/icn-cli"
    fi
    
    echo "icn-cli:ready:$CLI_PATH" >> "${PROJECT_ROOT}/.icn_services"
    log_message "icn-cli" "ICN CLI is ready to use at: $CLI_PATH" "SUCCESS"
    
    return 0
}

# Setup and initialize services
start_all_services() {
    log_message "SYSTEM" "Starting all ICN services..." "INFO"
    
    # Start Docker services
    start_docker_services || {
        log_message "SYSTEM" "Failed to start Docker services" "ERROR"
        return 1
    }
    
    # Check service health
    check_service_health || {
        log_message "SYSTEM" "Service health checks failed" "WARNING"
        # Continue anyway as services might still be initializing
    }
    
    # Prepare CLI tool
    start_icn_cli || {
        log_message "icn-cli" "Failed to prepare ICN CLI" "WARNING"
        # Continue anyway as this is not critical
    }
    
    log_message "SYSTEM" "All ICN services started" "SUCCESS"
    return 0
}

# Register trap for graceful shutdown on script exit
trap_shutdown() {
    log_message "SYSTEM" "Registering shutdown handler..." "INFO"
    
    # Register the trap for common signals
    trap 'echo -e "${YELLOW}\nShutdown signal received. Stopping all services...${NC}"; ./stop_icn.sh; exit' INT TERM EXIT
    
    log_message "SYSTEM" "Shutdown handler registered" "INFO"
}

# Main execution
main() {
    echo -e "${BLUE}========== ICN System Startup ==========${NC}"
    echo -e "${BLUE}Starting ICN system at $(date)${NC}"
    
    # Setup logging
    setup_logging
    
    # Register trap for graceful shutdown
    trap_shutdown
    
    # Ask for confirmation in interactive mode
    if [ -t 1 ]; then  # Check if stdout is a terminal
        echo -e "${YELLOW}This script will start all components of the ICN system.${NC}"
        echo -e "${YELLOW}Any existing processes will be stopped first.${NC}\n"
        
        read -p "Do you want to proceed? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${RED}Startup aborted by user.${NC}"
            exit 0
        fi
    fi
    
    # Stop any existing processes
    stop_existing_processes
    
    # Start all services
    start_all_services || {
        log_message "SYSTEM" "ICN system startup failed" "ERROR"
        echo -e "${RED}ICN system startup failed. Check logs for details.${NC}"
        echo -e "${YELLOW}Log file: ${MASTER_LOG}${NC}"
        exit 1
    }
    
    # Final status message
    log_message "SYSTEM" "ICN system startup completed" "SUCCESS"
    echo -e "${GREEN}=== ICN System Startup Completed ===${NC}"
    echo -e "${GREEN}All services are running.${NC}"
    
    # Print access information
    echo -e "\n${BLUE}Access your ICN system at:${NC}"
    echo -e "  - Web Interface: ${GREEN}http://localhost:3000${NC}"
    echo -e "  - Backend API: ${GREEN}http://localhost:8081/api/v1${NC}"
    
    # Print CLI usage
    echo -e "\n${BLUE}To use the ICN CLI:${NC}"
    if [ -f "${PROJECT_ROOT}/target/debug/icn-cli" ]; then
        echo -e "  ${GREEN}${PROJECT_ROOT}/target/debug/icn-cli --help${NC}"
    elif [ -f "${PROJECT_ROOT}/target/release/icn-cli" ]; then
        echo -e "  ${GREEN}${PROJECT_ROOT}/target/release/icn-cli --help${NC}"
    fi
    
    # Print monitoring commands
    echo -e "\n${BLUE}To monitor the system:${NC}"
    echo -e "  - ${GREEN}./check_icn_status.sh${NC}  (Check status of all components)"
    echo -e "  - ${GREEN}docker ps${NC}              (List running containers)"
    echo -e "  - ${GREEN}docker logs <container-id>${NC} (View container logs)"
    
    echo -e "\n${YELLOW}Press Ctrl+C to stop the system gracefully${NC}"
    
    # Keep script alive to maintain the trap
    while true; do
        sleep 3600 &
        wait $!
    done
}

# Execute main function
main

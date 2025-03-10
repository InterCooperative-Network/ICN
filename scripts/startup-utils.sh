#!/bin/bash

# ICN System Startup Utilities
# Provides functions for managing Docker containers, dependency checking, and system startup

# Define color codes for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Define service dependencies (format: SERVICE_DEPENDENCIES["service"]="dependency1,dependency2")
declare -A SERVICE_DEPENDENCIES
SERVICE_DEPENDENCIES["db"]=""                      # Database has no dependencies
SERVICE_DEPENDENCIES["backend"]="db"               # Backend depends on database
SERVICE_DEPENDENCIES["bootstrap"]="db"             # Bootstrap node depends on database
SERVICE_DEPENDENCIES["validator1"]="bootstrap,db"  # Validator1 depends on bootstrap and database
SERVICE_DEPENDENCIES["validator2"]="bootstrap,db"  # Validator2 depends on bootstrap and database
SERVICE_DEPENDENCIES["frontend"]="backend"         # Frontend depends on backend
SERVICE_DEPENDENCIES["identity"]="backend"         # Identity service depends on backend
SERVICE_DEPENDENCIES["reputation"]="backend"       # Reputation service depends on backend
SERVICE_DEPENDENCIES["governance"]="backend"       # Governance service depends on backend

# Define service ports (for health checks)
declare -A SERVICE_PORTS
SERVICE_PORTS["db"]="5432"
SERVICE_PORTS["backend"]="8081"
SERVICE_PORTS["bootstrap"]="8082"
SERVICE_PORTS["validator1"]="8083"
SERVICE_PORTS["validator2"]="8084"
SERVICE_PORTS["frontend"]="3000"
SERVICE_PORTS["identity"]="8085"
SERVICE_PORTS["reputation"]="8086"
SERVICE_PORTS["governance"]="8087"

# Setup logging infrastructure
setup_logging() {
  # Create log directory if it doesn't exist
  mkdir -p "${PROJECT_ROOT}/logs"
  
  # Create master log file with timestamp
  LOG_DATE=$(date +"%Y%m%d-%H%M%S")
  export MASTER_LOG="${PROJECT_ROOT}/logs/icn-master-${LOG_DATE}.log"
  touch "$MASTER_LOG"
  
  echo "--- ICN System Startup Log - $(date) ---" >> "$MASTER_LOG"
  log_message "SYSTEM" "Logging initialized" "INFO"
}

# Log a message with timestamp, level, and service
log_message() {
  local SERVICE=$1
  local MESSAGE=$2
  local LEVEL=${3:-INFO}
  local TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
  
  # Color coding for log levels when output is to terminal
  if [ -t 1 ]; then  # Check if stdout is a terminal
    case "$LEVEL" in
      "INFO")
        LEVEL_COLOR=$GREEN
        ;;
      "WARNING")
        LEVEL_COLOR=$YELLOW
        ;;
      "ERROR")
        LEVEL_COLOR=$RED
        ;;
      *)
        LEVEL_COLOR=$NC
        ;;
    esac
    
    echo -e "[$TIMESTAMP] [${LEVEL_COLOR}${LEVEL}${NC}] [${SERVICE}] ${MESSAGE}"
  else
    echo "[$TIMESTAMP] [${LEVEL}] [${SERVICE}] ${MESSAGE}"
  fi
  
  # Also log to files
  mkdir -p "${PROJECT_ROOT}/logs/${SERVICE}"
  echo "[$TIMESTAMP] [${LEVEL}] [${SERVICE}] ${MESSAGE}" >> "${PROJECT_ROOT}/logs/${SERVICE}/${SERVICE}.log"
  echo "[$TIMESTAMP] [${LEVEL}] [${SERVICE}] ${MESSAGE}" >> "${MASTER_LOG}"
}

# Load configuration from .env files
load_configuration() {
  log_message "CONFIG" "Loading environment configuration..." "INFO"
  
  # Load base configuration
  if [ -f "${PROJECT_ROOT}/.env" ]; then
    log_message "CONFIG" "Loading configuration from .env file" "INFO"
    export $(grep -v '^#' "${PROJECT_ROOT}/.env" | xargs)
  fi
  
  # Load docker environment configuration
  if [ -f "${PROJECT_ROOT}/docker/.env" ]; then
    log_message "CONFIG" "Loading Docker configuration from docker/.env file" "INFO"
    export $(grep -v '^#' "${PROJECT_ROOT}/docker/.env" | xargs)
  fi
  
  # Set default values if not defined
  export POSTGRES_USER=${POSTGRES_USER:-icnuser}
  export POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-icnpass}
  export POSTGRES_DB=${POSTGRES_DB:-icndb}
  export RUST_LOG=${RUST_LOG:-info}
  export RUST_BACKTRACE=${RUST_BACKTRACE:-0}
  export COOPERATIVE_ID=${COOPERATIVE_ID:-icn-primary}
  export ICN_NETWORK_MODE=${ICN_NETWORK_MODE:-development}
  
  # Validate configuration
  validate_configuration
}

# Validate configuration
validate_configuration() {
  local ISSUES=false
  
  # Check for required Docker installation
  if ! command -v docker &> /dev/null; then
    log_message "CONFIG" "Docker is not installed or not in PATH" "ERROR"
    ISSUES=true
  fi
  
  if ! command -v docker-compose &> /dev/null; then
    log_message "CONFIG" "docker-compose is not installed or not in PATH" "ERROR"
    ISSUES=true
  fi
  
  # Check Docker daemon is running
  if ! docker info &> /dev/null; then
    log_message "CONFIG" "Docker daemon is not running" "ERROR"
    ISSUES=true
  fi
  
  if [ "$ISSUES" = true ]; then
    return 1
  fi
  
  return 0
}

# Service discovery setup
setup_service_registry() {
  echo "# SERVICE_NAME,HOST,PORT,STATUS,CONTAINER_ID" > "${PROJECT_ROOT}/.service_registry"
  log_message "SYSTEM" "Service registry initialized" "INFO"
}

# Register a service in the registry
register_service() {
  local SERVICE_NAME=$1
  local SERVICE_HOST=$2
  local SERVICE_PORT=$3
  local STATUS=$4
  local CONTAINER_ID=$5
  
  echo "${SERVICE_NAME},${SERVICE_HOST},${SERVICE_PORT},${STATUS},${CONTAINER_ID}" >> "${PROJECT_ROOT}/.service_registry"
  log_message "$SERVICE_NAME" "Registered service on ${SERVICE_HOST}:${SERVICE_PORT} (${STATUS})" "INFO"
}

# Update a service status in the registry
update_service_registry() {
  local SERVICE_NAME=$1
  local NEW_STATUS=$2
  local TEMP_FILE="${PROJECT_ROOT}/.service_registry.tmp"
  
  # Update the status in the registry file
  awk -F, -v service="$SERVICE_NAME" -v status="$NEW_STATUS" '
    BEGIN { OFS = "," }
    $1 == service { $4 = status }
    { print }
  ' "${PROJECT_ROOT}/.service_registry" > "$TEMP_FILE"
  
  mv "$TEMP_FILE" "${PROJECT_ROOT}/.service_registry"
  log_message "$SERVICE_NAME" "Updated service status to ${NEW_STATUS}" "INFO"
}

# Check if Docker container is running
container_is_running() {
  local SERVICE_NAME=$1
  local CONTAINER_NAME="icn-${SERVICE_NAME}"
  
  # For db service, use the specific container name from docker-compose
  if [ "$SERVICE_NAME" = "db" ]; then
    CONTAINER_NAME="icn_db_1"
  fi
  
  # Check if container exists and is running
  if docker ps -q -f name=$CONTAINER_NAME | grep -q .; then
    return 0
  fi
  
  return 1
}

# Get Docker container ID
get_container_id() {
  local SERVICE_NAME=$1
  local CONTAINER_NAME="icn-${SERVICE_NAME}"
  
  # For db service, use the specific container name from docker-compose
  if [ "$SERVICE_NAME" = "db" ]; then
    CONTAINER_NAME="icn_db_1"
  fi
  
  local CONTAINER_ID=$(docker ps -q -f name=$CONTAINER_NAME)
  echo "$CONTAINER_ID"
}

# Check service health
service_health_check() {
  local SERVICE_NAME=$1
  local MAX_ATTEMPTS=${2:-30}
  local SLEEP_TIME=${3:-2}
  local CONTAINER_ID
  
  log_message "$SERVICE_NAME" "Performing health check..." "INFO"
  
  # Different health check methods based on service type
  case "$SERVICE_NAME" in
    "db")
      # Wait for postgres to be ready
      for ((i=1; i<=MAX_ATTEMPTS; i++)); do
        if docker exec $(get_container_id "db") pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB} > /dev/null 2>&1; then
          log_message "$SERVICE_NAME" "Health check passed (attempt $i/$MAX_ATTEMPTS)" "INFO"
          return 0
        fi
        log_message "$SERVICE_NAME" "Health check waiting (attempt $i/$MAX_ATTEMPTS)..." "INFO"
        sleep $SLEEP_TIME
      done
      ;;
      
    "backend"|"bootstrap"|"validator1"|"validator2")
      # Get the port from SERVICE_PORTS
      local PORT=${SERVICE_PORTS[$SERVICE_NAME]}
      local HEALTH_URL="http://localhost:${PORT}/api/v1/health"
      
      for ((i=1; i<=MAX_ATTEMPTS; i++)); do
        if curl -s --max-time 2 "$HEALTH_URL" > /dev/null; then
          log_message "$SERVICE_NAME" "Health check passed (attempt $i/$MAX_ATTEMPTS)" "INFO"
          return 0
        fi
        log_message "$SERVICE_NAME" "Health check waiting (attempt $i/$MAX_ATTEMPTS)..." "INFO"
        sleep $SLEEP_TIME
      done
      ;;
      
    "frontend")
      # Frontend health check - simple connection test
      local PORT=${SERVICE_PORTS[$SERVICE_NAME]}
      
      for ((i=1; i<=MAX_ATTEMPTS; i++)); do
        if curl -s --max-time 2 "http://localhost:${PORT}" > /dev/null; then
          log_message "$SERVICE_NAME" "Health check passed (attempt $i/$MAX_ATTEMPTS)" "INFO"
          return 0
        fi
        log_message "$SERVICE_NAME" "Health check waiting (attempt $i/$MAX_ATTEMPTS)..." "INFO"
        sleep $SLEEP_TIME
      done
      ;;
      
    *)
      # Default health check - just check if container is running
      for ((i=1; i<=MAX_ATTEMPTS; i++)); do
        if container_is_running "$SERVICE_NAME"; then
          log_message "$SERVICE_NAME" "Basic container health check passed (attempt $i/$MAX_ATTEMPTS)" "INFO"
          return 0
        fi
        log_message "$SERVICE_NAME" "Health check waiting (attempt $i/$MAX_ATTEMPTS)..." "INFO"
        sleep $SLEEP_TIME
      done
      ;;
  esac
  
  log_message "$SERVICE_NAME" "Health check failed after $MAX_ATTEMPTS attempts" "ERROR"
  return 1
}

# Check service dependencies
check_dependencies() {
  local SERVICE=$1
  local DEPS=${SERVICE_DEPENDENCIES[$SERVICE]}
  
  if [ -z "$DEPS" ]; then
    log_message "$SERVICE" "No dependencies to check" "INFO"
    return 0
  fi
  
  log_message "$SERVICE" "Checking dependencies: $DEPS" "INFO"
  
  IFS=',' read -ra DEP_ARRAY <<< "$DEPS"
  for DEP in "${DEP_ARRAY[@]}"; do
    log_message "$SERVICE" "Checking dependency: $DEP" "INFO"
    
    if ! container_is_running "$DEP"; then
      log_message "$SERVICE" "Dependency $DEP is not running" "ERROR"
      return 1
    fi
    
    if ! service_health_check "$DEP" 5 2; then
      log_message "$SERVICE" "Dependency $DEP is not healthy" "ERROR"
      return 1
    fi
    
    log_message "$SERVICE" "Dependency $DEP is running and healthy" "INFO"
  done
  
  return 0
}

# Stop existing containers
stop_existing_containers() {
  log_message "SYSTEM" "Stopping existing containers..." "INFO"
  
  # Check if docker-compose file exists
  if [ -f "${PROJECT_ROOT}/docker/docker-compose.yml" ]; then
    # Check which mode we're in
    local COMPOSE_FILE="${PROJECT_ROOT}/docker/docker-compose.yml"
    if [ "$ICN_NETWORK_MODE" = "development" ] && [ -f "${PROJECT_ROOT}/docker/docker-compose.dev.yml" ]; then
      COMPOSE_FILE="${PROJECT_ROOT}/docker/docker-compose.dev.yml"
    fi
    
    # Stop containers
    (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" down) || {
      log_message "SYSTEM" "Failed to stop containers with docker-compose" "ERROR"
      return 1
    }
    
    log_message "SYSTEM" "Existing containers stopped" "INFO"
    return 0
  else
    log_message "SYSTEM" "docker-compose.yml not found, no containers to stop" "WARNING"
    return 0
  fi
}

# Start service with Docker Compose
start_docker_service() {
  local SERVICE_NAME=$1
  local REQUIRED=${2:-true}  # Is this service required for system operation?
  local COMPOSE_FILE
  
  # Determine which compose file to use
  if [ "$ICN_NETWORK_MODE" = "development" ] && [ -f "${PROJECT_ROOT}/docker/docker-compose.dev.yml" ]; then
    COMPOSE_FILE="docker-compose.dev.yml"
    log_message "$SERVICE_NAME" "Using development compose file" "INFO"
  else
    COMPOSE_FILE="docker-compose.yml"
    log_message "$SERVICE_NAME" "Using production compose file" "INFO"
  fi
  
  log_message "$SERVICE_NAME" "Starting service with Docker Compose..." "INFO"
  
  # Check if the service is already running
  if container_is_running "$SERVICE_NAME"; then
    log_message "$SERVICE_NAME" "Service is already running" "INFO"
    local CONTAINER_ID=$(get_container_id "$SERVICE_NAME")
    register_service "$SERVICE_NAME" "localhost" "${SERVICE_PORTS[$SERVICE_NAME]}" "RUNNING" "$CONTAINER_ID"
    return 0
  fi
  
  # Check dependencies
  if ! check_dependencies "$SERVICE_NAME"; then
    log_message "$SERVICE_NAME" "Dependencies not satisfied" "ERROR"
    if [ "$REQUIRED" = true ]; then
      return 1
    else
      log_message "$SERVICE_NAME" "Service is optional, continuing without it" "WARNING"
      return 0
    fi
  fi
  
  # Start the service with docker-compose
  (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" up -d "$SERVICE_NAME") || {
    log_message "$SERVICE_NAME" "Failed to start service with docker-compose" "ERROR"
    if [ "$REQUIRED" = true ]; then
      return 1
    else
      log_message "$SERVICE_NAME" "Service is optional, continuing without it" "WARNING"
      return 0
    fi
  }
  
  # Check if service started successfully
  if ! container_is_running "$SERVICE_NAME"; then
    log_message "$SERVICE_NAME" "Service failed to start" "ERROR"
    if [ "$REQUIRED" = true ]; then
      return 1
    else
      log_message "$SERVICE_NAME" "Service is optional, continuing without it" "WARNING"
      return 0
    fi
  fi
  
  # Perform health check
  if ! service_health_check "$SERVICE_NAME"; then
    log_message "$SERVICE_NAME" "Service failed health check" "ERROR"
    if [ "$REQUIRED" = true ]; then
      # If the service is required but failed health check, stop the container
      (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" stop "$SERVICE_NAME")
      return 1
    else
      log_message "$SERVICE_NAME" "Service is optional, continuing despite health check failure" "WARNING"
      return 0
    fi
  fi
  
  # Register the service
  local CONTAINER_ID=$(get_container_id "$SERVICE_NAME")
  register_service "$SERVICE_NAME" "localhost" "${SERVICE_PORTS[$SERVICE_NAME]}" "RUNNING" "$CONTAINER_ID"
  log_message "$SERVICE_NAME" "Service started successfully" "INFO"
  
  return 0
}

# Start all ICN Docker services
start_all_docker_services() {
  log_message "SYSTEM" "Starting all ICN services with Docker..." "INFO"
  
  # 1. Start database (required)
  start_docker_service "db" true || {
    log_message "SYSTEM" "Failed to start database. Aborting startup." "ERROR"
    return 1
  }
  
  # 2. Start backend (required)
  start_docker_service "backend" true || {
    log_message "SYSTEM" "Failed to start backend. Aborting startup." "ERROR"
    return 1
  }
  
  # 3. Start bootstrap node (required)
  start_docker_service "bootstrap" true || {
    log_message "SYSTEM" "Failed to start bootstrap node. Aborting startup." "ERROR"
    return 1
  }
  
  # 4. Start validator nodes (optional)
  start_docker_service "validator1" false
  start_docker_service "validator2" false
  
  # 5. Start frontend (optional)
  start_docker_service "frontend" false
  
  # Print startup status
  print_docker_status
  
  return 0
}

# Print the status of Docker containers
print_docker_status() {
  log_message "SYSTEM" "ICN Docker Container Status:" "INFO"
  
  echo -e "\n${BLUE}=== ICN Docker Container Status ===${NC}"
  echo -e "${BLUE}Service\t\tStatus\t\tContainer ID${NC}"
  echo -e "${BLUE}-------\t\t------\t\t------------${NC}"
  
  for SERVICE in "${!SERVICE_DEPENDENCIES[@]}"; do
    local STATUS="STOPPED"
    local CONTAINER_ID=""
    
    if container_is_running "$SERVICE"; then
      STATUS="${GREEN}RUNNING${NC}"
      CONTAINER_ID=$(get_container_id "$SERVICE")
    else
      STATUS="${RED}STOPPED${NC}"
    fi
    
    printf "%-15s\t%-15s\t%s\n" "$SERVICE" "$STATUS" "$CONTAINER_ID"
  done
  
  echo -e "\n${BLUE}To check logs, use: docker logs <container-id>${NC}"
  echo -e "${BLUE}To stop a service, use: docker-compose stop <service-name>${NC}"
  echo -e "${BLUE}To stop all services, use: docker-compose down${NC}\n"
}

# Get services in reverse dependency order (for shutdown)
get_services_in_reverse_dependency_order() {
  # Start with all services
  local ALL_SERVICES=()
  for SERVICE in "${!SERVICE_DEPENDENCIES[@]}"; do
    ALL_SERVICES+=("$SERVICE")
  done
  
  # Simple topological sort
  local RESULT=()
  local REMAINING=("${ALL_SERVICES[@]}")
  
  while [ ${#REMAINING[@]} -gt 0 ]; do
    local FOUND=false
    
    for IDX in "${!REMAINING[@]}"; do
      local SERVICE="${REMAINING[$IDX]}"
      local DEPS="${SERVICE_DEPENDENCIES[$SERVICE]}"
      local ALL_DEPS_PROCESSED=true
      
      if [ -n "$DEPS" ]; then
        IFS=',' read -ra DEP_ARRAY <<< "$DEPS"
        for DEP in "${DEP_ARRAY[@]}"; do
          # Check if this dependency is still in remaining list
          if [[ " ${REMAINING[*]} " =~ " ${DEP} " ]] && [[ ! " ${RESULT[*]} " =~ " ${DEP} " ]]; then
            ALL_DEPS_PROCESSED=false
            break
          fi
        done
      fi
      
      if [ "$ALL_DEPS_PROCESSED" = true ]; then
        RESULT+=("$SERVICE")
        unset "REMAINING[$IDX]"
        REMAINING=("${REMAINING[@]}")  # Reindex array
        FOUND=true
        break
      fi
    done
    
    # If we couldn't find a service to add, we might have a cycle
    if [ "$FOUND" = false ]; then
      log_message "SYSTEM" "Warning: Possible circular dependency detected" "WARNING"
      # Add remaining services in any order
      RESULT+=("${REMAINING[@]}")
      break
    fi
  done
  
  # Return the reverse of the result for shutdown order
  local REVERSED=()
  for (( i=${#RESULT[@]}-1; i>=0; i-- )); do
    REVERSED+=("${RESULT[$i]}")
  done
  
  echo "${REVERSED[@]}"
}

# Graceful shutdown of Docker containers
shutdown_docker_services() {
  log_message "SYSTEM" "Initiating graceful shutdown of Docker services..." "INFO"
  
  # Determine which compose file to use
  local COMPOSE_FILE="docker-compose.yml"
  if [ "$ICN_NETWORK_MODE" = "development" ] && [ -f "${PROJECT_ROOT}/docker/docker-compose.dev.yml" ]; then
    COMPOSE_FILE="docker-compose.dev.yml"
  fi
  
  # Get services in reverse dependency order
  local SERVICES=($(get_services_in_reverse_dependency_order))
  
  for SERVICE in "${SERVICES[@]}"; do
    if container_is_running "$SERVICE"; then
      log_message "$SERVICE" "Stopping service..." "INFO"
      (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" stop "$SERVICE") || {
        log_message "$SERVICE" "Failed to stop service gracefully" "WARNING"
      }
      update_service_registry "$SERVICE" "STOPPED"
    fi
  done
  
  log_message "SYSTEM" "All Docker services stopped" "INFO"
}

# Main function to start ICN using Docker
start_icn_docker() {
  log_message "SYSTEM" "Starting ICN with Docker..." "INFO"
  
  # Load and validate configuration
  load_configuration || {
    log_message "SYSTEM" "Configuration validation failed. Aborting startup." "ERROR"
    return 1
  }
  
  # Initialize service registry
  setup_service_registry
  
  # Stop existing containers if needed
  stop_existing_containers
  
  # Start all services
  start_all_docker_services || {
    log_message "SYSTEM" "Failed to start all required services. Aborting." "ERROR"
    return 1
  }
  
  log_message "SYSTEM" "ICN system startup with Docker completed successfully" "INFO"
  return 0
}

# Register signal handlers for graceful shutdown
trap "shutdown_docker_services" EXIT INT TERM
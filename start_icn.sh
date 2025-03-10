#!/bin/bash

# Start ICN components script
# This script starts all the necessary components for the ICN project

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

echo -e "${BLUE}========== ICN Project Startup ==========${NC}"

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Rust/Cargo not found. Please install Rust: https://rustup.rs/${NC}"; exit 1; }
command -v psql >/dev/null 2>&1 || { echo -e "${YELLOW}PostgreSQL client not found. Database integration will be skipped.${NC}"; }

# Create necessary directories if they don't exist
echo -e "${YELLOW}Setting up directories...${NC}"
mkdir -p "${PROJECT_ROOT}/logs"
mkdir -p "${PROJECT_ROOT}/data"
mkdir -p "${PROJECT_ROOT}/data/bootstrap"
mkdir -p "${PROJECT_ROOT}/data/validator1"
mkdir -p "${PROJECT_ROOT}/data/validator2"

# Load environment variables
echo -e "${YELLOW}Loading configuration...${NC}"

# Initialize default values from the Docker Compose configuration
# Database Configuration
export POSTGRES_USER=icnuser
export POSTGRES_PASSWORD=icnpass
export POSTGRES_DB=icndb
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}"

# Network Configuration
export BOOTSTRAP_NODE_PORT=9000
export BOOTSTRAP_API_PORT=8086
export VALIDATOR1_NODE_PORT=9001
export VALIDATOR1_API_PORT=8087
export VALIDATOR2_NODE_PORT=9002
export VALIDATOR2_API_PORT=8088
export BACKEND_API_PORT=8081
export FRONTEND_PORT=3000

# ICN Server Configuration
export ICN_SERVER_PORT=8085
export ICN_SERVER_HOST=0.0.0.0
export ICN_CORS_ORIGINS=http://localhost:3000
export REACT_APP_API_URL=http://localhost:8085/api

# Node Configuration
export COOPERATIVE_ID=icn-primary
export RUST_LOG=info
export RUST_BACKTRACE=1

# Load .env file if it exists
if [ -f "${PROJECT_ROOT}/.env" ]; then
  echo -e "${GREEN}Loading environment variables from .env file${NC}"
  export $(grep -v '^#' "${PROJECT_ROOT}/.env" | xargs)
fi

# Kill any existing processes
echo -e "${YELLOW}Checking for existing processes...${NC}"
pkill -f "icn-server" >/dev/null 2>&1 || true
pkill -f "react-scripts start" >/dev/null 2>&1 || true
pkill -f "icn_bin" >/dev/null 2>&1 || true
pkill -f "simple_node" >/dev/null 2>&1 || true
pkill -f "icn-validator" >/dev/null 2>&1 || true
pkill -f "icn-identity" >/dev/null 2>&1 || true
pkill -f "icn-messaging" >/dev/null 2>&1 || true
echo -e "${GREEN}Cleared previous processes.${NC}"

# Array to store PIDs of all started processes
declare -a SERVICE_PIDS

# Function to add a service PID to the array and save to .icn_pids file
add_service_pid() {
  local SERVICE_NAME=$1
  local PID=$2
  SERVICE_PIDS+=($PID)
  echo "${SERVICE_NAME}:${PID}" >> "${PROJECT_ROOT}/.icn_services"
}

# Function to check if a service is responding
check_service() {
  local SERVICE_NAME=$1
  local URL=$2
  local MAX_RETRIES=$3
  local RETRY_COUNT=0
  local SERVICE_READY=false

  echo -e "${YELLOW}Waiting for ${SERVICE_NAME} to be available...${NC}"
  
  while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s "${URL}" > /dev/null; then
      SERVICE_READY=true
      break
    fi
    
    RETRY_COUNT=$((RETRY_COUNT+1))
    sleep 1
    echo -n "."
  done
  
  echo ""  # New line after dots
  
  if [ "$SERVICE_READY" = true ]; then
    echo -e "${GREEN}${SERVICE_NAME} is running and responding.${NC}"
    return 0
  else
    echo -e "${RED}${SERVICE_NAME} did not respond within the timeout period.${NC}"
    return 1
  fi
}

# Start PostgreSQL if not already running
DB_PID=""
if command -v psql >/dev/null 2>&1; then
  echo -e "${YELLOW}Checking PostgreSQL status...${NC}"
  if pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo -e "${GREEN}PostgreSQL is already running.${NC}"
  else
    echo -e "${YELLOW}Starting PostgreSQL...${NC}"
    
    # Check if we're on a system with systemd
    if command -v systemctl >/dev/null 2>&1; then
      sudo systemctl start postgresql
      if pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
        echo -e "${GREEN}PostgreSQL started successfully.${NC}"
      else
        echo -e "${RED}Failed to start PostgreSQL. Please start it manually.${NC}"
        echo -e "${YELLOW}Continuing without database...${NC}"
      fi
    else
      echo -e "${YELLOW}Systemd not found. Please start PostgreSQL manually.${NC}"
      echo -e "${YELLOW}Continuing without database...${NC}"
    fi
  fi
  
  # Create database if it doesn't exist
  if pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    if psql -h localhost -p 5432 -U ${POSTGRES_USER} -lqt | cut -d \| -f 1 | grep -qw ${POSTGRES_DB}; then
      echo -e "${GREEN}Database ${POSTGRES_DB} already exists.${NC}"
    else
      echo -e "${YELLOW}Creating database ${POSTGRES_DB}...${NC}"
      createdb -h localhost -p 5432 -U ${POSTGRES_USER} ${POSTGRES_DB} || {
        echo -e "${RED}Failed to create database. Please create it manually.${NC}"
      }
    fi
  fi
else
  echo -e "${YELLOW}PostgreSQL client not found. Database checks skipped.${NC}"
fi

# Initialize empty services file
> "${PROJECT_ROOT}/.icn_services"

# Start the ICN Server (main API server)
echo -e "${YELLOW}Starting ICN server on port ${ICN_SERVER_PORT}...${NC}"
cd "${PROJECT_ROOT}/icn-server" && cargo run > "${PROJECT_ROOT}/logs/icn-server.log" 2>&1 &
SERVER_PID=$!
echo -e "${GREEN}ICN server started with PID ${SERVER_PID}${NC}"
add_service_pid "icn-server" $SERVER_PID

# Wait for server to be available
if ! check_service "ICN Server" "http://localhost:${ICN_SERVER_PORT}/api/v1/health" 30; then
  echo -e "${RED}ICN Server failed to start. Check logs/icn-server.log for details.${NC}"
  cat "${PROJECT_ROOT}/logs/icn-server.log"
  exit 1
fi

# Change to project root
cd "${PROJECT_ROOT}"

# Build consensus binaries if needed
if [ -d "${PROJECT_ROOT}/icn-consensus" ] && [ -f "${PROJECT_ROOT}/icn-consensus/Cargo.toml" ]; then
  echo -e "${YELLOW}Building consensus binaries...${NC}"
  cargo build -p icn-consensus-bin --bin simple_node
  if [ ! -f "${PROJECT_ROOT}/target/debug/simple_node" ]; then
    echo -e "${RED}Failed to build consensus binaries. Skipping consensus nodes...${NC}"
  else
    # Start Bootstrap Node
    echo -e "${YELLOW}Starting Bootstrap Node on port ${BOOTSTRAP_NODE_PORT}...${NC}"
    ${PROJECT_ROOT}/target/debug/simple_node -t bootstrap \
      -p ${BOOTSTRAP_NODE_PORT} \
      -a ${BOOTSTRAP_API_PORT} \
      > "${PROJECT_ROOT}/logs/bootstrap.log" 2>&1 &
    BOOTSTRAP_PID=$!
    echo -e "${GREEN}Bootstrap Node started with PID ${BOOTSTRAP_PID}${NC}"
    add_service_pid "bootstrap" $BOOTSTRAP_PID
    
    # Wait for bootstrap node to be available (using health endpoint)
    if ! check_service "Bootstrap Node" "http://localhost:${BOOTSTRAP_API_PORT}/api/v1/health" 30; then
      echo -e "${RED}Bootstrap Node failed to start. Check logs/bootstrap.log for details.${NC}"
      cat "${PROJECT_ROOT}/logs/bootstrap.log"
      echo -e "${YELLOW}Continuing without Bootstrap Node...${NC}"
    fi
    
    # Start Validator 1
    echo -e "${YELLOW}Starting Validator Node 1 on port ${VALIDATOR1_NODE_PORT}...${NC}"
    ${PROJECT_ROOT}/target/debug/simple_node -t validator \
      -p ${VALIDATOR1_NODE_PORT} \
      -a ${VALIDATOR1_API_PORT} \
      -b "ws://localhost:${BOOTSTRAP_NODE_PORT}" \
      > "${PROJECT_ROOT}/logs/validator1.log" 2>&1 &
    VALIDATOR1_PID=$!
    echo -e "${GREEN}Validator Node 1 started with PID ${VALIDATOR1_PID}${NC}"
    add_service_pid "validator1" $VALIDATOR1_PID
    
    # Wait for validator 1 to be available
    if ! check_service "Validator Node 1" "http://localhost:${VALIDATOR1_API_PORT}/api/v1/health" 30; then
      echo -e "${RED}Validator Node 1 failed to start. Check logs/validator1.log for details.${NC}"
      cat "${PROJECT_ROOT}/logs/validator1.log"
      echo -e "${YELLOW}Continuing without Validator Node 1...${NC}"
    fi
    
    # Start Validator 2
    echo -e "${YELLOW}Starting Validator Node 2 on port ${VALIDATOR2_NODE_PORT}...${NC}"
    ${PROJECT_ROOT}/target/debug/simple_node -t validator \
      -p ${VALIDATOR2_NODE_PORT} \
      -a ${VALIDATOR2_API_PORT} \
      -b "ws://localhost:${BOOTSTRAP_NODE_PORT}" \
      > "${PROJECT_ROOT}/logs/validator2.log" 2>&1 &
    VALIDATOR2_PID=$!
    echo -e "${GREEN}Validator Node 2 started with PID ${VALIDATOR2_PID}${NC}"
    add_service_pid "validator2" $VALIDATOR2_PID
    
    # Wait for validator 2 to be available
    if ! check_service "Validator Node 2" "http://localhost:${VALIDATOR2_API_PORT}/api/v1/health" 30; then
      echo -e "${RED}Validator Node 2 failed to start. Check logs/validator2.log for details.${NC}"
      cat "${PROJECT_ROOT}/logs/validator2.log"
      echo -e "${YELLOW}Continuing without Validator Node 2...${NC}"
    fi
  fi
else
  echo -e "${YELLOW}Consensus components not found. Skipping...${NC}"
fi

# Start Identity Service if available
if [ -d "${PROJECT_ROOT}/identity" ]; then
  echo -e "${YELLOW}Starting Identity Service...${NC}"
  
  # Check for the appropriate executable
  if [ -f "${PROJECT_ROOT}/identity/service.py" ]; then
    # Python service (using module approach instead of relative import)
    cd "${PROJECT_ROOT}"
    python3 -m identity.service > "${PROJECT_ROOT}/logs/identity.log" 2>&1 &
    IDENTITY_PID=$!
    echo -e "${GREEN}Identity Service started with PID ${IDENTITY_PID}${NC}"
    add_service_pid "identity" $IDENTITY_PID
  elif [ -f "${PROJECT_ROOT}/identity/Cargo.toml" ]; then
    # Rust service
    cd "${PROJECT_ROOT}/identity" && cargo run > "${PROJECT_ROOT}/logs/identity.log" 2>&1 &
    IDENTITY_PID=$!
    echo -e "${GREEN}Identity Service started with PID ${IDENTITY_PID}${NC}"
    add_service_pid "identity" $IDENTITY_PID
  else
    echo -e "${RED}No executable found for Identity Service. Skipping...${NC}"
  fi
  
  cd "${PROJECT_ROOT}"
else
  echo -e "${YELLOW}Identity Service component not found. Skipping...${NC}"
fi

# Start Messaging Service if available
if [ -d "${PROJECT_ROOT}/messaging" ] && [ -f "${PROJECT_ROOT}/messaging/server.rs" ]; then
  echo -e "${YELLOW}Starting Messaging Service...${NC}"
  
  # Try to find the appropriate binary
  cd "${PROJECT_ROOT}/messaging"
  
  # Check available binaries
  MESSAGING_BIN=""
  if cargo build --bin messaging > /dev/null 2>&1; then
    MESSAGING_BIN="messaging"
  elif cargo build --bin icn_bin > /dev/null 2>&1; then
    MESSAGING_BIN="icn_bin"
    # Set specific env var to indicate messaging mode
    export SERVICE_TYPE=messaging
  fi
  
  if [ -n "$MESSAGING_BIN" ]; then
    cargo run --bin $MESSAGING_BIN > "${PROJECT_ROOT}/logs/messaging.log" 2>&1 &
    MESSAGING_PID=$!
    echo -e "${GREEN}Messaging Service started with PID ${MESSAGING_PID}${NC}"
    add_service_pid "messaging" $MESSAGING_PID
  else
    # Try to run it as a library instead
    echo -e "${YELLOW}No messaging binary found, trying to compile as a library...${NC}"
    cargo run --example messaging_server > "${PROJECT_ROOT}/logs/messaging.log" 2>&1 &
    MESSAGING_PID=$!
    echo -e "${GREEN}Messaging Service started with PID ${MESSAGING_PID}${NC}"
    add_service_pid "messaging" $MESSAGING_PID
  fi
  
  cd "${PROJECT_ROOT}"
else
  echo -e "${YELLOW}Messaging Service component not found. Skipping...${NC}"
fi

# Try to start the frontend if it exists
FRONTEND_PID=""
if [ -d "${PROJECT_ROOT}/frontend" ] && [ -f "${PROJECT_ROOT}/frontend/package.json" ]; then
  # Check if npm is installed
  if command -v npm >/dev/null 2>&1; then
    echo -e "${YELLOW}Starting frontend on port 3000...${NC}"
    cd "${PROJECT_ROOT}/frontend"
    
    # Check if we need to install dependencies
    if [ ! -d "node_modules" ]; then
      echo -e "${YELLOW}Installing frontend dependencies...${NC}"
      npm install --silent > "${PROJECT_ROOT}/logs/frontend-install.log" 2>&1 || {
        echo -e "${RED}Failed to install frontend dependencies. Check logs/frontend-install.log for details.${NC}"
        cat "${PROJECT_ROOT}/logs/frontend-install.log"
      }
    fi
    
    # Start the frontend if dependencies installed successfully
    if [ -d "node_modules" ]; then
      npm start > "${PROJECT_ROOT}/logs/frontend.log" 2>&1 &
      FRONTEND_PID=$!
      echo -e "${GREEN}Frontend started with PID ${FRONTEND_PID}${NC}"
      add_service_pid "frontend" $FRONTEND_PID
      
      # Brief pause to ensure frontend is starting
      sleep 3
      
      # Check if the frontend started successfully
      if ps -p $FRONTEND_PID > /dev/null; then
        echo -e "${GREEN}Frontend is running.${NC}"
      else
        echo -e "${RED}Frontend failed to start. Check logs/frontend.log for details.${NC}"
        echo -e "${YELLOW}Continuing without frontend...${NC}"
      fi
    else
      echo -e "${YELLOW}Frontend dependencies not available. Running in backend-only mode.${NC}"
    fi
  else
    echo -e "${YELLOW}npm not found. Running in backend-only mode.${NC}"
  fi
else
  echo -e "${YELLOW}Frontend directory not found. Running in backend-only mode.${NC}"
fi

# Print available endpoints
echo -e "${BLUE}========== ICN ENDPOINTS ==========${NC}"
echo -e "${GREEN}Main Server:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/health"
echo -e "${GREEN}Resources:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/resources"
echo -e "${GREEN}Identities:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/identities"
echo -e "${GREEN}Cooperatives:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/cooperatives"

if [ -f "${PROJECT_ROOT}/target/debug/simple_node" ]; then
  echo -e "${GREEN}Bootstrap Node:${NC} http://localhost:${BOOTSTRAP_API_PORT}/api/v1/health"
  echo -e "${GREEN}Validator 1:${NC} http://localhost:${VALIDATOR1_API_PORT}/api/v1/health"
  echo -e "${GREEN}Validator 2:${NC} http://localhost:${VALIDATOR2_API_PORT}/api/v1/health"
fi

if [ ! -z "$FRONTEND_PID" ]; then
  echo -e "${GREEN}Frontend:${NC} http://localhost:${FRONTEND_PORT}"
fi

echo -e "${BLUE}========== ICN RUNNING ==========${NC}"
echo -e "${YELLOW}To stop all services, run: ./stop_icn.sh${NC}"
echo -e "${YELLOW}Logs are available in the logs directory${NC}"

echo -e "${GREEN}Process IDs saved to .icn_services${NC}"

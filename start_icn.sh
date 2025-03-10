#!/bin/bash

# Start ICN components script
# This script starts all the necessary components for the ICN project

set -e  # Exit on error

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
command -v npm >/dev/null 2>&1 || { echo -e "${RED}Node.js/npm not found. Please install Node.js: https://nodejs.org/${NC}"; exit 1; }

# Create necessary directories if they don't exist
echo -e "${YELLOW}Setting up directories...${NC}"
mkdir -p "${PROJECT_ROOT}/logs"
mkdir -p "${PROJECT_ROOT}/data"

# Set environment variables for configuration
export ICN_SERVER_PORT=8085
export ICN_SERVER_HOST=0.0.0.0
export RUST_LOG=info
export ICN_CORS_ORIGINS=http://localhost:3000
export REACT_APP_API_URL=http://localhost:8085/api

# Load .env file if it exists
if [ -f "${PROJECT_ROOT}/.env" ]; then
  echo -e "${GREEN}Loading environment variables from .env file${NC}"
  export $(grep -v '^#' "${PROJECT_ROOT}/.env" | xargs)
fi

# Kill any existing processes running on our ports
echo -e "${YELLOW}Checking for existing processes...${NC}"
pkill -f "icn-server" >/dev/null 2>&1 || true
pkill -f "react-scripts start" >/dev/null 2>&1 || true
echo -e "${GREEN}Cleared previous processes.${NC}"

# Start the backend server
echo -e "${YELLOW}Starting ICN server on port ${ICN_SERVER_PORT}...${NC}"
cd "${PROJECT_ROOT}/icn-server" && cargo run > "${PROJECT_ROOT}/logs/icn-server.log" 2>&1 &
SERVER_PID=$!
echo -e "${GREEN}ICN server started with PID ${SERVER_PID}${NC}"

# Brief pause to ensure server is up
sleep 2

# Check if the server started successfully
if ps -p $SERVER_PID > /dev/null; then
    echo -e "${GREEN}Server is running.${NC}"
else
    echo -e "${RED}Server failed to start. Check logs/icn-server.log for details.${NC}"
    cat "${PROJECT_ROOT}/logs/icn-server.log"
    exit 1
fi

# Change to project root
cd "${PROJECT_ROOT}"

# Start the frontend
echo -e "${YELLOW}Starting frontend on port 3000...${NC}"
cd "${PROJECT_ROOT}/frontend" && npm start > "${PROJECT_ROOT}/logs/frontend.log" 2>&1 &
FRONTEND_PID=$!
echo -e "${GREEN}Frontend started with PID ${FRONTEND_PID}${NC}"

# Brief pause to ensure frontend is starting
sleep 3

# Check if the frontend started successfully
if ps -p $FRONTEND_PID > /dev/null; then
    echo -e "${GREEN}Frontend is running.${NC}"
else
    echo -e "${RED}Frontend failed to start. Check logs/frontend.log for details.${NC}"
    cat "${PROJECT_ROOT}/logs/frontend.log"
    exit 1
fi

# Print available endpoints
echo -e "${BLUE}========== ICN ENDPOINTS ==========${NC}"
echo -e "${GREEN}Backend API:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/health"
echo -e "${GREEN}Resources:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/resources"
echo -e "${GREEN}Identities:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/identities"
echo -e "${GREEN}Cooperatives:${NC} http://${ICN_SERVER_HOST}:${ICN_SERVER_PORT}/api/v1/cooperatives"
echo -e "${GREEN}Frontend:${NC} http://localhost:3000"

echo -e "${BLUE}========== ICN RUNNING ==========${NC}"
echo -e "${YELLOW}To stop all services, run:${NC}"
echo -e "${YELLOW}kill $SERVER_PID $FRONTEND_PID${NC}"
echo -e "${YELLOW}Logs are available in the logs directory${NC}"

# Save PIDs for easy shutdown
echo "$SERVER_PID $FRONTEND_PID" > "${PROJECT_ROOT}/.icn_pids"
echo -e "${GREEN}Process IDs saved to .icn_pids${NC}"

# Create a stop script for convenient shutdown
cat > "${PROJECT_ROOT}/stop_icn.sh" << EOL
#!/bin/bash
if [ -f "${PROJECT_ROOT}/.icn_pids" ]; then
  echo "Stopping ICN services..."
  kill \$(cat "${PROJECT_ROOT}/.icn_pids") 2>/dev/null || true
  rm "${PROJECT_ROOT}/.icn_pids"
  echo "All ICN services stopped."
else
  echo "No running ICN services found."
fi
EOL

chmod +x "${PROJECT_ROOT}/stop_icn.sh"
echo -e "${GREEN}Created stop_icn.sh for convenient shutdown${NC}"

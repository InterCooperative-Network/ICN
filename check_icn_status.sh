#!/bin/bash

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========== ICN Status Check ==========${NC}"

# Check if the services file exists
if [ ! -f "${PROJECT_ROOT}/.icn_services" ]; then
  echo -e "${RED}No ICN services are registered.${NC}"
  echo -e "${YELLOW}If you believe services are running, they might have been started outside the start_icn.sh script.${NC}"
  exit 1
fi

# Read configuration from .env file if it exists
if [ -f "${PROJECT_ROOT}/.env" ]; then
  source "${PROJECT_ROOT}/.env"
fi

# Set default values if not in .env
ICN_SERVER_PORT=${ICN_SERVER_PORT:-8085}
BOOTSTRAP_API_PORT=${BOOTSTRAP_API_PORT:-8082}
VALIDATOR1_API_PORT=${VALIDATOR1_API_PORT:-8083}
VALIDATOR2_API_PORT=${VALIDATOR2_API_PORT:-8084}
FRONTEND_PORT=${FRONTEND_PORT:-3000}

# Function to check if a service is responding
check_endpoint() {
  local NAME=$1
  local URL=$2
  
  echo -ne "${NAME}: "
  
  RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$URL" 2>/dev/null)
  
  if [ "$RESPONSE" = "200" ]; then
    echo -e "${GREEN}Running (HTTP 200)${NC}"
    return 0
  else
    echo -e "${RED}Not responding (HTTP $RESPONSE)${NC}"
    return 1
  fi
}

# Check PostgreSQL
if command -v pg_isready >/dev/null 2>&1; then
  echo -ne "PostgreSQL: "
  if pg_isready -h localhost -p 5432 -q; then
    echo -e "${GREEN}Running${NC}"
  else
    echo -e "${RED}Not running${NC}"
  fi
fi

# Check registered services
echo -e "\n${BLUE}Registered Services:${NC}"
while IFS=: read -r SERVICE_NAME PID || [ -n "$SERVICE_NAME" ]; do
  if [ -n "$PID" ]; then
    echo -ne "${SERVICE_NAME} (PID ${PID}): "
    if ps -p $PID > /dev/null; then
      echo -e "${GREEN}Running${NC}"
    else
      echo -e "${RED}Process is dead${NC}"
    fi
  fi
done < "${PROJECT_ROOT}/.icn_services"

# Check API endpoints
echo -e "\n${BLUE}API Endpoints:${NC}"
check_endpoint "Main Server" "http://localhost:${ICN_SERVER_PORT}/api/v1/health"
check_endpoint "Resources API" "http://localhost:${ICN_SERVER_PORT}/api/v1/resources"
check_endpoint "Identities API" "http://localhost:${ICN_SERVER_PORT}/api/v1/identities"
check_endpoint "Cooperatives API" "http://localhost:${ICN_SERVER_PORT}/api/v1/cooperatives"

# Check Bootstrap and Validator nodes if they exist
if grep -q "bootstrap:" "${PROJECT_ROOT}/.icn_services"; then
  check_endpoint "Bootstrap Node" "http://localhost:${BOOTSTRAP_API_PORT}/api/v1/status"
fi

if grep -q "validator1:" "${PROJECT_ROOT}/.icn_services"; then
  check_endpoint "Validator 1" "http://localhost:${VALIDATOR1_API_PORT}/api/v1/status"
fi

if grep -q "validator2:" "${PROJECT_ROOT}/.icn_services"; then
  check_endpoint "Validator 2" "http://localhost:${VALIDATOR2_API_PORT}/api/v1/status"
fi

# Check frontend if it exists
if grep -q "frontend:" "${PROJECT_ROOT}/.icn_services"; then
  echo -ne "Frontend: "
  RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${FRONTEND_PORT}" 2>/dev/null)
  
  if [ "$RESPONSE" = "200" ] || [ "$RESPONSE" = "304" ]; then
    echo -e "${GREEN}Running (HTTP $RESPONSE)${NC}"
  else
    echo -e "${RED}Not responding (HTTP $RESPONSE)${NC}"
  fi
fi

# System resource usage
echo -e "\n${BLUE}System Resource Usage:${NC}"
echo -e "CPU and Memory usage of ICN processes:"
echo "------------------------------------------"
ps -o pid,ppid,%cpu,%mem,cmd -p $(cut -d: -f2 "${PROJECT_ROOT}/.icn_services" | tr '\n' ' ') 2>/dev/null || echo "No processes found"

echo -e "\n${BLUE}Disk Usage:${NC}"
du -sh "${PROJECT_ROOT}/logs" "${PROJECT_ROOT}/data" 2>/dev/null

echo -e "\n${BLUE}Log File Sizes:${NC}"
ls -lh "${PROJECT_ROOT}/logs" | grep -v "^total" | awk '{print $5 "\t" $9}' 
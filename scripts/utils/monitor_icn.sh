#!/bin/bash

# ICN Service Monitor
# This script monitors the ICN services and restarts any that have failed

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========== ICN Service Monitor ==========${NC}"

# Check if the services file exists
if [ ! -f "${PROJECT_ROOT}/.icn_services" ]; then
  echo -e "${RED}No ICN services are registered.${NC}"
  echo -e "${YELLOW}Starting ICN services first...${NC}"
  ./start_icn.sh
  exit 0
fi

# Read configuration from .env file if it exists
if [ -f "${PROJECT_ROOT}/.env" ]; then
  source "${PROJECT_ROOT}/.env"
fi

# Set default values if not in .env
ICN_SERVER_PORT=${ICN_SERVER_PORT:-8085}

# Function to check if a service is responding
check_endpoint() {
  local URL=$1
  
  RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$URL" 2>/dev/null)
  
  if [ "$RESPONSE" = "200" ]; then
    return 0
  else
    return 1
  fi
}

# Check if the main server is running
if ! check_endpoint "http://localhost:${ICN_SERVER_PORT}/api/v1/health"; then
  echo -e "${RED}Main ICN server is not responding. Restarting services...${NC}"
  ./stop_icn.sh
  sleep 2
  ./start_icn.sh
  exit 0
fi

# Check each registered service
echo -e "${YELLOW}Checking registered services...${NC}"
RESTART_NEEDED=false

while IFS=: read -r SERVICE_NAME PID || [ -n "$SERVICE_NAME" ]; do
  if [ -n "$PID" ]; then
    echo -ne "${SERVICE_NAME} (PID ${PID}): "
    if ps -p $PID > /dev/null; then
      echo -e "${GREEN}Running${NC}"
    else
      echo -e "${RED}Process is dead${NC}"
      RESTART_NEEDED=true
    fi
  fi
done < "${PROJECT_ROOT}/.icn_services"

# Restart services if needed
if [ "$RESTART_NEEDED" = true ]; then
  echo -e "${YELLOW}Some services have failed. Restarting all services...${NC}"
  ./stop_icn.sh
  sleep 2
  ./start_icn.sh
else
  echo -e "${GREEN}All services are running.${NC}"
fi

# Print system resource usage
echo -e "\n${BLUE}System Resource Usage:${NC}"
echo -e "CPU and Memory usage of ICN processes:"
echo "------------------------------------------"
ps -o pid,ppid,%cpu,%mem,cmd -p $(cut -d: -f2 "${PROJECT_ROOT}/.icn_services" | tr '\n' ' ') 2>/dev/null || echo "No processes found"

echo -e "\n${BLUE}Disk Usage:${NC}"
du -sh "${PROJECT_ROOT}/logs" "${PROJECT_ROOT}/data" 2>/dev/null

echo -e "\n${YELLOW}To set up automatic monitoring, add this to your crontab:${NC}"
echo "*/5 * * * * $(realpath $0) > ${PROJECT_ROOT}/logs/monitor.log 2>&1" 
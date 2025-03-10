#!/bin/bash

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Stopping ICN services...${NC}"

# Check if services file exists
if [ -f "${PROJECT_ROOT}/.icn_services" ]; then
  # Read and process each service entry
  while IFS=: read -r SERVICE_NAME PID || [ -n "$SERVICE_NAME" ]; do
    if [ -n "$PID" ] && ps -p $PID > /dev/null; then
      echo -e "Stopping ${SERVICE_NAME} with PID ${PID}..."
      kill $PID 2>/dev/null || true
      sleep 0.5
    else
      echo -e "Process ${SERVICE_NAME} with PID ${PID} is not running."
    fi
  done < "${PROJECT_ROOT}/.icn_services"
  
  # Remove the services file
  rm "${PROJECT_ROOT}/.icn_services"
  echo -e "${GREEN}All ICN services stopped.${NC}"
else
  echo -e "${RED}No running ICN services found.${NC}"
  
  # Try to find and kill any running services anyway
  echo -e "${YELLOW}Attempting to find and stop any ICN processes...${NC}"
  pkill -f "icn-server" >/dev/null 2>&1 || true
  pkill -f "react-scripts start" >/dev/null 2>&1 || true
  pkill -f "icn_bin" >/dev/null 2>&1 || true
  pkill -f "simple_node" >/dev/null 2>&1 || true
  pkill -f "icn-validator" >/dev/null 2>&1 || true
  pkill -f "icn-identity" >/dev/null 2>&1 || true
  pkill -f "icn-messaging" >/dev/null 2>&1 || true
  echo -e "${GREEN}Done.${NC}"
fi

# Check for PostgreSQL
if command -v pg_isready >/dev/null 2>&1 && command -v psql >/dev/null 2>&1; then
  echo -e "${YELLOW}Do you want to stop PostgreSQL? (y/n)${NC}"
  read -r response
  if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    if command -v systemctl >/dev/null 2>&1; then
      echo -e "${YELLOW}Stopping PostgreSQL...${NC}"
      sudo systemctl stop postgresql
      echo -e "${GREEN}PostgreSQL stopped.${NC}"
    else
      echo -e "${RED}Systemd not found. Please stop PostgreSQL manually if needed.${NC}"
    fi
  else
    echo -e "${YELLOW}PostgreSQL will continue running.${NC}"
  fi
fi 
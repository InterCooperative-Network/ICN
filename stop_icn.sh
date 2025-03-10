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

if [ -f "${PROJECT_ROOT}/.icn_pids" ]; then
  PIDS=$(cat "${PROJECT_ROOT}/.icn_pids")
  for PID in $PIDS; do
    if ps -p $PID > /dev/null; then
      echo -e "Stopping process with PID ${PID}..."
      kill $PID 2>/dev/null || true
    else
      echo -e "Process with PID ${PID} is not running."
    fi
  done
  rm "${PROJECT_ROOT}/.icn_pids"
  echo -e "${GREEN}All ICN services stopped.${NC}"
else
  echo -e "${RED}No running ICN services found.${NC}"
  
  # Try to find and kill any running services anyway
  echo -e "${YELLOW}Attempting to find and stop any ICN processes...${NC}"
  pkill -f "icn-server" >/dev/null 2>&1 || true
  pkill -f "react-scripts start" >/dev/null 2>&1 || true
  echo -e "${GREEN}Done.${NC}"
fi 
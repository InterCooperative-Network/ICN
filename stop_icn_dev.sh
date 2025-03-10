#!/bin/bash
# Script to stop the ICN development processes

set -e

# Function to stop processes
stop_processes() {
  echo "Stopping ICN processes..."
  
  # Find and kill the backend process
  BACKEND_PID=$(pgrep -f "icn-backend")
  if [ -n "$BACKEND_PID" ]; then
    echo "Stopping backend (PID: $BACKEND_PID)..."
    kill $BACKEND_PID 2>/dev/null || true
  else
    echo "No backend process found."
  fi
  
  # Find and kill the frontend (React) process
  FRONTEND_PID=$(pgrep -f "react-scripts start")
  if [ -n "$FRONTEND_PID" ]; then
    echo "Stopping frontend (PID: $FRONTEND_PID)..."
    kill $FRONTEND_PID 2>/dev/null || true
  else
    echo "No frontend process found."
  fi
  
  echo "All ICN processes stopped."
}

# Execute the function
stop_processes 
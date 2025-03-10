#!/bin/bash
# Development script for running the ICN system locally

set -e

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Create necessary directories
mkdir -p "${PROJECT_ROOT}/data"
mkdir -p "${PROJECT_ROOT}/logs"

# Setup environment variables
export RUST_LOG=debug
export RUST_BACKTRACE=1
export API_PORT=8081
export API_HOST=0.0.0.0
export COOPERATIVE_ID=icn-primary
export ICN_NETWORK_MODE=development
export REACT_APP_API_URL=http://localhost:8081/api
export NODE_ENV=development

# Check if frontend or backend is specified
RUN_COMPONENT=${1:-"all"}  # Default to "all" if not specified

run_backend() {
  echo "Building and running ICN backend..."
  echo "Logs will be written to ${PROJECT_ROOT}/logs/backend.log"
  cd "${PROJECT_ROOT}/backend"
  cargo run -- --port ${API_PORT} --host ${API_HOST} > "${PROJECT_ROOT}/logs/backend.log" 2>&1 &
  echo "Backend started with PID $!"
}

run_frontend() {
  echo "Building and running ICN frontend..."
  echo "Logs will be written to ${PROJECT_ROOT}/logs/frontend.log"
  cd "${PROJECT_ROOT}/frontend"
  if [ ! -d "node_modules" ]; then
    echo "Installing frontend dependencies..."
    npm install
  fi
  npm start > "${PROJECT_ROOT}/logs/frontend.log" 2>&1 &
  echo "Frontend started with PID $!"
}

case ${RUN_COMPONENT} in
  "backend")
    run_backend
    ;;
  "frontend")
    run_frontend
    ;;
  "all")
    run_backend
    sleep 5  # Give the backend some time to start
    run_frontend
    echo "Both backend and frontend are now running."
    echo "Press Ctrl+C to stop both processes."
    trap "pkill -P $$; exit" SIGINT SIGTERM
    wait
    ;;
  *)
    echo "Unknown component: ${RUN_COMPONENT}"
    echo "Usage: $0 [backend|frontend|all]"
    exit 1
    ;;
esac

# If we're running a single component, keep it in the foreground
if [ "${RUN_COMPONENT}" != "all" ]; then
  echo "${RUN_COMPONENT} is running. Press Ctrl+C to stop."
  wait
fi 
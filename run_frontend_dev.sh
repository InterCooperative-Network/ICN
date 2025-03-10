#!/bin/bash
# Direct run script for the ICN frontend

set -e

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Setup environment variables
export REACT_APP_API_URL=http://localhost:8081/api
export NODE_ENV=development

# Build and run the frontend
echo "Building and running ICN frontend..."
cd frontend
npm install
npm start 
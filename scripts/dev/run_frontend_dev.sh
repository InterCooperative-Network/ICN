#!/bin/bash
# Development script for running the frontend

# Set error handling
set -e

# Load environment variables
if [ -f ".env" ]; then
    source .env
else
    echo "Error: .env file not found"
    exit 1
fi

# Ensure we're in the frontend directory
cd frontend || {
    echo "Error: frontend directory not found"
    exit 1
}

# Set default port if not specified
FRONTEND_PORT=${FRONTEND_PORT:-3000}
FRONTEND_HOST=${FRONTEND_HOST:-0.0.0.0}

echo "Starting frontend development server on $FRONTEND_HOST:$FRONTEND_PORT..."

# Run the frontend in development mode
npm run dev 
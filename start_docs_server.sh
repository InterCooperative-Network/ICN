#!/bin/bash

# Navigate to the project root directory
PROJECT_ROOT_DIR="$(dirname "$(readlink -f "$0")")"
cd "$PROJECT_ROOT_DIR/docker" || exit

# Stop any running documentation container if it exists
echo "Stopping any existing docs container..."
docker-compose down

# Rebuild and bring up the documentation server only
echo "Starting the documentation server..."
docker-compose up --build docs


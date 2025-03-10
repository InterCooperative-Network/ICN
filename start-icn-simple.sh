#!/bin/bash
# Simple script to start the ICN system

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Run the start script
"${PROJECT_ROOT}/docker/scripts/start-icn.sh"

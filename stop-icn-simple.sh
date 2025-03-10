#!/bin/bash
# Simple script to stop the ICN system

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Run the stop script
"${PROJECT_ROOT}/docker/scripts/stop-icn.sh"

#!/bin/bash

# ICN System Startup Script
# Version: 2.0
# Description: Starts all components of the ICN system with proper dependency management and error handling

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Initialize global variables
declare -a SERVICE_PIDS
export STARTUP_TIMESTAMP=$(date +"%Y%m%d-%H%M%S")

# Source the dependency management and utility functions
source "${PROJECT_ROOT}/scripts/startup-utils.sh"

# Main execution
main() {
    echo "=== ICN System Startup ==="
    echo "Starting ICN system at $(date)"
    
    # Setup logging
    setup_logging
    
    # Load and validate configuration
    log_message "SYSTEM" "Loading configuration..." "INFO"
    load_configuration || {
        log_message "SYSTEM" "Configuration validation failed. Aborting startup." "ERROR"
        exit 1
    }
    
    # Initialize service registry
    setup_service_registry
    
    # Kill any existing processes (with confirmation in interactive mode)
    if [ -t 1 ]; then  # Check if stdout is a terminal
        read -p "Do you want to stop any existing ICN processes? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            stop_existing_processes
        fi
    else
        stop_existing_processes
    fi
    
    # Start all services
    start_all_services
    
    # Final status message
    log_message "SYSTEM" "ICN system startup completed" "INFO"
    echo "=== ICN System Startup Completed ==="
    echo "Run './check_icn_status.sh' to view the status of all components"
}

# Execute main function
main

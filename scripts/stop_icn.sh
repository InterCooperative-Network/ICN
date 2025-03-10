#!/bin/bash

# ICN System Shutdown Script
# Version: 3.0
# Description: Stops all components of the ICN system in the correct dependency order

# Ensure we're in the right directory
cd "$(dirname "$0")/.."
export PROJECT_ROOT=$(pwd)

# Source the dependency management and utility functions
source "${PROJECT_ROOT}/scripts/startup-utils.sh"

# Main execution
main() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}     ICN System Docker Shutdown Tool    ${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "Stopping ICN system at $(date)"
    
    # Setup logging
    setup_logging
    
    # Ask for confirmation in interactive mode
    if [ -t 1 ]; then  # Check if stdout is a terminal
        echo -e "\n${YELLOW}This script will stop all ICN Docker containers.${NC}"
        echo -e "${YELLOW}All services will be gracefully shut down in the correct order.${NC}\n"
        
        read -p "Do you want to proceed? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${RED}Shutdown aborted by user.${NC}"
            exit 0
        fi
        
        echo -e "\n${YELLOW}Choose shutdown option:${NC}"
        echo -e "1) Stop containers (preserves data volumes)"
        echo -e "2) Stop and remove containers (preserves data volumes)"
        echo -e "3) Stop, remove containers and delete volumes (CAUTION: all data will be lost)\n"
        read -p "Enter option (1/2/3): " -n 1 -r
        echo
        
        case $REPLY in
            1)
                export SHUTDOWN_MODE="stop"
                echo -e "${GREEN}Stopping containers only${NC}"
                ;;
            2)
                export SHUTDOWN_MODE="down"
                echo -e "${GREEN}Stopping and removing containers${NC}"
                ;;
            3)
                export SHUTDOWN_MODE="down_volumes"
                echo -e "${RED}WARNING: All data will be lost! Stopping and removing containers and volumes${NC}"
                read -p "Are you sure? This will delete ALL DATA (y/n): " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    echo -e "${GREEN}Abort. Using 'stop' mode instead${NC}"
                    export SHUTDOWN_MODE="stop"
                fi
                ;;
            *)
                export SHUTDOWN_MODE="stop"
                echo -e "${GREEN}Invalid option. Using 'stop' mode${NC}"
                ;;
        esac
    else
        # Default to safe mode in non-interactive
        export SHUTDOWN_MODE="stop"
    fi
    
    # Load configuration for Docker settings
    load_configuration || {
        log_message "SYSTEM" "Configuration validation failed, but proceeding with shutdown" "WARNING"
    }
    
    # Determine which compose file to use
    local COMPOSE_FILE="docker-compose.yml"
    if [ "$ICN_NETWORK_MODE" = "development" ] && [ -f "${PROJECT_ROOT}/docker/docker-compose.dev.yml" ]; then
        COMPOSE_FILE="docker-compose.dev.yml"
        log_message "SYSTEM" "Using development compose file" "INFO"
    fi
    
    log_message "SYSTEM" "Starting shutdown process with mode: $SHUTDOWN_MODE" "INFO"
    
    # Execute the appropriate shutdown command
    case $SHUTDOWN_MODE in
        "stop")
            log_message "SYSTEM" "Stopping containers..." "INFO"
            (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" stop) || {
                log_message "SYSTEM" "Failed to stop containers" "ERROR"
                echo -e "${RED}Failed to stop containers. Try manually with:${NC}"
                echo -e "cd ${PROJECT_ROOT}/docker && docker-compose -f $COMPOSE_FILE stop"
                exit 1
            }
            ;;
        "down")
            log_message "SYSTEM" "Stopping and removing containers..." "INFO"
            (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" down) || {
                log_message "SYSTEM" "Failed to stop and remove containers" "ERROR"
                echo -e "${RED}Failed to stop and remove containers. Try manually with:${NC}"
                echo -e "cd ${PROJECT_ROOT}/docker && docker-compose -f $COMPOSE_FILE down"
                exit 1
            }
            ;;
        "down_volumes")
            log_message "SYSTEM" "Stopping and removing containers and volumes..." "INFO"
            (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" down -v) || {
                log_message "SYSTEM" "Failed to stop and remove containers and volumes" "ERROR"
                echo -e "${RED}Failed to stop and remove containers and volumes. Try manually with:${NC}"
                echo -e "cd ${PROJECT_ROOT}/docker && docker-compose -f $COMPOSE_FILE down -v"
                exit 1
            }
            ;;
    esac
    
    # Final status message
    echo -e "\n${GREEN}ICN system shutdown completed successfully!${NC}"
    
    # Remove service registry if it exists
    if [ -f "${PROJECT_ROOT}/.service_registry" ]; then
        rm "${PROJECT_ROOT}/.service_registry"
        log_message "SYSTEM" "Service registry removed" "INFO"
    fi
    
    echo -e "\n${BLUE}ICN system has been shut down.${NC}"
    echo -e "${YELLOW}Log file: ${MASTER_LOG}${NC}"
}

# Execute main function
main 
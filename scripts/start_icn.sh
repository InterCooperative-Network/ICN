#!/bin/bash

# ICN System Startup Script
# Version: 3.0
# Description: Starts all components of the ICN system using Docker containers with proper 
#              dependency management and error handling

# Ensure we're in the right directory
cd "$(dirname "$0")/.."
export PROJECT_ROOT=$(pwd)

# Source the dependency management and utility functions
source "${PROJECT_ROOT}/scripts/startup-utils.sh"

# Main execution
main() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}     ICN System Docker Startup Tool     ${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "Starting ICN system at $(date)"
    
    # Setup logging
    setup_logging
    
    # Ask for confirmation in interactive mode
    if [ -t 1 ]; then  # Check if stdout is a terminal
        echo -e "\n${YELLOW}This script will manage Docker containers for the ICN system.${NC}"
        echo -e "${YELLOW}It will start all required services in the correct dependency order.${NC}"
        echo -e "${YELLOW}Existing containers may be stopped if they're running.${NC}\n"
        
        read -p "Do you want to proceed? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${RED}Startup aborted by user.${NC}"
            exit 0
        fi
        
        echo -e "\n${YELLOW}Choose network mode:${NC}"
        echo -e "1) Development (with hot-reloading)"
        echo -e "2) Production\n"
        read -p "Enter option (1/2): " -n 1 -r
        echo
        
        if [[ $REPLY =~ ^[1]$ ]]; then
            export ICN_NETWORK_MODE="development"
            echo -e "${GREEN}Using development mode${NC}"
        else
            export ICN_NETWORK_MODE="production"
            echo -e "${GREEN}Using production mode${NC}"
        fi
    fi
    
    # Start ICN with Docker
    start_icn_docker || {
        echo -e "\n${RED}ICN system startup failed. Check logs for details.${NC}"
        echo -e "${YELLOW}Log file: ${MASTER_LOG}${NC}"
        exit 1
    }
    
    # Final status message
    echo -e "\n${GREEN}ICN system startup completed successfully!${NC}"
    echo -e "${YELLOW}You can monitor the system status with:${NC}"
    echo -e "  - docker ps                        (list running containers)"
    echo -e "  - docker logs <container-id>       (view container logs)"
    echo -e "  - docker-compose -f <file> ps      (show service status)"
    
    # Print access URLs
    echo -e "\n${BLUE}Access your ICN system at:${NC}"
    echo -e "  - Web Interface: http://localhost:${SERVICE_PORTS[frontend]}"
    echo -e "  - API Endpoint: http://localhost:${SERVICE_PORTS[backend]}/api/v1"
    echo -e "  - Bootstrap Node API: http://localhost:${SERVICE_PORTS[bootstrap]}/api/v1"
    
    echo -e "\n${YELLOW}To shut down the system:${NC}"
    echo -e "  - Use Ctrl+C to stop this script"
    echo -e "  - Or run: cd ${PROJECT_ROOT}/docker && docker-compose down"
    
    # Keep script running to handle graceful shutdown
    echo -e "\n${BLUE}Press Ctrl+C to stop the ICN system...${NC}"
    while true; do
        sleep 1
    done
}

# Execute main function
main
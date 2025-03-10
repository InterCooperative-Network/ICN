#!/bin/bash

# ICN System Status Script
# Version: 3.0
# Description: Shows the status of all ICN system components running in Docker

# Ensure we're in the right directory
cd "$(dirname "$0")/.."
export PROJECT_ROOT=$(pwd)

# Source the dependency management and utility functions
source "${PROJECT_ROOT}/scripts/startup-utils.sh"

# Function to check the health of services
check_service_health() {
    local SERVICE_NAME=$1
    local PORT=$2
    local HEALTH_URL=$3
    local HEALTH_STATUS="UNKNOWN"
    local INDICATOR="❓"
    
    # Skip if no port or health URL provided
    if [ -z "$PORT" ] || [ -z "$HEALTH_URL" ]; then
        echo -e "  ${YELLOW}❓ Health check not available${NC}"
        return
    fi
    
    if curl -s --max-time 2 "$HEALTH_URL" > /dev/null 2>&1; then
        HEALTH_STATUS="HEALTHY"
        INDICATOR="${GREEN}✓${NC}"
    else
        HEALTH_STATUS="UNHEALTHY"
        INDICATOR="${RED}✗${NC}"
    fi
    
    echo -e "  ${INDICATOR} Health: ${HEALTH_STATUS}"
}

# Function to display container details
display_container_details() {
    local SERVICE_NAME=$1
    local CONTAINER_NAME="icn-${SERVICE_NAME}"
    
    # For db service, use the specific container name from docker-compose
    if [ "$SERVICE_NAME" = "db" ]; then
        CONTAINER_NAME="icn_db_1"
    fi
    
    # Get container info if running
    local CONTAINER_ID=$(docker ps -q -f name=$CONTAINER_NAME)
    
    if [ -n "$CONTAINER_ID" ]; then
        # Container is running
        echo -e "\n${BLUE}=== ${SERVICE_NAME} (RUNNING) ===${NC}"
        
        # Get basic info
        local SHORT_ID=$(echo "$CONTAINER_ID" | cut -c1-12)
        local CONTAINER_NAME=$(docker inspect --format='{{.Name}}' "$CONTAINER_ID" | sed 's/^\///')
        local IMAGE=$(docker inspect --format='{{.Config.Image}}' "$CONTAINER_ID")
        local CREATED=$(docker inspect --format='{{.Created}}' "$CONTAINER_ID" | cut -d'T' -f1)
        local STATUS=$(docker inspect --format='{{.State.Status}}' "$CONTAINER_ID")
        local UPTIME=$(docker inspect --format='{{.State.StartedAt}}' "$CONTAINER_ID" | cut -d'T' -f2 | cut -d'.' -f1)
        local PORT_MAPPINGS=$(docker inspect --format='{{range $p, $conf := .NetworkSettings.Ports}}{{$p}} -> {{(index $conf 0).HostPort}}{{printf "\n"}}{{end}}' "$CONTAINER_ID")
        
        echo -e "  Container ID: ${SHORT_ID}"
        echo -e "  Name: ${CONTAINER_NAME}"
        echo -e "  Image: ${IMAGE}"
        echo -e "  Created: ${CREATED}"
        echo -e "  Status: ${STATUS}"
        echo -e "  Started at: ${UPTIME}"
        echo -e "  Port mappings:"
        echo -e "$PORT_MAPPINGS" | sed 's/^/    /'
        
        # Check service health
        local PORT=${SERVICE_PORTS[$SERVICE_NAME]}
        local HEALTH_URL
        
        case "$SERVICE_NAME" in
            "db")
                # Custom check for db
                if docker exec "$CONTAINER_ID" pg_isready -U ${POSTGRES_USER:-icnuser} -d ${POSTGRES_DB:-icndb} > /dev/null 2>&1; then
                    echo -e "  ${GREEN}✓${NC} Database: READY"
                else
                    echo -e "  ${RED}✗${NC} Database: NOT READY"
                fi
                ;;
            "backend"|"bootstrap"|"validator1"|"validator2")
                HEALTH_URL="http://localhost:${PORT}/api/v1/health"
                check_service_health "$SERVICE_NAME" "$PORT" "$HEALTH_URL"
                ;;
            "frontend")
                HEALTH_URL="http://localhost:${PORT}"
                check_service_health "$SERVICE_NAME" "$PORT" "$HEALTH_URL"
                ;;
        esac
        
        # Show recent logs (last 5 lines)
        echo -e "\n  ${BLUE}Recent logs:${NC}"
        docker logs --tail 5 "$CONTAINER_ID" | sed 's/^/    /'
        
    else
        # Container is not running
        echo -e "\n${BLUE}=== ${SERVICE_NAME} (${RED}STOPPED${BLUE}) ===${NC}"
        echo -e "  ${YELLOW}This service is not running${NC}"
        
        # Check if container exists but is stopped
        local STOPPED_ID=$(docker ps -a -q -f name=$CONTAINER_NAME)
        if [ -n "$STOPPED_ID" ]; then
            local EXIT_CODE=$(docker inspect --format='{{.State.ExitCode}}' "$STOPPED_ID")
            local FINISHED_AT=$(docker inspect --format='{{.State.FinishedAt}}' "$STOPPED_ID" | cut -d'T' -f2 | cut -d'.' -f1)
            echo -e "  Container exists but is stopped"
            echo -e "  Exit code: ${EXIT_CODE}"
            echo -e "  Stopped at: ${FINISHED_AT}"
            
            # Show error message if available
            if [ "$EXIT_CODE" != "0" ]; then
                echo -e "  ${RED}Error logs:${NC}"
                docker logs --tail 10 "$STOPPED_ID" | grep -i "error\|exception\|fail" | sed 's/^/    /' || echo -e "    No error logs found"
            fi
        fi
    fi
}

# Main function
main() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}      ICN System Status Checker         ${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "Checking ICN system status at $(date)"
    
    # Load configuration for Docker settings
    load_configuration || {
        echo -e "${YELLOW}Configuration validation failed, but proceeding with status check${NC}"
    }
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        echo -e "${RED}Error: Docker is not running${NC}"
        exit 1
    fi
    
    # Summary table
    echo -e "\n${BLUE}=== ICN Docker Container Status ===${NC}"
    echo -e "${BLUE}Service\t\tStatus\t\tPort\t\tHealth${NC}"
    echo -e "${BLUE}-------\t\t------\t\t----\t\t------${NC}"
    
    # Create array of service names
    local SERVICES=("db" "backend" "bootstrap" "validator1" "validator2" "frontend" "identity" "reputation" "governance")
    
    # Check status of each service
    for SERVICE in "${SERVICES[@]}"; do
        local STATUS="${RED}STOPPED${NC}"
        local CONTAINER_ID=""
        local PORT="${SERVICE_PORTS[$SERVICE]}"
        local HEALTH="--"
        local CONTAINER_NAME="icn-${SERVICE}"
        
        # For db service, use the specific container name from docker-compose
        if [ "$SERVICE" = "db" ]; then
            CONTAINER_NAME="icn_db_1"
        fi
        
        # Check if container is running
        if docker ps -q -f name=$CONTAINER_NAME | grep -q .; then
            STATUS="${GREEN}RUNNING${NC}"
            CONTAINER_ID=$(docker ps -q -f name=$CONTAINER_NAME)
            
            # Check health
            case "$SERVICE" in
                "db")
                    if docker exec $CONTAINER_ID pg_isready -U ${POSTGRES_USER:-icnuser} -d ${POSTGRES_DB:-icndb} > /dev/null 2>&1; then
                        HEALTH="${GREEN}HEALTHY${NC}"
                    else
                        HEALTH="${RED}UNHEALTHY${NC}"
                    fi
                    ;;
                "backend"|"bootstrap"|"validator1"|"validator2")
                    if curl -s --max-time 2 "http://localhost:${PORT}/api/v1/health" > /dev/null 2>&1; then
                        HEALTH="${GREEN}HEALTHY${NC}"
                    else
                        HEALTH="${RED}UNHEALTHY${NC}"
                    fi
                    ;;
                "frontend")
                    if curl -s --max-time 2 "http://localhost:${PORT}" > /dev/null 2>&1; then
                        HEALTH="${GREEN}HEALTHY${NC}"
                    else
                        HEALTH="${RED}UNHEALTHY${NC}"
                    fi
                    ;;
                *)
                    HEALTH="${YELLOW}UNKNOWN${NC}"
                    ;;
            esac
        fi
        
        printf "%-15s\t%-15s\t%-15s\t%-15s\n" "$SERVICE" "$STATUS" "$PORT" "$HEALTH"
    done
    
    # Ask if user wants detailed output for specific services
    if [ -t 1 ]; then  # Check if stdout is a terminal
        echo -e "\n${YELLOW}Would you like to see detailed information? (y/n)${NC}"
        read -p "" -n 1 -r
        echo
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}Choose an option:${NC}"
            echo "1) Show details for all services"
            echo "2) Show details for running services only"
            echo "3) Select a specific service"
            read -p "Enter option (1/2/3): " -n 1 -r
            echo
            
            case $REPLY in
                1)
                    # Show all services
                    for SERVICE in "${SERVICES[@]}"; do
                        display_container_details "$SERVICE"
                    done
                    ;;
                2)
                    # Show only running services
                    for SERVICE in "${SERVICES[@]}"; do
                        local CONTAINER_NAME="icn-${SERVICE}"
                        if [ "$SERVICE" = "db" ]; then
                            CONTAINER_NAME="icn_db_1"
                        fi
                        
                        if docker ps -q -f name=$CONTAINER_NAME | grep -q .; then
                            display_container_details "$SERVICE"
                        fi
                    done
                    ;;
                3)
                    # Show menu of services
                    echo -e "${YELLOW}Select a service:${NC}"
                    select SERVICE_CHOICE in "${SERVICES[@]}" "Exit"; do
                        if [ "$SERVICE_CHOICE" = "Exit" ]; then
                            break
                        elif [ -n "$SERVICE_CHOICE" ]; then
                            display_container_details "$SERVICE_CHOICE"
                            break
                        else
                            echo "Invalid selection"
                        fi
                    done
                    ;;
                *)
                    echo "Invalid option"
                    ;;
            esac
        fi
    fi
    
    echo -e "\n${BLUE}For more information on a specific container:${NC}"
    echo -e "  docker logs <container-id>"
    echo -e "  docker inspect <container-id>"
    
    echo -e "\n${BLUE}System Management Commands:${NC}"
    echo -e "  ${PROJECT_ROOT}/scripts/start_icn.sh  (Start the ICN system)"
    echo -e "  ${PROJECT_ROOT}/scripts/stop_icn.sh   (Stop the ICN system)"
}

# Execute main function
main 
#!/bin/bash

# ICN System Status Checker
# Description: Monitors all components of the ICN system and reports their status

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Set default values
ICN_SERVER_PORT=${ICN_SERVER_PORT:-8081}
BOOTSTRAP_API_PORT=${BOOTSTRAP_API_PORT:-8082}
VALIDATOR1_API_PORT=${VALIDATOR1_API_PORT:-8083}
VALIDATOR2_API_PORT=${VALIDATOR2_API_PORT:-8084}
FRONTEND_PORT=${FRONTEND_PORT:-3000}

# Function to check if a service is responding
check_endpoint() {
    local NAME=$1
    local URL=$2
    local REQUIRED=${3:-false}
    
    echo -ne "${NAME}: "
    
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$URL" 2>/dev/null)
    
    if [ "$RESPONSE" = "200" ]; then
        echo -e "${GREEN}OK (200)${NC}"
        return 0
    else
        if [ "$REQUIRED" = "true" ]; then
            echo -e "${RED}FAILED (${RESPONSE:-timeout})${NC}"
        else
            echo -e "${YELLOW}WARNING (${RESPONSE:-timeout})${NC}"
        fi
        return 1
    fi
}

# Function to check Docker container status
check_docker_container() {
    local CONTAINER_NAME=$1
    local STATUS
    local CONTAINER_ID
    local STATE
    local HEALTH
    
    echo -ne "Container ${CONTAINER_NAME}: "
    
    # Check if container exists
    CONTAINER_ID=$(docker ps -a -q -f name="${CONTAINER_NAME}" 2>/dev/null)
    
    if [ -z "$CONTAINER_ID" ]; then
        echo -e "${RED}NOT FOUND${NC}"
        return 1
    fi
    
    # Check container state
    STATE=$(docker inspect --format='{{.State.Status}}' "$CONTAINER_ID" 2>/dev/null)
    
    if [ "$STATE" = "running" ]; then
        # Check health if available
        HEALTH=$(docker inspect --format='{{if .State.Health}}{{.State.Health.Status}}{{else}}N/A{{end}}' "$CONTAINER_ID" 2>/dev/null)
        
        if [ "$HEALTH" = "healthy" ]; then
            echo -e "${GREEN}RUNNING (Healthy)${NC}"
        elif [ "$HEALTH" = "unhealthy" ]; then
            echo -e "${RED}RUNNING (Unhealthy)${NC}"
        else
            echo -e "${GREEN}RUNNING${NC}"
        fi
        
        # Show container uptime
        STARTED_AT=$(docker inspect --format='{{.State.StartedAt}}' "$CONTAINER_ID" | cut -d'T' -f2 | cut -d'.' -f1)
        echo -e "  ${GRAY}• Started at: ${STARTED_AT}${NC}"
        
        # Show port mappings
        PORTS=$(docker inspect --format='{{range $p, $conf := .NetworkSettings.Ports}}{{if $conf}}{{$p}} -> {{(index $conf 0).HostPort}}{{end}}{{end}}' "$CONTAINER_ID" | tr ' ' '\n' | grep -v '^$')
        if [ -n "$PORTS" ]; then
            echo -e "  ${GRAY}• Port mappings:${NC}"
            echo -e "$PORTS" | sed 's/^/    /'
        fi
        
        return 0
    else
        echo -e "${RED}NOT RUNNING (${STATE})${NC}"
        
        # Show exit code if container exited
        if [ "$STATE" = "exited" ]; then
            EXIT_CODE=$(docker inspect --format='{{.State.ExitCode}}' "$CONTAINER_ID")
            FINISHED_AT=$(docker inspect --format='{{.State.FinishedAt}}' "$CONTAINER_ID" | cut -d'T' -f2 | cut -d'.' -f1)
            echo -e "  ${GRAY}• Exit code: ${EXIT_CODE}${NC}"
            echo -e "  ${GRAY}• Finished at: ${FINISHED_AT}${NC}"
            
            # Show last few log lines if it exited with non-zero code
            if [ "$EXIT_CODE" != "0" ]; then
                echo -e "  ${GRAY}• Last few log lines:${NC}"
                docker logs --tail 5 "$CONTAINER_ID" | sed 's/^/    /'
            fi
        fi
        
        return 1
    fi
}

# Function to check system resources
check_system_resources() {
    echo -e "\n${BLUE}System Resources:${NC}"
    
    # Check memory usage of Docker containers
    echo -e "\n${BLUE}Docker Container Resource Usage:${NC}"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}" 2>/dev/null || echo "No Docker stats available"
    
    # Show disk usage
    echo -e "\n${BLUE}Disk Usage:${NC}"
    df -h . | awk 'NR==1 || NR==2 {print $0}'
}

# Function to check running processes
check_processes() {
    echo -e "\n${BLUE}Running ICN Processes:${NC}"
    
    # Check registered services in .icn_services if available
    if [ -f "${PROJECT_ROOT}/.icn_services" ]; then
        echo -e "Registered services:"
        
        while IFS=: read -r SERVICE_NAME PID EXTRA || [ -n "$SERVICE_NAME" ]; do
            if [ -n "$PID" ] && [ "$PID" != "ready" ]; then
                echo -ne "  - ${SERVICE_NAME} (PID ${PID}): "
                if ps -p $PID > /dev/null 2>&1; then
                    echo -e "${GREEN}RUNNING${NC}"
                    # Show memory usage
                    MEM_USAGE=$(ps -o rss= -p $PID 2>/dev/null | awk '{print $1/1024 " MB"}')
                    if [ -n "$MEM_USAGE" ]; then
                        echo -e "    ${GRAY}• Memory: ${MEM_USAGE}${NC}"
                    fi
                else
                    echo -e "${RED}NOT RUNNING${NC}"
                fi
            elif [ "$PID" = "ready" ]; then
                echo -e "  - ${SERVICE_NAME}: ${YELLOW}READY (not running)${NC}"
                if [ -n "$EXTRA" ]; then
                    echo -e "    ${GRAY}• Path: ${EXTRA}${NC}"
                fi
            fi
        done < "${PROJECT_ROOT}/.icn_services"
    else
        # Look for common ICN processes
        PROCESSES=$(ps aux | grep -E "icn-|react-scripts|icn_bin" | grep -v grep || true)
        
        if [ -n "$PROCESSES" ]; then
            echo "$PROCESSES" | while read -r line; do
                PID=$(echo "$line" | awk '{print $2}')
                CMD=$(echo "$line" | awk '{$1=$2=$3=$4=$5=$6=$7=$8=$9=$10=""; print substr($0,11)}')
                echo -e "  - PID $PID: ${CMD}"
            done
        else
            echo -e "${YELLOW}No ICN processes found running${NC}"
        fi
    fi
}

# Main execution
main() {
    echo -e "${BLUE}========== ICN System Status Check ==========${NC}"
    echo -e "Status as of $(date)\n"
    
    # Check Docker containers
    echo -e "${BLUE}Docker Containers:${NC}"
    check_docker_container "icn_db" || true
    check_docker_container "icn_backend" || true
    check_docker_container "icn_frontend" || true
    check_docker_container "icn_bootstrap" || true
    
    # Check API endpoints
    echo -e "\n${BLUE}API Endpoints:${NC}"
    check_endpoint "Backend API" "http://localhost:${ICN_SERVER_PORT}/api/v1/health" true
    check_endpoint "Frontend" "http://localhost:${FRONTEND_PORT}" false
    check_endpoint "Bootstrap Node" "http://localhost:${BOOTSTRAP_API_PORT}/api/v1/status" false
    check_endpoint "Validator 1" "http://localhost:${VALIDATOR1_API_PORT}/api/v1/status" false
    check_endpoint "Validator 2" "http://localhost:${VALIDATOR2_API_PORT}/api/v1/status" false
    
    # Check running processes
    check_processes
    
    # Check resource usage
    check_system_resources
    
    # System status summary
    echo -e "\n${BLUE}Overall System Status:${NC}"
    if docker ps | grep -q "icn_backend" && \
       docker ps | grep -q "icn_frontend" && \
       check_endpoint "Backend API" "http://localhost:${ICN_SERVER_PORT}/api/v1/health" true >/dev/null 2>&1; then
        echo -e "${GREEN}ICN system is operational${NC}"
        echo -e "${GREEN}✓ Core components are running${NC}"
    else
        echo -e "${RED}ICN system is incomplete or not fully operational${NC}"
        echo -e "${RED}✗ Some core components are not running properly${NC}"
        
        # Provide restart instructions
        echo -e "\n${YELLOW}To restart the ICN system:${NC}"
        echo -e "  1. Stop the system:  ${GRAY}./stop_icn.sh${NC}"
        echo -e "  2. Start the system: ${GRAY}./start_icn.sh${NC}"
    fi
}

# Execute main function
main
#!/bin/bash

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to check service health
check_service() {
    local service=$1
    local port=$2
    local endpoint=${3:-"/health"}
    
    if curl -s "http://localhost:${port}${endpoint}" > /dev/null; then
        echo -e "${GREEN}✓${NC} $service is healthy"
        return 0
    else
        echo -e "${RED}✗${NC} $service is not responding"
        return 1
    fi
}

# Function to show recent logs for a service
show_service_logs() {
    local service=$1
    local lines=${2:-50}
    echo -e "\n${BLUE}Recent logs for $service:${NC}"
    docker-compose -f docker/docker-compose.dev.yml logs --tail=$lines $service
}

# Main monitoring loop
monitor_services() {
    while true; do
        clear
        echo -e "${BLUE}=== ICN Development Environment Monitor ===${NC}"
        echo -e "Press Ctrl+C to exit\n"

        # Check core services
        echo -e "${YELLOW}Core Services Status:${NC}"
        check_service "Database" "5432" || show_service_logs "db" 10
        check_service "Backend" "8081" "/api/v1/health" || show_service_logs "backend" 10
        check_service "Frontend" "3000" || show_service_logs "frontend" 10

        # Check network nodes
        echo -e "\n${YELLOW}Network Nodes Status:${NC}"
        check_service "Bootstrap Node" "8082" "/api/v1/health" || show_service_logs "bootstrap" 10

        # Display resource usage
        echo -e "\n${YELLOW}Resource Usage:${NC}"
        docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"

        sleep 5
    done
}

# Start monitoring
monitor_services
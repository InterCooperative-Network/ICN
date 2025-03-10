#!/bin/bash

# ICN System Shutdown Script
# Description: Gracefully stops all components of the ICN system

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize log file
LOG_DATE=$(date +"%Y%m%d-%H%M%S")
SHUTDOWN_LOG="${PROJECT_ROOT}/logs/icn-shutdown-${LOG_DATE}.log"
mkdir -p "${PROJECT_ROOT}/logs"
touch "$SHUTDOWN_LOG"
echo "--- ICN System Shutdown Log - $(date) ---" >> "$SHUTDOWN_LOG"

# Log message to both console and log file
log_message() {
    local SERVICE=$1
    local MESSAGE=$2
    local LEVEL=${3:-INFO}
    local TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    
    case "$LEVEL" in
        "ERROR")
            echo -e "${RED}[ERROR][$SERVICE] $MESSAGE${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}[WARNING][$SERVICE] $MESSAGE${NC}"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS][$SERVICE] $MESSAGE${NC}"
            ;;
        *)
            echo -e "[INFO][$SERVICE] $MESSAGE"
            ;;
    esac
    
    echo "[$TIMESTAMP][$LEVEL][$SERVICE] $MESSAGE" >> "$SHUTDOWN_LOG"
}

# Stop Docker services using docker-compose
stop_docker_services() {
    local SHUTDOWN_MODE=$1
    local COMPOSE_FILE="docker-compose.dev.yml"
    
    # Check if the compose file exists
    if [ ! -f "${PROJECT_ROOT}/docker/$COMPOSE_FILE" ]; then
        log_message "SYSTEM" "Docker Compose file not found: $COMPOSE_FILE" "ERROR"
        return 1
    fi
    
    log_message "SYSTEM" "Stopping Docker services using $COMPOSE_FILE..." "INFO"
    
    case "$SHUTDOWN_MODE" in
        "stop")
            log_message "SYSTEM" "Stopping containers only..." "INFO"
            (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" stop) || {
                log_message "SYSTEM" "Failed to stop containers" "ERROR"
                return 1
            }
            ;;
        "down")
            log_message "SYSTEM" "Stopping and removing containers..." "INFO"
            (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" down) || {
                log_message "SYSTEM" "Failed to stop and remove containers" "ERROR"
                return 1
            }
            ;;
        "down_volumes")
            log_message "SYSTEM" "Stopping containers and removing volumes..." "INFO"
            (cd "${PROJECT_ROOT}/docker" && docker-compose -f "$COMPOSE_FILE" down -v) || {
                log_message "SYSTEM" "Failed to stop containers and remove volumes" "ERROR"
                return 1
            }
            ;;
        *)
            log_message "SYSTEM" "Invalid shutdown mode: $SHUTDOWN_MODE" "ERROR"
            return 1
            ;;
    esac
    
    log_message "SYSTEM" "Docker services stopped" "SUCCESS"
    return 0
}

# Stop any running ICN processes (non-Docker)
stop_processes() {
    log_message "SYSTEM" "Stopping any running ICN processes..." "INFO"
    
    # Check if the services file exists
    if [ -f "${PROJECT_ROOT}/.icn_services" ]; then
        # Read and process each service entry
        while IFS=: read -r SERVICE_NAME PID EXTRA || [ -n "$SERVICE_NAME" ]; do
            if [ -n "$PID" ] && [ "$PID" != "ready" ] && ps -p $PID > /dev/null 2>&1; then
                log_message "$SERVICE_NAME" "Stopping process with PID ${PID}..." "INFO"
                kill $PID 2>/dev/null || {
                    log_message "$SERVICE_NAME" "Failed to stop process with SIGTERM, using SIGKILL..." "WARNING"
                    kill -9 $PID 2>/dev/null || true
                }
                sleep 0.5
            elif [ "$PID" = "ready" ]; then
                log_message "$SERVICE_NAME" "Service is ready but not running as a process" "INFO"
            else
                log_message "$SERVICE_NAME" "Process with PID ${PID} is not running" "INFO"
            fi
        done < "${PROJECT_ROOT}/.icn_services"
        
        # Remove the services file
        rm "${PROJECT_ROOT}/.icn_services"
        log_message "SYSTEM" "Service registry removed" "INFO"
    else
        log_message "SYSTEM" "No service registry found" "WARNING"
    fi
    
    # Try to find and kill any remaining ICN processes
    log_message "SYSTEM" "Checking for any remaining ICN processes..." "INFO"
    
    pkill -f "icn-server" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped icn-server processes" "INFO" || true
    pkill -f "react-scripts start" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped frontend processes" "INFO" || true
    pkill -f "icn_bin" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped icn_bin processes" "INFO" || true
    pkill -f "icn-cli" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped icn-cli processes" "INFO" || true
    pkill -f "simple_node" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped simple_node processes" "INFO" || true
    pkill -f "icn-validator" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped icn-validator processes" "INFO" || true
    pkill -f "icn-identity" >/dev/null 2>&1 && log_message "SYSTEM" "Stopped icn-identity processes" "INFO" || true
    
    log_message "SYSTEM" "All processes stopped" "SUCCESS"
    return 0
}

# Main execution
main() {
    echo -e "${BLUE}========== ICN System Shutdown ==========${NC}"
    echo -e "${BLUE}Stopping ICN system at $(date)${NC}"
    
    local SHUTDOWN_MODE="stop"
    
    # Ask for confirmation and shutdown mode in interactive mode
    if [ -t 1 ]; then  # Check if stdout is a terminal
        echo -e "${YELLOW}This script will stop all ICN components.${NC}\n"
        
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
                SHUTDOWN_MODE="stop"
                echo -e "${GREEN}Stopping containers only${NC}"
                ;;
            2)
                SHUTDOWN_MODE="down"
                echo -e "${GREEN}Stopping and removing containers${NC}"
                ;;
            3)
                SHUTDOWN_MODE="down_volumes"
                echo -e "${GREEN}Stopping, removing containers and deleting volumes${NC}"
                echo -e "${RED}WARNING: All data will be lost!${NC}"
                read -p "Are you sure? This will delete all data. (y/n) " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    echo -e "${YELLOW}Switching to 'stop' mode to preserve data.${NC}"
                    SHUTDOWN_MODE="stop"
                fi
                ;;
            *)
                SHUTDOWN_MODE="stop"
                echo -e "${YELLOW}Invalid option. Defaulting to 'stop' mode.${NC}"
                ;;
        esac
    fi
    
    # Stop Docker services
    stop_docker_services "$SHUTDOWN_MODE" || {
        log_message "SYSTEM" "Failed to properly stop Docker services" "WARNING"
        # Continue with process shutdown anyway
    }
    
    # Stop any remaining processes
    stop_processes
    
    # Final status message
    echo -e "${GREEN}=== ICN System Shutdown Completed ===${NC}"
    echo -e "${GREEN}All services have been stopped.${NC}"
    echo -e "${YELLOW}Log file: ${SHUTDOWN_LOG}${NC}"
}

# Execute main function
main
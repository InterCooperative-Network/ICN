#!/bin/bash
# ICN Project - Run Script
# This script provides a unified interface for running and managing ICN services

# Set error handling
set -e

# Define colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration variables
LOG_DIR="logs"
BACKEND_PORT=8081
FRONTEND_PORT=3000
CONSENSUS_PORT=8088

# Function definitions
print_banner() {
    echo -e "${BLUE}==========================================${NC}"
    echo -e "${BLUE}       Inter-Cooperative Network          ${NC}"
    echo -e "${BLUE}==========================================${NC}"
}

print_help() {
    echo -e "${CYAN}Usage:${NC}"
    echo -e "  ${GREEN}./run.sh${NC} [command]"
    echo -e "\n${CYAN}Commands:${NC}"
    echo -e "  ${GREEN}start${NC} or empty   Start all ICN services"
    echo -e "  ${GREEN}backend${NC}          Start only backend services"
    echo -e "  ${GREEN}frontend${NC}         Start only frontend service"
    echo -e "  ${GREEN}consensus${NC}        Start only consensus engine"
    echo -e "  ${GREEN}stop${NC}             Stop all services"
    echo -e "  ${GREEN}status${NC}           Check service status"
    echo -e "  ${GREEN}logs${NC} [service]   View logs of a service (or all if not specified)"
    echo -e "  ${GREEN}test${NC}             Run tests"
    echo -e "  ${GREEN}clean${NC}            Clean up environment (stop services, remove temp files)"
    echo -e "  ${GREEN}help${NC}             Show this help message"
}

check_setup() {
    if [ ! -f "./setup.sh" ]; then
        echo -e "${YELLOW}WARNING: setup.sh not found. You may need to run setup first.${NC}"
        read -p "Would you like to run the setup script now? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            ./setup.sh
        fi
    fi
}

ensure_log_directory() {
    if [ ! -d "$LOG_DIR" ]; then
        mkdir -p "$LOG_DIR"
    fi
}

start_all() {
    echo -e "${BLUE}Starting all ICN services...${NC}"
    start_backend
    start_frontend
    start_consensus
}

start_backend() {
    echo -e "${BLUE}Starting backend services...${NC}"
    ensure_log_directory
    
    # Check if database is running
    if ! docker ps | grep -q postgres; then
        echo "Starting database..."
        docker-compose -f docker-compose.test.yml up -d postgres
    fi
    
    # Run backend service
    echo "Starting backend server..."
    cargo run --bin icn-server > "$LOG_DIR/backend.log" 2>&1 &
    echo $! > "$LOG_DIR/backend.pid"
    echo -e "${GREEN}Backend started on port $BACKEND_PORT${NC}"
    echo -e "${YELLOW}Logs available at $LOG_DIR/backend.log${NC}"
}

start_frontend() {
    if [ ! -d "frontend" ]; then
        echo -e "${RED}Frontend directory not found${NC}"
        return 1
    fi
    
    echo -e "${BLUE}Starting frontend services...${NC}"
    ensure_log_directory
    
    cd frontend
    if [ -f "package.json" ]; then
        echo "Starting frontend development server..."
        npm run dev > "../$LOG_DIR/frontend.log" 2>&1 &
        echo $! > "../$LOG_DIR/frontend.pid"
        echo -e "${GREEN}Frontend started on port $FRONTEND_PORT${NC}"
        echo -e "${YELLOW}Logs available at $LOG_DIR/frontend.log${NC}"
    else
        echo -e "${RED}No package.json found in frontend directory${NC}"
        return 1
    fi
    cd ..
}

start_consensus() {
    echo -e "${BLUE}Starting consensus engine...${NC}"
    ensure_log_directory
    
    cargo run --bin icn-consensus > "$LOG_DIR/consensus.log" 2>&1 &
    echo $! > "$LOG_DIR/consensus.pid"
    echo -e "${GREEN}Consensus engine started on port $CONSENSUS_PORT${NC}"
    echo -e "${YELLOW}Logs available at $LOG_DIR/consensus.log${NC}"
}

stop_services() {
    echo -e "${BLUE}Stopping ICN services...${NC}"
    
    # Stop backend if running
    if [ -f "$LOG_DIR/backend.pid" ]; then
        PID=$(cat "$LOG_DIR/backend.pid")
        if ps -p "$PID" > /dev/null; then
            echo "Stopping backend server..."
            kill "$PID"
        fi
        rm "$LOG_DIR/backend.pid"
    fi
    
    # Stop frontend if running
    if [ -f "$LOG_DIR/frontend.pid" ]; then
        PID=$(cat "$LOG_DIR/frontend.pid")
        if ps -p "$PID" > /dev/null; then
            echo "Stopping frontend server..."
            kill "$PID"
        fi
        rm "$LOG_DIR/frontend.pid"
    fi
    
    # Stop consensus if running
    if [ -f "$LOG_DIR/consensus.pid" ]; then
        PID=$(cat "$LOG_DIR/consensus.pid")
        if ps -p "$PID" > /dev/null; then
            echo "Stopping consensus engine..."
            kill "$PID"
        fi
        rm "$LOG_DIR/consensus.pid"
    fi
    
    # Stop docker containers
    if command -v docker-compose >/dev/null 2>&1; then
        echo "Stopping docker containers..."
        docker-compose -f docker-compose.test.yml down
    fi
    
    echo -e "${GREEN}All services stopped${NC}"
}

check_status() {
    echo -e "${BLUE}Checking ICN services status...${NC}"
    
    # Check docker services
    if command -v docker-compose >/dev/null 2>&1; then
        echo -e "${CYAN}Docker services:${NC}"
        docker-compose -f docker-compose.test.yml ps
    fi
    
    # Check backend
    echo -e "\n${CYAN}Backend service:${NC}"
    if [ -f "$LOG_DIR/backend.pid" ]; then
        PID=$(cat "$LOG_DIR/backend.pid")
        if ps -p "$PID" > /dev/null; then
            echo -e "${GREEN}Running (PID: $PID)${NC}"
        else
            echo -e "${RED}Not running (stale PID file)${NC}"
        fi
    else
        echo -e "${RED}Not running${NC}"
    fi
    
    # Check frontend
    echo -e "\n${CYAN}Frontend service:${NC}"
    if [ -f "$LOG_DIR/frontend.pid" ]; then
        PID=$(cat "$LOG_DIR/frontend.pid")
        if ps -p "$PID" > /dev/null; then
            echo -e "${GREEN}Running (PID: $PID)${NC}"
        else
            echo -e "${RED}Not running (stale PID file)${NC}"
        fi
    else
        echo -e "${RED}Not running${NC}"
    fi
    
    # Check consensus
    echo -e "\n${CYAN}Consensus engine:${NC}"
    if [ -f "$LOG_DIR/consensus.pid" ]; then
        PID=$(cat "$LOG_DIR/consensus.pid")
        if ps -p "$PID" > /dev/null; then
            echo -e "${GREEN}Running (PID: $PID)${NC}"
        else
            echo -e "${RED}Not running (stale PID file)${NC}"
        fi
    else
        echo -e "${RED}Not running${NC}"
    fi
}

view_logs() {
    local service=$1
    
    if [ ! -d "$LOG_DIR" ]; then
        echo -e "${RED}No logs directory found${NC}"
        return 1
    fi
    
    if [ -z "$service" ]; then
        # View all logs
        echo -e "${BLUE}Available logs:${NC}"
        ls -l "$LOG_DIR"/*.log 2>/dev/null || echo -e "${RED}No log files found${NC}"
    else
        # View specific service log
        if [ -f "$LOG_DIR/$service.log" ]; then
            echo -e "${BLUE}Showing logs for $service:${NC}"
            cat "$LOG_DIR/$service.log"
        else
            echo -e "${RED}No log file found for $service${NC}"
            echo -e "${YELLOW}Available logs:${NC}"
            ls -l "$LOG_DIR"/*.log 2>/dev/null || echo -e "${RED}No log files found${NC}"
        fi
    fi
}

run_tests() {
    echo -e "${BLUE}Running tests...${NC}"
    cargo test
}

clean_environment() {
    echo -e "${BLUE}Cleaning up environment...${NC}"
    
    # Stop all services first
    stop_services
    
    # Clean Cargo target directory
    echo "Cleaning Rust build artifacts..."
    cargo clean
    
    # Clean logs
    echo "Removing logs..."
    rm -f "$LOG_DIR"/*.log
    rm -f "$LOG_DIR"/*.pid
    
    echo -e "${GREEN}Environment cleaned${NC}"
}

# Check if we should run setup
check_setup

# Main execution
print_banner

case "$1" in
    start|"")
        start_all
        ;;
    backend)
        start_backend
        ;;
    frontend)
        start_frontend
        ;;
    consensus)
        start_consensus
        ;;
    stop)
        stop_services
        ;;
    status)
        check_status
        ;;
    logs)
        view_logs "$2"
        ;;
    test)
        run_tests
        ;;
    clean)
        clean_environment
        ;;
    help|--help|-h)
        print_help
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        print_help
        exit 1
        ;;
esac

exit 0
#!/bin/bash
# ICN Project - Complete Setup Script
# This script handles the entire setup process for the ICN project

# Set error handling
set -e

# Define colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Print banner
echo -e "${BLUE}==========================================${NC}"
echo -e "${BLUE}  Inter-Cooperative Network (ICN) Setup   ${NC}"
echo -e "${BLUE}==========================================${NC}"

# Function definitions
check_requirement() {
    local cmd=$1
    local name=$2
    local install_hint=$3
    
    echo -n "Checking for $name... "
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo -e "${RED}NOT FOUND${NC}"
        echo -e "${YELLOW}→ $install_hint${NC}"
        return 1
    fi
    echo -e "${GREEN}FOUND${NC}"
    return 0
}

create_directory() {
    local dir=$1
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        echo -e "${GREEN}Created directory: $dir${NC}"
    else
        echo -e "${BLUE}Directory exists: $dir${NC}"
    fi
}

setup_rust_environment() {
    echo -e "\n${BLUE}Setting up Rust environment...${NC}"
    
    # Make sure we're using the right Rust toolchain
    if [ -f "rust-toolchain.toml" ]; then
        echo "Using project-specific Rust toolchain..."
    else
        echo "Using default Rust toolchain..."
    fi
    
    # Check cargo installation
    cargo --version || {
        echo -e "${RED}Cargo not available. Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    }
    
    # Install required cargo components
    rustup component add rustfmt clippy
}

setup_database() {
    echo -e "\n${BLUE}Setting up database...${NC}"
    
    # Check if we have a PostgreSQL container running
    if docker ps | grep -q postgres; then
        echo -e "${GREEN}PostgreSQL is already running${NC}"
    else
        echo "Starting PostgreSQL container..."
        docker-compose -f docker-compose.test.yml up -d postgres || {
            echo -e "${RED}Failed to start PostgreSQL container${NC}"
            exit 1
        }
    fi
    
    echo "Waiting for PostgreSQL to start..."
    sleep 5
    
    # Run database migrations if needed
    if [ -d "backend/migrations" ]; then
        echo "Running database migrations..."
        # Add commands to run migrations if available
    fi
}

build_backend() {
    echo -e "\n${BLUE}Building backend components...${NC}"
    cargo build || {
        echo -e "${RED}Failed to build backend components${NC}"
        exit 1
    }
}

setup_frontend() {
    if [ -d "frontend" ]; then
        echo -e "\n${BLUE}Setting up frontend...${NC}"
        cd frontend
        
        if [ -f "package.json" ]; then
            npm install || {
                echo -e "${RED}npm install failed${NC}"
                exit 1
            }
            echo -e "${GREEN}Frontend dependencies installed${NC}"
        else
            echo -e "${YELLOW}Warning: No package.json found in frontend directory${NC}"
        fi
        
        cd ..
    fi
}

setup_config() {
    echo -e "\n${BLUE}Setting up configuration...${NC}"
    
    # Create default config if it doesn't exist
    if [ ! -f "config/settings.json" ]; then
        echo "Creating default configuration..."
        # Copy template if it exists
        if [ -f "config/settings.json.template" ]; then
            cp config/settings.json.template config/settings.json
        fi
    fi
    
    # Make sure log configuration exists
    if [ ! -f "config/log4rs.yaml" ]; then
        echo "Creating default logging configuration..."
        # Add logging setup if needed
    fi
}

# Main setup process
echo -e "\n${BLUE}Step 1: Checking prerequisites${NC}"
check_requirement "docker" "Docker" "Install from https://docs.docker.com/get-docker/" || exit 1
check_requirement "docker-compose" "Docker Compose" "Install from https://docs.docker.com/compose/install/" || exit 1
check_requirement "rustc" "Rust" "Install from https://rustup.rs" || {
    setup_rust_environment
}
check_requirement "npm" "Node.js/npm" "Install from https://nodejs.org/" || {
    echo -e "${YELLOW}Frontend development may be limited without Node.js/npm${NC}"
}

echo -e "\n${BLUE}Step 2: Creating project directories${NC}"
create_directory "logs"
create_directory "data"
create_directory "tools/doctools"

echo -e "\n${BLUE}Step 3: Setting up environment${NC}"
setup_rust_environment
setup_config
setup_database

echo -e "\n${BLUE}Step 4: Building components${NC}"
build_backend
setup_frontend

# Setup complete
echo -e "\n${GREEN}✅ Setup completed successfully!${NC}"
echo -e "\n${BLUE}Available Commands:${NC}"
echo "  ./run.sh               # Start all ICN services"
echo "  ./run.sh backend       # Start only the backend services"
echo "  ./run.sh frontend      # Start only the frontend service"
echo "  ./run.sh consensus     # Start only the consensus engine"
echo "  ./run.sh stop          # Stop all services"
echo "  ./run.sh status        # Check service status"

echo -e "\n${YELLOW}For more information, see the README.md file${NC}"
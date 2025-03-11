#!/bin/bash
# ICN Project - Dev Container Setup Script

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
echo -e "${BLUE}  ICN Setup (Dev Container Environment)   ${NC}"
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
    
    # Install required cargo components
    rustup component add rustfmt clippy
    
    # Build the project to ensure dependencies are downloaded
    echo "Building Rust project..."
    cargo build
}

setup_node_environment() {
    echo -e "\n${BLUE}Setting up Node.js environment...${NC}"
    
    # Install Node.js using nvm if not present
    if ! command -v node >/dev/null 2>&1; then
        echo "Installing Node.js..."
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
        export NVM_DIR="$HOME/.nvm"
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
        nvm install 18
        nvm use 18
    fi
    
    # Install frontend dependencies if package.json exists
    if [ -f "frontend/package.json" ]; then
        echo "Installing frontend dependencies..."
        cd frontend
        npm install
        cd ..
    fi
}

setup_config() {
    echo -e "\n${BLUE}Setting up configuration...${NC}"
    
    # Create default config directories
    create_directory "config"
    create_directory "data"
    create_directory "logs"
    
    # Copy environment template if it doesn't exist
    if [ ! -f ".env" ] && [ -f ".env.template" ]; then
        cp .env.template .env
        echo -e "${GREEN}Created .env file from template${NC}"
    fi
}

create_basic_structure() {
    echo -e "\n${BLUE}Creating project structure...${NC}"
    
    # Create main directories
    create_directory "backend"
    create_directory "frontend"
    create_directory "scripts/dev"
    create_directory "scripts/utils"
    create_directory "scripts/test"
    create_directory "config"
    create_directory "docs"
    create_directory "templates"
    
    # Create basic backend structure
    create_directory "backend/src"
    create_directory "backend/tests"
    
    # Create basic frontend structure
    create_directory "frontend/src"
    create_directory "frontend/public"
}

# Main setup process
echo -e "\n${BLUE}Step 1: Checking prerequisites${NC}"
check_requirement "rustc" "Rust" "Install from https://rustup.rs"
check_requirement "cargo" "Cargo" "Install from https://rustup.rs"
check_requirement "git" "Git" "Install git using your package manager"

echo -e "\n${BLUE}Step 2: Creating project structure${NC}"
create_basic_structure

echo -e "\n${BLUE}Step 3: Setting up environment${NC}"
setup_rust_environment
setup_node_environment
setup_config

# Setup complete
echo -e "\n${GREEN}✅ Setup completed successfully!${NC}"
echo -e "\n${BLUE}Next steps:${NC}"
echo "1. Edit .env file with your configuration"
echo "2. Start the backend: ./scripts/dev/run_backend_dev.sh"
echo "3. Start the frontend: ./scripts/dev/run_frontend_dev.sh"
echo "4. Monitor services: ./scripts/utils/monitor_icn.sh"

echo -e "\n${YELLOW}For more information, see the README.md file${NC}"
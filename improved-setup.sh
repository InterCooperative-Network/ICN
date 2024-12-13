#!/bin/bash
# ICN Project Setup Script
# Version: 1.0.0

# Set error handling
set -e

# Define colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Function definitions
check_requirement() {
    local cmd=$1
    local msg=$2
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo -e "${RED}Error: $msg is required but not installed.${NC}"
        return 1
    fi
    echo -e "${GREEN}Found: $msg${NC}"
    return 0
}

create_directory() {
    local dir=$1
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        echo -e "${GREEN}Created directory: $dir${NC}"
    else
        echo -e "${GREEN}Directory exists: $dir${NC}"
    fi
}

# Main setup process
echo -e "${GREEN}🚀 Starting ICN Development Environment Setup...${NC}"

# 1. Check prerequisites
echo "Checking prerequisites..."
check_requirement "docker" "Docker" || exit 1
check_requirement "cargo" "Rust/Cargo" || exit 1
check_requirement "npm" "Node.js/npm" || exit 1

# 2. Create required directories
echo "Creating project directories..."
create_directory ".logs"
create_directory ".data"
create_directory "tools/doctools"

# 3. Backend setup
if [ -d "backend" ]; then
    echo "Setting up backend..."
    cd backend
    if [ -f "Cargo.toml" ]; then
        echo "Running cargo check..."
        cargo check || {
            echo -e "${RED}Cargo check failed${NC}"
            exit 1
        }
        echo "Running cargo test..."
        cargo test || {
            echo -e "${RED}Tests failed${NC}"
            exit 1
        }
    else
        echo -e "${RED}Error: Cargo.toml not found in backend directory${NC}"
        exit 1
    fi
    cd ..
else
    echo -e "${RED}Error: Backend directory not found${NC}"
    exit 1
fi

# 4. Frontend setup
if [ -d "frontend" ]; then
    echo "Setting up frontend..."
    cd frontend
    if [ -f "package.json" ]; then
        npm install || {
            echo -e "${RED}npm install failed${NC}"
            exit 1
        }
    else
        echo -e "${RED}Warning: No package.json found in frontend directory${NC}"
    fi
    cd ..
fi

# 5. Docker environment setup
if [ -f "docker/docker-compose.yml" ]; then
    echo "Starting Docker services..."
    cd docker
    docker-compose up -d || {
        echo -e "${RED}Docker Compose failed${NC}"
        exit 1
    }
    cd ..
else
    echo -e "${RED}Warning: docker-compose.yml not found${NC}"
fi

# Setup complete
echo -e "${GREEN}✅ Setup completed successfully!${NC}"
echo "
Available Services:
------------------
Backend API: http://localhost:8081
WebSocket:   ws://localhost:8088

Development Resources:
--------------------
Logs: .logs/
Data: .data/

Next Steps:
----------
1. Check service status:
   docker-compose -f docker/docker-compose.yml ps

2. View logs:
   docker-compose -f docker/docker-compose.yml logs -f

3. Run backend tests:
   cd backend && cargo test
"
#!/bin/bash

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to create directory if it doesn't exist
create_directory() {
    if [ ! -d "$1" ]; then
        mkdir -p "$1"
        echo "Created directory: $1"
    fi
}

# Function to check requirements
check_requirement() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}Error: $2 is not installed${NC}"
        return 1
    fi
    echo -e "${GREEN}✓ Found $2${NC}"
    return 0
}

echo -e "${BLUE}Setting up ICN development environment...${NC}"

# Check requirements
check_requirement "cargo" "Rust/Cargo" || exit 1
check_requirement "npm" "Node.js/npm" || exit 1
check_requirement "docker" "Docker" || exit 1
check_requirement "docker-compose" "Docker Compose" || exit 1

# Create necessary directories
echo "Creating project directories..."
create_directory "backend/src"
create_directory "frontend/src"
create_directory "data"
create_directory "logs"
create_directory "config"

# Create .env from template if it doesn't exist
if [ ! -f .env ] && [ -f .env.template ]; then
    cp .env.template .env
    echo "Created .env file from template"
fi

# Backend setup
echo -e "\n${BLUE}Setting up backend...${NC}"
cd backend
if [ ! -f Cargo.toml ]; then
    cargo init
    # Add dependencies to Cargo.toml
    cat >> Cargo.toml << EOL
[dependencies]
tokio = { version = "1.0", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
log = "0.4"
env_logger = "0.9"
futures = "0.3"
thiserror = "1.0"
async-trait = "0.1"
EOL
fi

# Install backend dependencies
cargo check || {
    echo -e "${RED}Cargo check failed${NC}"
    exit 1
}

# Frontend setup
cd ../frontend
if [ ! -f package.json ]; then
    echo -e "\n${BLUE}Setting up frontend...${NC}"
    # Initialize a new React TypeScript project
    npx create-react-app . --template typescript

    # Install additional dependencies
    npm install \
        @types/react-router-dom \
        react-router-dom \
        @types/node \
        tailwindcss \
        @headlessui/react \
        @heroicons/react \
        recharts \
        @types/recharts

    # Initialize Tailwind CSS
    npx tailwindcss init
fi

# Install frontend dependencies
npm install || {
    echo -e "${RED}npm install failed${NC}"
    exit 1
}

# Setup development database
cd ../docker
echo -e "\n${BLUE}Setting up development database...${NC}"
docker-compose -f docker-compose.dev.yml up -d db || {
    echo -e "${RED}Failed to start development database${NC}"
    exit 1
}

cd ..

echo -e "\n${GREEN}Development environment setup completed!${NC}"
echo -e "\n${BLUE}Next steps:${NC}"
echo "1. Start the backend:"
echo "   cd backend && cargo run"
echo "2. Start the frontend:"
echo "   cd frontend && npm start"
echo "3. Access the dashboard at http://localhost:3000"
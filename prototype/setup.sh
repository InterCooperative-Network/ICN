#!/bin/bash
# ICN Prototype Setup Script

# Set error handling
set -e

# Define colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================${NC}"
echo -e "${GREEN}ICN P2P Cloud Computing Prototype Setup${NC}"
echo -e "${BLUE}=======================================${NC}"

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

# Check for Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: Docker Compose is not installed. Please install Docker Compose first.${NC}"
    exit 1
fi

echo -e "\n${GREEN}Creating necessary directories...${NC}"
mkdir -p data/bootstrap/keys data/node1/keys data/node2/keys logs

echo -e "\n${GREEN}Building Docker images...${NC}"
docker-compose build

echo -e "\n${GREEN}Starting ICN network...${NC}"
docker-compose up -d

echo -e "\n${GREEN}ICN network started successfully!${NC}"
echo -e "${BLUE}--------------------------------------${NC}"
echo -e "Services available at:"
echo -e "  Dashboard: http://localhost:8080"
echo -e "  Bootstrap Node API: http://localhost:3000/api/status"
echo -e "  Node 1 API: http://localhost:3001/api/status"
echo -e "  Node 2 API: http://localhost:3002/api/status"
echo -e "${BLUE}--------------------------------------${NC}"

echo -e "\n${GREEN}To view logs:${NC}"
echo -e "  docker-compose logs -f"
echo -e "\n${GREEN}To stop the network:${NC}"
echo -e "  docker-compose down"

echo -e "\n${GREEN}Setup complete!${NC}"
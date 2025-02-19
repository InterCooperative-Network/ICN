#!/bin/bash

# Source Rust environment if available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Auto-install docker if not installed
if ! command -v docker >/dev/null 2>&1; then
    echo "Docker not found. Attempting to install docker..."
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update && sudo apt-get install -y docker.io || { echo "Docker installation failed."; exit 1; }
    else
        echo "Error: docker not installed, please install docker manually." && exit 1
    fi
fi

# Auto-install docker-compose if not installed
if ! command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose not found. Attempting to install docker-compose..."
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update && sudo apt-get install -y docker-compose || { echo "docker-compose installation failed."; exit 1; }
    else
        echo "Error: docker-compose not installed, please install docker-compose manually." && exit 1
    fi
fi

# Wait for docker daemon to be active (timeout: 30 seconds)
timeout=30
interval=2
elapsed=0
check_docker_running() {
    if command -v systemctl >/dev/null 2>&1; then
        systemctl is-active --quiet docker
    elif command -v service >/dev/null 2>&1; then
        service docker status 2>&1 | grep -iq "running"
    else
        return 1
    fi
}
while ! check_docker_running; do
    if [ $elapsed -ge $timeout ]; then
        echo "Error: Docker daemon is not running after waiting for $timeout seconds. Please start it properly and try again."
        exit 1
    fi
    sleep $interval
    elapsed=$(( elapsed + interval ))
done

# Create necessary directories
mkdir -p backend/src
mkdir -p frontend/src
mkdir -p contracts
mkdir -p crates

# Initialize backend only if Cargo.toml doesn't exist
cd backend
if [ ! -f Cargo.toml ]; then
    if command -v cargo >/dev/null 2>&1; then
        cargo init
    else
        echo "cargo not installed, using Docker to initialize backend"
        docker run --rm -v "$(pwd)":/usr/src/app -w /usr/src/app rust:1.75-slim cargo init
    fi
else
    echo "Backend already initialized. Skipping cargo init."
fi
cd ..

# Initialize frontend only if package.json doesn't exist
cd frontend
if [ ! -f package.json ]; then
    if command -v npm >/dev/null 2>&1; then
        npm init -y
        npm install react react-dom @types/react @types/react-dom typescript
    else
        echo "npm not installed, using Docker to initialize frontend"
        docker run --rm -v "$(pwd)":/usr/src/app -w /usr/src/app node:16-slim npm init -y
        docker run --rm -v "$(pwd)":/usr/src/app -w /usr/src/app node:16-slim npm install react react-dom @types/react @types/react-dom typescript
    fi
else
    echo "Frontend already initialized. Skipping npm init."
fi
cd ..

# Create docker network (ignore if exists)
docker network create icn-network 2>/dev/null || true

# Build and start services (fail gracefully if config is missing)
if [ -f docker/docker-compose.yml ]; then
    docker-compose -f docker/docker-compose.yml build
    docker-compose -f docker/docker-compose.yml up -d
else
    echo "Warning: docker-compose.yml not found in docker directory"
fi

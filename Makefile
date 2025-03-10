.PHONY: all build clean test lint doc dev release help

# Project variables
PROJECT_NAME := icn
CARGO := cargo
DOCKER_COMPOSE := docker-compose

# Default target
all: build

# Build the project
build:
	@echo "Building $(PROJECT_NAME)..."
	$(CARGO) build

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf target/

# Run tests
test:
	@echo "Running tests..."
	$(CARGO) test

# Run specific tests
test-unit:
	@echo "Running unit tests..."
	$(CARGO) test --lib

test-integration:
	@echo "Running integration tests..."
	$(CARGO) test --test '*'

# Run linting tools
lint:
	@echo "Running linters..."
	$(CARGO) clippy -- -D warnings
	$(CARGO) fmt -- --check

# Fix linting issues
fix:
	@echo "Fixing formatting issues..."
	$(CARGO) fmt

# Generate documentation
doc:
	@echo "Generating documentation..."
	$(CARGO) doc --no-deps --open

# Start development environment
dev:
	@echo "Starting development environment..."
	./setup-dev.sh
	$(CARGO) build

# Build for release
release:
	@echo "Building for release..."
	$(CARGO) build --release

# Start backend services
backend:
	@echo "Starting backend services..."
	cd backend && $(CARGO) run

# Start CLI
cli:
	@echo "Running CLI..."
	$(CARGO) run --bin icn_cli

# Start frontend
frontend:
	@echo "Starting frontend..."
	cd frontend && npm start

# Build docker images
docker-build:
	@echo "Building Docker images..."
	$(DOCKER_COMPOSE) build

# Start all services with Docker
docker-up:
	@echo "Starting services with Docker..."
	$(DOCKER_COMPOSE) up -d

# Stop all Docker services
docker-down:
	@echo "Stopping Docker services..."
	$(DOCKER_COMPOSE) down

# Show help
help:
	@echo "Available targets:"
	@echo "  make build         - Build the project"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make test          - Run all tests"
	@echo "  make test-unit     - Run unit tests"
	@echo "  make test-integration - Run integration tests"
	@echo "  make lint          - Run linters"
	@echo "  make fix           - Fix formatting issues"
	@echo "  make doc           - Generate documentation"
	@echo "  make dev           - Set up development environment"
	@echo "  make release       - Build for release"
	@echo "  make backend       - Start backend services"
	@echo "  make cli           - Run CLI"
	@echo "  make frontend      - Start frontend"
	@echo "  make docker-build  - Build Docker images"
	@echo "  make docker-up     - Start all services with Docker"
	@echo "  make docker-down   - Stop all Docker services"
	@echo "  make help          - Show this help message" 
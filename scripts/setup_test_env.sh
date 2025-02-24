#!/bin/bash
set -e

# Function to install system dependencies
install_system_dependencies() {
    echo "Installing system dependencies..."
    if [ -f /etc/debian_version ]; then
        # Debian/Ubuntu
        sudo apt-get update
        sudo apt-get install -y postgresql postgresql-contrib postgresql-client libpq-dev
        
        # Start PostgreSQL service
        sudo service postgresql start
        
        # Configure PostgreSQL to allow local connections
        sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"
        sudo sed -i "s/#listen_addresses = 'localhost'/listen_addresses = '*'/" /etc/postgresql/13/main/postgresql.conf
        sudo sed -i "s/peer/trust/" /etc/postgresql/13/main/pg_hba.conf
        sudo sed -i "s/md5/trust/" /etc/postgresql/13/main/pg_hba.conf
        sudo service postgresql restart
    else
        echo "This script currently only supports Debian/Ubuntu"
        exit 1
    fi
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install dependencies if needed
if ! command_exists psql; then
    install_system_dependencies
fi

# Function to wait for PostgreSQL to be ready
wait_for_postgres() {
    echo "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if psql -h localhost -U postgres -c '\q' >/dev/null 2>&1; then
            echo "PostgreSQL is ready!"
            return 0
        fi
        echo "Waiting for PostgreSQL... ($i/30)"
        sleep 1
    done
    echo "PostgreSQL did not become ready in time"
    exit 1
}

# Ensure PostgreSQL is running
wait_for_postgres

# Create test database
echo "Creating test database..."
psql -h localhost -U postgres -c "DROP DATABASE IF EXISTS icndb_test;" || {
    echo "Failed to drop existing database. Make sure PostgreSQL is running and accessible."
    exit 1
}
psql -h localhost -U postgres -c "CREATE DATABASE icndb_test;" || {
    echo "Failed to create database. Make sure PostgreSQL is running and accessible."
    exit 1
}

# Initialize schema
echo "Initializing database schema..."
psql -h localhost -U postgres -d icndb_test -f backend/migrations/test/init.sql || {
    echo "Failed to initialize database schema."
    exit 1
}

# Set environment variables for tests
export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/icndb_test"
export RUST_LOG="debug"
export RUST_BACKTRACE=1

# Install Rust test tools if they're not already installed
echo "Installing Rust test tools..."
if ! command_exists cargo-tarpaulin; then
    cargo install cargo-tarpaulin
fi

if ! command_exists cargo-audit; then
    cargo install cargo-audit
fi

if ! command_exists cargo-deny; then
    cargo install cargo-deny
fi

# Install k6 for load testing if it's not already installed
if ! command_exists k6; then
    echo "Installing k6..."
    if [ -f /etc/debian_version ]; then
        sudo gpg -k
        sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
        echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
        sudo apt-get update
        sudo apt-get install k6
    else
        echo "Please install k6 manually for your operating system"
    fi
fi

# Create test results directory
mkdir -p test-results/k6
mkdir -p test-results/coverage

echo "Test environment setup complete!" 
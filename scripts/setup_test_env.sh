#!/bin/bash
set -e

# Create test database
PGPASSWORD=postgres psql -h localhost -U postgres -c "DROP DATABASE IF EXISTS icndb_test;"
PGPASSWORD=postgres psql -h localhost -U postgres -c "CREATE DATABASE icndb_test;"

# Initialize schema
PGPASSWORD=postgres psql -h localhost -U postgres -d icndb_test -f backend/migrations/test/init.sql

# Set environment variables for tests
export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/icndb_test"
export RUST_LOG="debug"
export RUST_BACKTRACE=1

# Install test dependencies
cargo install cargo-tarpaulin
cargo install cargo-audit
cargo install cargo-deny

# Install k6 for load testing
if ! command -v k6 &> /dev/null; then
    sudo gpg -k
    sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
    echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
    sudo apt-get update
    sudo apt-get install k6
fi

# Create test results directory
mkdir -p test-results/k6
mkdir -p test-results/coverage

echo "Test environment setup complete!" 
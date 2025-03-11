#!/bin/bash
set -e

# Load environment variables
if [ ! -f ".env" ]; then
    echo "Creating default .env file..."
    echo "DATABASE_URL=sqlite:backend/db/icn.db" > .env
fi
source .env

# Create SQLite database directory if it doesn't exist
mkdir -p backend/db

# Export the SQLite database URL if not already set
if [ -z "$DATABASE_URL" ]; then
    export DATABASE_URL="sqlite:backend/db/icn.db"
fi

# Function to cleanup on script exit
cleanup() {
    echo "Cleaning up..."
    pkill -P $$
}

# Set up cleanup trap
trap cleanup EXIT

# Install frontend dependencies
echo "Installing frontend dependencies..."
(cd frontend && npm install)

# Run database migrations
echo "Running database migrations..."
(cd backend && sqlite3 db/icn.db < migrations/20240311_initial_schema.sql)

# Start the backend server
echo "Starting backend server..."
(cd backend && cargo run) &

# Start the frontend development server
echo "Starting frontend development server..."
(cd frontend && npm run dev) &

# Wait for Ctrl+C
wait 
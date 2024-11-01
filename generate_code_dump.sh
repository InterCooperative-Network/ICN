#!/bin/bash

# Output file
OUTPUT_FILE="project_code_dump.txt"

# Create or clear the output file
echo "Project Code Dump - Generated $(date)" > $OUTPUT_FILE
echo "======================================" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Add project tree
echo "Project Tree:" >> $OUTPUT_FILE
echo "=============" >> $OUTPUT_FILE
tree -L 4 . >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Function to add a file to the dump
add_file() {
    local file=$1
    echo "===================" >> $OUTPUT_FILE
    echo "File: $file" >> $OUTPUT_FILE
    echo "===================" >> $OUTPUT_FILE
    cat "$file" >> $OUTPUT_FILE
    echo "" >> $OUTPUT_FILE
    echo "" >> $OUTPUT_FILE
}

# Backend Rust files
echo "Adding backend Rust files..."
find ./backend/src -name "*.rs" | while read file; do
    add_file "$file"
done

# Frontend TypeScript/React files
echo "Adding frontend files..."
find ./frontend/src -name "*.tsx" -o -name "*.ts" | while read file; do
    add_file "$file"
done

# Configuration files
echo "Adding configuration files..."
for config_file in \
    "./backend/Cargo.toml" \
    "./frontend/package.json" \
    "./frontend/tsconfig.json" \
    "./docker/docker-compose.yml" \
    "./docker/backend.Dockerfile" \
    "./docker/frontend.Dockerfile"
do
    if [ -f "$config_file" ]; then
        add_file "$config_file"
    fi
done

# Documentation files (if they exist)
echo "Adding documentation files..."
find ./docs -name "*.md" | while read file; do
    add_file "$file"
done

echo "Code dump generated in $OUTPUT_FILE"
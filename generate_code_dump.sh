#!/bin/bash

# Output file (default to "project_code_dump.txt" if not provided)
OUTPUT_FILE="${1:-project_code_dump.txt}"

# Directories to exclude
EXCLUDE_DIRS="target|node_modules|dist|docs|.git|coverage"

# Ensure required commands are available
for cmd in tree find cat; do
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $cmd command is required but not installed." >&2
        exit 1
    fi
done

# Create or clear the output file
echo "Project Code Dump - Generated $(date)" > $OUTPUT_FILE
echo "======================================" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Add project tree
echo "Project Tree:" >> $OUTPUT_FILE
echo "=============" >> $OUTPUT_FILE
tree -L 4 --prune -I "$EXCLUDE_DIRS" . >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Function to add a file to the dump, including empty files
add_file() {
    local file=$1
    echo "===================" >> $OUTPUT_FILE
    echo "File: $file" >> $OUTPUT_FILE
    echo "===================" >> $OUTPUT_FILE

    if [ -s "$file" ]; then
        # If the file is not empty, add its content
        cat "$file" >> $OUTPUT_FILE
    else
        # If the file is empty, add a placeholder message
        echo "<EMPTY FILE>" >> $OUTPUT_FILE
    fi

    echo "" >> $OUTPUT_FILE
    echo "" >> $OUTPUT_FILE
}

# Backend Rust files
echo "Adding backend Rust files..."
find ./backend/src -name "*.rs" -not -path "*/target/*" | while read file; do
    add_file "$file"
done

# Frontend TypeScript/React files
echo "Adding frontend files..."
find ./frontend/src -name "*.tsx" -o -name "*.ts" -not -path "*/node_modules/*" | while read file; do
    add_file "$file"
done

# Configuration files
echo "Adding configuration files..."
CONFIG_FILES=(
    "./backend/Cargo.toml"
    "./frontend/package.json"
    "./frontend/tsconfig.json"
    "./docker/docker-compose.yml"
    "./docker/backend.Dockerfile"
    "./docker/frontend.Dockerfile"
)
for config_file in "${CONFIG_FILES[@]}"; do
    add_file "$config_file"
done

# Summarize output
echo "Code dump generated in $OUTPUT_FILE"

#!/bin/bash

# Input validation
OUTPUT_FILE="${1:-project_code_dump.txt}"
if [[ -f "$OUTPUT_FILE" && ! -w "$OUTPUT_FILE" ]]; then
    echo "Error: Cannot write to $OUTPUT_FILE" >&2
    exit 1
fi

# Key directories and files to analyze
CORE_DIRS="backend/src frontend/src contracts"
EXCLUDE_DIRS="target|node_modules|dist|.git|coverage|docs|tests|examples|__tests__|.vscode"
IMPORTANT_FILES=(
    "backend/Cargo.toml"
    "backend/src/lib.rs" 
    "backend/src/main.rs"
    "frontend/package.json"
    "frontend/tsconfig.json"
    "contracts/*/src/lib.rs"
    "docker-compose.yml"
)

# Ensure required commands are available
for cmd in tree find cat; do
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $cmd command is required but not installed." >&2
        exit 1
    fi
done

# Project metadata and timestamp
echo "Project Code Dump - Generated $(date -u)" > "$OUTPUT_FILE"
echo "======================================" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Function to generate LLM context header
generate_llm_context() {
    cat << EOF >> "$OUTPUT_FILE"
LLM Context Information
======================
This is a distributed cooperative network system with the following key components:

Core Components:
- Backend: Rust-based node implementation
- Frontend: TypeScript/React web interface
- Contracts: Smart contracts for cooperative governance

Architecture Overview:
- Blockchain: Handles consensus and state management
- Identity: DID-based identity management
- Relationship: Tracks member interactions and relationships
- Reputation: Manages trust and reputation scores
- Governance: Handles proposals and voting
- WebSocket: Real-time communication layer

File Organization:
- /backend/src/: Core Rust implementation
- /frontend/src/: React frontend application
- /contracts/: Smart contract implementations
- /docker/: Deployment configurations

======================

EOF
}

# Add LLM context at the beginning
generate_llm_context

# Project tree section - limit depth and filter more aggressively
echo "Project Tree:" >> "$OUTPUT_FILE"
echo "=============" >> "$OUTPUT_FILE"
tree -L 3 -I "$EXCLUDE_DIRS" --noreport . >> "$OUTPUT_FILE" 2>/dev/null || ls -R . >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Function to add dependency context
add_dependency_context() {
    local filepath="$1"
    local filename=$(basename "$filepath")
    local dirpath=$(dirname "$filepath")

    # Extract imports and dependencies
    case "$filename" in
        *.rs)
            echo "Dependencies:" >> "$OUTPUT_FILE"
            grep -E "^(use|mod) " "$filepath" | sort | uniq >> "$OUTPUT_FILE"
            ;;
        *.ts|*.tsx)
            echo "Dependencies:" >> "$OUTPUT_FILE"
            grep -E "^import " "$filepath" | sort | uniq >> "$OUTPUT_FILE"
            ;;
    esac
    echo >> "$OUTPUT_FILE"
}

# Function to format and add a file to the dump
add_file() {
    local filepath="$1"
    local ext="${filepath##*.}"
    
    # Get file size in a more robust way
    local filesize=0
    if [[ "$OSTYPE" == "darwin"* ]]; then
        filesize=$(stat -f %z "$filepath" 2>/dev/null)
    else
        filesize=$(stat -c %s "$filepath" 2>/dev/null)
    fi

    # Get modification time
    local modified=""
    if [[ "$OSTYPE" == "darwin"* ]]; then
        modified=$(stat -f %Sm "$filepath" 2>/dev/null)
    else
        modified=$(stat -c %y "$filepath" 2>/dev/null)
    fi

    # Convert filesize to number and check
    filesize=${filesize:-0}
    if (( filesize > 150000 )); then
        echo "Skipping large file: $filepath ($filesize bytes)" >&2
        return
    fi

    # Skip empty files
    if [[ ! -s "$filepath" ]]; then
        return
    fi

    # Skip generated files and test files
    if [[ $filepath =~ .*\.(generated|test|spec)\..* ]]; then
        return
    fi

    echo "===================" >> "$OUTPUT_FILE"
    echo "File: $filepath" >> "$OUTPUT_FILE"
    echo "Size: $filesize bytes" >> "$OUTPUT_FILE"
    echo "Modified: $modified" >> "$OUTPUT_FILE"
    
    # Add dependency context
    add_dependency_context "$filepath"
    
    echo "===================" >> "$OUTPUT_FILE"
    
    # Only process code files
    case "$ext" in
        rs|ts|tsx|js|json|toml)
            echo "\`\`\`$ext" >> "$OUTPUT_FILE"
            cat "$filepath" >> "$OUTPUT_FILE"
            echo "\`\`\`" >> "$OUTPUT_FILE"
            ;;
    esac
    echo >> "$OUTPUT_FILE"
}

# Main file processing
{
    # Process IMPORTANT_FILES first
    for file in "${IMPORTANT_FILES[@]}"; do
        find . -type f -path "*/$file" 2>/dev/null
    done
    
    # Then process core API/interface files
    find . -type f \( \
        -path "*/src/api/*.rs" -o \
        -path "*/src/lib.rs" -o \
        -path "*/src/main.rs" -o \
        -path "*/src/types/*.ts" -o \
        -path "*/src/models/*.ts" -o \
        -path "*/src/interfaces/*.ts" \
    \) -not -path "*/\.*" -not -path "*/${EXCLUDE_DIRS}*"
} | while read -r file; do
    add_file "$file"
done

echo "Code dump generated successfully in $OUTPUT_FILE"

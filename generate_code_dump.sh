#!/bin/bash

# Base output filename prefix (will be used as prefix for multiple files)
OUTPUT_PREFIX="${1:-project_code}"

# Target output size for each file (approximate in bytes)
TARGET_SIZE=1500000  # ~1.5MB per file

# Exclude patterns - be very aggressive with dependencies and generated files
EXCLUDE_PATTERNS=(
    "*/node_modules/*"
    "*/target/*"
    "*/dist/*"
    "*/.git/*"
    "*/coverage/*"
    "*/.vscode/*"
    "*/build/*"
    "*/docs/*"
    "*/__tests__/*"
    "*/tests/*"
    "*/test/*"
    "*/examples/*"
    "*/.cache/*"
    "*/vendor/*"
)

# Dump file structure - define components and their patterns
declare -A COMPONENT_PATTERNS
# Core config files
COMPONENT_PATTERNS["01_config"]="Cargo.toml package.json tsconfig.json docker-compose.yml .env.example Makefile README.md rust-toolchain.toml"
# Backend (Rust) files
COMPONENT_PATTERNS["02_backend"]="backend/src/*.rs backend/src/*/*.rs src/*.rs src/*/*.rs src/*/*/*.rs"
# Frontend files
COMPONENT_PATTERNS["03_frontend"]="frontend/src/*.ts frontend/src/*.tsx frontend/src/*/*.ts frontend/src/*/*.tsx frontend/src/*/*/*.ts frontend/src/*/*/*.tsx"
# Contracts
COMPONENT_PATTERNS["04_contracts"]="contracts/*/src/*.rs"
# Core services
COMPONENT_PATTERNS["05_identity"]="identity/*.rs identity/*/*.rs"
COMPONENT_PATTERNS["06_governance"]="governance/*.rs governance/*/*.rs"
COMPONENT_PATTERNS["07_consensus"]="consensus/*.rs consensus/*/*.rs"
COMPONENT_PATTERNS["08_reputation"]="reputation/*.rs reputation/*/*.rs"
COMPONENT_PATTERNS["09_relationship"]="relationship/*.rs relationship/*/*.rs"
# Scripts and utilities
COMPONENT_PATTERNS["10_scripts"]="scripts/*.sh *.sh"

# Build the exclude args for find command
build_exclude_args() {
    local exclude_args=""
    for pattern in "${EXCLUDE_PATTERNS[@]}"; do
        exclude_args="$exclude_args -not -path \"$pattern\""
    done
    echo "$exclude_args"
}

# Ensure required commands are available
for cmd in find cat wc; do
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $cmd command is required but not installed." >&2
        exit 1
    fi
done

# Generate common header for all files
generate_header() {
    local component="$1"
    local output_file="$2"
    
    cat << EOF > "$output_file"
Project Code Dump - $component - Generated $(date -u)
=========================================================

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

Note: This is file $(echo "$component" | cut -d'_' -f1) of a multi-file dump.
All dependency directories are excluded from these dumps.
======================

EOF
}

# Function to format and add a file to the dump, with size tracking
add_file() {
    local filepath="$1"
    local output_file="$2"
    local current_size="$3"
    local max_size="$4"  # Optional max size for individual files
    
    # Get file extension
    local ext="${filepath##*.}"
    
    # Skip binary files like .png, .jpg, etc.
    if [[ "$ext" == "png" || "$ext" == "jpg" || "$ext" == "jpeg" || 
          "$ext" == "gif" || "$ext" == "woff" || "$ext" == "woff2" || 
          "$ext" == "ttf" || "$ext" == "eot" || "$ext" == "ico" ]]; then
        return "$current_size"
    fi
    
    # Get file size
    local filesize=$(wc -c < "$filepath" 2>/dev/null || echo 0)
    
    # Skip if filesize is 0 or exceeds max_size
    if [[ $filesize -eq 0 || ($max_size -gt 0 && $filesize -gt $max_size) ]]; then
        return "$current_size"
    fi
    
    # Guess if it's a binary file using the 'file' command
    if file "$filepath" | grep -q "binary"; then
        return "$current_size"
    fi
    
    # Calculate the size this entry would add to output
    # Include metadata (headers) + file content + markdown formatting
    local entry_size=$(( 100 + filesize + 10 ))
    
    # Check if adding this file would exceed our target size
    if [[ $(( current_size + entry_size )) -gt $TARGET_SIZE ]]; then
        return "$current_size"  # Return unchanged size if we'd exceed the limit
    fi
    
    # Add file to output
    {
        echo "==================="
        echo "File: $filepath"
        echo "Size: $filesize bytes"
        echo "==================="
        
        # Format code based on extension
        if [[ -n "$ext" ]]; then
            echo "\`\`\`$ext"
            cat "$filepath"
            echo "\`\`\`"
        else
            # Handle files without extension
            echo "\`\`\`"
            cat "$filepath"
            echo "\`\`\`"
        fi
        echo ""
    } >> "$output_file"
    
    # Return updated size
    echo $(( current_size + entry_size ))
}

# Process each component and create separate dump files
for component in "${!COMPONENT_PATTERNS[@]}"; do
    patterns="${COMPONENT_PATTERNS[$component]}"
    output_file="${OUTPUT_PREFIX}_${component}.txt"
    
    echo "Processing component: $component"
    echo "Output file: $output_file"
    
    # Generate header
    generate_header "$component" "$output_file"
    current_size=$(wc -c < "$output_file")
    
    # Process file patterns for this component
    file_list=()
    
    # Build a list of matching files
    for pattern in $patterns; do
        # Get exclude args
        exclude_args=$(build_exclude_args)
        
        # Build and execute find command
        find_cmd="find . -type f -path \"*/$pattern\" $exclude_args 2>/dev/null"
        while read -r file; do
            if [[ -n "$file" ]]; then
                file_list+=("$file")
            fi
        done < <(eval "$find_cmd")
        
        # Also try direct match for files in root directory
        if [[ "$pattern" != *"/"* ]]; then
            if [[ -f "$pattern" ]]; then
                file_list+=("$pattern")
            fi
        fi
    done
    
    # Remove duplicates
    unique_files=($(echo "${file_list[@]}" | tr ' ' '\n' | sort -u))
    
    echo "Found ${#unique_files[@]} files for component $component"
    
    # Process each file
    for file in "${unique_files[@]}"; do
        # Check if file still exists and is not a directory
        if [[ -f "$file" ]]; then
            # Skip if this file belongs to an excluded pattern
            skip_file=0
            for exclude in "${EXCLUDE_PATTERNS[@]}"; do
                if [[ "$file" == $exclude ]]; then
                    skip_file=1
                    break
                fi
            done
            
            if [[ $skip_file -eq 0 ]]; then
                new_size=$(add_file "$file" "$output_file" "$current_size" 50000)
                current_size=$new_size
            fi
        fi
    done
    
    # Add summary
    {
        echo "==================="
        echo "Summary for $component"
        echo "==================="
        echo "Total size of dump: $current_size bytes"
        echo "Patterns included:"
        for pattern in $patterns; do
            echo "- $pattern"
        done
        echo ""
        echo "Files processed: ${#unique_files[@]}"
        echo "==================="
    } >> "$output_file"
    
    echo "Component $component dump complete. Size: $current_size bytes"
    echo ""
done

# Generate a manifest file listing all components
manifest_file="${OUTPUT_PREFIX}_manifest.txt"
echo "Project Code Dump Manifest - Generated $(date -u)" > "$manifest_file"
echo "==========================================================" >> "$manifest_file"
echo "" >> "$manifest_file"
echo "This project has been split into multiple files to stay within size limits." >> "$manifest_file"
echo "Use these files in the following order:" >> "$manifest_file"
echo "" >> "$manifest_file"

for component in "${!COMPONENT_PATTERNS[@]}"; do
    output_file="${OUTPUT_PREFIX}_${component}.txt"
    file_size=$(wc -c < "$output_file")
    echo "- ${component}: ${output_file} ($(numfmt --to=iec-i --suffix=B --format="%.2f" $file_size))" >> "$manifest_file"
done

echo "" >> "$manifest_file"
echo "Total components: ${#COMPONENT_PATTERNS[@]}" >> "$manifest_file"

echo "Multi-file code dump complete. See $manifest_file for details."
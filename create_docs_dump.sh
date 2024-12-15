#!/bin/bash

# Output file (default to "project_docs_dump.txt" if not provided)
OUTPUT_FILE="${1:-project_docs_dump.txt}"

# Directories to exclude
EXCLUDE_DIRS="target|node_modules|dist|.git|coverage"

# Ensure required commands are available
for cmd in tree find cat; do
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $cmd command is required but not installed." >&2
        exit 1
    fi
done

# Create or clear the output file
echo "Project Docs Dump - Generated $(date)" > $OUTPUT_FILE
echo "======================================" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Add docs tree
echo "Docs Tree:" >> $OUTPUT_FILE
echo "=============" >> $OUTPUT_FILE
tree -L 5 --prune -I "$EXCLUDE_DIRS" ./docs >> $OUTPUT_FILE  # Relative path to docs
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

# Documentation files
echo "Adding documentation files..."
find ./docs -type f -not -path "*/.git/*" | while read file; do  # Relative path to docs
    add_file "$file"
done

# Summarize output
echo "Docs dump generated in $OUTPUT_FILE"
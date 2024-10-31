#!/bin/bash

# Output file
output_file="project_code_dump.txt"

# Clear the output file if it already exists
> "$output_file"

# Add the project tree at the top of the file
echo "Project Tree:" >> "$output_file"
echo "=============" >> "$output_file"
tree -a -I 'target|.*' . >> "$output_file"   # Exclude target and hidden files (adjust as necessary)
echo -e "\n\n" >> "$output_file"

# Function to add the file content with a heading
add_file_content() {
    local file_path=$1
    echo "===================" >> "$output_file"
    echo "File: $file_path" >> "$output_file"
    echo "===================" >> "$output_file"
    cat "$file_path" >> "$output_file"
    echo -e "\n\n" >> "$output_file"
}

# Iterate through each file in the src directory and add it to the output file
for file in $(find src -type f -name '*.rs'); do
    add_file_content "$file"
done

echo "Code dump generated in $output_file."

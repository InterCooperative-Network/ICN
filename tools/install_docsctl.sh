#!/bin/bash

# Get the virtual environment's bin directory
VENV_BIN="$VIRTUAL_ENV/bin"

if [ -z "$VIRTUAL_ENV" ]; then
    echo "Error: No virtual environment activated"
    exit 1
fi

# Create symlink to docsctl in the virtual environment's bin directory
ln -sf "$(pwd)/tools/doctools/docsctl" "$VENV_BIN/docsctl"

# Make sure the original script is executable
chmod +x "$(pwd)/tools/doctools/docsctl"

echo "Installation complete! You can now use 'docsctl' command."
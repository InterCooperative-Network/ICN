#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Activate the virtual environment
source "${SCRIPT_DIR}/.env/icn-docs/bin/activate"

# Add tools/doctools to PYTHONPATH
export PYTHONPATH="${SCRIPT_DIR}/doctools:${PYTHONPATH}"

# Add the virtual environment's bin to PATH
export PATH="$VIRTUAL_ENV/bin:$PATH"

echo "ICN documentation environment activated!"
echo "Use 'docsctl' to manage documentation"
echo "Use 'deactivate' to exit the virtual environment"
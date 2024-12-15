#!/bin/bash

# Create a .env directory in tools for the virtual environment
mkdir -p tools/.env

# Create the virtual environment
python3 -m venv tools/.env/icn-docs

# Create activation script for convenience
cat > tools/activate-docs-env.sh << 'EOL'
#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Activate the virtual environment
source "${SCRIPT_DIR}/.env/icn-docs/bin/activate"

# Add tools/doctools to PYTHONPATH
export PYTHONPATH="${SCRIPT_DIR}/doctools:${PYTHONPATH}"

echo "ICN documentation environment activated!"
echo "Use 'deactivate' to exit the virtual environment"
EOL

# Make the activation script executable
chmod +x tools/activate-docs-env.sh

# Create a .gitignore entry
cat >> .gitignore << 'EOL'

# Python virtual environments
tools/.env/
EOL

echo "Virtual environment setup complete!"
echo ""
echo "To use the environment:"
echo "1. Source the activation script:"
echo "   source tools/activate-docs-env.sh"
echo ""
echo "2. Install the requirements:"
echo "   pip install -r tools/requirements.txt"
echo ""
echo "3. When finished, type 'deactivate' to exit the environment"
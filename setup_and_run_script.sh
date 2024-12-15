#!/bin/bash

# Set up missing files and directories for documentation
mkdir -p docs/guides
mkdir -p docs/architecture/diagrams/images
mkdir -p docs/architecture/diagrams/mermaid

# Create missing Markdown files in docs/guides
touch docs/guides/getting-started.md
touch docs/guides/governance-guide.md
touch docs/guides/reputation-management.md

# Create placeholder image in docs/architecture/diagrams/images
touch docs/architecture/diagrams/images/component-diagram.png

# Create Mermaid diagram placeholder in docs/architecture/diagrams/mermaid
echo "%% Mermaid diagram placeholder" > docs/architecture/diagrams/mermaid/component-diagram.mermaid

# Add placeholder content to new files
echo "# Getting Started" > docs/guides/getting-started.md
echo "# Governance Guide" > docs/guides/governance-guide.md
echo "# Reputation Management" > docs/guides/reputation-management.md

# Change permissions to ensure Docker can access the docs directory
chmod -R 755 docs

# Navigate to docker folder and start the services
cd docker

# Bring down any running containers
sudo docker-compose down

# Build and start the docs service
sudo docker-compose up --build docs

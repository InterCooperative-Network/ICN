#!/bin/bash

# Create documentation tools structure
mkdir -p tools/doctools
touch tools/doctools/__init__.py

# Create documentation structure
mkdir -p docs/{architecture/{overview,backend,frontend,diagrams/{system,sequence,component}},specifications/{core,api,protocols},development/{setup,guides,plans},user/{guides,tutorials}}

# Create tools files
cat > tools/doctools/docmanager.py << 'EOL'
#!/usr/bin/env python3
import os
import yaml
from datetime import datetime
import argparse
from pathlib import Path
import re
import shutil

# [Previous docmanager.py content would go here]
EOL

cat > tools/doctools/specgen.py << 'EOL'
#!/usr/bin/env python3
import sys
from docmanager import DocManager
from pathlib import Path
import yaml

# [Previous specgen.py content would go here]
EOL

# Make the Python files executable
chmod +x tools/doctools/docmanager.py
chmod +x tools/doctools/specgen.py

# Create requirements.txt
cat > tools/requirements.txt << 'EOL'
pyyaml>=6.0
EOL

# Create tools README
cat > tools/README.md << 'EOL'
# ICN Documentation Tools

This directory contains tools for managing ICN project documentation.

## Setup

1. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

2. Add tools to your PATH:
   ```bash
   export PATH=$PATH:$(pwd)/tools/doctools
   ```

## Usage

Generate a new specification:
```bash
specgen.py --title "Component Name" --type "specification" --component "core-component"
```

List all documents:
```bash
docmanager.py list
```

Create documentation index:
```bash
docmanager.py index
```
EOL

# Create initial documentation templates
mkdir -p docs/templates

# Create core component template
cat > docs/templates/core-component-template.md << 'EOL'
# {title}

## 1. Overview

### 1.1 Purpose

### 1.2 Core Components

## 2. Detailed Specifications

### 2.1 Data Structures

### 2.2 Interfaces

### 2.3 Behaviors

## 3. Implementation Guidelines

### 3.1 Performance Requirements

### 3.2 Security Requirements

### 3.3 Error Handling

## 4. Testing Requirements

## 5. Monitoring and Metrics

## 6. Future Considerations
EOL

# Create API template
cat > docs/templates/api-template.md << 'EOL'
# {title} API Specification

## 1. API Overview

### 1.1 Purpose

### 1.2 Endpoints

## 2. Authentication

## 3. Endpoints Detail

## 4. Error Handling

## 5. Rate Limiting

## 6. Security Considerations

## 7. Example Usage
EOL

# Create root documentation index
cat > docs/README.md << 'EOL'
# ICN Documentation

## Structure

- `architecture/` - System architecture documentation
- `specifications/` - Technical specifications
- `development/` - Development guides and plans
- `user/` - User documentation and tutorials

## Getting Started

1. Install documentation tools:
   ```bash
   cd tools
   pip install -r requirements.txt
   ```

2. Generate documentation index:
   ```bash
   python tools/doctools/docmanager.py index
   ```

## Documentation Standards

Please refer to `development/guides/documentation-standards.md` for our documentation guidelines.
EOL

# Create initial documentation standards guide
mkdir -p docs/development/guides
cat > docs/development/guides/documentation-standards.md << 'EOL'
# Documentation Standards

## File Organization

- Use appropriate directory for document type
- Follow naming conventions
- Include required metadata

## Writing Style

- Be clear and concise
- Include code examples where appropriate
- Keep documentation up to date
- Use proper Markdown formatting

## Review Process

1. Create new document using tools
2. Submit for review
3. Address feedback
4. Update documentation index

## Templates

Use provided templates in `docs/templates/` for new documents.
EOL

# Create .gitignore for documentation
cat > .gitignore << 'EOL'
# Python
__pycache__/
*.py[cod]
*$py.class

# Environment
.env
.venv
env/
venv/

# IDE
.idea/
.vscode/
*.swp

# Documentation build
_build/
EOL

# Initialize documentation index
python tools/doctools/docmanager.py index

echo "Documentation structure and tools have been set up successfully!"
echo "Next steps:"
echo "1. Install Python dependencies: pip install -r tools/requirements.txt"
echo "2. Review the documentation structure in docs/"
echo "3. Start creating documentation using the tools in tools/doctools/"
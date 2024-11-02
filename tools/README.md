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

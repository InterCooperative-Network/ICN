#!/bin/bash

# Script to generate comprehensive documentation for the ICN project

set -e

DOCS_DIR="docs"
API_DOCS_DIR="$DOCS_DIR/api"
EXAMPLES_DIR="$DOCS_DIR/examples"
OUTPUT_DIR="target/doc"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Generating documentation for the ICN project...${NC}"

# Create documentation directories
mkdir -p "$API_DOCS_DIR"
mkdir -p "$EXAMPLES_DIR"

# Generate API documentation
echo -e "${YELLOW}Generating API documentation with rustdoc...${NC}"
cargo doc --no-deps --all-features

# Generate README for API docs to improve navigation
cat > "$OUTPUT_DIR/index.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ICN Documentation</title>
    <link rel="stylesheet" href="./rustdoc.css">
    <style>
        body {
            padding: 20px;
            font-family: "Open Sans", sans-serif;
            max-width: 1200px;
            margin: 0 auto;
        }
        h1, h2 {
            color: #000;
            border-bottom: 1px solid #ddd;
            padding-bottom: 5px;
        }
        .crate-list {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        .crate-card {
            border: 1px solid #ddd;
            border-radius: 5px;
            padding: 15px;
            background-color: #f9f9f9;
        }
        .crate-card h3 {
            margin-top: 0;
        }
        a {
            text-decoration: none;
            color: #4d76ae;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>Internet of Cooperative Networks (ICN) Documentation</h1>
    
    <p>Welcome to the ICN project documentation. This page provides links to API documentation for all crates in the ICN ecosystem.</p>
    
    <h2>Core Crates</h2>
    <div class="crate-list">
        <div class="crate-card">
            <h3><a href="./icn_types/index.html">icn-types</a></h3>
            <p>Common data types and structures used throughout the ICN project.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_common/index.html">icn-common</a></h3>
            <p>Shared utilities and helpers for the ICN platform.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_core/index.html">icn-core</a></h3>
            <p>Core functionality and business logic for the ICN platform.</p>
        </div>
    </div>
    
    <h2>Networking Crates</h2>
    <div class="crate-list">
        <div class="crate-card">
            <h3><a href="./icn_p2p/index.html">icn-p2p</a></h3>
            <p>Peer-to-peer networking for the ICN platform.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_federation/index.html">icn-federation</a></h3>
            <p>Federation management for cooperative networks.</p>
        </div>
    </div>
    
    <h2>Identity and Governance</h2>
    <div class="crate-list">
        <div class="crate-card">
            <h3><a href="./icn_identity/index.html">icn-identity</a></h3>
            <p>Identity management using DIDs and verifiable credentials.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_governance/index.html">icn-governance</a></h3>
            <p>Democratic governance mechanisms for cooperatives.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_reputation/index.html">icn-reputation</a></h3>
            <p>Reputation tracking and management across the network.</p>
        </div>
    </div>
    
    <h2>Resources and Consensus</h2>
    <div class="crate-list">
        <div class="crate-card">
            <h3><a href="./icn_resource/index.html">icn-resource</a></h3>
            <p>Resource allocation and management.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_consensus/index.html">icn-consensus</a></h3>
            <p>Consensus mechanisms for distributed decision making.</p>
        </div>
    </div>
    
    <h2>Tools and Utilities</h2>
    <div class="crate-list">
        <div class="crate-card">
            <h3><a href="./icn_cli/index.html">icn-cli</a></h3>
            <p>Command-line interface for interacting with ICN.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_crypto/index.html">icn-crypto</a></h3>
            <p>Cryptographic operations and utilities.</p>
        </div>
        <div class="crate-card">
            <h3><a href="./icn_zk/index.html">icn-zk</a></h3>
            <p>Zero-knowledge proof utilities and implementations.</p>
        </div>
    </div>
    
    <footer style="margin-top: 40px; border-top: 1px solid #ddd; padding-top: 10px; color: #666; font-size: 0.9em;">
        <p>Documentation generated on $(date)</p>
    </footer>
</body>
</html>
EOF

echo -e "${GREEN}API documentation generated at $OUTPUT_DIR${NC}"

# Generate Markdown documentation for GitHub
echo -e "${YELLOW}Generating Markdown documentation...${NC}"

# Create architecture overview
cat > "$DOCS_DIR/architecture.md" << EOF
# ICN Architecture Overview

This document provides a high-level overview of the ICN architecture.

## System Components

The ICN platform is composed of several modular components that work together to provide a complete cooperative network solution:

1. **Core Layer**: Provides the fundamental building blocks
   - **icn-types**: Common data types and structures
   - **icn-common**: Shared utilities and helpers
   - **icn-core**: Core business logic

2. **Networking Layer**: Manages communication between nodes
   - **icn-p2p**: Peer-to-peer networking
   - **icn-federation**: Federation management

3. **Identity and Governance Layer**: Manages identities and governance
   - **icn-identity**: Identity management
   - **icn-governance**: Governance mechanisms
   - **icn-reputation**: Reputation tracking

4. **Resource Layer**: Manages resource allocation
   - **icn-resource**: Resource allocation and management
   - **icn-mutual-credit**: Mutual credit system

5. **Consensus Layer**: Manages distributed consensus
   - **icn-consensus**: Consensus mechanisms

6. **Cryptography Layer**: Provides cryptographic functionality
   - **icn-crypto**: Cryptographic operations
   - **icn-zk**: Zero-knowledge proofs
   - **zk_snarks**: SNARK implementations

7. **Integration Layer**: Provides interfaces to the platform
   - **icn-cli**: Command-line interface
   - **backend**: REST API server
   - **frontend**: Web interface

## Component Interaction

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend / CLI                         │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                        Backend API                          │
└───┬───────────────┬──────────────┬────────────────┬─────────┘
    │               │              │                │
┌───▼───┐       ┌───▼───┐      ┌───▼───┐        ┌───▼───┐
│ Core  │◄─────►│Identity│◄────►│Resource│◄──────►│Consensus│
└───┬───┘       └───┬───┘      └───┬───┘        └───┬───┘
    │               │              │                │
    └───────┬───────┴──────┬───────┴────────┬──────┘
            │              │                │
        ┌───▼───┐      ┌───▼───┐        ┌───▼───┐
        │ P2P   │◄────►│Reputation│◄────►│Crypto │
        └───────┘      └─────────┘      └───────┘
```

## Data Flow

1. User requests enter through CLI or frontend interface
2. Backend API processes requests and routes to appropriate module
3. Business logic is executed in core and specialized modules
4. Changes are persisted to storage
5. Events are published to relevant modules
6. Responses are returned to the user

For more detailed information about specific components, please refer to the individual module documentation:

- [Core](./core.md)
- [Identity](./identity.md)
- [Governance](./governance.md)
- [Resource Allocation](./resources.md)
- [Consensus](./consensus.md)
EOF

echo -e "${GREEN}Markdown documentation generated at $DOCS_DIR${NC}"

# Copy example code to docs directory
echo -e "${YELLOW}Collecting examples...${NC}"
find examples -name "*.rs" -exec cp {} "$EXAMPLES_DIR/" \;

echo -e "${GREEN}Documentation generation completed!${NC}"
echo -e "To view the API documentation, open $OUTPUT_DIR/index.html in your browser." 
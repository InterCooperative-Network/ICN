#!/bin/bash

# Script to create a new crate with appropriate templates

set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <crate_name> <crate_description>"
    echo "Example: $0 metrics 'Metrics and telemetry system'"
    exit 1
fi

CRATE_NAME=$1
CRATE_DESCRIPTION=$2
CRATE_PURPOSE=${3:-"the specified functionality"}

# Convert crate name to PascalCase for struct/enum names
PASCAL_CRATE_NAME=$(echo "$CRATE_NAME" | sed -r 's/(^|_)([a-z])/\U\2/g')

# Create directory structure
CRATE_DIR="crates/icn-$CRATE_NAME"
mkdir -p "$CRATE_DIR/src"

# Check if templates directory exists
TEMPLATE_DIR="templates/crate_template"
if [ ! -d "$TEMPLATE_DIR" ]; then
    echo "Error: Templates directory not found at $TEMPLATE_DIR"
    exit 1
fi

# Process and copy Cargo.toml template
sed -e "s/{{crate_name}}/$CRATE_NAME/g" \
    -e "s/{{crate_description}}/$CRATE_DESCRIPTION/g" \
    "$TEMPLATE_DIR/Cargo.toml.template" > "$CRATE_DIR/Cargo.toml"

# Process and copy README.md template
sed -e "s/{{crate_name}}/$CRATE_NAME/g" \
    -e "s/{{crate_description}}/$CRATE_DESCRIPTION/g" \
    -e "s/{{crate_purpose}}/$CRATE_PURPOSE/g" \
    -e "s/{{pascal_crate_name}}/$PASCAL_CRATE_NAME/g" \
    "$TEMPLATE_DIR/README.md.template" > "$CRATE_DIR/README.md"

# Create source files
mkdir -p "$CRATE_DIR/src/models"
mkdir -p "$CRATE_DIR/src/utils"

# Process and copy lib.rs template
sed -e "s/{{crate_name}}/$CRATE_NAME/g" \
    -e "s/{{crate_description}}/$CRATE_DESCRIPTION/g" \
    -e "s/{{crate_purpose}}/$CRATE_PURPOSE/g" \
    -e "s/{{pascal_crate_name}}/$PASCAL_CRATE_NAME/g" \
    "$TEMPLATE_DIR/src/lib.rs.template" > "$CRATE_DIR/src/lib.rs"

# Create basic models.rs file
cat > "$CRATE_DIR/src/models.rs" << EOF
//! Models for the ICN $CRATE_NAME module

/// Example model struct
#[derive(Debug, Clone, PartialEq)]
pub struct ${PASCAL_CRATE_NAME}Config {
    pub enabled: bool,
    pub name: String,
}

impl Default for ${PASCAL_CRATE_NAME}Config {
    fn default() -> Self {
        Self {
            enabled: true,
            name: "default".to_string(),
        }
    }
}
EOF

# Create basic utils.rs file
cat > "$CRATE_DIR/src/utils.rs" << EOF
//! Utility functions for the ICN $CRATE_NAME module

/// Helper function
pub fn validate_input(input: &str) -> bool {
    !input.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input() {
        assert!(validate_input("valid"));
        assert!(!validate_input(""));
    }
}
EOF

# Create an example test file
mkdir -p "$CRATE_DIR/tests"
cat > "$CRATE_DIR/tests/integration_test.rs" << EOF
//! Integration tests for icn-$CRATE_NAME

use icn_${CRATE_NAME}::*;

#[test]
fn test_integration() {
    // Basic integration test
    let manager = ${PASCAL_CRATE_NAME}Manager::new();
    // Add assertions as needed
}
EOF

# Add crate to workspace in root Cargo.toml if not already added
if ! grep -q "\"crates/icn-$CRATE_NAME\"" Cargo.toml; then
    # Find the last member of the workspace
    LAST_MEMBER=$(grep -n "crates/" Cargo.toml | tail -1 | cut -d: -f1)
    # Insert the new crate after the last member
    sed -i "${LAST_MEMBER}a\\    \"crates/icn-$CRATE_NAME\"," Cargo.toml
    echo "Added icn-$CRATE_NAME to workspace members in Cargo.toml"
fi

# Update workspace.dependencies in root Cargo.toml
if ! grep -q "icn-$CRATE_NAME =" Cargo.toml; then
    # Find the position after the last workspace dependency
    LAST_DEP=$(grep -n "icn-.* = { path =" Cargo.toml | tail -1 | cut -d: -f1)
    # Insert the new dependency after the last dependency
    sed -i "${LAST_DEP}a\\icn-$CRATE_NAME = { path = \"crates/icn-$CRATE_NAME\" }" Cargo.toml
    echo "Added icn-$CRATE_NAME to workspace.dependencies in Cargo.toml"
fi

echo "Created new crate at $CRATE_DIR"
echo "Don't forget to run 'cargo build' to verify the new crate" 
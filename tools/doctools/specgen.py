#!/usr/bin/env python3
import sys
import argparse
from pathlib import Path
import yaml
from docmanager import DocManager

class SpecificationGenerator:
    def __init__(self, doc_manager):
        self.doc_manager = doc_manager
        self.templates_dir = Path(doc_manager.root_dir) / "templates"
        self.ensure_templates()

    def ensure_templates(self):
        """Ensure specification templates exist."""
        templates = {
            "core-component": """# {title}

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

### 4.1 Unit Tests

### 4.2 Integration Tests

### 4.3 Performance Tests

## 5. Monitoring and Metrics

## 6. Future Considerations
""",
            "api": """# {title} API Specification

## 1. API Overview

### 1.1 Purpose

### 1.2 Endpoints

## 2. Authentication

## 3. Endpoints Detail

### 3.1 Resource Endpoints

### 3.2 Request/Response Formats

## 4. Error Handling

## 5. Rate Limiting

## 6. Security Considerations

## 7. Example Usage
""",
            "protocol": """# {title} Protocol Specification

## 1. Protocol Overview

### 1.1 Purpose

### 1.2 Flow

## 2. Message Formats

## 3. State Machine

## 4. Security Model

## 5. Implementation Guidelines

## 6. Compatibility Requirements

## 7. Future Extensions
"""
        }

        self.templates_dir.mkdir(exist_ok=True)
        for name, content in templates.items():
            template_file = self.templates_dir / f"{name}-template.md"
            if not template_file.exists():
                template_file.write_text(content)

    def generate_specification(self, title, spec_type, component_type, authors=None, reviewers=None):
        """Generate a new specification document."""
        # Load appropriate template
        template_file = self.templates_dir / f"{component_type}-template.md"
        if not template_file.exists():
            raise ValueError(f"Template not found for component type: {component_type}")

        template_content = template_file.read_text()
        
        # Format template with title
        content = template_content.format(title=title)

        # Create document using doc manager
        related_docs = self.find_related_docs(title, component_type)
        
        return self.doc_manager.create_document(
            title=title,
            doc_type=spec_type,
            content=content,
            authors=authors or [],
            reviewers=reviewers or [],
            related_docs=related_docs,
            status="draft"
        )

    def find_related_docs(self, title, component_type):
        """Find potentially related documents based on title and type."""
        related = []
        words = set(title.lower().split())
        
        # List existing documents
        all_docs = self.doc_manager.list_documents()
        
        for doc in all_docs:
            doc_title = doc["metadata"]["title"].lower()
            doc_words = set(doc_title.split())
            
            # Check for word overlap
            if len(words & doc_words) >= 2:  # At least 2 words in common
                related.append(str(doc["path"].relative_to(self.doc_manager.root_dir)))
                
        return related


def main():
    parser = argparse.ArgumentParser(description="Generate technical specifications")
    parser.add_argument("--title", required=True, help="Specification title")
    parser.add_argument("--type", required=True, help="Specification type")
    parser.add_argument("--component", required=True, choices=["core-component", "api", "protocol"],
                      help="Component type")
    parser.add_argument("--authors", help="Comma-separated list of authors")
    parser.add_argument("--reviewers", help="Comma-separated list of reviewers")

    args = parser.parse_args()

    # Initialize managers
    doc_manager = DocManager()
    spec_generator = SpecificationGenerator(doc_manager)

    try:
        # Parse authors and reviewers
        authors = [a.strip() for a in args.authors.split(",")] if args.authors else []
        reviewers = [r.strip() for r in args.reviewers.split(",")] if args.reviewers else []

        # Generate specification
        doc_path = spec_generator.generate_specification(
            title=args.title,
            spec_type=args.type,
            component_type=args.component,
            authors=authors,
            reviewers=reviewers
        )
        
        print(f"Generated specification at: {doc_path}")
        
        # Update index
        doc_manager.create_index()
        
    except Exception as e:
        print(f"Error generating specification: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
#!/usr/bin/env python3
import os
import yaml
from datetime import datetime
from pathlib import Path
import re
import shutil

class DocManager:
    def __init__(self, root_dir="docs"):
        self.root_dir = Path(root_dir)
        self.templates_dir = self.root_dir / "templates"
        self.ensure_directory_structure()

    def ensure_directory_structure(self):
        """Create the base directory structure if it doesn't exist."""
        directories = [
            "architecture/overview",
            "architecture/backend",
            "architecture/frontend",
            "architecture/diagrams/system",
            "architecture/diagrams/sequence",
            "architecture/diagrams/component",
            "specifications/core",
            "specifications/api",
            "specifications/protocols",
            "development/setup",
            "development/guides",
            "development/plans",
            "user/guides",
            "user/tutorials",
            "templates",
        ]

        for dir_path in directories:
            (self.root_dir / dir_path).mkdir(parents=True, exist_ok=True)

    def create_metadata(self, title, doc_type, version="1.0.0", status="draft", 
                       authors=None, reviewers=None, related_docs=None):
        """Generate metadata for a document."""
        metadata = {
            "title": title,
            "version": version,
            "date": datetime.now().strftime("%Y-%m-%d"),
            "status": status,
            "authors": authors or [],
            "reviewers": reviewers or [],
            "related_docs": related_docs or [],
            "type": doc_type,
            "last_updated": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        }
        return metadata

    def generate_filename(self, title, doc_type):
        """Generate a filename from title and type."""
        # Convert title to kebab case
        filename = re.sub(r'[^a-zA-Z0-9\s-]', '', title.lower())
        filename = re.sub(r'\s+', '-', filename)
        
        # Add date prefix for versioned documents
        if doc_type in ["specification", "plan"]:
            date_prefix = datetime.now().strftime("%Y-%m-%d")
            return f"{date_prefix}-{filename}.md"
        
        return f"{filename}.md"

    def get_directory_for_type(self, doc_type):
        """Determine the appropriate directory for a document type."""
        type_dirs = {
            "specification": self.root_dir / "specifications/core",
            "architecture": self.root_dir / "architecture/overview",
            "guide": self.root_dir / "development/guides",
            "plan": self.root_dir / "development/plans",
            "tutorial": self.root_dir / "user/tutorials",
        }
        return type_dirs.get(doc_type, self.root_dir)

    def format_document(self, metadata, content):
        """Format a document with metadata and content."""
        yaml_metadata = yaml.dump(metadata, default_flow_style=False)
        return f"""---
{yaml_metadata}
---

{content}
"""

    def create_document(self, title, doc_type, content="", **metadata_kwargs):
        """Create a new document with proper metadata and structure."""
        # Generate filename
        filename = self.generate_filename(title, doc_type)
        
        # Generate metadata
        metadata = self.create_metadata(title, doc_type, **metadata_kwargs)
        
        # Determine directory based on doc_type
        doc_dir = self.get_directory_for_type(doc_type)
        
        # Create full document content
        full_content = self.format_document(metadata, content)
        
        # Save document
        file_path = doc_dir / filename
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_text(full_content)
            
        return file_path

    def update_document(self, file_path, new_content=None, **metadata_updates):
        """Update an existing document's content and/or metadata."""
        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"Document not found: {file_path}")

        # Read existing document
        content = file_path.read_text()

        # Split metadata and content
        parts = content.split("---\n", 2)
        if len(parts) < 3:
            raise ValueError("Invalid document format")

        # Update metadata
        metadata = yaml.safe_load(parts[1])
        metadata.update(metadata_updates)
        metadata["last_updated"] = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

        # Create updated document
        updated_content = self.format_document(
            metadata,
            new_content if new_content is not None else parts[2]
        )

        # Save updated document
        file_path.write_text(updated_content)

    def list_documents(self, doc_type=None):
        """List all documents, optionally filtered by type."""
        docs = []
        for path in self.root_dir.rglob("*.md"):
            if path.name.startswith("."):
                continue
                
            try:
                content = path.read_text()
                parts = content.split("---\n", 2)
                if len(parts) >= 2:
                    metadata = yaml.safe_load(parts[1])
                    if doc_type is None or metadata.get("type") == doc_type:
                        docs.append({
                            "path": path,
                            "metadata": metadata
                        })
            except Exception as e:
                print(f"Error reading {path}: {e}")
                
        return docs

    def create_index(self):
        """Create an index of all documents."""
        docs = self.list_documents()
        index_content = "# Documentation Index\n\n"
        
        # Group by type
        docs_by_type = {}
        for doc in docs:
            doc_type = doc["metadata"].get("type", "uncategorized")
            if doc_type not in docs_by_type:
                docs_by_type[doc_type] = []
            docs_by_type[doc_type].append(doc)

        # Generate index content
        for doc_type, type_docs in docs_by_type.items():
            index_content += f"\n## {doc_type.title()}\n\n"
            for doc in sorted(type_docs, key=lambda x: x["metadata"]["title"]):
                metadata = doc["metadata"]
                rel_path = doc["path"].relative_to(self.root_dir)
                index_content += f"- [{metadata['title']}]({rel_path}) "
                index_content += f"(v{metadata['version']}, {metadata['status']})\n"

        # Save index
        index_path = self.root_dir / "INDEX.md"
        index_path.write_text(index_content)


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Documentation Management System")
    parser.add_argument("command", choices=["create", "update", "list", "index"])
    parser.add_argument("--title", help="Document title")
    parser.add_argument("--type", help="Document type")
    parser.add_argument("--content", help="Document content file")
    parser.add_argument("--file", help="Document file to update")
    
    args = parser.parse_args()
    
    doc_manager = DocManager()
    
    if args.command == "create":
        if not args.title or not args.type:
            parser.error("--title and --type are required for create")
            
        content = ""
        if args.content:
            with open(args.content) as f:
                content = f.read()
                
        doc_path = doc_manager.create_document(args.title, args.type, content)
        print(f"Created document: {doc_path}")
        
    elif args.command == "update":
        if not args.file:
            parser.error("--file is required for update")
            
        updates = {}
        if args.content:
            with open(args.content) as f:
                updates["content"] = f.read()
                
        doc_manager.update_document(args.file, **updates)
        print(f"Updated document: {args.file}")
        
    elif args.command == "list":
        docs = doc_manager.list_documents(args.type)
        for doc in docs:
            print(f"{doc['path']}: {doc['metadata']['title']} (v{doc['metadata']['version']})")
            
    elif args.command == "index":
        doc_manager.create_index()
        print("Created documentation index")


if __name__ == "__main__":
    main()
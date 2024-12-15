---
authors:
  - Matt Faherty
date: '2024-11-18'
status: draft
title: Documentation Standards Guide
type: guide
version: 1.0.0
---

# ICN Documentation Standards Guide

## Overview

Effective documentation is crucial for the success of the Inter-Cooperative Network (ICN). It ensures that developers, contributors, and users understand how to use, modify, and extend the system. This guide provides the standards and best practices that all contributors must follow when documenting code, APIs, and other technical details within the ICN project.

### Purpose
- **Consistency**: Maintain a consistent style and format across all documentation.
- **Clarity**: Ensure that all documentation is easy to understand and navigate for both technical and non-technical audiences.
- **Comprehensiveness**: Provide sufficient information without overwhelming the reader, covering both conceptual and practical aspects.

## 1. Documentation Types

### 1.1 Code Documentation
This type refers to inline comments, function headers, and detailed explanations within the source code.
- **Inline Comments**: Should be used to explain the purpose and flow of complex logic or non-obvious code segments.
- **Function Headers**: Include a description of the function, input parameters, output values, and any potential exceptions.

#### Example Function Header
```rust
/// Calculates the reputation score for a member.
///
/// # Arguments
///
/// * `did` - The Decentralized Identifier of the member.
///
/// # Returns
///
/// * `i32` - The computed reputation score.
///
/// # Errors
///
/// Will return an error if the member does not exist in the database.
fn calculate_reputation(did: &str) -> Result<i32, String> {
    // Function implementation here
}
```

### 1.2 API Documentation
All public-facing APIs must be documented to ensure developers can integrate with ICN effectively.
- **Endpoint Description**: Provide a summary of what the endpoint does.
- **Parameters**: List all parameters, including their types, requirements, and descriptions.
- **Response Structure**: Include examples of both success and error responses.
- **Authentication**: Specify if authentication is required and what type.

#### Example API Documentation
```markdown
### GET /api/v1/resources/{resource_id}

**Description**: Retrieves details for a specific resource.

**Parameters**:
- `resource_id` (path, required): The ID of the resource to retrieve.

**Response**:
- **200 OK**:
  ```json
  {
    "resource_id": "abc123",
    "type": "physical",
    "availability": "Available",
    "owner": "did:icn:123456"
  }
  ```
- **404 Not Found**: Resource not found.

**Authentication**: Requires Bearer token.
```

### 1.3 User Guides
User guides are written for non-technical users to understand the functionality of ICN services.
- **Step-by-Step Instructions**: Break down processes into easy-to-follow steps.
- **Visual Aids**: Where applicable, include diagrams, screenshots, or flowcharts.

### 1.4 Development Guides
Development guides are aimed at helping developers contribute to ICN.
- **Environment Setup**: How to configure a local development environment.
- **Coding Standards**: Details of language-specific standards that developers should adhere to.
- **Contribution Workflow**: Guide on creating pull requests, writing unit tests, and ensuring code quality.

## 2. Formatting Standards

### 2.1 Markdown Conventions
- **Headings**: Use heading levels consistently (e.g., `#` for main sections, `##` for subsections).
- **Code Blocks**: Use triple backticks for code blocks and specify the language where applicable.
- **Lists**: Use hyphens (`-`) for bullet points and numbers (`1.`) for ordered lists.
- **Links**: Always use descriptive text for links rather than raw URLs.

#### Example Markdown Structure
```markdown
# Introduction

This is a high-level overview of the ICN governance system.

## Features
- **Democratic Decision-Making**: All members participate in governance.
- **Transparency**: Decisions are publicly recorded in the audit log.
```

### 2.2 Naming Conventions
- **File Names**: Use lowercase letters with hyphens as separators (e.g., `resource-sharing-system.md`).
- **Variable Names**: Use `snake_case` for variables and function names in Rust, and `camelCase` for JavaScript variables.
- **Document Titles**: Should be concise, descriptive, and reflect the document’s content.

## 3. Best Practices

### 3.1 Write for Your Audience
- **Technical Readers**: For developers, provide detailed code explanations, examples, and technical diagrams.
- **Non-Technical Readers**: For general users, focus on how to achieve tasks without delving into implementation details.

### 3.2 Keep Documentation Up-to-Date
- Update relevant documentation whenever a code change affects functionality or API behavior.
- Use version numbers and change logs to track major updates in documents.

### 3.3 Use Active Voice
Write in the active voice to keep sentences clear and direct.

**Example**:
- **Passive**: "The resource is allocated by the system."
- **Active**: "The system allocates the resource."

### 3.4 Short Paragraphs and Bullet Points
Use short paragraphs, bullet points, and numbered lists to improve readability. Avoid large blocks of text.

## 4. Review Process

### 4.1 Peer Review
All new documentation should go through a peer review process before being merged. Reviewers should verify:
- **Clarity**: Is the information presented in a clear and understandable manner?
- **Accuracy**: Are the technical details correct?
- **Consistency**: Does the documentation conform to the standards outlined in this guide?

### 4.2 Automated Linting Tools
Use tools like `markdownlint` to check for common formatting issues in Markdown files.
- **Install with npm**: `npm install -g markdownlint-cli`
- **Run Lint Check**: `markdownlint *.md`

## 5. Tools and Resources

### 5.1 Diagram Tools
- **Mermaid**: Use for creating system diagrams and flowcharts in `.mermaid` format.
- **Mermaid**: Preferred for creating sequence diagrams and other technical visualizations within code.

### 5.2 Writing Aids
- **Grammarly**: Use for grammar and spelling checks.
- **Vale**: A command-line tool to enforce style rules and ensure consistency across documentation.

### 5.3 Markdown Editors
- **Typora**: A popular WYSIWYG editor for Markdown.
- **Visual Studio Code**: With extensions like `Markdown All in One` for a seamless writing experience.

## Appendix

### A. Additional Resources
- **ICN Contribution Guide**: [Contribution Guide](./contributing.md)
- **Rust API Guidelines**: [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/)
- **Markdown Guide**: [https://www.markdownguide.org/](https://www.markdownguide.org/)


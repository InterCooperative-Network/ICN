# Contributing to Internet of Cooperative Networks (ICN)

Thank you for your interest in contributing to the ICN project! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
  - [Development Environment](#development-environment)
  - [Project Structure](#project-structure)
- [Contribution Workflow](#contribution-workflow)
  - [Creating Issues](#creating-issues)
  - [Pull Requests](#pull-requests)
  - [Code Review Process](#code-review-process)
- [Development Guidelines](#development-guidelines)
  - [Code Style](#code-style)
  - [Documentation](#documentation)
  - [Testing](#testing)
  - [Creating New Modules](#creating-new-modules)
- [Release Process](#release-process)
- [Community](#community)

## Code of Conduct

We expect all contributors to adhere to our Code of Conduct. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

## Getting Started

### Development Environment

1. Fork the repository and clone it locally:
   ```bash
   git clone https://github.com/your-username/icn.git
   cd icn
   ```

2. Set up the development environment:
   ```bash
   # Copy environment template
   cp .env.template .env
   
   # Edit .env file with your configuration
   
   # Run setup script
   ./setup-dev.sh
   ```

3. Build the project:
   ```bash
   cargo build
   ```

### Project Structure

Refer to the [README.md](README.md) for an overview of the project structure.

## Contribution Workflow

### Creating Issues

Before starting work on a new feature or fix, please search for existing issues. If there's none that addresses your idea, create a new issue describing:

1. What you want to accomplish
2. Why it's valuable
3. How you're thinking of implementing it

For bugs, please include:
1. Steps to reproduce
2. Expected behavior
3. Actual behavior
4. Environment information (OS, Rust version, etc.)

### Pull Requests

1. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following our [development guidelines](#development-guidelines).

3. Commit your changes with a descriptive message:
   ```bash
   git commit -m "Add feature: your feature description"
   ```

4. Push your branch to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

5. Create a pull request to our `main` branch.

### Code Review Process

After submitting a PR, team members will review your code. Please be responsive to feedback and be prepared to make changes if necessary. All PRs must:

1. Pass all CI checks
2. Receive approvals from at least one team member
3. Resolve all conversations

## Development Guidelines

### Code Style

We follow Rust's standard code style guidelines:

1. Run `cargo fmt` before committing to ensure consistent formatting.
2. Run `cargo clippy` to catch common mistakes and improve code quality.
3. Use meaningful variable and function names.
4. Keep functions short and focused on a single responsibility.

### Documentation

All code should be well-documented:

1. Use doc comments (`///` or `//!`) for public APIs.
2. Include examples in documentation when appropriate.
3. Update README and other documentation when changing functionality.
4. Write clear commit messages that explain the "why" not just the "what".

### Testing

All code should be tested:

1. Write unit tests for new functions and methods.
2. Write integration tests for new features.
3. Ensure existing tests pass with your changes.
4. Aim for high test coverage, especially for critical components.

Run tests with:
```bash
cargo test
```

### Creating New Modules

When creating new modules or crates:

1. Use our template system:
   ```bash
   ./scripts/create_crate.sh module-name "Description of the module"
   ```

2. Follow the established patterns for error handling, configuration, and API design.

3. Include appropriate documentation and tests.

## Release Process

1. Releases are managed by the core team.
2. We follow semantic versioning (`MAJOR.MINOR.PATCH`).
3. Release notes are generated from PR descriptions and commits.

## Community

Join our community:
- [Discord/Matrix/Telegram] - Link to chat
- [Forum] - Link to forum
- [Mailing List] - Link to mailing list

Thank you for contributing to the Internet of Cooperative Networks! 
---
authors:
  - Matt Faherty
date: '2024-11-18'
status: draft
title: Development Setup Guide
type: guide
version: 1.0.0
---

# ICN Development Setup Guide

## Overview

Welcome to the development setup guide for the Inter-Cooperative Network (ICN). This guide will walk you through setting up your local development environment to contribute effectively to ICN. The goal is to ensure that all developers can start coding with a minimal setup process, ensuring consistency across environments.

### Prerequisites
- Familiarity with Rust, Docker, and Kubernetes.
- Basic understanding of Git and version control.
- Linux-based development environment (Ubuntu recommended).

## 1. System Requirements

### 1.1 Hardware Requirements
- **CPU**: 4 cores minimum (Intel i5/AMD Ryzen 3 or higher recommended).
- **RAM**: 8GB minimum (16GB recommended).
- **Storage**: 100GB SSD (ensure sufficient space for containers, dependencies, and builds).

### 1.2 Software Requirements
- **OS**: Linux-based distribution (Ubuntu 24.04 or similar).
- **Docker**: Version 20.x or higher.
- **Kubernetes (Minikube)**: Version 1.26 or higher.
- **Rust**: Latest stable version.
- **Node.js & npm**: Version 16.x or higher (for front-end integration).
- **Git**: Version 2.34 or higher.

## 2. Setting Up Your Development Environment

### 2.1 Install Rust
The ICN backend is primarily built using Rust. Ensure you have the latest stable version installed.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update stable
```

### 2.2 Install Docker
Docker is used for containerizing ICN services. Install Docker by running:

```bash
sudo apt-get update
sudo apt-get install -y docker.io
sudo usermod -aG docker $USER
```
> Note: You may need to log out and log back in for Docker permissions to apply.

### 2.3 Set Up Kubernetes (Minikube)
Minikube helps in running a local Kubernetes cluster for testing ICN services.

```bash
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube
minikube start --driver=docker
```

### 2.4 Install Node.js and npm (Front-End Development)
The ICN frontend is built using modern JavaScript frameworks that require Node.js and npm.

```bash
curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g npm
```

### 2.5 Clone the Repository
Use Git to clone the ICN repository to your local machine:

```bash
git clone https://github.com/your-repo/icn.git
cd icn
```

### 2.6 Install Project Dependencies
Navigate to the backend and frontend directories and install the necessary dependencies.

#### Backend Dependencies
```bash
cd backend
cargo build
```

#### Frontend Dependencies
```bash
cd ../frontend
npm install
```

### 2.7 Configure Environment Variables
The ICN project requires specific environment variables for local development.
Create a `.env` file in the root of the repository and configure the following:

```env
DATABASE_URL=postgresql://icn_user:password@localhost:5432/icn_db
NODE_ENV=development
API_KEY=your-api-key-here
```

> Note: Refer to `env.example` for a full list of environment variables.

## 3. Running the Development Environment

### 3.1 Start the Backend
To start the backend services, navigate to the backend directory and run:

```bash
cargo run
```

This command will start the ICN backend services locally, making them accessible at `http://localhost:8000` by default.

### 3.2 Start the Frontend
Navigate to the frontend directory and run:

```bash
npm start
```

The frontend will start on `http://localhost:3000` by default, and it will connect to the backend services running locally.

### 3.3 Deploying with Minikube
To test the full ICN deployment using Kubernetes, use Minikube to deploy both the backend and frontend.

```bash
minikube kubectl -- apply -f deployment/icn-deployment.yaml
```

> Note: Ensure your Docker images are built locally and tagged correctly before deploying with Minikube.

## 4. Debugging and Troubleshooting

### 4.1 Common Issues
- **Docker Permission Denied**: If you encounter permission errors with Docker, ensure you have added your user to the Docker group and restarted your terminal.
- **Kubernetes Pod CrashLoopBackOff**: This usually indicates a configuration or resource issue. Run `minikube kubectl -- get pods` to inspect the status and logs.
- **Missing Dependencies**: Run `cargo check` and `npm audit` to check for any missing dependencies.

### 4.2 Logging
- **Backend**: Logs are output to the console by default. Use `RUST_LOG=debug cargo run` to see more detailed logs.
- **Frontend**: Browser console logs and terminal output provide insights into frontend issues.

## 5. Testing Your Setup

### 5.1 Running Unit Tests
Ensure your setup is functioning correctly by running the unit tests included in the backend and frontend.

#### Backend Tests
```bash
cargo test
```

#### Frontend Tests
```bash
npm test
```

### 5.2 Integration Tests
Integration tests verify that the frontend and backend communicate correctly.
To run integration tests, use:

```bash
cargo test --features integration
```

## 6. Contributing to ICN

### 6.1 Development Workflow
- **Fork the repository** on GitHub and create a new branch for your feature or bugfix.
- **Write unit tests** for new code and ensure all existing tests pass before making a pull request.
- **Submit a pull request** with a detailed description of your changes.

### 6.2 Code Standards
- **Rust**: Follow the Rust community guidelines and use `cargo fmt` to format your code.
- **JavaScript**: Adhere to the ES6 standards and use `eslint` to check your code.

## Appendix

### A. Useful Commands
- **Start Minikube Dashboard**: `minikube dashboard`
- **Access Minikube Services**: `minikube service list`
- **Run Backend in Watch Mode**: `cargo watch -x run`

### B. Additional Resources
- **Rust Documentation**: [https://doc.rust-lang.org/](https://doc.rust-lang.org/)
- **Docker Documentation**: [https://docs.docker.com/](https://docs.docker.com/)
- **Node.js Documentation**: [https://nodejs.org/en/docs/](https://nodejs.org/en/docs/)


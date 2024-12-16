# Inter-Cooperative Network (ICN)

## Table of Contents
- [Project Title and Description](#project-title-and-description)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Key Features](#key-features)
- [Usage](#usage)
- [Contributing](#contributing)
- [Testing](#testing)
- [Documentation](#documentation)
- [License](#license)

## Project Title and Description
The Inter-Cooperative Network (ICN) is a decentralized platform designed to facilitate cooperation and resource sharing among various cooperatives. The project aims to provide a robust infrastructure for identity management, reputation tracking, governance, and resource allocation, enabling cooperatives to operate transparently and efficiently.

## Getting Started
To set up the development environment for ICN, follow these steps:

### System Requirements
- **Hardware**: Minimum 4 cores CPU, 8GB RAM, 100GB SSD.
- **Software**: Linux-based OS (Ubuntu recommended), Docker 20.x or higher, Kubernetes (Minikube) 1.26 or higher, Rust (latest stable), Node.js & npm 16.x or higher, Git 2.34 or higher.

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update stable
```

### Install Docker
```bash
sudo apt-get update
sudo apt-get install -y docker.io
sudo usermod -aG docker $USER
```
> Note: You may need to log out and log back in for Docker permissions to apply.

### Set up Kubernetes (Minikube)
```bash
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube
minikube start --driver=docker
```

### Install Node.js and npm
```bash
curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g npm
```

### Clone the repository
```bash
git clone https://github.com/your-repo/icn.git
cd icn
```

### Install project dependencies
#### Backend dependencies
```bash
cd backend
cargo build
```

#### Frontend dependencies
```bash
cd ../frontend
npm install
```

### Configure environment variables
Create a `.env` file in the root of the repository and configure the following:
```env
DATABASE_URL=postgresql://icn_user:password@localhost:5432/icn_db
NODE_ENV=development
API_KEY=your-api-key-here
```
> Note: Refer to `env.example` for a full list of environment variables.

### Start the backend
```bash
cargo run
```
> This command will start the ICN backend services locally, making them accessible at `http://localhost:8000` by default.

### Start the frontend
```bash
npm start
```
> The frontend will start on `http://localhost:3000` by default, and it will connect to the backend services running locally.

### Deploying with Minikube
```bash
minikube kubectl -- apply -f deployment/icn-deployment.yaml
```
> Note: Ensure your Docker images are built locally and tagged correctly before deploying with Minikube.

### Running unit tests
#### Backend tests
```bash
cargo test
```

#### Frontend tests
```bash
npm test
```

### Running integration tests
```bash
cargo test --features integration
```

For more detailed instructions, refer to the [Development Setup Guide](docs/development/setup/development-setup-guide.md).

## Project Structure
The ICN project is organized into several directories, each serving a specific purpose:

- **backend**: Contains the Rust code for the backend services.
- **frontend**: Contains the JavaScript code for the frontend application.
- **contracts**: Contains the smart contracts for governance and cooperative operations.
- **crates**: Contains various Rust crates used by the backend services.
- **docker**: Contains Dockerfiles and Docker Compose configurations for containerizing the services.
- **docs**: Contains the project documentation, including setup guides, contribution guides, and API documentation.

## Key Features
- **Decentralized Identity Management**: Secure and verifiable identities using DIDs.
- **Reputation System**: Track and manage reputation across cooperatives.
- **Governance**: Democratic decision-making through proposals and voting.
- **Resource Sharing**: Efficient allocation and management of resources.
- **Consensus Mechanism**: Proof of Cooperation (PoC) for transaction validation.
- **Telemetry and Logging**: Integrated metrics and logging for monitoring and debugging.

## Usage
To run the ICN project locally, follow the instructions in the [Getting Started](#getting-started) section. For detailed usage instructions, refer to the [User Guides](docs/user/guides/index.md).

## Contributing
We welcome contributions from the community! To get started, please read the [ICN Contribution Guide](docs/development/guides/contributing.md) for guidelines on how to contribute to the project.

## Testing
The ICN project uses a comprehensive testing strategy to ensure the quality and reliability of the codebase. For detailed information on the testing strategy and how to run tests, refer to the [ICN Testing Strategy Guide](docs/development/guides/test-strategy.md).

## Documentation
For detailed documentation, including the Development Setup Guide, ICN Contribution Guide, and other relevant documents, refer to the [Documentation Index](docs/INDEX.md).

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

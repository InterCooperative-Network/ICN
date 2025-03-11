# Inter-Cooperative Network (ICN)

The Inter-Cooperative Network (ICN) is a decentralized platform for secure federation communications and cooperative resource sharing. It implements a novel Proof of Cooperation consensus mechanism with democratic validator governance.

## Features

- **Secure Federation Communications**: Uses the Secure Datagram Protocol (SDP) for reliable and secure inter-federation messaging
- **Proof of Cooperation Consensus**: Democratic validator selection with reputation-based incentives
- **Zero-Knowledge Proofs**: Privacy-preserving resource verification and governance
- **Resource Federation**: Dynamic resource pooling and allocation across federations
- **Distributed Identity**: Self-sovereign identity management for federation members
- **Smart Governance**: Automated proposal execution and dispute resolution

## Project Status

This project is in active development. The following components are currently functional:

- ✅ Core Federation Management
- ✅ Cryptographic Operations
- ✅ Basic Consensus Engine
- ✅ CLI Interface
- 🚧 Network Layer (In Progress)
- 🚧 Zero-Knowledge Proofs (In Progress)
- 📝 Frontend (Planned)

## Prerequisites

- Rust 1.70.0 or later
- PostgreSQL 15.0 or later
- Node.js 18.0 or later (for frontend)
- Docker and Docker Compose

## Quick Start

We've simplified the setup process with two easy-to-use scripts:

### 1. Setup

```bash
# Clone the repository (if you haven't already)
git clone https://github.com/yourusername/ICN.git
cd ICN

# Run the setup script
./setup.sh
```

The setup script will:
- Check for required dependencies
- Set up your Rust environment
- Configure the database
- Build the backend components
- Install frontend dependencies

### 2. Run

```bash
# Start all services
./run.sh

# Or start specific components
./run.sh backend    # Start only backend
./run.sh frontend   # Start only frontend
./run.sh consensus  # Start only consensus engine
```

### 3. Other Useful Commands

```bash
# Check status of services
./run.sh status

# View logs
./run.sh logs
./run.sh logs backend  # View specific service logs

# Stop all services
./run.sh stop

# Run tests
./run.sh test

# Clean up environment
./run.sh clean

# Show help
./run.sh help
```

## Project Structure

```
icn/
├── crates/                    # Core Rust crates
│   ├── icn-cli/               # Command-line interface
│   ├── icn-consensus/         # Consensus implementation
│   ├── icn-crypto/            # Cryptographic operations
│   ├── icn-federation/        # Federation management
│   ├── icn-networking/        # P2P networking layer
│   ├── icn-types/             # Common data types
│   └── icn-zk/                # Zero-knowledge proofs
├── backend/                   # Backend server implementation
├── frontend/                  # Frontend web application
├── docs/                      # Documentation
├── config/                    # Configuration files
├── scripts/                   # Utility scripts
└── docker/                    # Docker configurations
```

## Development

### Configuration

The system can be configured through:
- Environment variables (see `.env.template` if available)
- Configuration files in `config/`
- CLI arguments

## Documentation

- [Architecture Overview](docs/architecture/README.md)
- [API Documentation](docs/api/README.md)
- [Federation Protocol](docs/federation/README.md)
- [Consensus Mechanism](docs/consensus/README.md)
- [Contributing Guide](CONTRIBUTING.md)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

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

- Docker and Docker Compose v2.0+
- Git
- Make (optional)

For local development without Docker:
- Rust 1.70.0 or later
- PostgreSQL 15.0 or later
- Node.js 18.0 or later
- Redis 7.0 or later

## Quick Start

### 1. Clone and Configure

```bash
# Clone the repository
git clone https://github.com/yourusername/ICN.git
cd ICN

# Copy and configure environment variables
cp .env.template .env
# Edit .env with your preferred settings
```

### 2. Launch with Docker (Recommended)

```bash
# Start all services
docker compose up -d

# Or start specific services
docker compose up -d postgres redis  # Start dependencies
docker compose up -d backend        # Start backend
docker compose up -d frontend       # Start frontend
```

### 3. Launch for Development

```bash
# Run the setup script
./setup.sh

# Start services individually
./scripts/dev/run_backend_dev.sh
./scripts/dev/run_frontend_dev.sh
./scripts/dev/run_consensus.sh

# Monitor services
./scripts/utils/monitor_icn.sh
./scripts/utils/check_icn_status.sh
```

### 4. Run Tests

```bash
# Run all tests
./scripts/test/test_federation.sh
./scripts/test/test_icn_cli.sh
```

## Project Structure

```
icn/
├── backend/                   # Backend server implementation
├── frontend/                  # Frontend web application
├── crates/                    # Core Rust crates
│   ├── icn-cli/              # Command-line interface
│   ├── icn-consensus/        # Consensus implementation
│   ├── icn-crypto/           # Cryptographic operations
│   ├── icn-federation/       # Federation management
│   ├── icn-networking/       # P2P networking layer
│   ├── icn-types/            # Common data types
│   └── icn-zk/               # Zero-knowledge proofs
├── docker/                    # Docker configurations
├── scripts/                   # Utility scripts
│   ├── dev/                  # Development scripts
│   ├── test/                 # Test scripts
│   └── utils/                # Utility scripts
├── config/                    # Configuration files
├── docs/                      # Documentation
└── templates/                 # Template files
```

## Development

### Configuration

The system can be configured through:
1. Environment variables (copy `.env.template` to `.env`)
2. Configuration files in `config/`
3. CLI arguments

### Development Workflow

1. Start dependencies:
```bash
docker compose up -d postgres redis
```

2. Run backend in development mode:
```bash
./scripts/dev/run_backend_dev.sh
```

3. Run frontend in development mode:
```bash
./scripts/dev/run_frontend_dev.sh
```

4. Monitor services:
```bash
./scripts/utils/monitor_icn.sh
```

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

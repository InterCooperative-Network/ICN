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

1. Clone the repository:

```bash
git clone https://github.com/yourusername/ICN.git
cd ICN
```

2. Set up the development environment:

```bash
# Copy environment configuration
cp .env.template .env

# Start development containers
docker-compose -f docker-compose.dev.yml up -d

# Initialize the database
./scripts/setup_db.sh

# Build and start the ICN system
./scripts/run_icn_dev.sh
```

3. Run the CLI:

```bash
cargo run -p icn-cli -- --help
```

## Project Structure

```
icn/
├── crates/                    # Core Rust crates
│   ├── icn-cli/              # Command-line interface
│   ├── icn-consensus/        # Consensus implementation
│   ├── icn-crypto/           # Cryptographic operations
│   ├── icn-federation/       # Federation management
│   ├── icn-networking/       # P2P networking layer
│   ├── icn-types/           # Common data types
│   └── icn-zk/              # Zero-knowledge proofs
├── docs/                     # Documentation
├── scripts/                  # Utility scripts
└── docker/                   # Docker configurations
```

## Development

### Building Components

Build a specific component:

```bash
cargo build -p <crate-name>
```

Run tests:

```bash
cargo test
```

### Configuration

The system can be configured through:

- Environment variables (see `.env.template`)
- Configuration files in `config/`
- CLI arguments

### Docker Development Environment

For development with Docker:

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop environment
docker-compose -f docker-compose.dev.yml down
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

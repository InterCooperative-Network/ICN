# Inter-Cooperative Network (ICN)

The Inter-Cooperative Network (ICN) is a platform for cooperative resource sharing and governance.

## Project Status

This project is currently in development. Some components may not be fully functional.

## Getting Started

### Prerequisites

- Rust/Cargo (latest stable version)
- Node.js/npm (for frontend)
- Docker and Docker Compose (for services)

### Setup

1. Clone the repository:

```bash
git clone https://github.com/yourusername/ICN.git
cd ICN
```

2. Set up the development environment:

```bash
cp .env.template .env
./setup-dev.sh
```

### Building and Running

#### CLI

The CLI component is currently the most stable part of the project. You can build and run it using:

```bash
./build-and-run.sh --help
```

This will show you the available commands. For example, to check the health of the ICN API:

```bash
./build-and-run.sh health
```

See the [CLI README](crates/icn-cli/README.md) for more details.

#### Backend

The backend is still under development and may not build completely due to dependency issues. However, you can try building it:

```bash
cd backend
cargo build
```

#### Frontend

The frontend is also under development:

```bash
cd frontend
npm install
npm start
```

## Project Structure

- `backend/`: Backend API server
- `crates/`: Rust crates for various components
  - `icn-cli/`: Command-line interface
  - `icn-types/`: Common data types
  - `icn-common/`: Common utilities
  - `icn-core/`: Core functionality
  - `icn-crypto/`: Cryptographic operations
  - `icn-identity/`: Identity management
  - `icn-resource/`: Resource allocation
  - `icn-consensus/`: Consensus mechanisms
  - `icn-runtime/`: Runtime execution
  - `icn-zk/`: Zero-knowledge proofs
- `frontend/`: Web interface
- `docker/`: Docker configurations
- `scripts/`: Utility scripts

## Development

### Building Individual Components

To build a specific component:

```bash
cargo build -p <crate-name>
```

For example, to build the CLI:

```bash
cargo build -p icn-cli
```

### Running Tests

```bash
cargo test
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to the project.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

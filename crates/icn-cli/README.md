# ICN Command Line Interface

The ICN CLI is a powerful command-line tool for interacting with the Inter-Cooperative Network (ICN) API. It provides comprehensive functionality for managing your identity, cooperatives, resources, governance, and network operations.

## Features

- **Complete API Coverage**: Access all ICN functionality from the command line
- **Enhanced Network Management**: Monitor network peers, diagnostics, and performance
- **Detailed Output Options**: Format output as tables, JSON, or CSV
- **Configurable Verbosity**: Control logging levels with granular verbosity flags
- **Smart Error Handling**: Clear, actionable error messages with context
- **Performance Monitoring**: Track bandwidth usage and peer statistics

## Installation

### From Source

To build the CLI from source, you need Rust installed. Then run:

```bash
cargo install --path crates/icn-cli
```

Or clone the repository and build:

```bash
git clone https://github.com/icn/icn-cli.git
cd icn-cli
cargo build --release
```

The binary will be available at `target/release/icn`.

### Using Pre-built Binaries

Download the appropriate binary for your platform from the [releases page](https://github.com/icn/icn/releases).

## Configuration

The CLI can be configured using:

- Command-line arguments
- Environment variables (prefixed with `ICN_`)
- Configuration file (defaults to `~/.icn/config.toml`)

### Sample Configuration File

```toml
[api]
url = "http://localhost:8082"
timeout = 30

[network]
preferred_peers = ["peer1", "peer2"]
connect_timeout = 15

[output]
default_format = "table" 
```

## CLI Usage

```
icn [OPTIONS] [COMMAND]
```

### Global Options

- `--api-url <api-url>`: ICN API URL (default: http://localhost:8082)
- `-v, --verbose`: Enable verbose output (can be used multiple times for increased verbosity)
- `-t, --timeout <timeout>`: Global timeout for requests in seconds (default: 30)
- `-c, --config <config>`: Path to config file
- `-h, --help`: Print help
- `-V, --version`: Print version

### Commands

#### Health

Check the health of the ICN API:

```bash
icn health
```

#### Identity Management

```bash
icn identity create                 # Create a new DID identity
icn identity list                   # List existing DIDs
icn identity show <did>             # Show details of a specific DID
```

#### Cooperative Management

```bash
icn cooperative join <coop-id>                     # Join a cooperative
icn cooperative list                               # List available cooperatives
icn cooperative create <name> <description>        # Create a new cooperative
```

#### Resource Management

```bash
icn resource register <resource-type> <capacity>   # Register a new resource
icn resource list                                  # List available resources
```

#### Governance

```bash
icn governance propose <title> <description>       # Create a new proposal
icn governance vote <proposal-id> <vote>           # Vote on a proposal (yes/no)
icn governance list                                # List active proposals
```

#### Network Management

The network commands provide comprehensive functionality to manage and monitor your ICN network connections:

```bash
# Check the current network status
icn network status [--detail]

# List network peers with filtering and sorting options
icn network peers [--filter <connected|disconnected|all>] [--sort <latency|id|address>] [--output <table|json|csv>]

# Connect to a peer
icn network connect <address> [--timeout <seconds>]

# Disconnect from a peer
icn network disconnect <peer-id>

# Ping a peer to test connectivity
icn network ping <peer-id> [--count <number>] [--interval <milliseconds>]

# Run network diagnostics
icn network diagnostics [--comprehensive] [--output-file <path>]
```

## Examples

### Basic Usage

Check the health of the ICN API:

```bash
icn health
```

Create a new DID identity:

```bash
icn identity create
```

List available cooperatives:

```bash
icn cooperative list
```

### Network Management Examples

Check network status with detailed information:

```bash
icn network status --detail
```

List all peers sorted by latency in JSON format:

```bash
icn network peers --filter all --sort latency --output json
```

Connect to a peer with a custom timeout:

```bash
icn network connect 12.34.56.78:9000 --timeout 60
```

Ping a peer with custom settings:

```bash
icn network ping QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG --count 10 --interval 500
```

Run comprehensive network diagnostics and save to a file:

```bash
icn network diagnostics --comprehensive --output-file network-report.txt
```

### Verbose Output Examples

Get more detailed output for debugging:

```bash
icn -v network status
icn -vv network diagnostics  # Even more detailed output
```

## Error Handling

The CLI provides clear error messages with context when operations fail. For even more detail, use the verbose flag:

```bash
icn -v network connect 12.34.56.78:9000
```

## Development

The CLI is built using the following libraries:

- [clap](https://crates.io/crates/clap): Command line argument parsing
- [reqwest](https://crates.io/crates/reqwest): HTTP client
- [tokio](https://crates.io/crates/tokio): Async runtime
- [serde](https://crates.io/crates/serde): Serialization/deserialization
- [thiserror](https://crates.io/crates/thiserror): Error handling
- [prettytable-rs](https://crates.io/crates/prettytable-rs): Formatted table output
- [indicatif](https://crates.io/crates/indicatif): Progress indicators

### Adding New Commands

To add a new command, update the `main.rs` file with the new command definition and implement the corresponding functionality in the `client.rs` file.

### Running Tests

```bash
cargo test -p icn-cli
```

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../CONTRIBUTING.md) for more information.

## License

This project is licensed under the [MIT License](LICENSE). 
# ICN Command Line Interface

The ICN CLI is a command-line tool for interacting with the Inter-Cooperative Network (ICN) API.

## Building

To build the CLI, run:

```bash
cargo build -p icn-cli
```

Or use the provided script:

```bash
./build-and-run.sh --help
```

## Usage

```
icn-cli [OPTIONS] [COMMAND]
```

### Options

- `--api-url <api-url>`: ICN API URL (default: http://localhost:8081)
- `-h, --help`: Print help
- `-V, --version`: Print version

### Commands

- `health`: Check the health of the ICN API
- `identity`: Identity management commands
  - `create`: Create a new DID identity
  - `list`: List existing DIDs
- `cooperative`: Cooperative management commands
  - `join <coop-id>`: Join a cooperative
  - `list`: List available cooperatives
- `resource`: Resource management commands
  - `register <resource-type> <capacity>`: Register a new resource
  - `list`: List available resources
- `governance`: Governance commands
  - `propose <title> <description>`: Create a new governance proposal
  - `vote <proposal-id> <vote>`: Vote on a proposal

## Examples

Check the health of the ICN API:

```bash
./build-and-run.sh health
```

Create a new DID identity:

```bash
./build-and-run.sh identity create
```

List available cooperatives:

```bash
./build-and-run.sh cooperative list
```

## Development

The CLI is built using the following libraries:

- [clap](https://crates.io/crates/clap): Command line argument parsing
- [reqwest](https://crates.io/crates/reqwest): HTTP client
- [tokio](https://crates.io/crates/tokio): Async runtime
- [serde](https://crates.io/crates/serde): Serialization/deserialization

To add a new command, update the `main.rs` file with the new command definition and implement the corresponding functionality in the `client.rs` file. 
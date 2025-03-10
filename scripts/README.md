# ICN System Management Scripts

This directory contains scripts to manage the ICN (Inter-Cooperative Network) system Docker containers. These scripts provide a convenient way to start, stop, and monitor the system with proper dependency management and error handling.

## Overview

The ICN system consists of several Docker containers that work together:

| Service       | Purpose                                  | Dependencies            |
|---------------|------------------------------------------|-------------------------|
| db            | PostgreSQL database                      | None                    |
| backend       | Main API server                          | db                      |
| bootstrap     | Bootstrap consensus node                 | db                      |
| validator1    | Validator node 1                         | bootstrap, db           |
| validator2    | Validator node 2                         | bootstrap, db           |
| frontend      | Web interface                            | backend                 |
| identity      | Identity management service              | backend                 |
| reputation    | Reputation management service            | backend                 |
| governance    | Governance service                       | backend                 |

## Available Scripts

### Main Scripts

- **start_icn.sh**: Starts the ICN system with Docker
- **stop_icn.sh**: Stops the ICN system
- **status_icn.sh**: Shows the status of all ICN components

### Utility Scripts

- **startup-utils.sh**: Contains utility functions used by the main scripts

## Usage

### Starting the ICN System

```bash
./scripts/start_icn.sh
```

This script:
1. Prompts for confirmation and network mode selection (development/production)
2. Validates the Docker environment
3. Stops any existing containers if necessary
4. Starts all services in the correct dependency order
5. Performs health checks on each service
6. Displays the system status

The script stays running to handle graceful shutdown when interrupted with Ctrl+C.

### Stopping the ICN System

```bash
./scripts/stop_icn.sh
```

This script:
1. Prompts for confirmation and shutdown mode selection
2. Stops services in the correct order (reverse dependency order)
3. Optionally removes containers and/or volumes

### Checking System Status

```bash
./scripts/status_icn.sh
```

This script:
1. Displays a summary table of all services
2. Shows the status, port, and health of each service
3. Optionally provides detailed information about selected services

## Configuration

The scripts use the following configuration files:

- **/.env**: Main environment variables
- **/docker/.env**: Docker-specific environment variables

Important environment variables:

- `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`: Database credentials
- `RUST_LOG`: Logging level for Rust components
- `ICN_NETWORK_MODE`: "development" or "production"

## Service Management Details

### Dependency Management

The scripts enforce proper dependency order when starting services. For example:
- The database must be running before starting the backend
- The bootstrap node must be running before starting validator nodes

### Health Checks

Each service has appropriate health checks:
- Database: Postgres ready check
- Backend/nodes: HTTP health endpoint
- Frontend: HTTP connection check

### Error Handling

The scripts include:
- Proper error detection and reporting
- Configurable retries for transient failures
- Graceful shutdown on failure

## Logging

All operations are logged to:
- Console (with color-coded status)
- Service-specific log files in `/logs/{service}/`
- Master log file with timestamp in `/logs/`

## Advanced Usage

### Running in Production

For production environments, select "Production" mode when prompted. This uses the standard `docker-compose.yml` file.

### Development with Hot Reloading

For development, select "Development" mode when prompted. This uses the `docker-compose.dev.yml` file, which includes:
- Mounted source directories for hot code reloading
- Development ports and settings
- Extended debugging options

### Customizing Service Configuration

To customize service configuration:
1. Create or modify the `.env` file in the project root
2. Add or modify environment variables as needed
3. Restart the system with `./scripts/start_icn.sh`

## Troubleshooting

### Common Issues

1. **Docker not running**
   - Error: "Docker is not running"
   - Solution: Start the Docker daemon

2. **Port conflicts**
   - Error: "Failed to start service"
   - Solution: Check for port conflicts with `netstat -tulpn`

3. **Database connection issues**
   - Error: "Failed to start backend"
   - Solution: Check database credentials in `.env`

### Getting Help

For more detailed information:
1. Check the logs in the `/logs/` directory
2. Use `docker logs <container-id>` for container-specific logs
3. Run `./scripts/status_icn.sh` for system status 
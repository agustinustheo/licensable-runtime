# Licensable Runtime Testing Guide

This guide provides comprehensive instructions for testing the Licensable Runtime project, which consists of a Substrate blockchain node with offchain workers and a NestJS API service for license management.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Docker Setup](#docker-setup)
5. [Testing Procedures](#testing-procedures)
6. [Local Development](#local-development)
7. [Troubleshooting](#troubleshooting)
8. [API Reference](#api-reference)

## Architecture Overview

The project uses a **microservices architecture** with three separate Docker containers:

```
┌─────────────────────────────────────────────────────────┐
│                   Docker Network                        │
│              (licensable-network)                       │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  PostgreSQL  │  │  NestJS API  │  │   Substrate  │ │
│  │   Database   │  │   Service    │  │     Node     │ │
│  │              │  │              │  │              │ │
│  │  Container:  │◄─┤  Container:  │  │  Container:  │ │
│  │  postgres    │  │  api         │  │  substrate   │ │
│  │              │  │              │  │              │ │
│  │  Port: 5432  │  │  Port: 3000  │  │  Port: 9933  │ │
│  │              │  │              │  │  Port: 9944  │ │
│  │              │  │              │  │  Port: 30333 │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Component Details

| Component | Container Name | Purpose | Ports |
|-----------|---------------|---------|-------|
| **PostgreSQL** | `licensable-postgres` | Database for license storage | 5432 |
| **NestJS API** | `licensable-api` | REST API for license management | 3000 |
| **Substrate Node** | `licensable-substrate` | Blockchain with offchain workers | 9933, 9944, 30333, 9615 |

**Key Features**:
- Each service runs in its own container (microservices)
- Independent scaling and lifecycle management
- Clear separation of concerns
- Single process per container (Docker best practice)
- Services communicate via Docker network

**API Endpoints**: `/health`, `/license`
**Database**: `license_db` (auto-created on startup)

## Prerequisites

### Required Software

| Software | Minimum Version | Verification Command |
|----------|----------------|---------------------|
| Docker | 20.10+ | `docker --version` |
| Docker Compose | 2.0+ | `docker-compose --version` |
| Node.js | 18+ | `node --version` |
| pnpm | 8+ | `pnpm --version` |
| Rust | 1.70+ | `rustc --version` |

### System Requirements

- **RAM**: Minimum 4GB available
- **Disk Space**: Minimum 10GB available
- **OS**: Linux, macOS, or Windows with WSL2

## Quick Start

The fastest way to get started:

```bash
# Using the convenience script
.maintain/start-local.sh

# Or using pnpm scripts
pnpm docker
```

This will:
1. Build all Docker images
2. Start PostgreSQL, Substrate node, and NestJS API
3. Initialize the database
4. Seed test data
5. Display service logs

## Docker Setup

### Building and Starting Services

#### Option 1: Using pnpm Scripts (Recommended)

```bash
# Complete setup with logs
pnpm docker

# Individual commands
pnpm docker:build     # Build Docker images
pnpm docker:up        # Start services in background
pnpm docker:logs      # View live logs
pnpm docker:status    # Check service status
pnpm docker:seed      # Seed database with test data
```

#### Option 2: Using Docker Compose Directly

```bash
# From project root
docker-compose -f .maintain/docker-compose.yml build
docker-compose -f .maintain/docker-compose.yml up -d
docker-compose -f .maintain/docker-compose.yml logs -f
```

#### Option 3: Using the Local Startup Script

```bash
# Make the script executable (first time only)
chmod +x .maintain/start-local.sh

# Run the script
.maintain/start-local.sh
```

### Service Management

```bash
# Check service status
pnpm docker:status

# Restart services
pnpm docker:restart

# Stop services (preserves data)
pnpm docker:down

# Clean stop (removes volumes and data)
pnpm docker:clean
```

## Testing Procedures

### 1. Verify Service Health

#### Check All Services

```bash
# Quick health check script
curl -f http://localhost:3000/health && echo "✓ API is healthy" || echo "✗ API is down"
curl -f http://localhost:9933 && echo "✓ Node is healthy" || echo "✗ Node is down"
```

#### Detailed Service Checks

```bash
# Substrate Node
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933

# NestJS API
curl http://localhost:3000/health

# PostgreSQL
docker exec licensable-postgres pg_isready -U postgres
```

### 2. Test License Validation

The seed script creates three test licenses:

| License Key | Status | Description |
|-------------|--------|-------------|
| `valid-license-key-12345` | Valid | Active, expires in 30 days |
| `expired-license-key-67890` | Expired | Expired 30 days ago |
| `inactive-license-key-11111` | Inactive | Valid date but deactivated |

#### Test Each License

```bash
# Valid license
curl "http://localhost:3000/license?key=valid-license-key-12345"
# Expected: {"valid":true,"message":"License is valid"}

# Expired license
curl "http://localhost:3000/license?key=expired-license-key-67890"
# Expected: {"valid":false,"message":"License has expired"}

# Inactive license
curl "http://localhost:3000/license?key=inactive-license-key-11111"
# Expected: {"valid":false,"message":"License is inactive"}

# Non-existent license
curl "http://localhost:3000/license?key=nonexistent-key"
# Expected: {"valid":false,"message":"License not found"}
```

### 3. Test Substrate RPC

```bash
# Get chain information
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain"}' \
  http://localhost:9933

# Get node version
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_version"}' \
  http://localhost:9933

# Get peer count
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers"}' \
  http://localhost:9933
```

### 4. Run Unit Tests

```bash
# Rust tests (run locally, not in Docker)
cargo test --all

# Or using pnpm
pnpm test

# Run with verbose output
cargo test --all -- --nocapture
```

## Local Development

### Running Without Docker

If you prefer to run services locally without Docker:

#### 1. Start PostgreSQL

```bash
# Using Docker for PostgreSQL only
docker run -d \
  --name local-postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=license_db \
  -p 5432:5432 \
  postgres:15-alpine
```

#### 2. Start the API Service

```bash
cd api-service

# Install dependencies
pnpm install

# Build the service
pnpm build

# Set environment variables
export DB_HOST=localhost
export DB_PORT=5432
export DB_USERNAME=postgres
export DB_PASSWORD=postgres
export DB_NAME=license_db
export PORT=3000

# Run the seed script
pnpm seed

# Start the API
pnpm start
```

#### 3. Start the Substrate Node

```bash
# Build the node
cargo build --release

# Run in development mode
./target/release/solochain-template-node --dev
```

### Development Workflow

1. **Make code changes**
2. **Rebuild the affected service**:
   ```bash
   # For API changes
   cd api-service && pnpm build

   # For Substrate changes
   cargo build --release
   ```
3. **Rebuild Docker image** (if using Docker):
   ```bash
   # Rebuild all services
   pnpm docker:build

   # Or rebuild specific service
   docker-compose -f .maintain/docker-compose.yml build api
   docker-compose -f .maintain/docker-compose.yml build substrate
   ```
4. **Restart services**:
   ```bash
   # Restart all services
   pnpm docker:restart

   # Or restart specific service
   docker-compose -f .maintain/docker-compose.yml restart api
   docker-compose -f .maintain/docker-compose.yml restart substrate
   ```

## Troubleshooting

### Common Issues and Solutions

#### Port Already in Use

```bash
# Find process using a port (e.g., 3000)
lsof -i :3000

# Kill the process
kill -9 <PID>

# Or change the port in docker-compose.yml
```

#### Database Connection Failed

```bash
# Check PostgreSQL status
docker ps | grep postgres

# Check PostgreSQL logs
docker logs licensable-postgres

# Verify connection manually
PGPASSWORD=postgres psql -h localhost -U postgres -d license_db
```

#### Build Failures

```bash
# Clean Docker cache
docker system prune -a

# Remove old volumes
docker volume prune

# Fresh rebuild
pnpm docker:clean && pnpm docker:build
```

#### Rosetta/Architecture Issues (Apple Silicon)

If you see `rosetta error: failed to open elf` in the logs:

```bash
# The Docker image was built for x86_64 but you're on ARM
# Solution 1: Rebuild the image natively
.maintain/start-local.sh --build

# Solution 2: Use Omni-Node instead
.maintain/start-local.sh --omni

# Check if you're on ARM
uname -m  # Should show "arm64" on Apple Silicon
```

**Why this happens**: The Substrate node binary in the Docker image was compiled for a different architecture (x86_64 vs ARM64). Rebuilding the image on your local machine will compile it for the correct architecture.

#### Seed Script Errors

```bash
# Check if database exists
docker exec licensable-postgres psql -U postgres -c "\l"

# Recreate database
docker exec licensable-postgres psql -U postgres -c "DROP DATABASE IF EXISTS license_db"
docker exec licensable-postgres psql -U postgres -c "CREATE DATABASE license_db"

# Re-run seed
pnpm docker:seed
```

### Debugging

#### Access Container Shell

```bash
# Access Substrate node container
docker exec -it licensable-substrate bash

# Access API container
docker exec -it licensable-api sh

# Access PostgreSQL container
docker exec -it licensable-postgres psql -U postgres -d license_db
```

#### View Detailed Logs

```bash
# All services
pnpm docker:logs

# Specific service
docker logs licensable-substrate --tail 50 -f
docker logs licensable-api --tail 50 -f
docker logs licensable-postgres --tail 50 -f

# View logs for all containers
docker-compose -f .maintain/docker-compose.yml logs -f
```

**Note**: The traditional setup no longer uses supervisor. Each service runs as a single process in its own container, making logs simpler to access.

## API Reference

### Health Check

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2024-11-30T12:00:00.000Z"
}
```

### License Validation

**Endpoint:** `GET /license`

**Query Parameters:**
- `key` (required): License key to validate

**Response Examples:**

Valid license:
```json
{
  "valid": true,
  "message": "License is valid"
}
```

Invalid/expired license:
```json
{
  "valid": false,
  "message": "License has expired"
}
```

Not found:
```json
{
  "valid": false,
  "message": "License not found"
}
```

### Substrate RPC Methods

Common RPC methods available at `http://localhost:9933`:

| Method | Description |
|--------|-------------|
| `system_health` | Get node health status |
| `system_chain` | Get chain name |
| `system_version` | Get node version |
| `system_peers` | Get connected peers |
| `chain_getBlock` | Get block by hash |
| `chain_getHeader` | Get block header |
| `state_getMetadata` | Get runtime metadata |

Example RPC call:
```bash
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "METHOD_NAME", "params": []}' \
  http://localhost:9933
```

## Additional Resources

- [Substrate Documentation](https://docs.substrate.io/)
- [NestJS Documentation](https://docs.nestjs.com/)
- [TypeORM Documentation](https://typeorm.io/)
- [Docker Documentation](https://docs.docker.com/)

## Support

For issues or questions:
1. Check the [Troubleshooting](#troubleshooting) section
2. Review logs with `pnpm docker:logs`
3. Open an issue on the project repository
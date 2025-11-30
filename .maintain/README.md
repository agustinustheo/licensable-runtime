# Maintenance Documentation

This directory contains all Docker-related files and maintenance documentation for the Licensable Runtime project.

## Directory Structure

```
.maintain/
├── Dockerfile                    # Substrate node only (microservice)
├── Dockerfile.api                # NestJS API only (microservice)
├── docker-compose.yml            # Multi-container setup (Substrate + API + PostgreSQL)
├── Dockerfile.omni-node          # Docker build file for Polkadot Omni-Node setup
├── docker-compose.omni-node.yml  # Docker Compose configuration for Omni-Node
├── parachain-spec-template.json  # Chain specification template for parachain
├── start-local.sh                # Convenience script for local development
├── DOCKER-ARCHITECTURE.md        # Detailed architecture documentation
├── .env.docker                   # Docker environment configuration
└── README.md                     # This file
```

## Architecture Overview

The project uses a **microservices architecture** with three separate containers:

```
┌─────────────────────────────────────────────────────────┐
│                   Docker Network                        │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  PostgreSQL  │  │  NestJS API  │  │   Substrate  │ │
│  │   Database   │  │   Service    │  │     Node     │ │
│  │              │  │              │  │              │ │
│  │  Port: 5432  │◄─┤  Port: 3000  │  │  Port: 9933  │ │
│  │              │  │              │  │  Port: 9944  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Containers

| Container | Purpose | Dockerfile | Image |
|-----------|---------|------------|-------|
| `licensable-postgres` | PostgreSQL database | Official image | `postgres:15-alpine` |
| `licensable-api` | NestJS API service | `Dockerfile.api` | `maintain-api:latest` |
| `licensable-substrate` | Substrate blockchain node | `Dockerfile` | `maintain-substrate:latest` |

## Quick Start Commands

### Using the Convenience Script (Recommended)

```bash
# Start with traditional node (uses existing images if available)
.maintain/start-local.sh

# Force rebuild and start
.maintain/start-local.sh --build

# Start with Polkadot Omni-Node
.maintain/start-local.sh --omni

# Clean start (remove all data)
.maintain/start-local.sh --clean

# Rebuild omni-node setup
.maintain/start-local.sh --omni --build

# Show help
.maintain/start-local.sh --help
```

### Using pnpm Scripts

```bash
# Traditional node setup
pnpm docker           # Build and start with logs
pnpm docker:build     # Build all Docker images
pnpm docker:up        # Start all services
pnpm docker:logs      # View logs
pnpm docker:status    # Check service status
pnpm docker:down      # Stop all services
pnpm docker:clean     # Stop and remove volumes
pnpm docker:restart   # Restart all services
pnpm docker:seed      # Seed the database with test data

# Polkadot Omni-Node setup
pnpm omni:docker           # Build and start omni-node
pnpm omni:docker:build     # Build omni-node image
pnpm omni:docker:up        # Start omni-node services
pnpm omni:docker:logs      # View omni-node logs
pnpm omni:docker:down      # Stop omni-node services
pnpm omni:docker:clean     # Clean omni-node data
```

## Docker Configuration Details

### Microservices Architecture

Unlike traditional monolithic setups, this project separates concerns into individual services:

#### 1. Substrate Node (`Dockerfile`)

**Purpose**: Build and run only the Substrate blockchain node

**Multi-stage Build**:
1. **Builder Stage**: Uses `paritytech/ci-unified` to compile Rust binary
2. **Runtime Stage**: Minimal Ubuntu image with only the node binary

**Key Features**:
- Single process per container (no supervisor)
- Runs as non-root user (`substrate`)
- Minimal dependencies (curl, ca-certificates)
- Health check on RPC endpoint (port 9933)
- Volume for blockchain data persistence

**Ports Exposed**:
- `30333` - P2P networking
- `9933` - RPC HTTP endpoint
- `9944` - RPC WebSocket endpoint
- `9615` - Prometheus metrics

#### 2. NestJS API Service (`Dockerfile.api`)

**Purpose**: Build and run only the NestJS license API service

**Multi-stage Build**:
1. **Builder Stage**: Builds TypeScript application with pnpm
2. **Runtime Stage**: Minimal Node.js Alpine image

**Key Features**:
- Waits for PostgreSQL before starting
- Auto-creates database if missing
- Runs as non-root user (`nestjs`)
- Health check on `/health` endpoint
- Includes postgresql-client for database operations

**Ports Exposed**:
- `3000` - REST API endpoint

#### 3. PostgreSQL Database

**Purpose**: Store license information

**Configuration**:
- Official `postgres:15-alpine` image
- Persistent volume for data
- Health check with `pg_isready`

**Ports Exposed**:
- `5432` - PostgreSQL server

### Omni-Node Setup (`Dockerfile.omni-node`)

The `Dockerfile.omni-node` is optimized for parachain deployment:

1. **Stage 1**: Builds the runtime WASM only
2. **Stage 2**: Builds the NestJS API
3. **Stage 3**: Uses pre-built `polkadot-omni-node` binary + supervisor

**Key Differences from Traditional**:
- Builds only runtime WASM (not full node)
- Uses official `polkadot-omni-node` binary
- Includes chain-spec generation
- Configured for parachain consensus
- Still uses supervisor (combined container)

### Docker Compose Configurations

#### Traditional Setup (`docker-compose.yml`)

Three-service microservices architecture:

```yaml
services:
  postgres:
    image: postgres:15-alpine
    container_name: licensable-postgres
    # Database configuration

  api:
    build:
      dockerfile: .maintain/Dockerfile.api
    container_name: licensable-api
    depends_on:
      postgres:
        condition: service_healthy
    # API configuration

  substrate:
    build:
      dockerfile: .maintain/Dockerfile
    container_name: licensable-substrate
    depends_on:
      - postgres
      - api
    # Substrate node configuration
```

**Service Startup Order**:
1. PostgreSQL starts first, waits for health check
2. API starts when PostgreSQL is healthy
3. Substrate node starts after both are running

#### Omni-Node Setup (`docker-compose.omni-node.yml`)

Two-service setup:
- PostgreSQL database
- Omni-node container (with API built-in using supervisor)

## Environment Variables

The `.env.docker` file contains default environment variables:

```env
NODE_ENV=production
DB_HOST=postgres
DB_PORT=5432
DB_USERNAME=postgres
DB_PASSWORD=postgres
DB_NAME=license_db
PORT=3000
```

## Container Management

### Accessing Container Shells

```bash
# Substrate node container
docker exec -it licensable-substrate bash

# API service container
docker exec -it licensable-api sh

# Omni-node container (if using omni setup)
docker exec -it licensable-omni-node bash

# PostgreSQL container
docker exec -it licensable-postgres psql -U postgres -d license_db
```

### Viewing Logs

```bash
# All services
docker-compose -f .maintain/docker-compose.yml logs -f

# Specific service
docker logs licensable-substrate -f
docker logs licensable-api -f
docker logs licensable-postgres -f

# Last 50 lines
docker logs licensable-substrate --tail 50

# For omni-node (uses supervisor)
docker exec -it licensable-omni-node tail -f /var/log/supervisor/substrate.log
docker exec -it licensable-omni-node tail -f /var/log/supervisor/api.log
```

### Service Health Checks

```bash
# Check all containers
docker-compose -f .maintain/docker-compose.yml ps

# Check container health status
docker ps --format "table {{.Names}}\t{{.Status}}"

# Test endpoints
curl http://localhost:9933                                      # Substrate RPC
curl http://localhost:3000/health                               # API health
curl "http://localhost:3000/license?key=valid-license-key-12345" # License validation
```

## Database Management

### Seeding the Database

```bash
# Using pnpm script (recommended)
pnpm docker:seed

# Or manually
docker exec -it licensable-api sh -c 'cd /app && node dist/seed.js'
```

This creates three test licenses:
- `valid-license-key-12345` - Valid, expires in 30 days
- `expired-license-key-67890` - Expired 30 days ago
- `inactive-license-key-11111` - Valid date but inactive

### Database Operations

```bash
# Connect to PostgreSQL
docker exec -it licensable-postgres psql -U postgres -d license_db

# Backup database
docker exec licensable-postgres pg_dump -U postgres license_db > backup.sql

# Restore database
docker exec -i licensable-postgres psql -U postgres license_db < backup.sql

# List databases
docker exec -it licensable-postgres psql -U postgres -c "\l"

# List tables
docker exec -it licensable-postgres psql -U postgres -d license_db -c "\dt"
```

## Maintenance Tasks

### Rebuilding After Code Changes

```bash
# Using the convenience script
.maintain/start-local.sh --build

# Or manually
docker-compose -f .maintain/docker-compose.yml build
docker-compose -f .maintain/docker-compose.yml up -d

# Rebuild specific service
docker-compose -f .maintain/docker-compose.yml build api
docker-compose -f .maintain/docker-compose.yml build substrate
docker-compose -f .maintain/docker-compose.yml up -d
```

### Updating Dependencies

```bash
# For Substrate node
cargo update
.maintain/start-local.sh --build

# For API service
cd api-service && pnpm update
docker-compose -f .maintain/docker-compose.yml build api
```

### Clean Rebuild

```bash
# Stop and remove everything
pnpm docker:clean

# Rebuild from scratch
.maintain/start-local.sh --build --clean

# Or manually
docker-compose -f .maintain/docker-compose.yml down -v
docker-compose -f .maintain/docker-compose.yml build --no-cache
docker-compose -f .maintain/docker-compose.yml up -d
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Port Conflicts

**Problem**: `Bind for 0.0.0.0:XXXX failed: port is already allocated`

**Solution**:
```bash
# Stop all containers from both setups
docker-compose -f .maintain/docker-compose.yml down
docker-compose -f .maintain/docker-compose.omni-node.yml down

# Or modify port mappings in docker-compose.yml
ports:
  - "19933:9933"  # Change external port
```

#### 2. Database Connection Failed

**Problem**: API can't connect to PostgreSQL

**Solutions**:
```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Check PostgreSQL health
docker exec licensable-postgres pg_isready -U postgres

# Check API logs
docker logs licensable-api

# Verify network connectivity
docker network inspect maintain_licensable-network
```

#### 3. Substrate Node Not Starting

**Problem**: Node crashes on startup or doesn't respond

**Solutions**:
```bash
# Check logs
docker logs licensable-substrate

# For architecture issues (Apple Silicon)
.maintain/start-local.sh --build  # Rebuild for ARM64

# Check if binary is compatible
docker exec -it licensable-substrate /usr/local/bin/solochain-template-node --version
```

#### 4. Rosetta/Architecture Issues (Apple Silicon)

**Problem**: `rosetta error: failed to open elf`

**Solutions**:
```bash
# Rebuild the image natively for ARM64
.maintain/start-local.sh --build

# Or try Omni-Node
.maintain/start-local.sh --omni

# Check your architecture
uname -m  # Should show "arm64" on Apple Silicon
```

#### 5. Build Failures

**Problem**: Docker build fails

**Solutions**:
```bash
# Clear Docker cache
docker system prune -a

# Remove old images
docker images | grep maintain | awk '{print $3}' | xargs docker rmi -f

# Rebuild
.maintain/start-local.sh --build --clean
```

#### 6. Image Not Found

**Problem**: Script can't find Docker images

**Solutions**:
```bash
# List available images
docker images | grep maintain

# Rebuild missing images
.maintain/start-local.sh --build
```

## Chain Specification Template

The `parachain-spec-template.json` contains the base configuration for the parachain:

- **Para ID**: 2000
- **Token Symbol**: LIC
- **Token Decimals**: 12
- **Relay Chain**: rococo-local
- **Initial Accounts**: Alice and Bob with balances
- **License Key**: Configured in genesis

## Performance Optimization

### Resource Limits

Add resource limits to docker-compose.yml for production:

```yaml
services:
  substrate:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

### Build Cache

Optimize build times using Docker BuildKit:

```bash
export DOCKER_BUILDKIT=1
docker-compose -f .maintain/docker-compose.yml build
```

## Security Considerations

### Production Deployment Checklist

- [ ] Remove `--dev` flag from Substrate node command
- [ ] Use secure passwords for PostgreSQL
- [ ] Implement SSL/TLS with reverse proxy
- [ ] Restrict RPC methods (remove `--rpc-methods unsafe`)
- [ ] Use specific image tags instead of `latest`
- [ ] Run containers as non-root user (already configured)
- [ ] Set up proper network isolation
- [ ] Enable Docker content trust
- [ ] Implement secrets management
- [ ] Set up monitoring and logging

### Environment Variables

Never commit sensitive data. Use Docker secrets:

```yaml
secrets:
  db_password:
    external: true

services:
  postgres:
    secrets:
      - db_password
```

## Migration Guides

### From Monolithic to Microservices

The project was refactored from a monolithic container (using supervisor) to a microservices architecture:

**Old Architecture** (Before):
- Single container running both Substrate node and API
- Supervisor managing processes
- Container name: `licensable-runtime`

**New Architecture** (After):
- Three separate containers
- No supervisor needed (except omni-node)
- Container names: `licensable-substrate`, `licensable-api`, `licensable-postgres`

**Migration Steps**:
1. Stop old setup: `docker stop licensable-runtime && docker rm licensable-runtime`
2. Rebuild with new architecture: `.maintain/start-local.sh --build`
3. Update any scripts using old container names

### From Traditional to Omni-Node

1. Build runtime WASM: `pnpm omni:build`
2. Start omni-node setup: `.maintain/start-local.sh --omni`
3. Generate chain spec with Para ID
4. Update environment variables if needed

## File Locations Reference

- **Substrate Dockerfile**: `.maintain/Dockerfile`
- **API Dockerfile**: `.maintain/Dockerfile.api`
- **Omni-Node Dockerfile**: `.maintain/Dockerfile.omni-node`
- **Traditional Compose**: `.maintain/docker-compose.yml`
- **Omni-Node Compose**: `.maintain/docker-compose.omni-node.yml`
- **Environment Config**: `.maintain/.env.docker`
- **Chain Spec Template**: `.maintain/parachain-spec-template.json`
- **Architecture Docs**: `.maintain/DOCKER-ARCHITECTURE.md`
- **Startup Script**: `.maintain/start-local.sh`
- **Main Documentation**: `../README.md`
- **Testing Guide**: `../GUIDE.md`

## Additional Resources

- [DOCKER-ARCHITECTURE.md](./DOCKER-ARCHITECTURE.md) - Detailed architecture documentation
- [GUIDE.md](../GUIDE.md) - Comprehensive testing guide
- [README.md](../README.md) - Main project documentation

## Support

For issues:
1. Check container logs: `docker logs <container-name>`
2. Verify service health: `docker-compose ps`
3. Review environment variables: `docker exec <container> env`
4. Check network connectivity: `docker network inspect maintain_licensable-network`
5. Consult troubleshooting guide above
6. Review [GUIDE.md](../GUIDE.md) for testing procedures

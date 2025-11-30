# Docker Architecture

This document describes the Docker setup for the Licensable Runtime project.

## Overview

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

## Components

### 1. PostgreSQL Database (`postgres`)
- **Image**: `postgres:15-alpine`
- **Container**: `licensable-postgres`
- **Port**: 5432
- **Purpose**: Stores license information
- **Volume**: `postgres-data`

### 2. NestJS API Service (`api`)
- **Dockerfile**: `.maintain/Dockerfile.api`
- **Image**: `maintain-api:latest`
- **Container**: `licensable-api`
- **Port**: 3000
- **Purpose**: REST API for license management
- **Dependencies**: Waits for PostgreSQL to be healthy
- **Features**:
  - Automatic database initialization
  - Health check endpoint
  - Built with multi-stage Docker build

### 3. Substrate Node (`substrate`)
- **Dockerfile**: `.maintain/Dockerfile`
- **Image**: `maintain-substrate:latest`
- **Container**: `licensable-substrate`
- **Ports**:
  - 30333 (P2P)
  - 9933 (RPC HTTP)
  - 9944 (RPC WebSocket)
  - 9615 (Prometheus metrics)
- **Purpose**: Blockchain node with offchain workers
- **Volume**: `substrate-data`
- **Dependencies**: Starts after API is running

## Dockerfiles

### `.maintain/Dockerfile` (Substrate Node)
**Purpose**: Build and run only the Substrate blockchain node

**Stages**:
1. **Builder**: Uses `paritytech/ci-unified` to compile Rust binary
2. **Runtime**: Minimal Ubuntu image with only the node binary

**Key Features**:
- Clean separation of build and runtime
- Runs as non-root user (`substrate`)
- Minimal dependencies (only curl and ca-certificates)
- Health check on RPC endpoint

### `.maintain/Dockerfile.api` (NestJS API)
**Purpose**: Build and run only the NestJS API service

**Stages**:
1. **Builder**: Builds TypeScript application with pnpm
2. **Runtime**: Minimal Node.js Alpine image

**Key Features**:
- Waits for PostgreSQL before starting
- Auto-creates database if missing
- Runs as non-root user (`nestjs`)
- Health check on `/health` endpoint
- Includes postgresql-client for database operations

## Docker Compose Setup

### Service Startup Order
1. **postgres** - Starts first, waits for health check
2. **api** - Starts when postgres is healthy
3. **substrate** - Starts after both postgres and api

### Networking
- All services connected via `licensable-network` bridge network
- Services can communicate using container names as hostnames
- External ports exposed for development access

### Volumes
- `postgres-data`: Persistent database storage
- `substrate-data`: Persistent blockchain data

## Usage

### Build and Start All Services
```bash
# Using the convenience script
.maintain/start-local.sh

# Or using Docker Compose directly
docker-compose -f .maintain/docker-compose.yml up -d
```

### Build Individual Services
```bash
# Build only Substrate node
docker-compose -f .maintain/docker-compose.yml build substrate

# Build only API
docker-compose -f .maintain/docker-compose.yml build api

# Build all services
docker-compose -f .maintain/docker-compose.yml build
```

### Access Individual Services
```bash
# Check Substrate node logs
docker logs licensable-substrate

# Check API logs
docker logs licensable-api

# Check PostgreSQL logs
docker logs licensable-postgres

# Execute commands in API container
docker exec -it licensable-api sh

# Execute commands in Substrate container
docker exec -it licensable-substrate bash
```

### Seed Database
```bash
# Using pnpm script
pnpm docker:seed

# Or directly
docker exec -it licensable-api sh -c 'cd /app && node dist/seed.js'
```

## Benefits of This Architecture

### 1. **Separation of Concerns**
- Each service has its own Dockerfile
- Clear responsibilities for each component
- Easier to maintain and debug

### 2. **Independent Scaling**
- Can scale API independently from Substrate node
- Database can be replaced with external service easily

### 3. **Development Flexibility**
- Can run individual services for testing
- Easy to mock or replace components
- Better resource management

### 4. **Production Ready**
- Each container runs single process
- Proper health checks
- Non-root users for security
- Minimal attack surface

### 5. **Easier Debugging**
- Clear separation of logs
- Can restart individual services
- Simpler troubleshooting

## Comparison with Previous Setup

### Before (Monolithic)
- Single container with supervisor running both services
- Harder to debug which service is failing
- Tightly coupled components
- Larger container image

### After (Microservices)
- Three separate containers
- Clear service boundaries
- Independent lifecycle management
- Smaller, focused container images

## Container Details

### Image Sizes (Approximate)
- `postgres:15-alpine`: ~380 MB
- `maintain-api:latest`: ~200 MB (after build)
- `maintain-substrate:latest`: ~400 MB (after build)

### Resource Requirements
- **PostgreSQL**: ~50 MB RAM (idle)
- **NestJS API**: ~100 MB RAM
- **Substrate Node**: ~500 MB RAM (varies with chain activity)

## Troubleshooting

### API Can't Connect to Database
```bash
# Check if postgres is healthy
docker-compose -f .maintain/docker-compose.yml ps postgres

# Check API logs
docker logs licensable-api
```

### Substrate Node Not Starting
```bash
# Check logs for errors
docker logs licensable-substrate

# Rebuild for correct architecture
.maintain/start-local.sh --build
```

### Port Conflicts
```bash
# Stop all services
docker-compose -f .maintain/docker-compose.yml down

# Remove containers from both setups
docker-compose -f .maintain/docker-compose.yml down
docker-compose -f .maintain/docker-compose.omni-node.yml down
```

## Future Enhancements

Potential improvements to consider:

1. **Add Redis** for caching license validation results
2. **Add Nginx** as reverse proxy for API
3. **Separate dev/prod** compose files
4. **Add monitoring** with Prometheus and Grafana
5. **Multi-stage healthchecks** with dependency waiting
6. **Environment-specific** configuration files

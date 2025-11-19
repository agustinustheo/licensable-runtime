# Maintenance Documentation

This directory contains all Docker-related files and maintenance documentation for the Licensable Runtime project.

## Directory Structure

```
.maintain/
├── Dockerfile                    # Main Docker build file for traditional node
├── docker-compose.yml            # Docker Compose configuration for traditional setup
├── Dockerfile.omni-node          # Docker build file for Polkadot Omni-Node setup
├── docker-compose.omni-node.yml  # Docker Compose configuration for Omni-Node
├── parachain-spec-template.json  # Chain specification template for parachain
└── .env.docker                   # Docker environment configuration
```

## Quick Start Commands

### Traditional Node Setup

```bash
# Build and start everything with Docker
pnpm docker

# Or use individual commands:
pnpm docker:build     # Build the Docker images
pnpm docker:up        # Start all services
pnpm docker:logs      # View logs
pnpm docker:status    # Check service status
pnpm docker:down      # Stop all services
pnpm docker:clean     # Stop and remove volumes
pnpm docker:restart   # Restart all services
pnpm docker:seed      # Seed the database with test data
```

### Polkadot Omni-Node Setup

```bash
# Build and run with omni-node (recommended for parachains)
pnpm omni:docker

# Or use individual commands:
pnpm omni:docker:build   # Build the Docker image
pnpm omni:docker:up      # Start services
pnpm omni:docker:logs    # View logs
```

## Docker Configuration Details

### Traditional Node (Dockerfile)

The main `Dockerfile` creates a multi-stage build:

1. **Stage 1 - Rust Builder**: Builds the Substrate node
2. **Stage 2 - Node.js Builder**: Builds the NestJS API
3. **Stage 3 - Runtime**: Combines both binaries with Supervisor

**Key Features:**
- Uses Ubuntu 22.04 as base runtime image
- Installs Supervisor for process management
- Runs both substrate node and API service concurrently
- Non-root user setup (`substrate` user)
- Health check endpoint on port 3000

### Omni-Node Setup (Dockerfile.omni-node)

The `Dockerfile.omni-node` is optimized for parachain deployment:

1. **Stage 1**: Builds the runtime WASM only
2. **Stage 2**: Builds the NestJS API
3. **Stage 3**: Uses pre-built `polkadot-omni-node` binary

**Key Differences from Traditional:**
- Builds only runtime WASM (not full node)
- Uses official `polkadot-omni-node` binary
- Includes chain-spec generation
- Configured for parachain consensus

### Docker Compose Configurations

#### Traditional Setup (`docker-compose.yml`)

```yaml
services:
  postgres:
    image: postgres:14
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: license_db
    volumes:
      - postgres_data:/var/lib/postgresql/data

  licensable-runtime:
    build:
      context: ..
      dockerfile: .maintain/Dockerfile
    ports:
      - "9933:9933"  # RPC HTTP
      - "9944:9944"  # RPC WebSocket
      - "30333:30333" # P2P
      - "3000:3000"  # NestJS API
    depends_on:
      - postgres
    environment:
      - NODE_ENV=production
      - DB_HOST=postgres
      # ... other env vars
```

#### Omni-Node Setup (`docker-compose.omni-node.yml`)

Similar structure but uses:
- `Dockerfile.omni-node` for building
- Additional volume for chain spec
- Parachain-specific environment variables

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

## Supervisor Configuration

Both Docker setups use Supervisor to manage processes. The configuration is embedded in the Dockerfiles:

```ini
[program:substrate]
command=/usr/local/bin/solochain-template-node --dev --rpc-external --rpc-cors all
autostart=true
autorestart=true
stderr_logfile=/var/log/supervisor/substrate.err.log
stdout_logfile=/var/log/supervisor/substrate.log

[program:api]
command=node /home/substrate/api/dist/main.js
autostart=true
autorestart=true
environment=NODE_ENV=production,DB_HOST=postgres,...
stderr_logfile=/var/log/supervisor/api.err.log
stdout_logfile=/var/log/supervisor/api.log
```

## Chain Specification Template

The `parachain-spec-template.json` contains the base configuration for the parachain:

- **Para ID**: 2000
- **Token Symbol**: LIC
- **Token Decimals**: 12
- **Relay Chain**: rococo-local
- **Initial Accounts**: Alice and Bob with balances
- **License Key**: Configured in genesis

## Maintenance Tasks

### Rebuilding After Code Changes

```bash
# For traditional node
pnpm docker:build && pnpm docker:restart

# For omni-node
pnpm omni:docker:build && pnpm omni:docker:up
```

### Accessing Container Shells

```bash
# Traditional node container
docker exec -it licensable-runtime bash

# Omni-node container
docker exec -it licensable-omni-node bash

# PostgreSQL container
docker exec -it licensable-postgres psql -U postgres
```

### Viewing Logs

```bash
# All services
docker-compose -f .maintain/docker-compose.yml logs -f

# Specific service
docker-compose -f .maintain/docker-compose.yml logs -f licensable-runtime

# Supervisor logs inside container
docker exec -it licensable-runtime tail -f /var/log/supervisor/substrate.log
docker exec -it licensable-runtime tail -f /var/log/supervisor/api.log
```

### Database Management

```bash
# Connect to PostgreSQL
docker exec -it licensable-postgres psql -U postgres -d license_db

# Backup database
docker exec licensable-postgres pg_dump -U postgres license_db > backup.sql

# Restore database
docker exec -i licensable-postgres psql -U postgres license_db < backup.sql

# Seed database with test data
pnpm docker:seed
```

### Health Monitoring

```bash
# Check container health
docker ps --format "table {{.Names}}\t{{.Status}}"

# Check services inside container
docker exec -it licensable-runtime supervisorctl status

# Test endpoints
curl http://localhost:9933  # Substrate RPC
curl http://localhost:3000/health  # API health
curl "http://localhost:3000/license?key=valid-license-key-12345"  # License validation
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Port Conflicts

**Problem**: Ports already in use
**Solution**: Modify port mappings in docker-compose.yml:
```yaml
ports:
  - "19933:9933"  # Change external port
  - "19944:9944"
  - "13000:3000"
```

#### 2. Database Connection Failed

**Problem**: API can't connect to PostgreSQL
**Solutions**:
1. Check PostgreSQL is running: `docker ps | grep postgres`
2. Verify network connectivity: `docker network ls`
3. Check environment variables in container: `docker exec licensable-runtime env | grep DB_`

#### 3. Substrate Node Not Starting

**Problem**: Node crashes on startup
**Solutions**:
1. Check logs: `docker exec -it licensable-runtime tail -f /var/log/supervisor/substrate.log`
2. Verify chain spec is valid (for omni-node)
3. Ensure WASM runtime is built correctly

#### 4. Build Failures

**Problem**: Docker build fails
**Solutions**:
1. Clear Docker cache: `docker system prune -a`
2. Update Rust toolchain: Check rust-toolchain.toml
3. Verify all dependencies in Cargo.toml

#### 5. Supervisor Issues

**Problem**: Services not managed properly
**Solutions**:
1. Check supervisor status: `docker exec -it licensable-runtime supervisorctl status`
2. Restart specific service: `docker exec -it licensable-runtime supervisorctl restart substrate`
3. Reload configuration: `docker exec -it licensable-runtime supervisorctl reload`

## Performance Optimization

### Resource Limits

Add resource limits to docker-compose.yml for production:

```yaml
services:
  licensable-runtime:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

### Volume Optimization

Use named volumes for better performance:

```yaml
volumes:
  substrate_data:
    driver: local
  api_data:
    driver: local
  postgres_data:
    driver: local
```

### Build Cache

Optimize build times using Docker BuildKit:

```bash
export DOCKER_BUILDKIT=1
docker-compose build
```

## Security Considerations

### Production Deployment

1. **Remove --dev flag** from substrate node command
2. **Use secure passwords** for PostgreSQL
3. **Implement SSL/TLS** with reverse proxy
4. **Restrict RPC methods** (remove `--rpc-methods unsafe`)
5. **Use specific image tags** instead of `latest`
6. **Run containers as non-root user** (already configured)

### Environment Variables

Never commit sensitive data. Use Docker secrets or external secret management:

```yaml
secrets:
  db_password:
    external: true

services:
  postgres:
    secrets:
      - db_password
```

### Network Isolation

Create custom networks for service isolation:

```yaml
networks:
  backend:
    driver: bridge
    internal: true
  frontend:
    driver: bridge
```

## Backup and Recovery

### Automated Backups

Create a backup service in docker-compose:

```yaml
services:
  backup:
    image: postgres:14
    command: |
      sh -c 'while true; do
        pg_dump -h postgres -U postgres license_db > /backups/backup_$$(date +%Y%m%d_%H%M%S).sql
        sleep 86400
      done'
    volumes:
      - ./backups:/backups
    depends_on:
      - postgres
```

### Manual Recovery Process

1. Stop services: `pnpm docker:down`
2. Restore database: `docker-compose run --rm postgres psql -U postgres -d license_db < backup.sql`
3. Start services: `pnpm docker:up`

## Monitoring and Logging

### Log Aggregation

For production, consider using log aggregation:

```yaml
services:
  licensable-runtime:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Metrics Collection

Add Prometheus metrics:

```yaml
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Docker Build
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build Docker image
        run: |
          docker build -f .maintain/Dockerfile -t licensable-runtime:${{ github.sha }} .
      - name: Push to registry
        run: |
          docker push licensable-runtime:${{ github.sha }}
```

## Migration Guides

### From Traditional to Omni-Node

1. Build runtime WASM: `pnpm omni:build`
2. Update docker-compose to use omni-node setup
3. Generate chain spec with Para ID
4. Update environment variables
5. Deploy with `pnpm omni:docker`

### Version Upgrades

1. Update Rust toolchain in rust-toolchain.toml
2. Update Node.js version in Dockerfile
3. Test locally: `pnpm docker:build`
4. Deploy: `pnpm docker:up`

## File Locations Reference

- **Dockerfiles**: `.maintain/Dockerfile`, `.maintain/Dockerfile.omni-node`
- **Docker Compose**: `.maintain/docker-compose.yml`, `.maintain/docker-compose.omni-node.yml`
- **Environment Config**: `.maintain/.env.docker`
- **Chain Spec Template**: `.maintain/parachain-spec-template.json`
- **Main Documentation**: `../README.md`

## Support

For detailed documentation on specific components, refer to the main [README.md](../README.md).

For issues:
1. Check container logs
2. Verify service health
3. Review environment variables
4. Check network connectivity
5. Consult troubleshooting guide above
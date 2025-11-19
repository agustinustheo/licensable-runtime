# Docker Setup for Licensable Runtime

This Docker setup runs both the Substrate node and NestJS licensing API together in a single container, with PostgreSQL as the database.

## Architecture

The Docker setup includes:
- **Substrate Node**: Running on ports 9933 (RPC), 9944 (WebSocket), 30333 (P2P)
- **NestJS API**: Running on port 3000 for license validation
- **PostgreSQL**: Database for storing license information
- **Supervisor**: Process manager to run both services concurrently

## Prerequisites

- Docker Engine 20.10 or higher
- Docker Compose 2.0 or higher
- At least 4GB of available RAM
- 10GB of available disk space

## Quick Start

### 1. Build and Run with npm/pnpm scripts

```bash
# Build and start all services (recommended)
pnpm docker

# Or use individual commands:
pnpm docker:build     # Build the Docker images
pnpm docker:up        # Start all services
pnpm docker:logs      # View logs
pnpm docker:status    # Check service status
pnpm docker:down      # Stop all services
pnpm docker:clean     # Stop and remove volumes (clean slate)
```

### Alternative: Using Docker Compose directly

```bash
# From project root
docker-compose -f .maintain/docker-compose.yml up -d
docker-compose -f .maintain/docker-compose.yml logs -f
docker-compose -f .maintain/docker-compose.yml down
```

### 2. Verify Services

After starting, verify that all services are running:

```bash
# Check Substrate node
curl http://localhost:9933

# Check NestJS API health
curl http://localhost:3000/health

# Check license validation endpoint
curl "http://localhost:3000/license?key=valid-license-key-12345"
```

## Service URLs

- **Substrate RPC HTTP**: http://localhost:9933
- **Substrate RPC WebSocket**: ws://localhost:9944
- **NestJS API**: http://localhost:3000
- **PostgreSQL**: localhost:5432 (database: license_db)

## Database Initialization

### Seed the Database

Run the seed script to create sample licenses:

```bash
# Using npm/pnpm script
pnpm docker:seed

# Or manually connect to the container
docker exec -it licensable-runtime bash
cd /home/substrate/api
DB_HOST=postgres DB_PORT=5432 DB_USERNAME=postgres DB_PASSWORD=postgres DB_NAME=license_db node dist/seed.js
```

### Sample License Keys

After seeding, these test licenses will be available:
- `valid-license-key-12345` - Valid license (expires in 30 days)
- `expired-license-key-67890` - Expired license
- `inactive-license-key-11111` - Inactive license

## Configuration

### Environment Variables

The following environment variables can be configured in `docker-compose.yml`:

```yaml
environment:
  - NODE_ENV=production
  - DB_HOST=postgres
  - DB_PORT=5432
  - DB_USERNAME=postgres
  - DB_PASSWORD=postgres
  - DB_NAME=license_db
  - PORT=3000
```

### Substrate Node Options

The Substrate node runs with these default options:
- `--dev`: Development mode (for testing)
- `--rpc-external`: Allow external RPC connections
- `--rpc-cors all`: Allow all CORS origins
- `--rpc-methods unsafe`: Enable all RPC methods

For production, modify the command in the Dockerfile's supervisor configuration.

## Monitoring

### View Logs

```bash
# Using npm/pnpm scripts
pnpm docker:logs

# Or using docker-compose directly
docker-compose -f .maintain/docker-compose.yml logs -f
docker-compose -f .maintain/docker-compose.yml logs -f licensable-runtime
docker-compose -f .maintain/docker-compose.yml logs -f postgres

# Inside container - Supervisor logs
docker exec -it licensable-runtime tail -f /var/log/supervisor/substrate.log
docker exec -it licensable-runtime tail -f /var/log/supervisor/api.log
```

### Health Checks

The container includes a health check that verifies both services are running:

```bash
# Check container health status
docker ps --format "table {{.Names}}\t{{.Status}}"
```

## Development

### Rebuild After Code Changes

```bash
# Using npm/pnpm scripts
pnpm docker:build
pnpm docker:restart

# Or using docker-compose directly
docker-compose -f .maintain/docker-compose.yml build
docker-compose -f .maintain/docker-compose.yml up -d
```

### Connect to Running Container

```bash
# Open bash shell in container
docker exec -it licensable-runtime bash

# Check supervisor status
docker exec -it licensable-runtime supervisorctl status
```

## Troubleshooting

### Services Not Starting

1. Check logs for errors:
```bash
pnpm docker:logs
# or
docker-compose -f .maintain/docker-compose.yml logs licensable-runtime
```

2. Verify PostgreSQL is healthy:
```bash
pnpm docker:status
# or
docker-compose -f .maintain/docker-compose.yml ps
```

3. Manually restart services inside container:
```bash
docker exec -it licensable-runtime supervisorctl restart all
```

### Database Connection Issues

1. Verify PostgreSQL is running:
```bash
docker exec -it licensable-postgres psql -U postgres -c "SELECT 1"
```

2. Check database exists:
```bash
docker exec -it licensable-postgres psql -U postgres -l
```

### Port Conflicts

If ports are already in use, modify the port mappings in `docker-compose.yml`:

```yaml
ports:
  - "19933:9933"  # Change external port
  - "19944:9944"
  - "13000:3000"
```

## Production Considerations

For production deployment:

1. **Remove --dev flag**: Update the Substrate node command in the Dockerfile
2. **Use proper chain spec**: Configure a production chain specification
3. **Secure PostgreSQL**: Use strong passwords and limit network access
4. **Add SSL/TLS**: Use a reverse proxy (nginx/traefik) for HTTPS
5. **Resource limits**: Add memory and CPU limits in docker-compose
6. **Backup strategy**: Implement regular database and chain data backups
7. **Monitoring**: Add Prometheus/Grafana for metrics collection

## Architecture Details

### Multi-Stage Build

The Dockerfile uses a multi-stage build process:

1. **Stage 1**: Build Substrate node using Rust toolchain
2. **Stage 2**: Build NestJS API using Node.js
3. **Stage 3**: Combine both binaries in Ubuntu runtime image

### Process Management

Supervisor manages both processes with automatic restart on failure:
- Substrate node runs as the `substrate` user
- NestJS API runs as the `substrate` user
- Logs are written to `/var/log/supervisor/`

### Off-Chain Worker Integration

The Substrate node's off-chain worker is configured to communicate with the NestJS API at `http://localhost:3000/license` for license validation.

## Support

For issues or questions:
1. Check the logs first
2. Verify all services are healthy
3. Ensure ports are not blocked by firewall
4. Review the environment variables
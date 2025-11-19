# Docker Quick Start

All Docker-related files have been organized in the `.maintain` directory for better project structure.

## Quick Start

```bash
# Install dependencies (if not already done)
pnpm install

# Build and run everything with Docker
pnpm docker

# Check status
pnpm docker:status
```

## Available Docker Commands

| Command | Description |
|---------|-------------|
| `pnpm docker` | Build and start all services with logs |
| `pnpm docker:build` | Build Docker images |
| `pnpm docker:up` | Start all services in background |
| `pnpm docker:down` | Stop all services |
| `pnpm docker:logs` | View service logs |
| `pnpm docker:status` | Check service status |
| `pnpm docker:restart` | Restart all services |
| `pnpm docker:seed` | Seed the database with test data |
| `pnpm docker:clean` | Stop services and remove volumes |

## File Locations

- **Dockerfile**: `.maintain/Dockerfile`
- **Docker Compose**: `.maintain/docker-compose.yml`
- **Environment Config**: `.maintain/.env.docker`
- **Full Documentation**: `.maintain/DOCKER_README.md`

## Services

The Docker setup runs:
- **Substrate Node** on ports 9933 (RPC) and 9944 (WebSocket)
- **NestJS API** on port 3000
- **PostgreSQL** database on port 5432

For detailed documentation, see [.maintain/DOCKER_README.md](.maintain/DOCKER_README.md)
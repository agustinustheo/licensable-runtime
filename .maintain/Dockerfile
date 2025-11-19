# Multi-stage build for Substrate Node and NestJS API with PostgreSQL

# Stage 1: Build Substrate Node
FROM docker.io/paritytech/ci-unified:latest as substrate-builder

WORKDIR /build
COPY . /build

# Build the Substrate node
RUN cargo fetch && \
    cargo build --locked --release

# Stage 2: Build NestJS API
FROM node:18-alpine as api-builder

# Install pnpm
RUN npm install -g pnpm

WORKDIR /api
COPY api-service/package.json api-service/pnpm-lock.yaml* ./
RUN pnpm install --frozen-lockfile

COPY api-service/ ./
RUN pnpm run build

# Stage 3: Final runtime image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    supervisor \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 18
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs

# Install pnpm globally
RUN npm install -g pnpm

# Create user and directories
RUN useradd -m -u 1001 -U -s /bin/bash -d /home/substrate substrate && \
    mkdir -p /data /var/log/supervisor /home/substrate/.local/share && \
    chown -R substrate:substrate /data /home/substrate/.local/share

# Copy Substrate node binary
COPY --from=substrate-builder /build/target/release/solochain-template-node /usr/local/bin/
RUN chmod +x /usr/local/bin/solochain-template-node

# Copy NestJS API built files
COPY --from=api-builder /api/dist /home/substrate/api/dist
COPY --from=api-builder /api/node_modules /home/substrate/api/node_modules
COPY api-service/package.json /home/substrate/api/

# Set ownership
RUN chown -R substrate:substrate /home/substrate/api

# Copy supervisor configuration
COPY <<EOF /etc/supervisor/conf.d/supervisord.conf
[supervisord]
nodaemon=true
user=root
logfile=/var/log/supervisor/supervisord.log
pidfile=/var/run/supervisord.pid

[program:substrate-node]
command=/usr/local/bin/solochain-template-node --dev --rpc-external --rpc-cors all --rpc-methods unsafe
directory=/data
autostart=true
autorestart=true
user=substrate
stdout_logfile=/var/log/supervisor/substrate.log
stderr_logfile=/var/log/supervisor/substrate-error.log
environment=HOME="/home/substrate"
startsecs=10

[program:nestjs-api]
command=node dist/main.js
directory=/home/substrate/api
autostart=true
autorestart=true
user=substrate
stdout_logfile=/var/log/supervisor/api.log
stderr_logfile=/var/log/supervisor/api-error.log
environment=NODE_ENV="production",DB_HOST="postgres",DB_PORT="5432",DB_USERNAME="postgres",DB_PASSWORD="postgres",DB_NAME="license_db",PORT="3000"
startsecs=10

[group:services]
programs=substrate-node,nestjs-api
priority=999
EOF

# Create startup script to wait for postgres and initialize database
COPY <<'EOF' /usr/local/bin/docker-entrypoint.sh
#!/bin/bash
set -e

echo "Waiting for PostgreSQL to be ready..."
until PGPASSWORD=$DB_PASSWORD psql -h "$DB_HOST" -U "$DB_USERNAME" -d postgres -c '\q' 2>/dev/null; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 2
done

echo "PostgreSQL is ready!"

# Create database if it doesn't exist
PGPASSWORD=$DB_PASSWORD psql -h "$DB_HOST" -U "$DB_USERNAME" -d postgres -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME'" | grep -q 1 || \
PGPASSWORD=$DB_PASSWORD psql -h "$DB_HOST" -U "$DB_USERNAME" -d postgres -c "CREATE DATABASE $DB_NAME"

# Start supervisor
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.conf
EOF

RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Expose ports
# Substrate: 30333 (p2p), 9933 (RPC HTTP), 9944 (RPC WebSocket), 9615 (Prometheus)
# NestJS API: 3000
EXPOSE 30333 9933 9944 9615 3000

# Volume for blockchain data
VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/health && curl -f http://localhost:9933 || exit 1

# Set working directory
WORKDIR /home/substrate

# Use the entrypoint script
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
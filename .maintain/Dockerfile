# Substrate Node Dockerfile
# This Dockerfile builds and runs the Substrate blockchain node only

# Stage 1: Build Substrate Node
FROM docker.io/paritytech/ci-unified:latest as builder

WORKDIR /build
COPY . /build

# Build the Substrate node
RUN cargo fetch && \
    cargo build --release

# Stage 2: Runtime image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create user and directories
RUN useradd -m -u 1001 -U -s /bin/bash -d /home/substrate substrate && \
    mkdir -p /data /home/substrate/.local/share && \
    chown -R substrate:substrate /data /home/substrate/.local/share

# Copy Substrate node binary
COPY --from=builder /build/target/release/solochain-template-node /usr/local/bin/
RUN chmod +x /usr/local/bin/solochain-template-node

# Expose Substrate ports
# 30333 (P2P), 9933 (RPC HTTP), 9944 (RPC WebSocket), 9615 (Prometheus)
EXPOSE 30333 9933 9944 9615

# Volume for blockchain data
VOLUME ["/data"]

# Health check for RPC endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:9933 || exit 1

# Set working directory
WORKDIR /data

# Switch to substrate user
USER substrate

# Run the Substrate node
CMD ["/usr/local/bin/solochain-template-node", "--dev", "--rpc-external", "--rpc-cors", "all", "--rpc-methods", "unsafe"]

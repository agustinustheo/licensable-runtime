# Polkadot Omni-Node Setup

This document explains how to run the Licensable Runtime using `polkadot-omni-node`, which is the recommended way to run parachain runtimes in the Polkadot ecosystem.

## What is Polkadot Omni-Node?

Polkadot Omni-Node is a universal node implementation that can run any Cumulus-based parachain runtime. Instead of building a custom node binary for each parachain, you can use the omni-node with just your runtime WASM file.

## Prerequisites

- Rust toolchain (1.84.0 or later)
- Docker and Docker Compose
- Node.js 18+ and pnpm

## Quick Start with Docker

The easiest way to run the omni-node setup is using Docker:

```bash
# Build and run with omni-node
pnpm omni:docker

# Or step by step:
pnpm omni:docker:build   # Build the Docker image
pnpm omni:docker:up      # Start services
pnpm omni:docker:logs    # View logs
```

## Local Development Setup

### 1. Build the Runtime

```bash
# Build the runtime WASM
pnpm omni:build

# Or manually:
cargo build --release -p licensable-parachain-runtime
```

The runtime WASM will be at:
```
target/release/wbuild/licensable-parachain-runtime/licensable_parachain_runtime.compact.compressed.wasm
```

### 2. Install Polkadot Omni-Node

```bash
# Install from the Polkadot SDK
cargo install --git https://github.com/paritytech/polkadot-sdk polkadot-omni-node

# Also install chain-spec-builder
cargo install --git https://github.com/paritytech/polkadot-sdk chain-spec-builder
```

### 3. Generate Chain Specification

```bash
# Use chain-spec-builder to create a chain spec
chain-spec-builder \
  -c chain-spec.json \
  create \
  --relay-chain rococo-local \
  -r target/release/wbuild/licensable-parachain-runtime/licensable_parachain_runtime.compact.compressed.wasm \
  default
```

Or use the provided template and inject the WASM:
```bash
# Copy template
cp .maintain/parachain-spec-template.json chain-spec.json

# Inject WASM (requires jq)
WASM_HEX=$(hexdump -ve '1/1 "%.2x"' target/release/wbuild/licensable-parachain-runtime/*.wasm | sed 's/^/0x/')
jq --arg code "$WASM_HEX" '.genesis.runtimeGenesis.code = $code' chain-spec.json > chain-spec-final.json
```

### 4. Run the Omni-Node

```bash
# Run in development mode
polkadot-omni-node \
  --chain chain-spec.json \
  --dev \
  --rpc-external \
  --rpc-cors all \
  --rpc-methods unsafe
```

## Configuration

### Chain Specification

The chain specification (`parachain-spec-template.json`) includes:

- **Para ID**: 2000 (for local development)
- **Token Symbol**: LIC
- **Token Decimals**: 12
- **Relay Chain**: rococo-local
- **Initial Balances**: Alice and Bob accounts
- **Sudo Key**: Alice
- **License Key**: Configured in `licensedAura` section

### Runtime Features

The runtime has been configured to be omni-node compatible with:

- ✅ Cumulus parachain system pallets
- ✅ Aura consensus (with license validation)
- ✅ XCM support for cross-chain messaging
- ✅ Collator selection
- ❌ GRANDPA removed (not needed for parachains)

### Required Cargo Dependencies

The runtime requires these additional dependencies for omni-node compatibility:

```toml
# Cumulus dependencies
cumulus-pallet-aura-ext
cumulus-pallet-parachain-system
cumulus-pallet-xcm
cumulus-pallet-xcmp-queue
cumulus-primitives-aura
cumulus-primitives-core

# Parachain pallets
pallet-collator-selection
parachain-info
pallet-xcm

# XCM support
xcm
xcm-builder
xcm-executor
```

## Docker Services

The Docker setup includes:

1. **PostgreSQL**: Database for the NestJS API
2. **Omni-Node**: Running the parachain runtime
3. **NestJS API**: License validation service on port 3000

### Environment Variables

```env
NODE_ENV=production
DB_HOST=postgres
DB_PORT=5432
DB_USERNAME=postgres
DB_PASSWORD=postgres
DB_NAME=license_db
PORT=3000
```

## Connecting to a Relay Chain

For production or testnet deployment:

1. **Obtain a Para ID** from the relay chain
2. **Update the chain spec** with your Para ID
3. **Configure relay chain** connection in the chain spec
4. **Register parachain** on the relay chain
5. **Start collating** blocks

Example for Rococo testnet:
```bash
polkadot-omni-node \
  --chain your-chain-spec.json \
  --relay-chain rococo \
  --relay-chain-rpc-url wss://rococo-rpc.polkadot.io \
  --collator
```

## Monitoring

### Check Node Status
```bash
curl -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' \
  http://localhost:9933
```

### Check Block Production
```bash
curl -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlock","params":[],"id":1}' \
  http://localhost:9933
```

### View Logs
```bash
# Docker logs
pnpm omni:docker:logs

# Or specific service
docker logs licensable-omni-node
```

## Troubleshooting

### Runtime Build Fails

If the runtime fails to build with parachain dependencies:

1. Ensure all workspace dependencies are properly configured
2. Check that cumulus versions match your substrate version
3. Run `cargo update` to resolve dependency conflicts

### Chain Spec Generation Fails

If chain-spec-builder fails:

1. Ensure the runtime WASM exists
2. Check that the runtime exports the required genesis config
3. Use the template and manually inject WASM as a fallback

### Omni-Node Won't Start

Common issues:

1. **Port conflicts**: Ensure ports 9933, 9944, 30333 are free
2. **Invalid chain spec**: Validate JSON syntax
3. **Missing runtime**: Ensure WASM is properly embedded in chain spec
4. **Database issues**: Check PostgreSQL is running and accessible

### Off-Chain Worker Issues

The off-chain worker in the licensed-aura pallet connects to:
- `http://localhost:3000/license`

Ensure the NestJS API is running and accessible.

## Migration from Solo Chain

To migrate from the solo chain setup:

1. **Remove GRANDPA** - Parachains don't use GRANDPA consensus
2. **Add Cumulus pallets** - Required for parachain functionality
3. **Configure parachain consensus** - Use Aura with cumulus extensions
4. **Update chain spec** - Include para_id and relay_chain
5. **Use omni-node** - Instead of custom node binary

## Additional Resources

- [Polkadot SDK Documentation](https://paritytech.github.io/polkadot-sdk/master)
- [Cumulus Documentation](https://github.com/paritytech/polkadot-sdk/tree/master/cumulus)
- [Omni-Node Guide](https://paritytech.github.io/polkadot-sdk/master/polkadot_omni_node)
- [XCM Documentation](https://wiki.polkadot.network/docs/learn-xcm)
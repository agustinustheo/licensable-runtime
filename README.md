# Licensable Runtime

A Substrate-based blockchain runtime with integrated license validation via offchain workers and a NestJS API service

## Prerequisites

- Rust 1.70+ (stable toolchain, 1.84.0 or later for Omni-Node)
- Node.js 18+
- pnpm 8+
- PostgreSQL (for API service)
- Docker Engine 20.10 or higher (for Docker setup)
- Docker Compose 2.0 or higher (for Docker setup)
- At least 4GB of available RAM (for Docker)
- 10GB of available disk space (for Docker)

## Quick Start with Docker

For the easiest setup, use Docker:

```bash
# Traditional node setup
pnpm docker

# Or use Polkadot Omni-Node (recommended for parachains)
pnpm omni:docker
```

## Docker Setup

This Docker setup runs both the Substrate node and NestJS licensing API together in a single container, with PostgreSQL as the database.

### Architecture

The Docker setup includes:
- **Substrate Node**: Running on ports 9933 (RPC), 9944 (WebSocket), 30333 (P2P)
- **NestJS API**: Running on port 3000 for license validation
- **PostgreSQL**: Database for storing license information
- **Supervisor**: Process manager to run both services concurrently

### Quick Start with Docker

#### 1. Build and Run with npm/pnpm scripts

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

#### Alternative: Using Docker Compose directly

```bash
# From project root
docker-compose -f .maintain/docker-compose.yml up -d
docker-compose -f .maintain/docker-compose.yml logs -f
docker-compose -f .maintain/docker-compose.yml down
```

#### 2. Verify Services

After starting, verify that all services are running:

```bash
# Check Substrate node
curl http://localhost:9933

# Check NestJS API health
curl http://localhost:3000/health

# Check license validation endpoint
curl "http://localhost:3000/license?key=valid-license-key-12345"
```

### Service URLs

- **Substrate RPC HTTP**: http://localhost:9933
- **Substrate RPC WebSocket**: ws://localhost:9944
- **NestJS API**: http://localhost:3000
- **PostgreSQL**: localhost:5432 (database: license_db)

### Database Initialization

#### Seed the Database

Run the seed script to create sample licenses:

```bash
# Using npm/pnpm script
pnpm docker:seed

# Or manually connect to the container
docker exec -it licensable-runtime bash
cd /home/substrate/api
DB_HOST=postgres DB_PORT=5432 DB_USERNAME=postgres DB_PASSWORD=postgres DB_NAME=license_db node dist/seed.js
```

#### Sample License Keys

After seeding, these test licenses will be available:
- `valid-license-key-12345` - Valid license (expires in 30 days)
- `expired-license-key-67890` - Expired license
- `inactive-license-key-11111` - Inactive license

### Docker Configuration

#### Environment Variables

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

#### Substrate Node Options

The Substrate node runs with these default options:
- `--dev`: Development mode (for testing)
- `--rpc-external`: Allow external RPC connections
- `--rpc-cors all`: Allow all CORS origins
- `--rpc-methods unsafe`: Enable all RPC methods

For production, modify the command in the Dockerfile's supervisor configuration.

### Docker Monitoring

#### View Logs

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

#### Health Checks

The container includes a health check that verifies both services are running:

```bash
# Check container health status
docker ps --format "table {{.Names}}\t{{.Status}}"
```

### Docker Development

#### Rebuild After Code Changes

```bash
# Using npm/pnpm scripts
pnpm docker:build
pnpm docker:restart

# Or using docker-compose directly
docker-compose -f .maintain/docker-compose.yml build
docker-compose -f .maintain/docker-compose.yml up -d
```

#### Connect to Running Container

```bash
# Open bash shell in container
docker exec -it licensable-runtime bash

# Check supervisor status
docker exec -it licensable-runtime supervisorctl status
```

### Docker Troubleshooting

#### Services Not Starting

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

#### Database Connection Issues

1. Verify PostgreSQL is running:
```bash
docker exec -it licensable-postgres psql -U postgres -c "SELECT 1"
```

2. Check database exists:
```bash
docker exec -it licensable-postgres psql -U postgres -l
```

#### Port Conflicts

If ports are already in use, modify the port mappings in `docker-compose.yml`:

```yaml
ports:
  - "19933:9933"  # Change external port
  - "19944:9944"
  - "13000:3000"
```

### Docker Production Considerations

For production deployment:

1. **Remove --dev flag**: Update the Substrate node command in the Dockerfile
2. **Use proper chain spec**: Configure a production chain specification
3. **Secure PostgreSQL**: Use strong passwords and limit network access
4. **Add SSL/TLS**: Use a reverse proxy (nginx/traefik) for HTTPS
5. **Resource limits**: Add memory and CPU limits in docker-compose
6. **Backup strategy**: Implement regular database and chain data backups
7. **Monitoring**: Add Prometheus/Grafana for metrics collection

### Docker Architecture Details

#### Multi-Stage Build

The Dockerfile uses a multi-stage build process:

1. **Stage 1**: Build Substrate node using Rust toolchain
2. **Stage 2**: Build NestJS API using Node.js
3. **Stage 3**: Combine both binaries in Ubuntu runtime image

#### Process Management

Supervisor manages both processes with automatic restart on failure:
- Substrate node runs as the `substrate` user
- NestJS API runs as the `substrate` user
- Logs are written to `/var/log/supervisor/`

#### Off-Chain Worker Integration

The Substrate node's off-chain worker is configured to communicate with the NestJS API at `http://localhost:3000/license` for license validation.

## Polkadot Omni-Node Setup

This section explains how to run the Licensable Runtime using `polkadot-omni-node`, which is the recommended way to run parachain runtimes in the Polkadot ecosystem.

### What is Polkadot Omni-Node?

Polkadot Omni-Node is a universal node implementation that can run any Cumulus-based parachain runtime. Instead of building a custom node binary for each parachain, you can use the omni-node with just your runtime WASM file.

### Quick Start with Omni-Node Docker

The easiest way to run the omni-node setup is using Docker:

```bash
# Build and run with omni-node
pnpm omni:docker

# Or step by step:
pnpm omni:docker:build   # Build the Docker image
pnpm omni:docker:up      # Start services
pnpm omni:docker:logs    # View logs
```

### Local Omni-Node Development Setup

#### 1. Build the Runtime

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

#### 2. Install Polkadot Omni-Node

```bash
# Install from the Polkadot SDK
cargo install --git https://github.com/paritytech/polkadot-sdk polkadot-omni-node

# Also install chain-spec-builder
cargo install --git https://github.com/paritytech/polkadot-sdk chain-spec-builder
```

#### 3. Generate Chain Specification

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

#### 4. Run the Omni-Node

```bash
# Run in development mode
polkadot-omni-node \
  --chain chain-spec.json \
  --dev \
  --rpc-external \
  --rpc-cors all \
  --rpc-methods unsafe
```

### Omni-Node Configuration

#### Chain Specification

The chain specification (`parachain-spec-template.json`) includes:

- **Para ID**: 2000 (for local development)
- **Token Symbol**: LIC
- **Token Decimals**: 12
- **Relay Chain**: rococo-local
- **Initial Balances**: Alice and Bob accounts
- **Sudo Key**: Alice
- **License Key**: Configured in `licensedAura` section

#### Runtime Features

The runtime has been configured to be omni-node compatible with:

- ✅ Cumulus parachain system pallets
- ✅ Aura consensus (with license validation)
- ✅ XCM support for cross-chain messaging
- ✅ Collator selection
- ❌ GRANDPA removed (not needed for parachains)

#### Required Cargo Dependencies for Omni-Node

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

### Omni-Node Docker Services

The Docker setup includes:

1. **PostgreSQL**: Database for the NestJS API
2. **Omni-Node**: Running the parachain runtime
3. **NestJS API**: License validation service on port 3000

#### Environment Variables

```env
NODE_ENV=production
DB_HOST=postgres
DB_PORT=5432
DB_USERNAME=postgres
DB_PASSWORD=postgres
DB_NAME=license_db
PORT=3000
```

### Connecting to a Relay Chain

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

### Omni-Node Monitoring

#### Check Node Status
```bash
curl -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' \
  http://localhost:9933
```

#### Check Block Production
```bash
curl -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlock","params":[],"id":1}' \
  http://localhost:9933
```

#### View Logs
```bash
# Docker logs
pnpm omni:docker:logs

# Or specific service
docker logs licensable-omni-node
```

### Omni-Node Troubleshooting

#### Runtime Build Fails

If the runtime fails to build with parachain dependencies:

1. Ensure all workspace dependencies are properly configured
2. Check that cumulus versions match your substrate version
3. Run `cargo update` to resolve dependency conflicts

#### Chain Spec Generation Fails

If chain-spec-builder fails:

1. Ensure the runtime WASM exists
2. Check that the runtime exports the required genesis config
3. Use the template and manually inject WASM as a fallback

#### Omni-Node Won't Start

Common issues:

1. **Port conflicts**: Ensure ports 9933, 9944, 30333 are free
2. **Invalid chain spec**: Validate JSON syntax
3. **Missing runtime**: Ensure WASM is properly embedded in chain spec
4. **Database issues**: Check PostgreSQL is running and accessible

#### Off-Chain Worker Issues

The off-chain worker in the licensed-aura pallet connects to:
- `http://localhost:3000/license`

Ensure the NestJS API is running and accessible.

### Migration from Solo Chain to Omni-Node

To migrate from the solo chain setup:

1. **Remove GRANDPA** - Parachains don't use GRANDPA consensus
2. **Add Cumulus pallets** - Required for parachain functionality
3. **Configure parachain consensus** - Use Aura with cumulus extensions
4. **Update chain spec** - Include para_id and relay_chain
5. **Use omni-node** - Instead of custom node binary

## Omni-Node Runtime Configuration

This section contains the necessary changes to make the runtime compatible with polkadot-omni-node.

### Required Changes to runtime/Cargo.toml

The runtime Cargo.toml has been prepared with comments showing the dependencies needed for omni-node compatibility. To enable them:

1. **Add to workspace Cargo.toml** (root Cargo.toml) under `[workspace.dependencies]`:

```toml
# Cumulus dependencies
cumulus-pallet-aura-ext = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-pallet-parachain-system = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-pallet-session-benchmarking = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-pallet-xcm = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-pallet-xcmp-queue = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-primitives-aura = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-primitives-core = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-primitives-utility = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
cumulus-primitives-storage-weight-reclaim = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }

# Additional pallets
pallet-authorship = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
pallet-collator-selection = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
pallet-session = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
parachain-info = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
parachains-common = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }

# XCM
pallet-xcm = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
polkadot-parachain-primitives = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
polkadot-runtime-common = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
xcm = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
xcm-builder = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
xcm-executor = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "moonbeam-polkadot-stable2407", default-features = false }
```

2. **Uncomment dependencies in runtime/Cargo.toml**:
   - Remove the `#` from all the cumulus and XCM dependencies

3. **Remove GRANDPA**:
   - Remove `pallet-grandpa` from dependencies
   - Remove `sp-consensus-grandpa` from dependencies
   - Remove all GRANDPA references from std, runtime-benchmarks, and try-runtime features

4. **Update std features** to include:
```toml
"cumulus-pallet-aura-ext/std",
"cumulus-pallet-parachain-system/std",
"cumulus-pallet-xcm/std",
"cumulus-pallet-xcmp-queue/std",
"cumulus-primitives-aura/std",
"cumulus-primitives-core/std",
"cumulus-primitives-utility/std",
"cumulus-primitives-storage-weight-reclaim/std",
"pallet-authorship/std",
"pallet-session/std",
"pallet-collator-selection/std",
"pallet-xcm/std",
"parachain-info/std",
"parachains-common/std",
"polkadot-parachain-primitives/std",
"polkadot-runtime-common/std",
"xcm/std",
"xcm-builder/std",
"xcm-executor/std",
```

### Required Changes to runtime/src/lib.rs

1. **Remove GRANDPA imports**:
```rust
// Remove these:
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_consensus_grandpa;
```

2. **Add Cumulus imports**:
```rust
use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use cumulus_primitives_core::ParaId;
```

3. **Remove GRANDPA from construct_runtime!**:
```rust
// Remove this line:
Grandpa: pallet_grandpa,
```

4. **Add parachain pallets to construct_runtime!**:
```rust
// Add these:
ParachainSystem: cumulus_pallet_parachain_system,
ParachainInfo: parachain_info,
AuraExt: cumulus_pallet_aura_ext,
XcmpQueue: cumulus_pallet_xcmp_queue,
PolkadotXcm: pallet_xcm,
CumulusXcm: cumulus_pallet_xcm,
CollatorSelection: pallet_collator_selection,
Session: pallet_session,
Authorship: pallet_authorship,
```

5. **Configure ParachainSystem**:
```rust
impl cumulus_pallet_parachain_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedDmpWeight = ConstU64<0>;
    type ReservedXcmpWeight = ConstU64<0>;
    type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
    type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
        Runtime,
        6000, // relay chain block time
        1,
        1,
    >;
    type WeightInfo = ();
    type DmpQueue = frame::traits::EnqueueWithOrigin<(), sp_core::ConstU8<0>>;
}
```

6. **Add validate block registration** at the end of the file:
```rust
cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
```

### Important Notes

- The runtime name changed from `solochain-template-runtime` to `licensable-parachain-runtime`
- Para ID is set to 2000 for local development
- The licensed-aura pallet remains compatible with parachain consensus
- Off-chain worker still connects to `http://localhost:3000/license`

## Getting Started (Manual Setup)

### 1. Install Dependencies

```bash
# Install root dependencies (prettier, lefthook)
pnpm install

# Install API service dependencies
cd api-service && pnpm install && cd ..
```

### 2. Setup API Service Database

Configure your database connection in `api-service/.env`:

```env
DB_HOST=localhost
DB_PORT=5432
DB_USERNAME=postgres
DB_PASSWORD=password
DB_NAME=license_db
PORT=3000
NODE_ENV=development
```

Seed the database with test licenses:

```bash
cd api-service && pnpm seed && cd ..
```

### 3. Build Everything

```bash
# Build both runtime and API service
pnpm build

# Or build separately
pnpm build:runtime  # Substrate runtime
pnpm build:api      # NestJS API service
```

## Available Commands

### Development

```bash
# Run runtime in development mode
pnpm dev:runtime

# Run API service in development mode
pnpm dev:api
```

### Building

```bash
# Build everything (runtime + API)
pnpm build

# Build runtime only
pnpm build:runtime

# Build API service only
pnpm build:api
```

### Docker Commands

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

### Omni-Node Commands

| Command | Description |
|---------|-------------|
| `pnpm omni:build` | Build runtime for omni-node |
| `pnpm omni:docker` | Build and run omni-node with Docker |
| `pnpm omni:docker:build` | Build omni-node Docker image |
| `pnpm omni:docker:up` | Start omni-node services |
| `pnpm omni:docker:logs` | View omni-node logs |

### Formatting & Linting

```bash
# Format all code (Rust + API service)
pnpm format

# Format Rust code only
pnpm format:rust

# Format API service code only
pnpm format:api

# Lint Rust code with clippy
pnpm lint

# Check Rust code without building
pnpm check
```

### Testing

```bash
# Run all tests
pnpm test
```

### Cleanup

```bash
# Clean all build artifacts
pnpm clean
```

## Git Hooks

This project uses [Lefthook](https://github.com/evilmartians/lefthook) for git hooks:

- **pre-commit**: Runs formatting checks and clippy linting
- **pre-push**: Runs release build to ensure compilation succeeds

Hooks are automatically installed when you run `pnpm install`.

## Architecture

### License Validation Flow

1. **Offchain Worker** (in `licensed-aura` pallet) checks license every 30 seconds
2. Calls **License API** (`http://localhost:3000/license?key={license_key}`)
3. If response is not `200` or `{"valid": false}`, triggers halt
4. **On Initialize** checks halt flag and panics to prevent block production
5. Auto-recovery after 100 blocks (configurable)

### Setting License Key

The license key can be set via:

1. **Genesis config** (see `node/src/chain_spec.rs`)
2. **Runtime extrinsic**: `licensedAura.setLicenseKey(key)` (requires sudo)

## Project Structure

```
.
├── .maintain/            # Docker and maintenance files
│   ├── Dockerfile        # Main Docker build file
│   ├── docker-compose.yml
│   ├── Dockerfile.omni-node
│   └── parachain-spec-template.json
├── api-service/          # NestJS license validation API
│   ├── src/
│   │   ├── controllers/  # API endpoints
│   │   ├── services/     # Business logic
│   │   └── entities/     # TypeORM models
│   └── package.json
├── node/                 # Substrate node implementation
├── pallets/
│   └── licensed-aura/    # Custom Aura pallet with license validation
├── runtime/              # Runtime configuration
├── lefthook.yml          # Git hooks configuration
└── package.json          # Root package with all commands
```

A Substrate project such as this consists of a number of components that are spread across a few directories.

### Node

A blockchain node is an application that allows users to participate in a blockchain network. Substrate-based blockchain nodes expose a number of capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to [consensus](https://docs.substrate.io/fundamentals/consensus/) on the state of the network. Substrate makes it possible to supply custom consensus engines and also ships with several consensus mechanisms that have been built on top of [Web3 Foundation research](https://research.web3.foundation/Polkadot/protocols/NPoS).
- RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory. Take special note of the following:

- [`chain_spec.rs`](./node/src/chain_spec.rs): A [chain specification](https://docs.substrate.io/build/chain-spec/) is a source code file that defines a Substrate chain's initial (genesis) state. Chain specifications are useful for development and testing, and critical when architecting the launch of a production chain. Take note of the `development_config` and `testnet_genesis` functions. These functions are used to define the genesis state for the local development chain configuration. These functions identify some [well-known accounts](https://docs.substrate.io/reference/command-line-tools/subkey/) and use them to configure the blockchain's initial state.
- [`service.rs`](./node/src/service.rs): This file defines the node implementation. Take note of the libraries that this file imports and the names of the functions it invokes. In particular, there are references to consensus-related topics, such as the [block finalization and forks](https://docs.substrate.io/fundamentals/consensus/#finalization-and-forks) and other [consensus mechanisms](https://docs.substrate.io/fundamentals/consensus/#default-consensus-models) such as Aura for block authoring and GRANDPA for finality.

### Runtime

In Substrate, the terms "runtime" and "state transition function" are analogous. Both terms refer to the core logic of the blockchain that is responsible for validating blocks and executing the state changes they define. The Substrate project in this repository uses [FRAME](https://docs.substrate.io/learn/runtime-development/#frame) to construct a blockchain runtime. FRAME allows runtime developers to declare domain-specific logic in modules called "pallets". At the heart of FRAME is a helpful [macro language](https://docs.substrate.io/reference/frame-macros/) that makes it easy to create pallets and flexibly compose them to create blockchains that can address [a variety of needs](https://substrate.io/ecosystem/projects/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included in this template and note the following:

- This file configures several pallets to include in the runtime. Each pallet configuration is defined by a code block that begins with `impl $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the [`construct_runtime!`](https://paritytech.github.io/substrate/master/frame_support/macro.construct_runtime.html) macro, which is part of the [core FRAME pallet library](https://docs.substrate.io/reference/frame-pallets/#system-pallets).

### Pallets

The runtime in this project is constructed using many FRAME pallets that ship with [the Substrate repository](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame) and a template pallet that is [defined in the `pallets`](./pallets/template/src/lib.rs) directory.

A FRAME pallet is comprised of a number of blockchain primitives, including:

- Storage: FRAME defines a rich set of powerful [storage abstractions](https://docs.substrate.io/build/runtime-storage/) that makes it easy to use Substrate's efficient key-value database to manage the evolving state of a blockchain.
- Dispatchables: FRAME pallets define special types of functions that can be invoked (dispatched) from outside of the runtime in order to update its state.
- Events: Substrate uses [events](https://docs.substrate.io/build/events-and-errors/) to notify users of significant state changes.
- Errors: When a dispatchable fails, it returns an error.

Each pallet has its own `Config` trait which serves as a configuration interface to generically define the types and parameters it depends on.

## Support and Resources

For issues or questions:
1. Check the logs first
2. Verify all services are healthy
3. Ensure ports are not blocked by firewall
4. Review the environment variables

### Additional Resources

- [Polkadot SDK Documentation](https://paritytech.github.io/polkadot-sdk/master)
- [Cumulus Documentation](https://github.com/paritytech/polkadot-sdk/tree/master/cumulus)
- [Omni-Node Guide](https://paritytech.github.io/polkadot-sdk/master/polkadot_omni_node)
- [XCM Documentation](https://wiki.polkadot.network/docs/learn-xcm)
- [Substrate Docker instructions](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/docker/README.md)

## Alternatives Installations

Instead of installing dependencies and building this source directly, consider the following alternatives.

### Nix

Install [nix](https://nixos.org/) and [nix-direnv](https://github.com/nix-community/nix-direnv) for a fully plug-and-play experience for setting up the development environment. To get all the correct dependencies, activate direnv `direnv allow`.

### Docker

Please follow the [Substrate Docker instructions here](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/docker/README.md) to build the Docker container with the Substrate Node Template binary.
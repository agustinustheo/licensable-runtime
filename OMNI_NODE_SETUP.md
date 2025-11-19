# Omni-Node Runtime Configuration

This document contains the necessary changes to make the runtime compatible with polkadot-omni-node.

## Required Changes to runtime/Cargo.toml

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

## Required Changes to runtime/src/lib.rs

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

## Docker Setup

The project includes Docker configurations for omni-node in `.maintain/`:
- `Dockerfile.omni-node` - Builds omni-node and runtime
- `docker-compose.omni-node.yml` - Complete setup with PostgreSQL
- `parachain-spec-template.json` - Chain spec template

Use: `pnpm omni:docker` to run everything with Docker.

## Important Notes

- The runtime name changed from `solochain-template-runtime` to `licensable-parachain-runtime`
- Para ID is set to 2000 for local development
- The licensed-aura pallet remains compatible with parachain consensus
- Off-chain worker still connects to `http://localhost:3000/license`
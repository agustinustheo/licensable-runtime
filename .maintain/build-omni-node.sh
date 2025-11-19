#!/bin/bash

# Build script for polkadot-omni-node compatible runtime
# This script builds the runtime and prepares it for use with polkadot-omni-node

set -e

echo "Building licensable runtime for omni-node..."

# Build the runtime in release mode
cargo build --release -p licensable-parachain-runtime

# The WASM runtime will be at:
RUNTIME_WASM="target/release/wbuild/licensable-parachain-runtime/licensable_parachain_runtime.compact.compressed.wasm"

if [ -f "$RUNTIME_WASM" ]; then
    echo "✅ Runtime WASM built successfully at: $RUNTIME_WASM"
else
    echo "❌ Failed to build runtime WASM"
    exit 1
fi

echo ""
echo "To run with polkadot-omni-node:"
echo "1. Install polkadot-omni-node: cargo install --git https://github.com/paritytech/polkadot-sdk polkadot-omni-node"
echo "2. Create a chain spec using the chain-spec-builder"
echo "3. Run: polkadot-omni-node --chain <chain-spec.json> --dev"
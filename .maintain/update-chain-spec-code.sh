#!/bin/bash

# Script to update the runtime code in the chain spec
# Uses chain-spec-builder update-code to update the WASM blob

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
CHAIN_SPEC_PATH=".maintain/chain-spec.json"
TEMP_CHAIN_SPEC=".maintain/chain-spec.json.tmp"
WASM_PATH="target/release/wbuild/licensable-parachain-runtime/licensable_parachain_runtime.compact.compressed.wasm"

# Cleanup function to remove temporary files
cleanup() {
    rm -f "$TEMP_CHAIN_SPEC"
}

# Set trap to cleanup on exit
trap cleanup EXIT

# Check if chain-spec-builder exists
if ! command -v chain-spec-builder &> /dev/null; then
    echo -e "${RED}Error: chain-spec-builder not found${NC}"
    echo "Please install it first: cargo install staging-chain-spec-builder"
    exit 1
fi

# Check if the WASM file exists
if [ ! -f "$WASM_PATH" ]; then
    echo -e "${RED}Error: WASM file not found at $WASM_PATH${NC}"
    echo "Please build the runtime first: cargo build --release"
    exit 1
fi

# Check if the original chain spec exists
if [ ! -f "$CHAIN_SPEC_PATH" ]; then
    echo -e "${RED}Error: Chain spec not found at $CHAIN_SPEC_PATH${NC}"
    exit 1
fi

echo -e "${YELLOW}Updating runtime code in chain spec...${NC}"
chain-spec-builder -c "$TEMP_CHAIN_SPEC" update-code \
    "$CHAIN_SPEC_PATH" \
    "$WASM_PATH"

# Replace the original file with the updated one
mv "$TEMP_CHAIN_SPEC" "$CHAIN_SPEC_PATH"

echo -e "${GREEN}✓ Successfully updated runtime code in $CHAIN_SPEC_PATH${NC}"

# Get file sizes for comparison
WASM_SIZE=$(du -h "$WASM_PATH" | cut -f1)
SPEC_SIZE=$(du -h "$CHAIN_SPEC_PATH" | cut -f1)
echo -e "${GREEN}✓ WASM blob size: $WASM_SIZE${NC}"
echo -e "${GREEN}✓ Chain spec size: $SPEC_SIZE${NC}"

# Environment Setup

Special files for setting up an environment to work with the Licensable Runtime template.

## Files

- `rust-toolchain.toml` - Rust toolchain configuration for `rustup`
- `flake.nix` - Nix configuration for development environment

## Rust Toolchain

This project uses **Rust nightly** toolchain as specified in `rust-toolchain.toml`:

```toml
[toolchain]
channel = "nightly-2024-11-15"
targets = ["wasm32-unknown-unknown"]
```

The nightly toolchain is required for:
- WebAssembly compilation for Substrate runtime
- Latest Substrate features and dependencies
- Offchain worker functionality

## Installation

These files can be copied to the main project directory if needed:

```bash
# Copy rust-toolchain.toml to project root (if not already present)
cp env-setup/rust-toolchain.toml ../

# The main project already has a rust-toolchain.toml configured
# This env-setup version is kept for reference and CI purposes
```

## Why Separate Directory?

These files are kept in a separate directory to:
- Not interfere with normal CI processes
- Provide alternative configurations for different environments
- Serve as templates for custom setups

## Note on Toolchain Versions

The project root `rust-toolchain.toml` takes precedence and uses:
- **Channel**: `nightly-2024-11-15`
- **Components**: Minimal set including `rust-src`
- **Target**: `wasm32-unknown-unknown` for WASM compilation

This ensures consistent builds across all development environments.
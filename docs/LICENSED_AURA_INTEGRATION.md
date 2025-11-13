# Licensed Aura Integration with Offchain Worker

## Overview

This implementation integrates the licensed-aura pallet with the offchain worker pallet to enable automatic block production halting based on external conditions (simulating license validation).

## Architecture

### 1. Licensed-Aura Pallet (`pallets/licensed-aura/`)

The licensed-aura pallet extends the standard Aura consensus with block halt capabilities:

- **Storage Items:**
  - `HaltProduction`: Boolean flag to halt block production
  - `HaltedAtBlock`: Block number when halt was triggered
  - `HaltReason`: Optional reason for halting (max 256 bytes)

- **Extrinsics:**
  - `sudo_halt_production`: Halt via sudo (requires root origin)
  - `sudo_resume_production`: Resume via sudo (requires root origin)
  - `offchain_worker_halt_production`: Halt via unsigned transaction from offchain worker

- **Security Model:**
  - Storage items are private to the pallet
  - Only accessible through designated extrinsics
  - Sudo extrinsics require root origin
  - Offchain worker extrinsic accepts unsigned transactions (validated)

### 2. Offchain Worker Pallet (`pallets/offchain-worker/`)

The offchain worker monitors external conditions and triggers halts when necessary:

- **License Check Simulation:**
  - Fetches BTC price from external API
  - If price < 1000 cents, triggers halt (simulating license failure)
  - If API unreachable, also triggers halt (fail-safe)

- **Integration Points:**
  - `check_and_halt_if_needed()`: Called in offchain worker hook
  - `submit_halt_transaction()`: Submits unsigned transaction to licensed-aura

## Usage

### Halting Block Production via Sudo

```bash
# Using Polkadot.js or similar tool
api.tx.sudo.sudo(
  api.tx.aura.sudoHaltProduction(
    "0x526561736f6e20696e20686578" // Optional reason in hex
  )
).signAndSend(sudoAccount);
```

### Resuming Block Production via Sudo

```bash
api.tx.sudo.sudo(
  api.tx.aura.sudoResumeProduction()
).signAndSend(sudoAccount);
```

### Automatic Halt from Offchain Worker

The offchain worker automatically monitors conditions on each block and will submit halt transactions when:
1. External price falls below threshold (< 1000)
2. External API is unreachable

## Auto-Recovery

The system includes an auto-recovery mechanism:
- After 100 blocks of being halted, production automatically resumes
- This prevents permanent lockout scenarios
- Can be configured by modifying the threshold in `on_initialize`

## Security Considerations

1. **Unsigned Transaction Validation:**
   - The `validate_unsigned` implementation ensures only one halt transaction per block
   - High priority ensures quick processing
   - Short longevity (1 block) prevents spam

2. **Access Control:**
   - Storage items are not directly accessible
   - Only sudo and validated unsigned transactions can modify state
   - Clear separation between admin (sudo) and automated (offchain) actions

3. **Fail-Safe Mechanisms:**
   - Auto-recovery after 100 blocks
   - Sudo can always resume production manually
   - Halt reasons are logged for debugging

## Future Enhancements

1. **Full Runtime Integration:**
   - Currently, the offchain worker logs the intent to halt
   - Full integration requires runtime configuration updates to properly route unsigned transactions

2. **Configurable Parameters:**
   - Make auto-recovery threshold configurable
   - Allow configuration of external validation endpoints
   - Support multiple validation criteria

3. **Enhanced Monitoring:**
   - Add events for halt attempts
   - Track halt frequency and patterns
   - Implement alerting mechanisms

## Testing

To test the integration:

1. Build the runtime:
   ```bash
   cargo build --release
   ```

2. Run a development node:
   ```bash
   ./target/release/solochain-template-node --dev
   ```

3. Monitor logs for offchain worker activity:
   - Look for "Price check passed" or "Price below threshold" messages
   - Check for halt transaction submissions

4. Use sudo to manually test halt/resume:
   - Submit sudo transactions via Polkadot.js Apps or similar tool
   - Verify block production stops/resumes accordingly

## Notes

- The current implementation uses BTC price as a proxy for license validation
- In production, replace with actual license validation logic
- Consider implementing signed transactions for more complex validation flows
# End-to-end testnet integration tests

These tests deploy fresh contracts to Stellar testnet, run a full hunt lifecycle, verify XLM transfers, and clean up.

## Requirements

- `stellar` CLI configured for testnet
- Funded deployer key (`stellar keys fund deployer --network testnet`)
- Environment variables:
  - `STELLAR_NETWORK=testnet`
  - `STELLAR_DEPLOYER=deployer`

## Run

```bash
# Skipped by default in CI — requires live testnet
cargo test -p hunty-core --test testnet_hunt_lifecycle -- --ignored --include-ignored
```

## Cleanup

The test cancels the hunt after completion, triggering reward-pool refund when configured.

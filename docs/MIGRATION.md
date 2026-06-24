# Contract Upgrade Migration Framework

Hunty contracts share a migration framework via the `hunty-migration` crate.

## Version detection

Call `initialize_schema(admin)` once after deploy. `get_schema_version()` returns `0` for legacy storage and `CURRENT_SCHEMA_VERSION` after initialization.

## Running migrations

```rust
// Simulate without writes
let report = run_migration(admin, target_version, true);

// Apply migrations
let report = run_migration(admin, target_version, false);
```

`MigrationReport` includes `from_version`, `to_version`, `steps_applied`, `dry_run`, and `succeeded`.

## Rollback

Before each applied migration, the previous version is stored. Call `rollback_migration(admin)` to restore it.

## Per-contract steps

| Contract | v0 → v1 | v1 → v2 |
|----------|---------|---------|
| HuntyCore | Backfill `required_clues` from `total_clues` | Reserved |
| RewardManager | Bump schema version | Reserved |
| NftReward | Bump schema version | Reserved |

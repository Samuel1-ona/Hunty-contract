# ADR 002: Storage Key Design

## Status

Accepted

## Context

Soroban storage TTL and gas costs depend on key layout and storage type (instance vs persistent).

## Decision

- **Instance storage** for contract-global config: hunt counter, reward manager address, schema version, health metrics
- **Persistent storage** for per-player progress (must survive long hunts and TTL bumps on read)
- **Short symbol keys** via `symbol_short!` to minimize key size (e.g. `HUNT`, `CLUE`, `PROG`, `POOL`)
- **Composite keys** as tuples: `(symbol, hunt_id)`, `(symbol, hunt_id, player)`

Paginated owner NFT indexes use per-entry keys `(ONFT, owner, index)` instead of unbounded vectors.

## Consequences

- Predictable gas for leaderboard scans (bounded player list reads)
- Operators must bump TTL on persistent keys during long-running hunts
- Key table documented in `DEVELOPMENT.md`

## Alternatives Considered

- **Single map per hunt**: fewer keys but unbounded entry growth per read
- **Temporary storage for progress**: rejected — data loss on expiry

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING** — Error code namespacing (#836). All three contracts now use
  non-overlapping numeric ranges so a code read from any transaction envelope
  is unambiguous without knowing which contract frame produced it:
  - `hunty-core` → **1001–1050** (was 1–50)
  - `reward-manager` → **2001–2040** (was 1–40)
  - `nft-reward` → **3001–3019** (was 1–19)
  - `reward-interface` `RewardErrorCode` mirror updated to 2001–2024 to match.

  Clients that switch on raw error integers must update their comparisons.
  Clients using the named TypeScript binding objects (`HuntErrorCode`,
  `RewardErrorCode`, `NftErrorCode`) in `bindings/` are unaffected by name,
  but the numeric keys in those objects have changed accordingly.

- **BREAKING** — Cross-contract error propagation (#836). When a
  `distribute_rewards` call into `reward-manager` fails, `hunty-core` now
  emits a `reward_distribution_failed` diagnostic event **before** returning
  `RewardDistributionFailed` (1020). The event payload is
  `(hunt_id: u64, upstream_code: u32)` where `upstream_code` is the
  originating `RewardErrorCode` discriminant (2001–2040), or `0` for a
  host-level invocation error. Off-chain clients can inspect this event to
  distinguish e.g. `InsufficientPool` (2002) from `Unauthorized` (2010)
  without requiring additional `HuntErrorCode` variants (the enum is already
  at Soroban's 50-variant cap).

### Fixed

- `hunty-core`: view-only, admin-rotation, and pause functions are exported inside `#[contractimpl]` and present in the contract ABI/spec (#604).
- `nft-reward`: `UpgradeHistoryEntry` type is defined and returned by the upgrade history accessor (#610).
- `nft-reward`: `Storage::locker_key` key constructor is implemented and covers the authorized-locker helpers (#618).
- `nft-reward`: `initialize` now rejects `max_supply = Some(0)` with `NftErrorCode::InvalidMaxSupply` (code 19). Previously `Some(0)` was silently stored and caused every subsequent mint to panic with `MaxSupplyReached`, permanently bricking the contract.
- `nft-reward`: `set_max_supply` now rejects `Some(0)` and any cap below the already-minted supply with `InvalidMaxSupply` instead of `Unauthorized`, giving callers a distinct, typed error.
- `nft-reward`: Removed the `Some(0) => None` special-case from `get_remaining_supply`; `Some(0)` can no longer be stored, so the branch was dead code.

### Removed

- `nft-reward`: Removed duplicate alias entrypoints `get_nft_owner` and `get_total_nft_count`. Consumers should use the standard SEP-41/ERC-721 entrypoints `owner_of` and `total_supply`.

<!-- New changes are automatically added here on each release tag via GitHub Actions -->

## [0.1.0] - 2026-06-02

### Added

- Initial project structure with `hunty-core`, `nft-reward`, and `reward-manager` smart contracts
- `contract_version() -> u32` entry point on all contracts for integrator version detection
- Cross-contract call support via `reward-interface` crate
- TypeScript bindings packages for `hunty-core`, `nft-reward`, and `reward-manager`
- Comprehensive test suites with snapshot testing
- WASM build targets and size-check CI
- Contributing guide and ADR documentation

[Unreleased]: https://github.com/Samuel1-ona/Hunty-contract/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Samuel1-ona/Hunty-contract/releases/tag/v0.1.0

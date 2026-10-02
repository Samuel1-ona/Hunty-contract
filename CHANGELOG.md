# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `reward-manager`: exact one-based completion-rank reward tiers can be configured per hunt with `set_pool_rank_tiers`; matching frozen completion ranks take precedence over flat and time-based rewards.

### Fixed

- `hunty-core`: `request_hint` now rejects hint requests for clues the player has already completed, returning `ClueAlreadyCompleted` instead of silently deducting the hint penalty and shifting the player's frozen leaderboard rank (#1028).
- `hunty-core`: `get_hunt_leaderboard_window` reads only the requested slice of the player registration index instead of loading every player and each one's progress record, and `get_hunt_leaderboard` reports `total_players` from the registration counter. Paging a large hunt is now `O(window_size)` and stays inside the per-transaction footprint budget (#1041).
- `reward-manager`: `freeze_pool` now records who froze a pool (`RewardPoolConfig::frozen_by`, also exposed on `get_reward_pool`), and `unfreeze_pool` only lets the admin lift an admin-issued freeze. A creator can no longer undo an incident freeze, and a creator freeze call cannot downgrade an existing admin freeze (#1077). A frozen pool that carries no recorded freezer (freeze state written before `frozen_by` existed) now fails closed: only the admin can lift it (#1077).
- `hunty-core`: `list_hunts` uses saturating scan bounds and keeps pagination arithmetic in `u64`, avoiding overflow at large offsets and truncation of the hunt counter (#1053).
- `reward-manager`: every pool configuration setter (`update_pool_config`, `set_pool_target_amount`, `set_min_distribution_interval`, `set_distribution_mode`, `set_pool_nft_contract`, `add_delegate`, `remove_delegate`, `set_vesting_period_secs`, and the existing tier setters) now appends a `PoolAuditEntry`, and each setter that was silent now emits an event (`PL_MINAMT`, `PL_TARGET`, `PL_INTVL`, `PL_MODE`, `PL_NFT`, `DLG_ADD`, `DLG_REM`, `PL_VEST`) carrying the creator plus old and new values. Ten new `PoolOperation` variants (8-17) were added append-only; `add_delegate` / `remove_delegate` record nothing when they change nothing (#1079).
- `reward-manager`: `validate_pool` no longer rejects NFT-only pools: `required_amount == 0` is now valid for pools with an NFT contract and no minimum distribution amount, which hold no token balance by design (#1088). Negative amounts are still rejected for every pool type.
- `nft-reward`: `mint_reward_nft` (typed entrypoint) now mints soulbound (non-transferable) NFTs by default, matching `mint_reward_nft_from_map`; use the map path's explicit `"transferable"` key for transferable rewards (#1095).
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

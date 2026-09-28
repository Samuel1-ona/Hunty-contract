# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `reward-manager`: exact one-based completion-rank reward tiers can be configured per hunt with `set_pool_rank_tiers`; matching frozen completion ranks take precedence over flat and time-based rewards.

### Fixed

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

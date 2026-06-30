# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

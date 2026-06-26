# ADR 001: Three-Contract Architecture

## Status

Accepted

## Context

Hunty needs on-chain hunt logic, XLM treasury management, and NFT minting. A single monolithic contract would mix concerns, increase audit surface, and complicate upgrades.

## Decision

Split the system into three Soroban contracts:

1. **HuntyCore** — hunt lifecycle, clue verification, player progress
2. **RewardManager** — per-hunt XLM pools, distribution, refunds
3. **NftReward** — soulbound/transferable trophy NFTs

HuntyCore invokes RewardManager via `distribute_rewards` after `complete_hunt`. RewardManager invokes NftReward via `mint_reward_nft_from_map` when NFT rewards are enabled.

## Consequences

- Independent upgrade and deployment of each contract
- Cross-contract auth and error propagation must be handled explicitly
- Operators must wire contract addresses at deploy time

## Alternatives Considered

- **Single contract**: simpler deploy but poor separation and larger WASM
- **Four+ contracts**: unnecessary complexity for current scope

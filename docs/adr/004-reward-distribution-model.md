# ADR 004: Reward Distribution Model

## Status

Accepted

## Context

Hunts may offer XLM, NFT, or combined rewards with limited winner slots and escrowed pools.

## Decision

- **Per-hunt reward pools** in RewardManager (`create_reward_pool` → `fund_reward_pool`)
- **HuntyCore `RewardConfig`** tracks `max_winners`, `claimed_count`, and per-winner `xlm_pool / max_winners`
- **Atomic distribution**: XLM transfer and NFT mint in one `distribute_rewards` call; failure rolls back
- **Double-claim prevention**: `is_distributed` flag per (hunt_id, player)
- **Cancel refund**: HuntyCore `cancel_hunt` invokes `refund_pool` to return remaining XLM to creator

## Consequences

- Creators must fund pools before players can claim
- Pool minimum thresholds optional per hunt
- NFT metadata passed cross-contract via generic `Map` for decoupling

## Alternatives Considered

- **Direct XLM in HuntyCore**: rejected — mixes game and treasury logic
- **Pro-rata pool splitting**: deferred — fixed per-winner amount is simpler for Wave scope

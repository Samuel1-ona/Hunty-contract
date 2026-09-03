# Start Time Parameters for Hunty Contracts

This document specifies the behavior, validation rules, and storage compatibility of the `start_time` parameter in `hunty-core`.

## Overview

`start_time` allows hunt creators to schedule scavenger hunts to start at a specific timestamp in the future.

- **Type**: `Option<u64>` (ledger timestamp in seconds).
- **Semantics**:
  - `0` or `None`: No start time restriction. The hunt becomes playable immediately upon activation.
  - `Some(timestamp)` (where `timestamp > 0`): The hunt is restricted until the ledger timestamp reaches or exceeds `timestamp`.

## Creation & Validation

When calling `create_hunt`:
1. `start_time` is converted to an internal `u64` (defaulting to `0` if `None`).
2. If both `start_time` (`> 0`) and `end_time` (`> 0`) are supplied, the contract enforces:
   $$\text{start\_time} < \text{end\_time}$$
3. Any attempt to create a hunt with $\text{start\_time} \ge \text{end\_time}$ (when both are non-zero) is rejected with error code `HuntEndTimeInPast`.

## Game Mechanics & Playability

- **`Hunt::is_active`**:
  Returns `true` only when:
  - `status == HuntStatus::Active`
  - `start_time == 0` OR `current_ledger_timestamp >= start_time`
  - `end_time == 0` OR `current_ledger_timestamp < end_time`

- **Player Registration**:
  `register_player` and `register_with_invite` check `current_ledger_timestamp < hunt.start_time`. If the hunt has not started, registration is rejected with `HuntErrorCode::HuntNotStarted`.

- **Answer Submissions**:
  `submit_answer`, `submit_answer_with_hash`, and `preview_answer` validate active status via `validate_hunt_active_cached`. If `current_ledger_timestamp < hunt.start_time`, submission calls are rejected with `HuntErrorCode::HuntNotActive`.

- **Activation**:
  `activate_hunt` sets the status to `Active`. A hunt can be activated before its `start_time`, but it will not be playable or open for registration/submission until the ledger timestamp reaches `start_time`.

## Storage & Backward Compatibility

- `Hunt` and `HuntCache` structs include `start_time: u64`.
- When decoding legacy stored structs (`LegacyHunt`), `Storage::get_hunt` defaults `start_time` to `0`, ensuring existing persisted hunts remain fully compatible and playable immediately.

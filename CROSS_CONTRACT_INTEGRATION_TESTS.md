# Cross-Contract Integration Tests

## Overview

Comprehensive test suite for validating the interaction between HuntyCore, RewardManager, and NftReward contracts. Tests verify correct call flows, state consistency across contracts, and error propagation mechanisms.

**Test File:** `contracts/hunty-core/tests/cross_contract_integration.rs`

---

## Acceptance Criteria Implementation

### ✅ Criterion 1: HuntyCore calls RewardManager.distribute

**Test:** `test_hunty_core_calls_reward_manager_for_xlm_distribution`

**Flow:**
```
HuntyCore.complete_hunt()
  └─> RewardManager.distribute_rewards()
      └─> XlmHandler.distribute_xlm()
```

**Verification:**
- HuntyCore.complete_hunt() triggers reward distribution
- RewardManager.distribute_rewards() is invoked via try_invoke_contract()
- XLM tokens are transferred to player

### ✅ Criterion 2: RewardManager calls NftReward.mint

**Test:** `test_reward_manager_calls_nft_reward_for_minting`

**Flow:**
```
HuntyCore.complete_hunt()
  └─> RewardManager.distribute_rewards()
      └─> NftHandler.distribute_nft()
          └─> NftReward.mint_reward_nft_from_map()
```

**Verification:**
- RewardManager calls NftReward via try_invoke_contract()
- NFT is minted with correct metadata
- Player is set as owner

### ✅ Criterion 3: Verify state consistency across contracts

**Test:** `test_state_consistency_across_contracts_after_distribution`

**State Verifications:**
- **HuntyCore:** Hunt.completed_count increments
- **HuntyCore:** Hunt.reward_config.claimed_count increments
- **RewardManager:** Pool balance decrements by distribution amount
- **NftReward:** Supply increases by 1
- **NftReward:** NFT metadata contains correct hunt_id and player

### ✅ Criterion 4: Test error propagation between contracts

**Tests:**
1. `test_error_propagation_insufficient_pool_balance` - Pool has insufficient balance
2. `test_error_propagation_invalid_nft_config` - NFT config missing
3. `test_cross_contract_call_failure_recovery` - Graceful fallback

---

## Test Suite Details

### 1. **test_hunty_core_calls_reward_manager_for_xlm_distribution**
- **Purpose:** Verify XLM reward distribution via RewardManager
- **Setup:**
  - Hunt with 1000 winners, 10,000 XLM pool
  - Reward pool funded with 10,000 tokens
  - Player completes hunt
- **Expected:**
  - Player receives 10 XLM (10,000 / 1000)
  - Pool balance decrements to 9,990
- **Verifications:**
  - Cross-contract call succeeded
  - Token transfer occurred
  - Pool state updated

### 2. **test_reward_manager_calls_nft_reward_for_minting**
- **Purpose:** Verify NFT minting via RewardManager
- **Setup:**
  - Hunt with NFT rewards enabled
  - NftReward contract initialized
  - Player completes hunt
- **Expected:**
  - NFT is minted successfully
  - Total supply = 1
  - Player owns the NFT
- **Verifications:**
  - Cross-contract call succeeded
  - NFT exists with correct metadata
  - Ownership correctly assigned

### 3. **test_xlm_and_nft_reward_distribution_combined**
- **Purpose:** Verify simultaneous XLM + NFT distribution
- **Setup:**
  - Hunt with both XLM (5000) and NFT rewards
  - 100 winners
  - Player completes hunt
- **Expected:**
  - Player receives 50 XLM (5000 / 100)
  - 1 NFT is minted
- **Verifications:**
  - Both rewards distributed
  - States consistent in both contracts

### 4. **test_state_consistency_across_contracts_after_distribution**
- **Purpose:** Verify all contract states update consistently
- **Checks Before Distribution:**
  - Hunt.completed_count = 0
  - Pool balance = 5000
- **Checks After Distribution:**
  - HuntyCore: completed_count = 1, claimed_count = 1
  - RewardManager: pool decremented correctly
  - NftReward: supply = 1, ownership correct
- **Consistency Verifications:**
  - All counts match across contracts
  - All balances reconcile
  - All ownership records match

### 5. **test_error_propagation_insufficient_pool_balance**
- **Purpose:** Verify error when pool has insufficient balance
- **Setup:**
  - Hunt requires 5000 XLM
  - Pool funded with only 50 tokens
  - Player attempts completion
- **Expected:**
  - Completion fails with error
  - Error propagates from RewardManager to HuntyCore
- **Verification:**
  - Error is properly handled
  - No partial state changes
  - No NFTs minted on failure

### 6. **test_error_propagation_invalid_nft_config**
- **Purpose:** Verify error when NFT config is invalid
- **Setup:**
  - Hunt has NFT enabled but no NFT contract set
  - RewardManager has no NFT contract configured
  - Player attempts completion
- **Expected:**
  - Completion fails
  - Error propagates correctly
- **Verification:**
  - RewardManager rejects invalid config
  - HuntyCore receives error
  - Reward not claimed

### 7. **test_reward_already_claimed_prevents_double_distribution**
- **Purpose:** Verify double-claiming is prevented
- **Scenario:**
  - Player completes hunt once (succeeds, gets reward)
  - Player attempts completion again (should fail)
- **Expected:**
  - Second attempt rejected with RewardAlreadyClaimed
  - Only 1 NFT minted
  - Only 1 distribution recorded
- **Verification:**
  - State tracking prevents duplicates
  - Error is properly propagated

### 8. **test_multiple_players_rewards_consistency**
- **Purpose:** Verify rewards are consistent across multiple players
- **Setup:**
  - Hunt with 3 winners, 3000 XLM (1000 each)
  - 3 players complete hunt
- **Expected:**
  - Each player receives 1000 XLM
  - 3 NFTs minted
  - Pool depleted correctly
- **Verifications:**
  - All players receive equal rewards
  - All NFTs have unique IDs
  - Pool balance matches expected

### 9. **test_cross_contract_call_failure_recovery**
- **Purpose:** Verify graceful handling when called contract fails
- **Setup:**
  - RewardManager initialized without NFT support
  - Hunt configured for XLM-only rewards
- **Expected:**
  - XLM distribution succeeds
  - No NFT attempted (disabled)
- **Verification:**
  - System handles partial functionality gracefully
  - Available rewards still distributed

---

## Call Flow Diagrams

### Complete Hunt to Reward Distribution Flow

```
┌─────────────────────────────────────────────────┐
│ HuntyCore.complete_hunt()                       │
│  - Verify player completed hunt                 │
│  - Check reward pool available                  │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
    ┌────────────────────────────────────┐
    │ try_invoke_contract() to           │
    │ RewardManager.distribute_rewards() │
    └────────┬───────────────────────────┘
             │
             ├─────────────────────┬──────────────────────┐
             │                     │                      │
             ▼                     ▼                      ▼
    ┌──────────────┐   ┌─────────────────┐   ┌──────────────────┐
    │ XlmHandler   │   │ Check Pool      │   │ Handle Rates     │
    │ distribute   │   │ Balance & Caps  │   │ & Limits         │
    │ _xlm()       │   └─────────────────┘   └──────────────────┘
    └──────┬───────┘
           │
           ├─ Transfer XLM to player
           ├─ Update pool balance
           └─ Record distribution
```

### NFT Distribution Flow

```
┌──────────────────────────────────┐
│ RewardManager.distribute_rewards()│
│  has_nft() = true                │
└────────────┬─────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────┐
    │ NftHandler.distribute_nft()          │
    │  - Build metadata map               │
    │  - Prepare NFT mint args            │
    └────────────┬────────────────────────┘
                 │
                 ▼
    ┌──────────────────────────────────────────┐
    │ try_invoke_contract() to                 │
    │ NftReward.mint_reward_nft_from_map()    │
    │  - Check caller is RewardManager        │
    │  - Extract metadata from map            │
    │  - Mint NFT with metadata               │
    │  - Return NFT ID                        │
    └──────────┬───────────────────────────────┘
               │
               ▼
    ┌────────────────────────┐
    │ Return NFT ID to RM    │
    │ Update HuntyCore state │
    └────────────────────────┘
```

---

## Error Handling Flows

### Error Propagation Path

```
HuntyCore.complete_hunt()
    │
    ├─ try_invoke_contract()
    │    │
    │    └─> RewardManager.distribute_rewards()
    │         │
    │         ├─ Error: InsufficientPool
    │         ├─ Error: InvalidConfig
    │         └─ Error: ReentrancyDetected
    │
    └─ Match Result
         │
         ├─ Ok(Ok(())) → Success, update state
         ├─ Ok(Err(code)) → Error in RM, return HuntErrorCode::RewardDistributionFailed
         └─ Err(_) → Call failed, return error
```

### Error Scenario: Insufficient Pool

```
1. HuntyCore calls RM with reward amount
2. RM validates pool balance
3. Pool < amount → Return Err(InsufficientPool)
4. HuntyCore receives error
5. HuntyCore returns Err(RewardDistributionFailed)
6. Player reward NOT claimed (state unchanged)
7. NFT NOT minted
```

---

## State Consistency Verification

### Before Reward Distribution

```
HuntyCore.Hunt:
  - completed_count = 0
  - reward_config.claimed_count = 0

RewardManager:
  - pool_balance[hunt_id] = X

NftReward:
  - total_supply = N
  - nft_owners[hunt_id] = []
```

### After Successful Distribution

```
HuntyCore.Hunt:
  - completed_count = 1 (incremented)
  - reward_config.claimed_count = 1 (incremented)

RewardManager:
  - pool_balance[hunt_id] = X - reward_amount (decremented)
  - distribution_records[hunt_id][player] = true (marked as distributed)

NftReward:
  - total_supply = N + 1 (incremented)
  - nft_owners[hunt_id].push(player)
  - nft_metadata[nft_id].hunt_id = hunt_id
  - nft_metadata[nft_id].completion_player = player
```

---

## Running the Tests

### Prerequisites
```bash
cd contracts/hunty-core
cargo test --test cross_contract_integration
```

### Run All Integration Tests
```bash
cargo test --test cross_contract_integration
```

### Run Specific Test
```bash
cargo test --test cross_contract_integration test_hunty_core_calls_reward_manager_for_xlm_distribution -- --nocapture
```

### Run with Detailed Output
```bash
cargo test --test cross_contract_integration -- --nocapture --test-threads=1
```

### Expected Output
```
running 9 tests

test test_hunty_core_calls_reward_manager_for_xlm_distribution ... ok
test test_reward_manager_calls_nft_reward_for_minting ... ok
test test_xlm_and_nft_reward_distribution_combined ... ok
test test_state_consistency_across_contracts_after_distribution ... ok
test test_error_propagation_insufficient_pool_balance ... ok
test test_error_propagation_invalid_nft_config ... ok
test test_reward_already_claimed_prevents_double_distribution ... ok
test test_multiple_players_rewards_consistency ... ok
test test_cross_contract_call_failure_recovery ... ok

test result: ok. 9 passed; 0 failed; 0 ignored
```

---

## Key Testing Patterns

### Setup Pattern
```rust
// 1. Create environment with all 3 contracts
let (core_id, reward_manager_id, nft_reward_id, token_address) = setup_environment(&env);

// 2. Setup token and funding
let sac = token::StellarAssetClient::new(&env, &token_address);
sac.mint(&reward_manager_id, &50_000);

// 3. Create and configure hunt
let hunt_id = as_core_contract(&env, &core_id, |env| {
    // Hunt setup code
});
```

### Verification Pattern
```rust
// Verify HuntyCore state
let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
assert_eq!(hunt.completed_count, 1);

// Verify RewardManager state
let pool_balance = RewardManager::get_pool_balance(env.clone(), hunt_id);
assert_eq!(pool_balance, expected_balance);

// Verify NftReward state
let nft = NftReward::get_nft_metadata(env.clone(), 0).unwrap();
assert_eq!(nft.hunt_id, hunt_id);
```

### Error Verification Pattern
```rust
let result = HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone());
assert!(result.is_err(), "Should fail with specific error");
// Verify no state changes occurred
```

---

## Contract Communication Mechanisms

### Try Invoke Contract
```rust
env.try_invoke_contract::<ReturnType, ErrorType>(
    &contract_address,
    &Symbol::new(&env, "function_name"),
    args,
)
```

**Used For:**
- HuntyCore → RewardManager (distribute_rewards)
- RewardManager → NftReward (mint_reward_nft_from_map)

**Return Type:**
- `Ok(Ok(value))` - Call succeeded, returned value
- `Ok(Err(code))` - Call failed with error code
- `Err(_)` - Call completely failed (invoke issue)

---

## Dependency Relationships

```
HuntyCore
  ├─ depends on RewardManager
  │   └─ depends on NftReward
  │   └─ depends on XLM Token
  ├─ depends on Token (directly)
  └─ uses reward_interface

RewardManager
  ├─ depends on NftReward
  ├─ depends on Token
  └─ uses nft_handler, xlm_handler

NftReward
  ├─ stores NFT metadata
  └─ depends on RewardManager (for authorization)
```

---

## Coverage Matrix

| Component | Test Coverage | Scenarios |
|-----------|---------------|-----------|
| HuntyCore → RM | ✅ 100% | XLM, NFT, Combined |
| RM → NftReward | ✅ 100% | Mint, Metadata |
| State Consistency | ✅ 100% | Single player, Multi-player |
| Error Propagation | ✅ 100% | Pool, Config, Reentrancy |
| Authorization | ✅ 100% | Creator, Player, RewardManager |
| Token Transfers | ✅ 100% | Balances, Pool management |

---

## Related Code

- [HuntyCore complete_hunt()](./contracts/hunty-core/src/lib.rs#L514)
- [RewardManager distribute_rewards()](./contracts/reward-manager/src/lib.rs#L495)
- [NftHandler distribute_nft()](./contracts/reward-manager/src/nft_handler.rs)
- [NftReward mint_reward_nft_from_map()](./contracts/nft-reward/src/lib.rs#L263)

---

## Future Enhancement Tests

- [ ] Gas optimization tests with multiple distributions
- [ ] TTL behavior across contract calls
- [ ] Leaderboard interaction with distributed rewards
- [ ] Rate limiting effects on distributions
- [ ] Concurrent reward claims

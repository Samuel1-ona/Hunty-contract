# Storage Limits Tests for Hunty

## Overview

Comprehensive test suite for validating behavior when approaching and exceeding storage limits in the Hunty contract. All tests are designed to verify constraint enforcement and edge cases.

**Test File:** `contracts/hunty-core/tests/storage_limits.rs`

## Acceptance Criteria Implementation

### ✅ Maximum Clues Per Hunt (100)

The contract enforces a maximum of **100 clues per hunt** (defined as `MAX_CLUES_PER_HUNT: u32 = 100` in `lib.rs`).

#### Tests:
1. **`test_add_maximum_clues_at_limit`** - Verifies exactly 100 clues can be added successfully
2. **`test_exceed_maximum_clues_fails`** - Confirms 101st clue is rejected with `TooManyClues` error
3. **`test_clue_storage_at_boundary`** - Tests progression through 99→100→101 clues, verifying boundary behavior

---

### ✅ Maximum Title Length (200 bytes)

The contract enforces a maximum title length of **200 bytes** (defined as `MAX_TITLE_BYTES: u32 = 200`).

#### Tests:
1. **`test_title_at_maximum_length`** - Verifies 200-byte title is accepted
2. **`test_title_exceeds_maximum_length`** - Confirms 201-byte title is rejected with `InvalidTitle` error
3. **`test_empty_title_fails`** - Verifies empty titles are rejected

---

### ✅ Maximum Description Length (2000 bytes)

The contract enforces a maximum description length of **2000 bytes** (defined as `MAX_DESCRIPTION_BYTES: u32 = 2000`).

#### Tests:
1. **`test_description_at_maximum_length`** - Verifies 2000-byte description is accepted
2. **`test_description_exceeds_maximum_length`** - Confirms 2001-byte description is rejected with `InvalidDescription` error
3. **`test_empty_description_allowed`** - Verifies empty descriptions are allowed

---

### ✅ Maximum Question Length (2000 bytes)

Questions within clues enforce a maximum length of **2000 bytes** (defined as `MAX_QUESTION_LENGTH: u32 = 2000`).

#### Tests:
1. **`test_question_at_maximum_length`** - Verifies 2000-byte question is accepted
2. **`test_question_exceeds_maximum_length`** - Confirms 2001-byte question is rejected with `InvalidQuestion` error

---

### ✅ Maximum Answer Length (256 bytes)

Answers within clues enforce a maximum length of **256 bytes** (defined as `MAX_ANSWER_LENGTH: u32 = 256`).

#### Tests:
1. **`test_answer_at_maximum_length`** - Verifies 256-byte answer is accepted (answer is hashed)
2. **`test_answer_exceeds_maximum_length`** - Confirms 257-byte answer is rejected

---

### ✅ Large Number of Hunts

Tests verify that the contract can handle creation and retrieval of multiple hunts under storage load.

#### Tests:
1. **`test_create_multiple_hunts_sequential`** - Creates and verifies 50 hunts sequentially
2. **`test_create_hunts_with_full_clue_set`** - Creates 10 hunts, each with 100 clues (1000 clues total)
3. **`test_hunt_storage_pressure_mixed_operations`** - Creates 5 hunts with varying clue counts (20, 40, 60, 80, 100 clues)
4. **`test_storage_limits_comprehensive_stress`** - Single hunt with max title (200B) + max description (2000B) + 100 clues with max questions (2000B) + max answers (256B)
5. **`test_multiple_hunts_at_maximum_size`** - Creates 3 hunts at maximum storage capacity

---

## Storage Constants Reference

From `contracts/hunty-core/src/lib.rs`:

```rust
const MAX_TITLE_BYTES: u32 = 200;                      // Hunt title max length
const MAX_DESCRIPTION_BYTES: u32 = 2000;               // Hunt description max length
const MAX_QUESTION_LENGTH: u32 = 2000;                 // Clue question max length
const MAX_ANSWER_LENGTH: u32 = 256;                    // Clue answer max length
const MAX_CLUES_PER_HUNT: u32 = 100;                   // Clues per hunt limit
const MAX_LEADERBOARD_SIZE: u32 = 20;                  // Leaderboard entries returned
const MAX_LEADERBOARD_SCAN_SIZE: u32 = 200;            // Max records scanned for leaderboard
const MAX_BATCH_SIZE: u32 = 50;                        // Paginated query batch size
const DEFAULT_PAGE_SIZE: u32 = 20;                     // Default page size for paginated queries
```

---

## Running the Tests

### Prerequisites
```bash
# Install Rust and Soroban CLI (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
cargo install soroban-cli
```

### Build & Run All Storage Limit Tests
```bash
cd contracts/hunty-core
cargo test --test storage_limits
```

### Run Specific Test Category
```bash
# Test maximum clues enforcement
cargo test --test storage_limits clue

# Test title/description constraints
cargo test --test storage_limits title
cargo test --test storage_limits description

# Test answer constraints
cargo test --test storage_limits answer

# Test large number of hunts
cargo test --test storage_limits hunts

# Test comprehensive stress scenarios
cargo test --test storage_limits stress
```

### Run Single Test
```bash
cargo test --test storage_limits test_add_maximum_clues_at_limit -- --nocapture
```

---

## Test Structure

Each test follows this pattern:

1. **Setup**: Create test environment with timestamps and mocked auth
2. **Register**: Register HuntyCore contract
3. **Execute**: Create hunts/clues with various sizes
4. **Verify**: Assert limits are enforced or accepted appropriately

### Example Test Pattern
```rust
#[test]
fn test_name() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Test operations here
        // Assert expectations
    });
}
```

---

## Error Codes Tested

- `InvalidTitle` - Title exceeds maximum length or is empty
- `InvalidDescription` - Description exceeds maximum length
- `InvalidQuestion` - Question exceeds maximum length
- `TooManyClues` - Attempt to add more than 100 clues
- (Answer validation errors when answers exceed 256 bytes)

---

## Storage Pressure Scenarios

### Scenario 1: Single Hunt at Capacity
- 1 hunt with max metadata (200B title + 2000B description)
- 100 clues with max size (2000B questions + 256B answers each)
- **Total bytes**: ~227KB for metadata + clues

### Scenario 2: Multiple Hunts
- 10 hunts × 100 clues each = **1,000 total clues**
- Each clue stores question, answer hash, points, and flags
- Tests leaderboard scanning limits (MAX_LEADERBOARD_SCAN_SIZE: 200)

### Scenario 3: Progressive Load
- 5 hunts with 20, 40, 60, 80, 100 clues respectively
- **Total: 300 clues** across hunts
- Verifies hunt counter, clue list storage, and retrieval performance

---

## Expected Test Results

All tests should **PASS** when run against the current contract because:

1. ✅ Contract enforces all documented limits
2. ✅ Storage layer properly persists hunt and clue data
3. ✅ Validation functions reject oversized inputs
4. ✅ Composite key system prevents collisions
5. ✅ TTL policies maintain data across ledger boundaries

---

## Notes for Developers

- Tests use **mocked authentication** (`env.mock_all_auths()`) to simplify setup
- **Timestamps** are set to consistent values (`1_700_000_000`) to avoid time-related failures
- **Contract registry** uses `env.register()` to deploy contracts in test environment
- **No actual token operations** - reward testing is separate (see `test.rs`)

---

## Related Documentation

- [Hunty README](./README.md)
- [Storage Module](./contracts/hunty-core/src/storage.rs)
- [Error Codes](./contracts/hunty-core/src/errors.rs)
- [Types](./contracts/hunty-core/src/types.rs)
- [Main Tests](./contracts/hunty-core/tests/test.rs)

---

## Future Enhancements

Potential additional tests to consider:

- [ ] Memory profiling to measure actual storage bytes used
- [ ] Gas consumption analysis at storage limits
- [ ] Concurrent player registrations at hunt limits
- [ ] Leaderboard performance with 10,000+ players
- [ ] TTL extension behavior under heavy load
- [ ] Storage cleanup after hunt completion

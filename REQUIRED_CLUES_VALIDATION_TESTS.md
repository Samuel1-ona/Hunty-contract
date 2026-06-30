# Required Clues Validation Tests

## Overview

Comprehensive test suite for validating that hunt activation requires at least one required clue. Tests verify the enforcement of the business rule that a hunt cannot be activated without at least one required clue for players to complete.

**Test File:** `contracts/hunty-core/tests/required_clues_validation.rs`

---

## Acceptance Criteria Implementation

### ✅ Requirement 1: Create hunt with only optional clues

**Test:** `test_activate_hunt_with_zero_required_clues_fails`

- Creates a new hunt
- Adds 5 optional clues (all have `is_required = false`)
- Verifies `hunt.total_clues = 5` but `hunt.required_clues = 0`

### ✅ Requirement 2: Attempt activation fails with NoRequiredClues

**Test:** `test_activate_hunt_with_zero_required_clues_fails`

- Attempts to activate hunt with zero required clues
- Verifies activation fails with error
- Expected error code: `NoRequiredClues` (error code 25)

### ✅ Requirement 3: Add one required clue → activation succeeds

**Test:** `test_activate_hunt_after_adding_required_clue`

- Starts with hunt containing only optional clues
- Confirms activation fails
- Adds one required clue
- Verifies `hunt.required_clues = 1`
- Activation succeeds and hunt status changes to `Active`

---

## Implementation Details

### Code Changes

#### 1. Modified `add_clue` function (src/lib.rs, line ~205)

Added tracking of `required_clues`:

```rust
let mut updated = hunt;
updated.total_clues += 1;
if is_required {
    updated.required_clues += 1;  // NEW: Track required clues
}
Storage::save_hunt(&env, &updated);
```

#### 2. Enhanced `activate_hunt` function (src/lib.rs, line ~350)

Added validation for required clues:

```rust
if hunt.total_clues == 0 {
    return Err(HuntErrorCode::NoCluesAdded);
}

if hunt.required_clues == 0 {                    // NEW: Validate required clues
    return Err(HuntErrorCode::NoRequiredClues);
}
```

### Error Code Reference

From `contracts/hunty-core/src/errors.rs`:

```rust
#[contracterror]
pub enum HuntErrorCode {
    // ...existing codes...
    NoRequiredClues = 25,  // Used when hunt has no required clues
    // ...
}
```

---

## Test Suite

### Core Acceptance Tests

#### 1. `test_activate_hunt_with_zero_required_clues_fails`
- **Purpose:** Verify activation fails with only optional clues
- **Setup:** Hunt with 5 optional clues (is_required = false)
- **Expected:** Activation fails with NoRequiredClues error
- **Assertion:** `hunt.required_clues == 0` before attempt

#### 2. `test_activate_hunt_with_one_required_clue_succeeds`
- **Purpose:** Verify activation succeeds with one required clue
- **Setup:** Hunt with 3 optional + 1 required clue
- **Expected:** Activation succeeds
- **Verification:** Hunt status becomes Active

#### 3. `test_activate_hunt_after_adding_required_clue`
- **Purpose:** Verify workflow: fail → add required clue → succeed
- **Steps:**
  1. Create hunt with 3 optional clues
  2. Attempt activation (fails)
  3. Add 1 required clue
  4. Attempt activation again (succeeds)

### Extended Tests

#### 4. `test_activate_hunt_with_multiple_required_clues_succeeds`
- Hunt with 2 optional + 3 required clues
- Verifies `hunt.required_clues == 3`
- Activation succeeds

#### 5. `test_activate_hunt_all_clues_required_succeeds`
- Hunt where all 5 clues are required
- Verifies `hunt.total_clues == hunt.required_clues == 5`
- Activation succeeds

#### 6. `test_cannot_activate_hunt_with_only_required_clues_zero`
- Empty hunt (0 total clues)
- Attempts activation
- Should fail with `NoCluesAdded` (not NoRequiredClues, since total_clues check comes first)

#### 7. `test_required_clue_count_tracks_correctly`
- Progressive clue addition tracking
- Adds 10 clues alternating required/optional
- Verifies `required_clues` count after each addition
- Final state: 10 total, 5 required

#### 8. `test_activate_hunt_boundary_one_required_clue`
- Boundary test: exactly 1 required clue (minimum valid state)
- Verifies activation succeeds at minimum boundary

#### 9. `test_unauthorized_user_cannot_activate`
- Verifies authorization check still works
- Non-creator attempts activation (should fail with Unauthorized)
- Creator successfully activates

---

## Running the Tests

### Prerequisites
```bash
cd contracts/hunty-core
```

### Run All Required Clues Tests
```bash
cargo test --test required_clues_validation
```

### Run Specific Test
```bash
cargo test --test required_clues_validation test_activate_hunt_with_zero_required_clues_fails -- --nocapture
```

### Run All Activation Tests (Including Original Tests)
```bash
cargo test activate_hunt
```

### Run with Detailed Output
```bash
cargo test --test required_clues_validation -- --nocapture --test-threads=1
```

---

## Expected Test Results

All 9 tests should **PASS**:

```
test test_activate_hunt_with_zero_required_clues_fails ... ok
test test_activate_hunt_with_one_required_clue_succeeds ... ok
test test_activate_hunt_after_adding_required_clue ... ok
test test_activate_hunt_with_multiple_required_clues_succeeds ... ok
test test_activate_hunt_all_clues_required_succeeds ... ok
test test_cannot_activate_hunt_with_only_required_clues_zero ... ok
test test_required_clue_count_tracks_correctly ... ok
test test_activate_hunt_boundary_one_required_clue ... ok
test test_unauthorized_user_cannot_activate ... ok

test result: ok. 9 passed; 0 failed; 0 ignored
```

---

## Test Coverage

### Business Rules Verified

✅ **Rule 1:** Hunt cannot be activated with zero required clues  
✅ **Rule 2:** Hunt can be activated with exactly one required clue  
✅ **Rule 3:** Hunt can be activated with multiple required clues  
✅ **Rule 4:** Optional clues do not satisfy requirement  
✅ **Rule 5:** Required clues counter tracks correctly  
✅ **Rule 6:** Authorization still enforced during activation  
✅ **Rule 7:** Empty hunts still fail (NoCluesAdded before NoRequiredClues)

### Edge Cases Tested

✅ **Boundary:** Exactly 1 required clue (minimum valid)  
✅ **Boundary:** 0 required with multiple optional clues  
✅ **Mixed:** Combination of required and optional clues  
✅ **Sequential:** Adding required clue changes outcome  
✅ **Authorization:** Non-creator cannot activate  

---

## Key Test Patterns

### Pattern 1: Setup Environment
```rust
let env = Env::default();
env.ledger().set_timestamp(1_700_000_000);
env.mock_all_auths();

let core_id = setup_core_contract(&env);
let creator = Address::generate(&env);
```

### Pattern 2: Create Hunt with Clues
```rust
let hunt_id = HuntyCore::create_hunt(
    env.clone(),
    creator.clone(),
    String::from_str(env, "Hunt Title"),
    String::from_str(env, "Description"),
    None,
    None,
    0,
    None,
).unwrap();

// Add optional clue
HuntyCore::add_clue(
    env.clone(),
    hunt_id,
    question,
    answer,
    10,
    false,  // is_required
    None,
).unwrap();

// Add required clue
HuntyCore::add_clue(
    env.clone(),
    hunt_id,
    question,
    answer,
    20,
    true,   // is_required
    None,
).unwrap();
```

### Pattern 3: Verify State Before Activation
```rust
let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
assert_eq!(hunt.total_clues, 4);
assert_eq!(hunt.required_clues, 1);
```

### Pattern 4: Test Activation
```rust
let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
assert!(result.is_ok(), "Activation should succeed");

// or

assert!(result.is_err(), "Activation should fail");
```

---

## Interaction with Other Features

### Player Completion
The `check_all_required_clues_completed` function verifies players complete all required clues before hunt completion is recognized.

### Leaderboard
Leaderboards may prioritize hunts with required clues to ensure meaningful completion criteria.

### Reward Distribution
Rewards are only distributed once all required clues are completed.

---

## Related Code

- [Hunt Types](./contracts/hunty-core/src/types.rs#L52) - Hunt struct with required_clues field
- [Error Codes](./contracts/hunty-core/src/errors.rs#L25) - NoRequiredClues error definition
- [Add Clue Implementation](./contracts/hunty-core/src/lib.rs#L158) - add_clue function
- [Activate Hunt Implementation](./contracts/hunty-core/src/lib.rs#L328) - activate_hunt function
- [Main Tests](./contracts/hunty-core/tests/test.rs) - Other integration tests

---

## Future Enhancements

- [ ] Test concurrent activation attempts
- [ ] Test TTL behavior for required clues metadata
- [ ] Test storage gas costs with varying required/optional ratios
- [ ] Test leaderboard behavior based on required clue completion
- [ ] Add metrics for required vs optional clue completion rates

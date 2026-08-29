# TODO - Compact player registration storage (hunty-core)

## Step 1: Implement bit flags for boolean fields

- File: `contracts/hunty-core/src/types.rs`
- Change `StoredPlayerProgress`:
  - remove `is_completed: bool`, `reward_claimed: bool`
  - add `flags: u8` with bits for both
- Update `PlayerProgress::to_stored()` and `PlayerProgress::from_stored()` accordingly
- ✅ Done (in this PR)

## Step 2: Ensure compilation/test fixes

- File: `contracts/hunty-core/src/test.rs`
- Fix any tests/uses that directly access stored boolean fields (public view should remain unchanged)

## Step 3: Add benchmark-style test harness (CI)

- Add a lightweight test that repeatedly registers/saves player progress and asserts functional correctness
- Optionally compare/record gas usage by measuring test execution pattern (best-effort)

## Step 4: Document results

- Note that timestamp packing was not changed (safety), only boolean flags were packed
- Provide expected/observed footprint reduction plan

# NFT Total Supply Cap Implementation

## Summary
This implementation adds support for enforcing a maximum total supply cap on NFT rewards in the Hunty system.

## Features Implemented

### 1. **Max Supply Configuration** 
- Initialize contracts with an optional max supply cap
- `max_supply: Option<u64>` parameter in `initialize()` function
- When `None`, supply is unlimited
- When `Some(n)`, maximum `n` NFTs can be minted

### 2. **Mint-Time Validation**
- Before each NFT mint, the contract checks if the supply limit has been reached
- If `current_supply >= max_supply`, minting fails with `MaxSupplyReached` error
- Provides protection against over-minting

### 3. **Query Functions**

#### `get_max_supply(env: Env) -> u64`
Returns the configured maximum supply:
- `0` if unlimited (no cap set)
- Positive integer if cap is enforced

**Example:**
```rust
let max_supply = client.get_max_supply();
if max_supply == 0 {
    println!("Unlimited supply");
} else {
    println!("Max supply: {}", max_supply);
}
```

#### `get_remaining_supply(env: Env) -> u64`
Returns the number of NFTs that can still be minted:
- `u64::MAX` if supply is unlimited
- `0` if at or above capacity
- `max - current` for capped supplies

**Example:**
```rust
let remaining = client.get_remaining_supply();
if remaining == u64::MAX {
    println!("Unlimited remaining");
} else {
    println!("Can mint {} more NFTs", remaining);
}
```

## Code Changes

### [lib.rs](contracts/nft-reward/src/lib.rs)

#### 1. Updated `NftData` struct
```rust
pub struct NftData {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub completion_player: Address,      // NEW: Track original completer
    pub metadata: NftMetadata,
    pub transferable: bool,
    pub minted_at: u64,
}
```

#### 2. New query functions
```rust
pub fn get_max_supply(env: Env) -> u64 { ... }
pub fn get_remaining_supply(env: Env) -> u64 { ... }
```

### [storage.rs](contracts/nft-reward/src/storage.rs)

Cleaned up duplicate/conflicting functions:
- Removed unused `save_admin()`, `get_admin()` methods
- Removed unused `add_minter()`, `remove_minter()`, `is_minter()` methods
- Consolidated duplicate `get_max_supply()` and `is_initialized()` implementations
- Used consistent instance storage for initialization and max_supply

## Testing

### Running Tests
```bash
cd contracts/nft-reward
make test
# or
cargo test
```

### New Test Cases
The following tests verify the implementation:

1. **test_get_remaining_supply_unlimited**
   - Verifies that unlimited supply returns `u64::MAX`

2. **test_get_remaining_supply_with_limit**
   - Verifies that a limit is correctly stored and returned

3. **test_get_remaining_supply_decreases_after_mint**
   - Verifies that remaining supply decreases with each mint

4. **test_get_remaining_supply_at_capacity**
   - Verifies that remaining supply becomes 0 at capacity

5. **test_get_max_supply_returns_zero_for_unlimited**
   - Verifies that unlimited returns 0

6. **test_get_max_supply_returns_actual_value**
   - Verifies that limits are correctly returned

### Existing Tests Still Pass
- `test_max_supply_enforced` - Verifies panic on over-supply
- `test_no_max_supply_allows_unlimited_mints` - Verifies unlimited works
- `test_max_supply_cap_blocks_additional_mints` - Verifies blocking

## Usage Example

```rust
// Initialize with a cap of 100 NFTs
let client = NftRewardClient::new(&env, &contract_id);
client.initialize(&Some(100));

// Query the limits
let max = client.get_max_supply();        // Returns 100
let remaining = client.get_remaining_supply(); // Returns 100

// Mint NFTs
for i in 1..=50 {
    let nft_id = client.mint_reward_nft(/* ... */);
    let remaining = client.get_remaining_supply();
    println!("Remaining: {}", remaining); // Decreases: 99, 98, ...
}

// After 100 mints, this would fail:
client.mint_reward_nft(/* ... */); // Error: MaxSupplyReached
```

## Acceptance Criteria Met

✅ **Set max_supply during initialization**
- Implemented via `initialize(env, max_supply: Option<u64>)`
- Stored in instance storage for safety and gas efficiency

✅ **Check before each mint**
- Implemented in `mint_reward_nft_impl()`
- Returns `MaxSupplyReached` error when limit exceeded

✅ **get_remaining_supply query**
- New public function returns remaining mintable NFTs
- Returns `u64::MAX` for unlimited supplies
- Returns `0` when at or above capacity

✅ **0 means unlimited**
- When `max_supply` is `None`, supply is unlimited
- `get_max_supply()` returns `0` for unlimited
- `get_remaining_supply()` returns `u64::MAX` for unlimited

## Security Considerations

1. **Atomic Storage**: Max supply is set atomically during initialization
2. **Immutable After Init**: Once set, max supply cannot be changed
3. **Gas Efficient**: Uses instance storage (low-cost) not persistent storage
4. **Error Handling**: Panics with specific error code for clear failure feedback

## Migration Notes

- NFTs minted before this update will work normally
- Existing contracts can upgrade to support supply caps via re-initialization
- The `completion_player` field in `NftData` properly tracks hunt completion regardless of ownership transfers

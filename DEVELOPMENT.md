# Development Guide

## Quick Start

### Prerequisites Installation


**macOS:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stellar CLI
# Follow instructions at https://soroban.stellar.org/docs/getting-started/setup
```

**Linux:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stellar CLI
# Follow instructions at https://soroban.stellar.org/docs/getting-started/setup
```

**Windows:**

```bash
# Install Rust
# Download from https://rustup.rs/

# Install Stellar CLI
# Follow instructions at https://soroban.stellar.org/docs/getting-started/setup
```

### Verify Installation

```bash
rustc --version
cargo --version
stellar --version
```

### Project Setup

1. **Clone and navigate:**

```bash
git clone https://github.com/Samuel1-ona/Hunty-contract.git
cd Hunty-contract
```

2. **Build all contracts:**

```bash
# Build hunty-core
cd contracts/hunty-core
make build

# Build reward-manager
cd ../reward-manager
make build

# Build nft-reward
cd ../nft-reward
make build
```

3. **Run tests:**

```bash
# From each contract directory
make test
```

---

## Storage Architecture & TTL Management

> **This section is required reading before modifying any contract storage or deploying to mainnet.**
> Soroban's rent / TTL model differs fundamentally from EVM storage — data that is not periodically
> "bumped" will be **archived (evicted) automatically** by the network after its time-to-live expires.
> Evicted data is not deleted forever, but restoring it requires a separate `restore_footprint`
> transaction and a fee; if operators are unaware of this, live hunts can become silently unplayable.

### Soroban Storage Types — Primer

Soroban exposes three storage buckets, each with different TTL defaults and use cases:

| Type           | Accessed via                 | Default TTL (Futurenet / Testnet)                                                                    | Typical use                                                                |
| -------------- | ---------------------------- | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| **Instance**   | `env.storage().instance()`   | Tied to the contract instance entry; extended whenever the contract is invoked                       | Admin config, contract-level counters, flags that must always be available |
| **Persistent** | `env.storage().persistent()` | Network-configured minimum (≈ 120 days on mainnet at time of writing); **not** auto-bumped on invoke | Per-hunt data, per-player progress, clue answers, reward pools             |
| **Temporary**  | `env.storage().temporary()`  | Short-lived (minutes to hours); **auto-deleted** when expired, not archivable                        | Nonces, short-lived session flags — **never used for game state**          |

> The exact ledger counts for minimum/maximum TTL are governance-controlled and may change. Always
> check the [Stellar Network Settings](https://stellar.expert/explorer/mainnet/network-settings) for
> current values before deploying.

### Key → Storage Type Mapping

The table below documents every storage key used across the three Hunty contracts and which storage
type it lives in. Keep this table up-to-date whenever a new key is introduced.

#### HuntyCore (`contracts/hunty-core/src/storage.rs`)

| Storage Key                                | Type           | Description                                                          | Eviction risk                                                                         |
| ------------------------------------------ | -------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `DataKey::Hunt(hunt_id)`                   | **Persistent** | Full `Hunt` struct (title, description, status, creator, clue count) | High — must be bumped while hunt is active                                            |
| `DataKey::Clue(hunt_id, clue_id)`          | **Persistent** | `Clue` struct including SHA-256 answer hash                          | High — unreadable clues silently block answer verification                            |
| `DataKey::PlayerProgress(hunt_id, player)` | **Persistent** | `PlayerProgress` struct (completed clues, score, timestamps)         | High — lost progress means players cannot complete hunts                              |
| `DataKey::HuntPlayers(hunt_id)`            | **Persistent** | `Vec<Address>` of registered players (used by leaderboard)           | Medium — leaderboard queries degrade gracefully, but re-registration would be blocked |
| `DataKey::HuntCount`                       | **Instance**   | Monotonic counter used to generate hunt IDs                          | Low — bumped automatically on every invocation                                        |
| `DataKey::RewardManager`                   | **Instance**   | Address of the registered `RewardManager` contract                   | Low — bumped automatically on every invocation                                        |

#### RewardManager (`contracts/reward-manager/src/lib.rs`)

| Storage Key                    | Type           | Description                                                         | Eviction risk                                                                        |
| ------------------------------ | -------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `DataKey::RewardPool(hunt_id)` | **Persistent** | `RewardPool` struct (balance, min distribution amount, funded flag) | **Critical** — eviction causes reward distribution to fail with a storage-miss panic |
| `DataKey::NftRewardContract`   | **Instance**   | Address of the `NftReward` contract                                 | Low — bumped automatically on every invocation                                       |
| `DataKey::Admin`               | **Instance**   | Admin `Address` authorised to configure the contract                | Low — bumped automatically on every invocation                                       |
| `DataKey::XlmToken`            | **Instance**   | SAC address for native XLM                                          | Low — bumped automatically on every invocation                                       |

#### NftReward (`contracts/nft-reward/src/lib.rs`)

| Storage Key            | Type           | Description                                         | Eviction risk                                              |
| ---------------------- | -------------- | --------------------------------------------------- | ---------------------------------------------------------- |
| `DataKey::Nft(nft_id)` | **Persistent** | `NftData` struct (owner, hunt_id, player, metadata) | High — evicted NFTs appear burned to any off-chain indexer |
| `DataKey::NftCount`    | **Instance**   | Monotonic counter for NFT IDs                       | Low — bumped automatically                                 |

### TTL Bump Strategy

#### What "bumping" means

Calling `env.storage().persistent().extend_ttl(&key, threshold, extend_to)` restores the remaining
TTL of a persistent entry to `extend_to` ledgers **only if** its current remaining TTL has fallen
below `threshold`. This prevents unnecessary writes and keeps fees low.

#### Recommended bump constants

Define these in a shared `ttl.rs` (or at the top of `storage.rs`) per contract:

```rust
/// Minimum remaining TTL before we extend (≈ 30 days at 5s/ledger).
pub const TTL_BUMP_THRESHOLD: u32 = 518_400;

/// Target TTL after a bump (≈ 1 year at 5s/ledger).
pub const TTL_BUMP_TARGET: u32 = 6_307_200;
```

> **Calibration note**: Stellar's ledger closes approximately every 5 seconds, giving ~6 307 200
> ledgers per year. Adjust `TTL_BUMP_TARGET` to match the expected lifetime of the data (e.g. a
> 90-day hunt needs at minimum 90 days × 17 280 ledgers/day ≈ 1 555 200 ledgers).

#### Where to call `extend_ttl`

Bump persistent storage entries at **read time** (inside helper getters in `storage.rs`), not only
at write time. This ensures that data accessed frequently during active hunts keeps its TTL refreshed
without any additional operator intervention.

**Pattern — bump-on-read in `storage.rs`:**

```rust
pub fn get_hunt(env: &Env, hunt_id: u64) -> Option<Hunt> {
    let key = DataKey::Hunt(hunt_id);
    let result = env.storage().persistent().get::<DataKey, Hunt>(&key);
    if result.is_some() {
        env.storage().persistent().extend_ttl(
            &key,
            TTL_BUMP_THRESHOLD,
            TTL_BUMP_TARGET,
        );
    }
    result
}

pub fn get_player_progress(env: &Env, hunt_id: u64, player: &Address) -> Option<PlayerProgress> {
    let key = DataKey::PlayerProgress(hunt_id, player.clone());
    let result = env.storage().persistent().get::<DataKey, PlayerProgress>(&key);
    if result.is_some() {
        env.storage().persistent().extend_ttl(
            &key,
            TTL_BUMP_THRESHOLD,
            TTL_BUMP_TARGET,
        );
    }
    result
}
```

**Pattern — bump-on-write:**

```rust
pub fn set_hunt(env: &Env, hunt_id: u64, hunt: &Hunt) {
    let key = DataKey::Hunt(hunt_id);
    env.storage().persistent().set(&key, hunt);
    env.storage().persistent().extend_ttl(
        &key,
        TTL_BUMP_THRESHOLD,
        TTL_BUMP_TARGET,
    );
}
```

#### Instance storage bump

Instance storage is bumped automatically whenever the contract is invoked, but you may also bump it
explicitly at the top of each contract function for extra safety:

```rust
env.storage().instance().extend_ttl(TTL_BUMP_THRESHOLD, TTL_BUMP_TARGET);
```

### Operator Runbook — Monitoring & Manual Bumps

Even with bump-on-read in place, operators should monitor storage TTLs independently, because:

- **Inactive hunts** (no player activity for weeks) will not trigger bump-on-read.
- **Reward pools** for future hunts may sit idle for months before the hunt is activated.

#### Checking remaining TTL via Stellar CLI

```bash
# Check the TTL of a specific contract data entry (persistent)
stellar contract data get \
  --id <CONTRACT_ID> \
  --key '<XDR_KEY>' \
  --network mainnet \
  --durability persistent

# The response includes `live_until_ledger`. Compare to current ledger:
stellar ledgers --network mainnet | jq '.sequence'
```

#### Manually bumping an entry

If an entry's TTL is dangerously low, bump it using a `restore_footprint` or `extend_footprint_ttl`
operation before it expires:

```bash
# Extend the contract instance (covers all instance-storage keys)
stellar contract extend \
  --id <CONTRACT_ID> \
  --ledgers-to-extend 6307200 \
  --source <ADMIN_KEYPAIR> \
  --network mainnet \
  --durability persistent
```

> The `stellar contract extend` command targets the contract instance entry. To bump individual
> persistent data entries (e.g. a specific `RewardPool`), you currently need a dedicated contract
> invocation that calls `extend_ttl` internally, or use the Stellar SDK to construct a
> `BumpFootprintExpirationOp` directly.

#### Recommended monitoring schedule

| Data category                                 | Check frequency           | Alert threshold         |
| --------------------------------------------- | ------------------------- | ----------------------- |
| Active hunt data (Hunt, Clue, PlayerProgress) | Daily during active hunts | < 7 days remaining TTL  |
| Reward pools for scheduled hunts              | Weekly                    | < 30 days remaining TTL |
| NFT records                                   | Monthly                   | < 60 days remaining TTL |
| Instance storage (all contracts)              | Monthly                   | < 30 days remaining TTL |

### Known Risks & Edge Cases

| Risk                       | Scenario                                                                                               | Mitigation                                                                                       |
| -------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| **Silent answer lock-out** | `Clue` entries expire mid-hunt; players receive a storage-miss error on `submit_answer`                | Bump clue entries in `add_clue` and in `get_clue`; alert if TTL < 7 days                         |
| **Lost player progress**   | `PlayerProgress` entry expires before `complete_hunt` is called                                        | Bump in `register_player`, `submit_answer`, and `complete_hunt`                                  |
| **Reward pool eviction**   | `RewardPool` expires before hunt is completed and rewards are distributed; `distribute_rewards` panics | Bump `RewardPool` when pool is created and funded; set TTL to at least `hunt_end_date + 90 days` |
| **Orphaned NFT**           | `Nft(nft_id)` entry expires; NFT appears burned to off-chain tools but is actually archivable          | Bump on mint; schedule periodic operator bump for all NFT IDs                                    |
| **Hunt-count desync**      | `HuntCount` (instance) expires and resets; new hunts get IDs that collide with old data                | Instance storage is bumped on every invocation — this risk is negligible in practice             |

### References

- [Soroban Storage & State Archival Documentation](https://developers.stellar.org/docs/learn/encyclopedia/storage/state-archival)
- [Stellar Network Settings (live TTL values)](https://stellar.expert/explorer/mainnet/network-settings)
- [BumpFootprintExpirationOp XDR reference](https://developers.stellar.org/docs/learn/fundamentals/stellar-data-structures/operations-and-transactions)

---

## Development Workflow

### Working on a Feature

1. **Create a feature branch:**

```bash
git checkout -b feature/your-feature-name
```

2. **Make changes:**

   - Edit source files
   - Add tests
   - Update documentation

3. **Test your changes:**

```bash
make test
make build
```

4. **Format code:**

```bash
make fmt
```

5. **Commit and push:**

```bash
git add .
git commit -m "feat: description of changes"
git push origin feature/your-feature-name
```

### Running Tests

**Individual contract tests:**

```bash
cd contracts/hunty-core
cargo test
```

**All tests:**

```bash
cargo test --workspace
```

**With output:**

```bash
cargo test -- --nocapture
```

### Building Contracts

**Build a single contract:**

```bash
cd contracts/hunty-core
make build
```

**Build all contracts:**

```bash
# From project root
for dir in contracts/*/; do
  cd "$dir" && make build && cd ../..
done
```

**Check build output:**

```bash
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

## Code Organization

### HuntyCore Contract

**File Structure:**

- `lib.rs` - Main contract implementation
- `types.rs` - Data structures (Hunt, Clue, PlayerProgress)
- `storage.rs` - Storage access patterns
- `errors.rs` - Custom error types
- `test.rs` - Test suite

**Key Functions to Implement:**

- `create_hunt()` - Create new hunt
- `add_clue()` - Add clue to hunt
- `register_player()` - Register player for hunt
- `submit_answer()` - Submit and verify answer
- `complete_hunt()` - Mark hunt complete

## Security Notes

- Never forward `Hunt.description` into NFT metadata during reward distribution.
- The cross-contract NFT path must only pass public hunt fields such as title and pre-approved metadata.
- Keep this rule covered by tests, because the code comment alone is advisory and can be missed during refactors.

### RewardManager Contract

**File Structure:**

- `lib.rs` - Main reward distribution logic
- `xlm_handler.rs` - XLM token handling
- `nft_handler.rs` - NFT coordination
- `test.rs` - Test suite

**Key Functions to Implement:**

- `distribute_rewards()` - Main distribution entry
- `handle_xlm_rewards()` - XLM transfer logic
- `handle_nft_rewards()` - NFT minting coordination

### NftReward Contract

**File Structure:**

- `lib.rs` - NFT contract implementation
- `test.rs` - Test suite

**Key Functions to Implement:**

- `mint_reward_nft()` - Mint NFT for reward
- `transfer_nft()` - Transfer NFT to player
- `get_nft_metadata()` - Retrieve NFT info

## Testing Guidelines

### Unit Tests

Test individual functions:

```rust
#[test]
fn test_create_hunt() {
    let env = Env::default();
    // Test implementation
}
```

### Integration Tests

Test cross-contract interactions:

```rust
#[test]
fn test_reward_distribution() {
    // Test HuntyCore -> RewardManager -> NftReward flow
}
```

### Test Coverage

Aim for >80% code coverage. Run:

```bash
cargo test --workspace -- --nocapture
```

## Debugging

### Common Issues

1. **Build errors:**

   - Check Rust version: `rustc --version`
   - Clean and rebuild: `make clean && make build`

2. **Test failures:**

   - Run with output: `cargo test -- --nocapture`
   - Check error messages carefully

3. **Storage issues:**
   - Verify storage keys are unique
   - Check data serialization
   - **Check TTL** — see [Storage Architecture & TTL Management](#storage-architecture--ttl-management)

### Debug Tools

**Print debugging:**

```rust
env.logs().add("Debug message", &value);
```

**Check storage:**

```rust
// In tests
let stored_value = env.storage().get(&key);
```

## Code Style

### Formatting

Always format before committing:

```bash
make fmt
# or
cargo fmt --all
```

### Naming Conventions

- Functions: `snake_case`
- Types: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`
- Storage keys: `snake_case`

### Documentation

Add doc comments:

```rust
/// Creates a new hunt with the given parameters.
///
/// # Arguments
/// * `env` - The environment
/// * `creator` - Address of the hunt creator
///
/// # Returns
/// Hunt ID
pub fn create_hunt(env: Env, creator: Address) -> u64 {
    // Implementation
}
```

## Deployment

Hunty requires deploying three contracts in the correct order and wiring them together. The steps below cover both **testnet** and **mainnet**. Replace `--network testnet` with `--network mainnet` (and use a funded mainnet key) for production deployments.

### Prerequisites

1. **Stellar CLI** installed and on your PATH (`stellar --version`).
2. A funded deployer keypair. On testnet, use the friendbot:

```bash
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
```

3. All contracts built (`.wasm` files present):

```bash
cargo build --target wasm32-unknown-unknown --release
ls target/wasm32-unknown-unknown/release/*.wasm
```

### Step 1 — Deploy NftReward

NftReward has no initializer, so it can be deployed and used immediately.

```bash
NFT_CONTRACT=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/nft_reward.wasm \
  --source deployer \
  --network testnet)

echo "NftReward: $NFT_CONTRACT"
```

### Step 2 — Deploy RewardManager

```bash
REWARD_MANAGER=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/reward_manager.wasm \
  --source deployer \
  --network testnet)

echo "RewardManager: $REWARD_MANAGER"
```

#### 2a — Identify the XLM SAC address

The XLM Stellar Asset Contract address differs by network.

| Network | XLM SAC address                                            |
| ------- | ---------------------------------------------------------- |
| Testnet | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |
| Mainnet | `CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA` |

> Tip: verify with `stellar contract id asset --asset native --network testnet`.

```bash
# Testnet
XLM_SAC="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
```

#### 2b — Initialize RewardManager

`initialize` sets the admin keypair and the XLM SAC. It can only be called once.

```bash
DEPLOYER_ADDRESS=$(stellar keys address deployer)

stellar contract invoke \
  --id "$REWARD_MANAGER" \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin "$DEPLOYER_ADDRESS" \
  --xlm_token "$XLM_SAC"
```

#### 2c — Register NftReward with RewardManager

```bash
stellar contract invoke \
  --id "$REWARD_MANAGER" \
  --source deployer \
  --network testnet \
  -- set_nft_reward_contract \
  --admin "$DEPLOYER_ADDRESS" \
  --nft_contract "$NFT_CONTRACT"
```

### Step 3 — Deploy HuntyCore

```bash
HUNTY_CORE=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hunty_core.wasm \
  --source deployer \
  --network testnet)

echo "HuntyCore: $HUNTY_CORE"
```

#### 3a — Register RewardManager with HuntyCore

`set_reward_manager` tells HuntyCore where to send reward-distribution calls.

```bash
stellar contract invoke \
  --id "$HUNTY_CORE" \
  --source deployer \
  --network testnet \
  -- set_reward_manager \
  --reward_manager "$REWARD_MANAGER"
```

### Step 4 — Persist contract addresses

Save the addresses so they can be reused across sessions and by your frontend.

```bash
cat << EOF > .env.testnet
HUNTY_CORE=$HUNTY_CORE
REWARD_MANAGER=$REWARD_MANAGER
NFT_CONTRACT=$NFT_CONTRACT
XLM_SAC=$XLM_SAC
NETWORK=testnet
EOF
```

### Step 5 — Fund a reward pool (hunt creator workflow)

Before a hunt can pay out XLM rewards, its pool must be created and funded.
Amounts are in **stroops** (1 XLM = 10 000 000 stroops).

```bash
HUNT_ID=1          # replace with your hunt ID after create_hunt
AMOUNT=100000000   # 10 XLM in stroops

# Create the pool
stellar contract invoke \
  --id "$REWARD_MANAGER" \
  --source deployer \
  --network testnet \
  -- create_reward_pool \
  --creator "$DEPLOYER_ADDRESS" \
  --hunt_id "$HUNT_ID" \
  --min_distribution_amount 0

# Fund the pool (transfers XLM from the creator's account)
stellar contract invoke \
  --id "$REWARD_MANAGER" \
  --source deployer \
  --network testnet \
  -- fund_reward_pool \
  --funder "$DEPLOYER_ADDRESS" \
  --hunt_id "$HUNT_ID" \
  --amount "$AMOUNT"
```

### Step 6 — Verify the deployment

```bash
# Check reward pool status
stellar contract invoke \
  --id "$REWARD_MANAGER" \
  --source deployer \
  --network testnet \
  -- get_reward_pool \
  --hunt_id "$HUNT_ID"

# Check NftReward supply (should be 0 before any completions)
stellar contract invoke \
  --id "$NFT_CONTRACT" \
  --source deployer \
  --network testnet \
  -- total_supply
```

### Mainnet checklist

- Use a hardware wallet or a dedicated deployment keypair; never use a hot key holding user funds.
- Replace `--network testnet` with `--network mainnet` in every command above.
- Use the mainnet XLM SAC: `CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA`.
- Verify each contract ID with `stellar contract info --id <ID> --network mainnet` before calling `initialize`.
- Keep `.env.mainnet` out of version control (add it to `.gitignore`).
- **Review the [Storage Architecture & TTL Management](#storage-architecture--ttl-management) section** and confirm bump-on-read is implemented before deploying.

## Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar SDK Reference](https://docs.rs/soroban-sdk/)
- [Rust Book](https://doc.rust-lang.org/book/)

## Getting Help

- Check [ARCHITECTURE.md](ARCHITECTURE.md) for system design
- Review [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
- Open an issue on GitHub for questions

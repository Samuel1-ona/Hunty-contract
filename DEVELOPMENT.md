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
```powershell
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

| Network  | XLM SAC address |
|----------|----------------|
| Testnet  | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |
| Mainnet  | `CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA` |

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

## Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar SDK Reference](https://docs.rs/soroban-sdk/)
- [Rust Book](https://doc.rust-lang.org/book/)

## Getting Help

- Check [ARCHITECTURE.md](ARCHITECTURE.md) for system design
- Review [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
- Open an issue on GitHub for questions



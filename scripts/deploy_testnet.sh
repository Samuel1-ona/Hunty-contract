#!/usr/bin/env bash
# deploy_testnet.sh — Automated testnet deployment for Hunty contracts
#
# USAGE:
#   [TESTNET_ADMIN_ADDRESS=G...] \
#   bash scripts/deploy_testnet.sh

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
RPC_URL="https://soroban-testnet.stellar.org"
DEPLOY_DIR="docs/deployment-testnet"
ADDRESS_FILE="${DEPLOY_DIR}/deployed-addresses.md"
CONTRACTS=("nft-reward" "reward-manager" "hunty-core")
WASM_NAMES=("nft_reward" "reward_manager" "hunty_core")

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"; }
die()  { echo "ERROR: $*" >&2; exit 1; }

require_cmd() { command -v "$1" &>/dev/null || die "'$1' not found in PATH"; }
require_cmd stellar
require_cmd sha256sum
require_cmd jq

mkdir -p "$DEPLOY_DIR"

# ── Step 1: Set up deployer keys ─────────────────────────────────────────────
log "=== Hunty Testnet Deployment ==="

# Check if deployer key exists, if not generate it (which automatically funds it)
if ! stellar keys address deployer &>/dev/null; then
  log "Generating and funding testnet deployer key..."
  stellar keys generate --network testnet deployer
else
  log "Using existing deployer key."
fi

DEPLOYER_ADDRESS=$(stellar keys address deployer)
ADMIN_ADDRESS="${TESTNET_ADMIN_ADDRESS:-$DEPLOYER_ADDRESS}"

log "Deployer Address : $DEPLOYER_ADDRESS"
log "Admin Address    : $ADMIN_ADDRESS"
log "RPC URL         : $RPC_URL"

# ── Step 2: Build WASM ───────────────────────────────────────────────────────
log "Building contracts..."
stellar contract build

# Handle differences in build target directory output
if [ -d "target/wasm32v1-none/release" ]; then
  WASM_DIR="target/wasm32v1-none/release"
else
  WASM_DIR="target/wasm32-unknown-unknown/release"
fi
log "Using WASM directory: $WASM_DIR"

# Record WASM hashes
log "WASM artefact hashes:"
MANIFEST_FILE="${DEPLOY_DIR}/manifest.txt"
: > "$MANIFEST_FILE"
for wasm_name in "${WASM_NAMES[@]}"; do
  wasm_path="${WASM_DIR}/${wasm_name}.wasm"
  [[ -f "$wasm_path" ]] || die "WASM not found: $wasm_path"
  hash=$(sha256sum "$wasm_path" | awk '{print $1}')
  echo "  ${wasm_name}.wasm  sha256:${hash}"
  echo "${wasm_name}.wasm  sha256:${hash}" >> "$MANIFEST_FILE"
done

# ── Step 3: Query Native SAC Address ─────────────────────────────────────────
log "Querying native XLM token address on testnet..."
XLM_TOKEN_ADDRESS=$(stellar contract id asset --asset native --network testnet)
log "Native XLM token address: $XLM_TOKEN_ADDRESS"

# ── Step 4: Deploy contracts (dependency order) ────────────────────────────────
declare -A NEW_IDS
for i in "${!CONTRACTS[@]}"; do
  contract="${CONTRACTS[$i]}"
  wasm_name="${WASM_NAMES[$i]}"
  wasm_path="${WASM_DIR}/${wasm_name}.wasm"

  log "Deploying $contract..."

  # Upload WASM and capture hash
  wasm_hash=$(stellar contract upload \
    --wasm "$wasm_path" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source deployer)

  log "  WASM uploaded: $wasm_hash"

  # Deploy contract instance
  contract_id=$(stellar contract deploy \
    --wasm-hash "$wasm_hash" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source deployer)

  log "  Contract ID  : $contract_id"
  NEW_IDS["$contract"]="$contract_id"
done

# ── Step 5: Initialize and link contracts ─────────────────────────────────────
log "Initializing contracts..."

# 1. Initialize nft-reward
stellar contract invoke \
  --id "${NEW_IDS[nft-reward]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source deployer \
  -- initialize \
  --admin "$ADMIN_ADDRESS"
log "  nft-reward initialized."

# 2. Initialize reward-manager
stellar contract invoke \
  --id "${NEW_IDS[reward-manager]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source deployer \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --xlm_token "$XLM_TOKEN_ADDRESS"
log "  reward-manager initialized."

# 3. Link nft-reward to reward-manager
stellar contract invoke \
  --id "${NEW_IDS[reward-manager]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source deployer \
  -- set_nft_reward_contract \
  --admin "$ADMIN_ADDRESS" \
  --nft_contract "${NEW_IDS[nft-reward]}"
log "  nft-reward linked to reward-manager."

# 4. Initialize hunty-core
stellar contract invoke \
  --id "${NEW_IDS[hunty-core]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source deployer \
  -- initialize_admin \
  --admin "$ADMIN_ADDRESS"
log "  hunty-core initialized."

# 5. Link reward-manager to hunty-core
stellar contract invoke \
  --id "${NEW_IDS[hunty-core]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source deployer \
  -- set_reward_manager \
  --admin "$ADMIN_ADDRESS" \
  --reward_manager "${NEW_IDS[reward-manager]}"
log "  reward-manager linked to hunty-core."

# ── Step 6: Write deployed addresses ──────────────────────────────────────────
TIMESTAMP=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
cat >> "$ADDRESS_FILE" <<EOF

## Deployment — $TIMESTAMP

| Contract | ID |
|---|---|
| nft-reward | ${NEW_IDS[nft-reward]} |
| reward-manager | ${NEW_IDS[reward-manager]} |
| hunty-core | ${NEW_IDS[hunty-core]} |

WASM manifest: $MANIFEST_FILE
EOF

log ""
log "=== Testnet Deployment Complete ==="
log "Addresses saved to $ADDRESS_FILE"
log "To verify deployment, run:"
log "  ./scripts/verify_deployment.sh testnet"

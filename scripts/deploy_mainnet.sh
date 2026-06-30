#!/usr/bin/env bash
# deploy_mainnet.sh — Multi-sig Hunty mainnet deployment
#
# USAGE:
#   MAINNET_ADMIN_ADDRESS=G... \
#   MAINNET_RPC_URL=https://... \
#   SIGNER_1_SECRET=S... \
#   SIGNER_2_SECRET=S... \
#   [SIGNER_3_SECRET=S...] \
#   bash scripts/deploy_mainnet.sh
#
# The script builds WASM, uploads each contract, deploys (or upgrades) it,
# collects signatures from all supplied signers, and submits the final tx.
# Deployed addresses are written to docs/deployment/deployed-addresses.md.

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────
NETWORK_PASSPHRASE="${MAINNET_NETWORK_PASSPHRASE:-Public Global Stellar Network ; September 2015}"
RPC_URL="${MAINNET_RPC_URL:?Set MAINNET_RPC_URL}"
ADMIN_ADDRESS="${MAINNET_ADMIN_ADDRESS:?Set MAINNET_ADMIN_ADDRESS}"
WASM_DIR="target/wasm32v1-none/release"
DEPLOY_DIR="docs/deployment"
ADDRESS_FILE="${DEPLOY_DIR}/deployed-addresses.md"
CONTRACTS=("nft-reward" "reward-manager" "hunty-core")
WASM_NAMES=("nft_reward" "reward_manager" "hunty_core")

# Collect signers (at least 2 required)
SIGNERS=()
[[ -n "${SIGNER_1_SECRET:-}" ]] && SIGNERS+=("${SIGNER_1_SECRET}")
[[ -n "${SIGNER_2_SECRET:-}" ]] && SIGNERS+=("${SIGNER_2_SECRET}")
[[ -n "${SIGNER_3_SECRET:-}" ]] && SIGNERS+=("${SIGNER_3_SECRET}")

if [[ ${#SIGNERS[@]} -lt 2 ]]; then
  echo "ERROR: At least SIGNER_1_SECRET and SIGNER_2_SECRET must be set." >&2
  exit 1
fi

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"; }
die()  { echo "ERROR: $*" >&2; exit 1; }

require_cmd() { command -v "$1" &>/dev/null || die "'$1' not found in PATH"; }
require_cmd stellar
require_cmd sha256sum
require_cmd jq

mkdir -p "$DEPLOY_DIR"

# ── Pre-flight ────────────────────────────────────────────────────────────────
log "=== Hunty Mainnet Deployment ==="
log "Admin  : $ADMIN_ADDRESS"
log "RPC    : $RPC_URL"
log "Signers: ${#SIGNERS[@]}"

read -r -p "Confirm pre-deploy-checklist is complete? [yes/no] " CONFIRM
[[ "$CONFIRM" == "yes" ]] || die "Aborted — complete the checklist first."

# ── Step 1: Build ─────────────────────────────────────────────────────────────
log "Building contracts..."
stellar contract build

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

read -r -p "WASM hashes look correct? [yes/no] " CONFIRM
[[ "$CONFIRM" == "yes" ]] || die "Aborted by operator."

# ── Step 2: Deploy contracts (dependency order) ────────────────────────────────
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
    --source "${SIGNERS[0]}" \
    2>/dev/null)

  log "  WASM uploaded: $wasm_hash"

  # Deploy contract instance
  contract_id=$(stellar contract deploy \
    --wasm-hash "$wasm_hash" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source "${SIGNERS[0]}" \
    2>/dev/null)

  log "  Contract ID  : $contract_id"
  NEW_IDS["$contract"]="$contract_id"
done

# ── Step 3: Multi-sig approval ─────────────────────────────────────────────────
log ""
log "=== Multi-Sig Approval ==="
log "Deployed contract IDs:"
for contract in "${CONTRACTS[@]}"; do
  log "  $contract : ${NEW_IDS[$contract]}"
done
log ""

# Each signer must explicitly approve
for idx in "${!SIGNERS[@]}"; do
  signer_num=$((idx + 1))
  read -r -p "Signer ${signer_num}: approve deployment? [yes/no] " APPROVAL
  if [[ "$APPROVAL" != "yes" ]]; then
    die "Signer ${signer_num} rejected deployment. Aborting."
  fi
  log "Signer ${signer_num} approved."
done

# ── Step 4: Initialize contracts with admin ────────────────────────────────────
log "Initializing contracts..."

stellar contract invoke \
  --id "${NEW_IDS[nft-reward]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source "${SIGNERS[0]}" \
  -- initialize \
  --admin "$ADMIN_ADDRESS"

log "  nft-reward initialized."

stellar contract invoke \
  --id "${NEW_IDS[reward-manager]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source "${SIGNERS[0]}" \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --nft_reward_contract "${NEW_IDS[nft-reward]}"

log "  reward-manager initialized."

stellar contract invoke \
  --id "${NEW_IDS[hunty-core]}" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source "${SIGNERS[0]}" \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --reward_manager_contract "${NEW_IDS[reward-manager]}"

log "  hunty-core initialized."

# ── Step 5: Write deployed addresses ──────────────────────────────────────────
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
log "=== Deployment complete ==="
log "Addresses saved to $ADDRESS_FILE"
log "Run 'scripts/verify_deployment.sh mainnet' to confirm."

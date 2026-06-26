#!/usr/bin/env bash
# verify_deployment.sh — Post-deployment health checks for Hunty contracts
#
# USAGE:
#   HUNTY_CORE_ID=C...  REWARD_MANAGER_ID=C...  NFT_REWARD_ID=C... \
#   bash scripts/verify_deployment.sh [mainnet|testnet]
#
# Exits 0 only when all checks pass. Non-zero exit means deployment is unsafe.

set -euo pipefail

NETWORK="${1:-testnet}"

# ── Config ────────────────────────────────────────────────────────────────────
case "$NETWORK" in
  mainnet)
    PASSPHRASE="Public Global Stellar Network ; September 2015"
    RPC_URL="${MAINNET_RPC_URL:?Set MAINNET_RPC_URL}"
    ;;
  testnet)
    PASSPHRASE="Test SDF Network ; September 2015"
    RPC_URL="${TESTNET_RPC_URL:-https://soroban-testnet.stellar.org}"
    ;;
  *) echo "Usage: $0 [mainnet|testnet]" >&2; exit 1 ;;
esac

CORE_ID="${HUNTY_CORE_ID:?Set HUNTY_CORE_ID}"
RM_ID="${REWARD_MANAGER_ID:?Set REWARD_MANAGER_ID}"
NFT_ID="${NFT_REWARD_ID:?Set NFT_REWARD_ID}"

EXPECTED_CORE_VERSION="${EXPECTED_CORE_VERSION:-1}"
EXPECTED_RM_VERSION="${EXPECTED_RM_VERSION:-1}"
EXPECTED_NFT_VERSION="${EXPECTED_NFT_VERSION:-1}"

# ── Helpers ───────────────────────────────────────────────────────────────────
PASS=0; FAIL=0
check() {
  local label="$1"; shift
  if "$@" &>/dev/null; then
    echo "  [PASS] $label"
    ((PASS++)) || true
  else
    echo "  [FAIL] $label"
    ((FAIL++)) || true
  fi
}

invoke() {
  stellar contract invoke \
    --id "$1" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$PASSPHRASE" \
    --source-account "${SOURCE_ACCOUNT:-}" \
    -- "${@:2}" 2>/dev/null
}

version_check() {
  local id="$1" expected="$2" label="$3"
  local got
  got=$(invoke "$id" contract_version 2>/dev/null || echo "")
  if [[ "$got" == *"$expected"* ]]; then
    echo "  [PASS] $label version = $expected"
    ((PASS++)) || true
  else
    echo "  [FAIL] $label version: expected $expected, got '$got'"
    ((FAIL++)) || true
  fi
}

# ── Run checks ────────────────────────────────────────────────────────────────
echo "=== Hunty Post-Deploy Verification ($NETWORK) ==="
echo "HuntyCore      : $CORE_ID"
echo "RewardManager  : $RM_ID"
echo "NftReward      : $NFT_ID"
echo ""

echo "--- Contract Versions ---"
version_check "$CORE_ID" "$EXPECTED_CORE_VERSION" "HuntyCore"
version_check "$RM_ID"   "$EXPECTED_RM_VERSION"   "RewardManager"
version_check "$NFT_ID"  "$EXPECTED_NFT_VERSION"  "NftReward"

echo ""
echo "--- Liveness (get_health_dashboard) ---"
check "HuntyCore health"     stellar contract invoke --id "$CORE_ID" --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_health_dashboard
check "RewardManager health" stellar contract invoke --id "$RM_ID"  --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_health_dashboard

echo ""
echo "--- Cross-Contract Wiring ---"
# RewardManager must reference NftReward; HuntyCore must reference RewardManager.
# We invoke a lightweight read that would revert if the linked address is wrong.
check "HuntyCore→RewardManager link" \
  stellar contract invoke --id "$CORE_ID" --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_reward_manager_address
check "RewardManager→NftReward link" \
  stellar contract invoke --id "$RM_ID" --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_nft_reward_address

echo ""
echo "--- Schema Versions ---"
check "HuntyCore schema"    stellar contract invoke --id "$CORE_ID" --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_schema_version
check "RewardManager schema" stellar contract invoke --id "$RM_ID"  --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_schema_version
check "NftReward schema"    stellar contract invoke --id "$NFT_ID"  --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" -- get_schema_version

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="

if [[ $FAIL -gt 0 ]]; then
  echo "DEPLOYMENT VERIFICATION FAILED — do not proceed to production traffic."
  exit 1
fi

echo "All checks passed. Deployment verified."
exit 0

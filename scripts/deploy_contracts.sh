#!/usr/bin/env bash
set -euo pipefail

ENVIRONMENT="${1:-}"
SOURCE_ACCOUNT="${2:-}"

case "$ENVIRONMENT" in
  testnet|staging)
    NETWORK="testnet"
    RPC_URL="${TESTNET_RPC_URL:-https://soroban-testnet.stellar.org}"
    PASSPHRASE="Test SDF Network ; September 2015"
    ;;
  mainnet)
    NETWORK="mainnet"
    RPC_URL="${MAINNET_RPC_URL:?Set MAINNET_RPC_URL}"
    PASSPHRASE="Public Global Stellar Network ; September 2015"
    ;;
  *)
    echo "Usage: $0 <testnet|staging|mainnet> <stellar-source-account>" >&2
    exit 1
    ;;
esac

if [[ -z "$SOURCE_ACCOUNT" ]]; then
  echo "Missing Stellar source account name or secret key" >&2
  echo "Usage: $0 <testnet|staging|mainnet> <stellar-source-account>" >&2
  exit 1
fi

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }
}

deploy_contract() {
  local name="$1"
  local wasm="target/wasm32v1-none/release/${name}.wasm"

  if [[ ! -f "$wasm" ]]; then
    echo "Missing WASM: $wasm" >&2
    echo "Run 'stellar contract build' from the repository root first." >&2
    exit 1
  fi

  stellar contract deploy \
    --wasm "$wasm" \
    --source "$SOURCE_ACCOUNT" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$PASSPHRASE"
}

require_cmd stellar

echo "Deploying Hunty contracts to $ENVIRONMENT ($NETWORK)"

NFT_REWARD_ID="$(deploy_contract nft_reward)"
REWARD_MANAGER_ID="$(deploy_contract reward_manager)"
HUNTY_CORE_ID="$(deploy_contract hunty_core)"

cat > "config/contracts.${ENVIRONMENT}.json" <<JSON
{
  "environment": "${ENVIRONMENT}",
  "network": "${NETWORK}",
  "rpcUrl": "${RPC_URL}",
  "networkPassphrase": "${PASSPHRASE}",
  "contracts": {
    "huntyCore": "${HUNTY_CORE_ID}",
    "rewardManager": "${REWARD_MANAGER_ID}",
    "nftReward": "${NFT_REWARD_ID}"
  }
}
JSON

echo "HuntyCore      : $HUNTY_CORE_ID"
echo "RewardManager  : $REWARD_MANAGER_ID"
echo "NftReward      : $NFT_REWARD_ID"
echo "Wrote config/contracts.${ENVIRONMENT}.json"

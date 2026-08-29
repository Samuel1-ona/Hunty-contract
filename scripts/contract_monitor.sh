#!/usr/bin/env bash
# Hunty contract health monitor — polls on-chain health dashboards via stellar CLI.
set -euo pipefail

NETWORK="${NETWORK:-testnet}"
CORE_ID="${HUNTY_CORE_ID:-}"
RM_ID="${REWARD_MANAGER_ID:-}"

if [[ -z "$CORE_ID" || -z "$RM_ID" ]]; then
  echo "Set HUNTY_CORE_ID and REWARD_MANAGER_ID" >&2
  exit 1
fi

echo "=== Hunty Contract Monitor ($NETWORK) ==="

core_health=$(stellar contract invoke --id "$CORE_ID" --network "$NETWORK" -- get_health_dashboard 2>/dev/null || echo "unavailable")
rm_health=$(stellar contract invoke --id "$RM_ID" --network "$NETWORK" -- get_health_dashboard 2>/dev/null || echo "unavailable")

echo "HuntyCore health: $core_health"
echo "RewardManager health: $rm_health"

# Alert thresholds (operator-tunable)
FAILURE_RATE_BPS_ALERT=500
LARGE_WITHDRAWAL_ALERT=1

if echo "$core_health" | grep -q "failure_rate_bps"; then
  rate=$(echo "$core_health" | sed -n 's/.*failure_rate_bps: \([0-9]*\).*/\1/p')
  if [[ -n "$rate" && "$rate" -gt "$FAILURE_RATE_BPS_ALERT" ]]; then
    echo "ALERT: HuntyCore failure rate ${rate}bps exceeds threshold"
    exit 2
  fi
fi

echo "Monitor check passed"

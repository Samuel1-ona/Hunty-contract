# Rollback Procedure — Hunty Mainnet

Use this document when `scripts/verify_deployment.sh mainnet` fails, or when a critical bug is detected after deployment. All rollback actions require the same multi-sig quorum as the original deployment.

---

## Decision Criteria — Roll Back or Stay Forward?

| Severity | Examples | Action |
|---|---|---|
| Critical | Funds inaccessible, wrong admin, broken cross-contract link | Roll back immediately |
| High | Version mismatch, schema corruption | Roll back after quick investigation |
| Medium | Single non-critical function broken | Hot-fix forward if possible |
| Low | Cosmetic issue, leaderboard edge case | Hot-fix forward |

When in doubt, roll back. The cost of re-deploying a fix is lower than the cost of leaving a broken contract on mainnet.

---

## Pre-Rollback: Record Current State

Before touching anything, capture the current IDs:

```bash
# Read from docs/deployment/deployed-addresses.md
BROKEN_CORE_ID=<current hunty-core ID>
BROKEN_RM_ID=<current reward-manager ID>
BROKEN_NFT_ID=<current nft-reward ID>

# Record in a timestamped note
echo "Rollback initiated $(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
  >> docs/deployment/deployed-addresses.md
```

---

## Rollback Option A — Storage Migration Rollback (same contract binary)

Use this when: data was migrated but the new schema is broken. The contract binary is fine; only storage needs reverting.

```bash
# Requires admin signer
stellar contract invoke \
  --id "$BROKEN_CORE_ID" \
  --rpc-url "$MAINNET_RPC_URL" \
  --network-passphrase "Public Global Stellar Network ; September 2015" \
  --source "$SIGNER_1_SECRET" \
  -- rollback_migration \
  --admin "$MAINNET_ADMIN_ADDRESS"
```

Repeat for `reward-manager` and `nft-reward` if they were also migrated.

Verify schema versions reverted:
```bash
stellar contract invoke --id "$BROKEN_CORE_ID" \
  --rpc-url "$MAINNET_RPC_URL" \
  --network-passphrase "Public Global Stellar Network ; September 2015" \
  -- get_schema_version
```

---

## Rollback Option B — Contract Upgrade Rollback (redeploy previous WASM)

Use this when: the new WASM itself is broken. You will redeploy the previous WASM hash.

### Step 1 — Retrieve the previous WASM hash

Previous hashes are in `docs/deployment/manifest.txt` from the last known-good deployment. Alternatively:

```bash
# List recent WASM uploads on-chain (requires horizon or RPC)
stellar contract info --id "$BROKEN_CORE_ID" \
  --rpc-url "$MAINNET_RPC_URL" \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

### Step 2 — Multi-sig approval for rollback

Both required signers must acknowledge:

```
Signer 1: I approve rolling back hunty-core to WASM hash <previous_hash>
Signer 2: I approve rolling back hunty-core to WASM hash <previous_hash>
```

Record approval in a GitHub issue or signed message before proceeding.

### Step 3 — Upgrade contract to previous WASM

```bash
stellar contract invoke \
  --id "$BROKEN_CORE_ID" \
  --rpc-url "$MAINNET_RPC_URL" \
  --network-passphrase "Public Global Stellar Network ; September 2015" \
  --source "$SIGNER_1_SECRET" \
  -- upgrade \
  --new_wasm_hash "<previous_wasm_hash>"
```

Repeat in reverse dependency order: `hunty-core` → `reward-manager` → `nft-reward`.

### Step 4 — Roll back storage migration if needed

After re-deploying the old binary, if the schema was also bumped, call `rollback_migration` as in Option A.

---

## Rollback Option C — Full Redeploy (last resort)

Use this when: the contract state is irrecoverable and Option A/B are not viable.

1. Run the full `scripts/deploy_mainnet.sh` with the last known-good tag:
   ```bash
   git checkout <last-good-tag>
   SIGNER_1_SECRET=... SIGNER_2_SECRET=... bash scripts/deploy_mainnet.sh
   ```
2. Update all references (frontend, environment configs) to the new contract IDs.
3. Announce the new IDs to all stakeholders.

**Note**: A full redeploy means new contract IDs. Existing on-chain state (hunts, player progress) in the broken contracts is abandoned. Only use this if the state is already corrupted.

---

## Post-Rollback Verification

After any rollback option:

```bash
HUNTY_CORE_ID=<rolled-back-id> \
REWARD_MANAGER_ID=<rolled-back-id> \
NFT_REWARD_ID=<rolled-back-id> \
bash scripts/verify_deployment.sh mainnet
```

The script must exit 0 before resuming normal operations.

---

## Communication Checklist

- [ ] Incident channel notified with: what broke, what was rolled back, new contract IDs if changed
- [ ] `docs/deployment/deployed-addresses.md` updated with rollback entry and timestamp
- [ ] Post-mortem issue opened in the repository
- [ ] Frontend / SDK consumers updated if contract IDs changed
- [ ] Monitoring (`scripts/contract_monitor.sh`) restarted against the rolled-back IDs

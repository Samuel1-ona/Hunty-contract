# Pre-Deployment Verification Checklist — Hunty Mainnet

Complete every item in order. Sign off with your name/date at the bottom before initiating the multi-sig approval.

---

## 1. Code Quality

- [ ] All tests pass: `cargo test --workspace`
- [ ] No Clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] `CONTRACT_VERSION` bumped in each contract being deployed (`contracts/<name>/src/lib.rs`)
- [ ] If any public function signature changed, `REQUIRED_*_VERSION` updated in all dependent contracts
- [ ] Cross-contract compatibility matrix in `docs/versioning/README.md` updated

## 2. Build Artefacts

- [ ] Clean build from source: `make build`
- [ ] WASM sizes are reasonable (no unexpected bloat):
  ```
  ls -lh target/wasm32v1-none/release/*.wasm
  ```
- [ ] WASM hashes recorded and match what is checked into `docs/deployment/manifest.txt`:
  ```
  sha256sum target/wasm32v1-none/release/*.wasm
  ```

## 3. Testnet Smoke Test

- [ ] All three contracts deployed to testnet and addresses recorded
- [ ] `contract_version()` returns expected version on testnet for each contract
- [ ] End-to-end hunt lifecycle executed on testnet (create → add clues → activate → register → solve → complete)
- [ ] Reward distribution (XLM + NFT) verified on testnet
- [ ] `scripts/verify_deployment.sh testnet` exits 0

## 4. Security Review

- [ ] No private keys, mnemonics, or secrets committed to the repository
- [ ] Admin account is a multi-sig account (≥ 2-of-N signers) — confirm with `stellar account info $MAINNET_ADMIN_ADDRESS`
- [ ] Reward pool funded with correct amount and admin is set correctly in RewardManager
- [ ] NftReward admin set to RewardManager contract address (only it can mint)
- [ ] HuntyCore admin set to multi-sig admin account

## 5. Deployment Configuration

- [ ] `MAINNET_ADMIN_ADDRESS` set and signers confirmed
- [ ] `MAINNET_RPC_URL` set (e.g. `https://mainnet.stellar.validationcloud.io/v1/<key>`)
- [ ] `MAINNET_NETWORK_PASSPHRASE` = `Public Global Stellar Network ; September 2015`
- [ ] Deployment order confirmed: `nft-reward` → `reward-manager` → `hunty-core`
- [ ] At least 2 signers available and reachable for multi-sig approval

## 6. Rollback Readiness

- [ ] Previous contract IDs (if upgrading) recorded in `docs/deployment/deployed-addresses.md`
- [ ] `rollback_migration(admin)` has been tested on testnet
- [ ] `docs/deployment/rollback-procedure.md` reviewed and understood by all signers

---

## Sign-Off

| Role | Name | Date |
|---|---|---|
| Lead Developer | | |
| Signer 2 | | |
| Signer 3 (if applicable) | | |

**DO NOT proceed to `scripts/deploy_mainnet.sh` until all boxes are checked and sign-offs recorded.**

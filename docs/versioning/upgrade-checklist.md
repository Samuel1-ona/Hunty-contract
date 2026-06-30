# Upgrade Checklist

Follow these steps whenever you release a new contract version.

## 1 – Code changes

- [ ] Bump `CONTRACT_VERSION` in `contracts/<name>/src/lib.rs`.
- [ ] If any public function signature changed or was removed, mark the change **breaking**.

## 2 – Breaking changes only

- [ ] Bump `REQUIRED_<NAME>_VERSION` in every contract that calls the changed contract.
- [ ] Re-run all cross-contract tests to confirm compatibility.

## 3 – Documentation

- [ ] Update the **Current Version** column in `docs/versioning/README.md`.
- [ ] Add a row to the History table in `docs/versioning/compatibility-matrix.md`.

## 4 – Deploy

- [ ] Build the changed contract: `cd contracts/<name> && make build`
- [ ] Deploy to testnet and record the new contract address.
- [ ] Run smoke-test: call `contract_version()` on the deployed contract and
  confirm it returns the expected number.
- [ ] If dependent contracts changed, deploy them in dependency order:
  `nft-reward` → `reward-manager` → `hunty-core`.

## 5 – Post-deploy

- [ ] Update deployed addresses in your environment config / frontend.
- [ ] Notify stakeholders of the new version.

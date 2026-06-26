# Contract Versioning & Cross-Contract Compatibility

This directory documents how Hunty contract versions are tracked and how
cross-contract compatibility is enforced at runtime.

## Contracts

| Contract | Crate | Current Version |
|---|---|---|
| HuntyCore | `hunty-core` | 1 |
| RewardManager | `reward-manager` | 1 |
| NftReward | `nft-reward` | 1 |

## Versioning Scheme

Each contract exposes a `contract_version() -> u32` function that returns its
current integer version. Versions are bumped according to this policy:

- **Patch** – bug-fix only, no interface change → no version bump needed.
- **Minor** – new functions added, all existing call-sites still valid → bump version.
- **Breaking** – existing function signatures change or are removed → bump version AND
  update the `REQUIRED_*_VERSION` constant in every dependent contract.

## Compatibility Matrix

`REQUIRED_X_VERSION` is the **minimum** version of contract X that a dependent
contract requires. If the on-chain version is lower, initialisation is blocked.

| Caller | Callee | `REQUIRED_*_VERSION` constant | Location |
|---|---|---|---|
| HuntyCore | RewardManager | `REQUIRED_REWARD_MANAGER_VERSION = 1` | `hunty-core/src/lib.rs` |
| RewardManager | NftReward | `REQUIRED_NFT_REWARD_VERSION = 1` | `reward-manager/src/lib.rs` |

## How It Works

1. Every contract stores its own version in **instance storage** under the key
   `"CVER"` during `initialize`.
2. `contract_version()` reads that value (falling back to the compiled constant).
3. Dependent contracts call `check_compatibility` during their own `initialize`
   to verify the callee version meets the minimum requirement.
4. `get_contract_version(env, address)` is a lightweight cross-contract query
   helper used by the compatibility check.

## Upgrading

When you release a new version of a contract:

1. Bump `CONTRACT_VERSION` in `lib.rs`.
2. Re-deploy the contract.
3. If the change is breaking, bump the corresponding `REQUIRED_*_VERSION`
   constant in every dependent contract and re-deploy those too.
4. Update the Compatibility Matrix table above.

## Files

| File | Purpose |
|---|---|
| `README.md` | This overview |
| `compatibility-matrix.md` | Machine-friendly version compatibility table |
| `upgrade-checklist.md` | Step-by-step checklist for safe upgrades |

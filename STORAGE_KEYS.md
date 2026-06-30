Storage key prefixes audit and migration plan
==========================================

Summary
-------
This document lists the storage-related symbol keys that were shortened to reduce per-entry storage costs. The changes in this branch replace longer symbol strings used as storage prefixes with minimal unique prefixes while preserving uniqueness within each contract module.

Mapping (old -> new)
---------------------

hunty-core
- HUNT -> HUNT
- CLUE -> CLU
- PROG -> PR
- PLRS -> PL
- CLST -> CLS
- CNTR -> CN
- CCNT -> CC
- RWDMGR -> R
- BAN -> BA
- SUBMIT -> S
- ADMIN -> AD
- VIEW -> V
- GVW -> GV
- PAUSE_REG -> PAUSE_RE
- PAUSE_ANS -> PAUSE_A
- PAUSE_RWD -> PAUSE_RW

nft-reward
- NFT -> NF
- CNTR -> CN
- ONFC -> ONFC
- HNFC -> HN
- MAXS -> MA
- INIT -> I
- ADMIN -> A
- MNTR -> MN
- RWDMGR -> R
- NVER -> NV
- THUNTS -> TH
- TOWNRS -> TO

reward-manager
- ADMIN -> ADMI
- XLMTKN -> X
- NFTADR -> NFTA
- DPCAP -> DPC
- DGRCAP -> DGR
- DPDST -> DPD
- DGDST -> DGD
- DIST -> DI
- DREC -> DR
- POOL -> POOL
- PCFG -> PC
- PDEP -> PDE
- PDST -> PDS
- TXLMDST -> TXL
- HCORE -> H
- TXDST -> TXD
- IN_DIST -> IN_
- PAUSED -> PA
- EMLOG -> EML

Notes
-----
- Only storage-related keys defined in contract storage modules were changed. Event names (symbols used for events) were left untouched to avoid changing external event schema.
- The shortened prefixes are chosen to be the shortest substring that remains unique among keys in the same contract.

Migration plan
--------------
High level
- Because Soroban persistent storage keys are typed tuples that begin with these symbol prefixes, changing the prefix means previously written entries will remain under the old key names. To preserve data, we need a migration that copies existing entries from old keys to the new keys.

Recommended approach (safe, off-chain driver)
1. Deploy a migration helper (off-chain) that connects to a node with administrative privileges for the contract.
2. For each contract:
   - Read counters (e.g. hunt counter) using the old key names to determine ranges.
   - For each discovered item (hunt, clue, player progress, distribution records, history entries, etc.), read the value under the old key and write it under the new key using the same tuple shape but first element replaced by the new symbol.
   - Do this in idempotent batches and verify checksums/hashes of the copied values before optionally removing the old key.

Dry-run vs apply
- First run in dry-run mode: only enumerate keys and report planned copy operations and sizes.
- Then run apply mode: perform copies, validate, and keep old keys for a verification period.
- Optionally run a cleanup pass to remove old keys after verification.

On-chain migration option
- It's possible to implement an on-chain migration method in the `migration` contract that, when authorized and called by an admin, performs the same copy operations. This can be expensive in gas and may hit execution limits; prefer the off-chain batched approach.

What I changed in this branch
- Shortened storage symbol strings in the following files:
  - `contracts/hunty-core/src/storage.rs`
  - `contracts/nft-reward/src/storage.rs`
  - `contracts/reward-manager/src/storage.rs`
- Added this `STORAGE_KEYS.md` documenting the mapping and migration plan.

Next steps I can take for you
----------------------------
- Implement a safe, idempotent off-chain migration script (TypeScript or Rust) that uses RPC to enumerate and copy keys in batches.
- Implement an on-chain migration function (carefully) in the `migration` contract that copies a small, bounded set of keys or is driven by batches.
- Create the branch and push to your fork (I need your fork URL or a PAT to push).

Execution
---------
To push this branch to your fork (once created locally), either add your fork remote and push, or provide a PAT and fork URL and I will push the branch for you.

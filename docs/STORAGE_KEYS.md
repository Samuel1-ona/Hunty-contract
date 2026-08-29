# Storage Keys Registry

Canonical registry of Soroban storage key prefixes used by Hunty contracts.
Every `*_KEY` constant and composite key constructor in `contracts/*/src/storage.rs`
(and related monitoring modules) must appear here. Storage key collisions are
unrecoverable after deploy — update this file in the same PR that adds or changes a key.


## Convention

- Prefer `symbol_short!("...")` prefixes (≤ 9 chars).
- Instance storage: contract-global config / counters / pause flags.
- Persistent storage: per-entity data (hunts, NFTs, pools, progress).
- Composite keys are tuples such as `(Symbol, id)` or `(Symbol, Address, ...)`.

A CI/script check (`scripts/ci/check_storage_keys_doc.sh`) asserts that every
`const *_KEY` declared under `contracts/` is named in this document (`docs/STORAGE_KEYS.md`).

---

## hunty-core (`contracts/hunty-core/src/storage.rs`)

> **Garbage collection (issue #446).** Soroban cannot scan by key prefix, so
> `Storage::sweep_hunt_storage` reconstructs a hunt's entries from the same key
> builders that wrote them. Any **per-hunt** key added below must also be added
> to that function, or it becomes a permanent leak: once the hunt record is gone
> there is nothing left to enumerate it from.

| Constant | Symbol | Shape / notes |
|---|---|---|
| `HUNT_KEY` | `HUNT` | `(HUNT, hunt_id)` — hunt record |
| `HUNT_CACHE_KEY` | `HC` | `(HC, hunt_id)` — instance cache |
| `CLUE_KEY` | `CLU` | `(CLU, clue_id)` |
| `PROGRESS_KEY` | `PR` | `(PR, hunt_id, player)` |
| `PLAYERS_LIST_KEY` | `PL` | players list for a hunt |
| `LEADERBOARD_KEY` | `LBD` | leaderboard index |
| `CLUES_LIST_KEY` | `CLS` | clues list |
| `PLAYER_ENTRY_KEY` | `PLRS` | `(PLRS, hunt_id, index)` |
| `PLAYER_COUNT_KEY` | `PLCT` | `(PLCT, hunt_id)` |
| `CLUE_ENTRY_KEY` | `CLST` | clue list entry |
| `CLUE_LIST_COUNT_KEY` | `CLCT` | clue list count |
| `HUNT_COUNTER_KEY` | `CN` | hunt id counter |
| `CLUE_COUNTER_KEY` | `CC` | clue id counter |
| `REWARD_MGR_KEY` | `R` | reward-manager address |
| `BAN_KEY` | `BA` | ban / blacklist entry |
| `SUBMISSION_KEY` | `S` | answer submission |
| `ADMIN_KEY` | `AD` | admin address |
| `PENDING_ADMIN_KEY` | `ADM_PEND` | pending admin transfer |
| `VIEW_ONLY_KEY` | `V` | per-hunt view-only |
| `GLOBAL_VIEW_ONLY_KEY` | `GV` | global view-only |
| `PAUSE_REGISTRATIONS_KEY` | `PAUSE_RE` | pause registrations |
| `PAUSE_ANSWERS_KEY` | `PAUSE_A` | pause answers |
| `PAUSE_REWARDS_KEY` | `PAUSE_RW` | pause rewards |
| `CONTRACT_PAUSED_KEY` | `CPAUSED` | global emergency pause |
| `PAUSE_KEY` | `PAUSE` | legacy / alternate pause flag |
| `BLACKLIST_KEY` | `BLKLST` | blacklist map |
| `BLACKLIST_VEC_KEY` | `BLKLST_V` | blacklist vector |
| `REQUIRED_CLUES_KEY` | `REQCL` | required clues config |
| `CACHE_HIT_KEY` | `CHIT` | cache hit counter |
| `CACHE_MISS_KEY` | `CMISS` | cache miss counter |
| `PLAYER_HUNTS_KEY` | `PHNT` | `(PHNT, player)` — hunts joined by player |
| `TEAM_KEY` | `TEAM` | `(TEAM, hunt_id, team_id)` |
| `TEAM_COUNT_KEY` | `TMCT` | `(TMCT, hunt_id)` |
| `PLAYER_TEAM_KEY` | `PLTM` | `(PLTM, hunt_id, player)` |
| `TEAM_PROGRESS_KEY` | `TMPR` | `(TMPR, hunt_id, team_id)` |

### Inline / helper symbols (hunty-core)

| Symbol | Usage |
|---|---|
| `CLEX` | clue existence helper |
| `PLEX` | player existence helper |
| `CVER` | contract version (inline) |
| `HRLADM` / `HRLCT` / `HRLDEF` / `HRLOVR` | rate-limit helpers |

### Monitoring (`contracts/hunty-core/src/monitoring.rs`)

| Constant | Symbol |
|---|---|
| `INVOCATIONS_KEY` | `INVCT` |
| `FAILURES_KEY` | `FAILCT` |
| `GAS_UNITS_KEY` | `GASUN` |
| `ALERTS_KEY` | `ALERT` |

---

## nft-reward (`contracts/nft-reward/src/storage.rs`)

| Constant | Symbol | Shape / notes |
|---|---|---|
| `NFT_KEY` | `NF` | `(NF, nft_id)` — full NFT blob |
| `NFT_CORE_KEY` | `NC` | `(NC, nft_id)` — core fields |
| `NFT_META_KEY` | `NM` | `(NM, nft_id)` — metadata |
| `NFT_VERSION_KEY` | `NFTV` | `(NFTV, nft_id)` — per-NFT metadata version (distinct from `CTRV`) |
| `NFT_COUNTER_KEY` | `CN` | next NFT id |
| `OWNER_NFT_COUNT_KEY` | `ONFC` | `(ONFC, owner)` |
| `HUNT_NFT_COUNT_KEY` | `HNFC` | `(HNFC, hunt_id)` — per-hunt mint count index |
| `MAX_SUPPLY_KEY` | `MAXS` | collection max supply |
| `INITIALIZED_KEY` | `INIT` | init flag |
| `ADMIN_KEY` | `ADMIN` | admin (also historically `ADMN`) |
| `MINTER_KEY` | `MNTR` | `(MNTR, minter)` whitelist |
| `REWARD_MGR_KEY` | `RWDMGR` | reward-manager address (also historically `RWMG`) |
| `COLLECTION_METADATA_KEY` | `COLL` | collection metadata |
| `HAS_AUTH_KEY` | `HAUTH` | auth-initialized flag |
| `ALL_NFTS_KEY` | `ALLNFT` | vector of all minted NFT ids |
| `TOTAL_HUNTS_KEY` | `TH` | distinct hunts that minted |
| `TOTAL_OWNERS_KEY` | `TO` | distinct owners counter |
| `CONTRACT_VERSION_KEY` | `CTRV` | contract version (instance storage) |
| `OPERATOR_KEY` | `OPKEY` | `(OPKEY, owner, operator)` — operator approval flag |

### Composite constructors (nft-reward)

| Constructor | Prefix | Shape |
|---|---|---|
| `operator_key(owner, operator)` | `OPKEY` | `(OPKEY, owner, operator)` — operator approval |
| `locker_key(locker)` | `LOCKR` | `(LOCKR, locker)` — authorized locker |
| `owner_nft_entry_key` | `ONFT` | `(ONFT, owner, index)` |
| `owner_nft_exist_key` | `ONFX` | `(ONFX, owner, nft_id)` |
| `hunt_nft_entry_key` | `HNFT` | `(HNFT, hunt_id, index)` |
| `hunt_nft_exist_key` | `HNFX` | `(HNFX, hunt_id, nft_id)` |
| `authorized_contract_key` | `AUTH` | `(AUTH, contract)` |
| hunt-minted flag | `HMNT` | `(HMNT, hunt_id)` |

---

## reward-manager (`contracts/reward-manager/src/storage.rs`)

| Constant | Symbol | Shape / notes |
|---|---|---|
| `ADMIN_KEY` | `ADMI` | admin address |
| `PENDING_ADMIN_KEY` | `PADMI` | pending admin transfer |
| `XLM_TOKEN_KEY` | `X` | native/XLM SAC address |
| `NFT_CONTRACT_KEY` | `NFTA` | nft-reward contract address |
| `HUNTY_CORE_KEY` | `HCORE` | hunty-core contract address |
| `DAILY_POOL_CAP_KEY` | `DPC` | `(DPC, hunt_id)` |
| `DAILY_GLOBAL_CAP_KEY` | `DGR` | global daily cap |
| `DAILY_POOL_DIST_KEY` | `DPD` | `(DPD, hunt_id, day)` |
| `DAILY_GLOBAL_DIST_KEY` | `DGD` | `(DGD, day)` |
| `DISTRIBUTION_KEY` | `DI` | `(DI, hunt_id, player)` |
| `DIST_RECORD_KEY` | `DR` | `(DR, hunt_id, player)` |
| `DIST_NONCE_KEY` | `DN` | `(DN, hunt_id, player)` |
| `DIST_RESOLVE_KEY` | `DRS` | `(DRS, hunt_id, player)` |
| `DIST_PROOF_KEY` | `DPRF` | `(DPRF, hunt_id, player)` |
| `LAST_DIST_TS_KEY` | `LDTS` | `(LDTS, hunt_id)` |
| `POOL_KEY` | `POOL` | `(POOL, hunt_id)` |
| `POOL_CFG_KEY` | `PCFG` | `(PCFG, hunt_id)` |
| `POOL_DEP_KEY` | `PDEP` | `(PDEP, hunt_id)` deposited |
| `POOL_DST_KEY` | `PDST` | `(PDST, hunt_id)` distributed |
| `POOL_RFD_KEY` | `PRFD` | `(PRFD, hunt_id)` total refunded (issue #628) |
| `POOL_DIST_COUNT_KEY` | `PDCNT` | `(PDCNT, hunt_id)` distribution count |
| `POOL_LAST_DIST_TS_KEY` | `PLDTS` | `(PLDTS, hunt_id)` |
| `POOL_DISTRIBUTIONS_KEY` | `PLDIST` | `(PLDIST, hunt_id)` distribution list |
| `TOTAL_XLM_DST_KEY` | `TXDST` | total XLM distributed |
| `IN_DISTRIBUTION_KEY` | `IN_DIST` | re-entrancy / in-flight flag |
| `HAS_AUTH_KEY` | `HAUTH` | auth-initialized flag |
| `AUDIT_COUNT_KEY` | `AUDC` | `(AUDC, hunt_id)` — total audit entries appended |
| `AUDIT_LOG_KEY` | `AUDL` | `(AUDL, hunt_id, index)` — ring-buffer slot (`index % MAX_AUDIT_ENTRIES_PER_POOL`) |
| `PAUSED_KEY` | `PAUSE` | emergency pause — also seen as `PAUSED` / `PAUS` |
| `PAUSE_FUNDING_KEY` | `PAUSE_FD` | granular funding pause (issue #628) |
| `PAUSE_DIST_KEY` | `PAUSE_DS` | granular distribution pause (issue #628) |
| `EMERGENCY_LOG_KEY` | `EMLOG` | emergency action log — also seen as `ELOG` |
| `PENDING_NFT_KEY` | `PNFT` | `(PNFT, hunt_id, player)` pending mint |
| `VESTING_KEY` | `VEST` | `(VEST, hunt_id, player)` vesting record |
| `POOL_FUNDERS_KEY` | `PFNDRS` | `(PFNDRS, hunt_id)` — list of distinct funders not yet refunded |
| `POOL_FUNDER_CONTRIB_KEY` | `PFCONT` | `(PFCONT, hunt_id, funder)` — cumulative unrefunded contribution |

### Audit log capacity (reward-manager)

| Constant | Value | Notes |
|---|---|---|
| `MAX_AUDIT_ENTRIES_PER_POOL` | `50` | Ring-buffer cap per hunt; not a storage key — bounds `(AUDL, hunt_id, index)` slots |

### Inline / helper symbols (reward-manager)

| Symbol | Usage |
|---|---|
| `AUTH` | authorized contract helper |
| `CVER` | contract version (inline) |

### Monitoring (`contracts/reward-manager/src/monitoring.rs`)

| Constant | Symbol |
|---|---|
| `INVOCATIONS_KEY` | `INVCT` |
| `FAILURES_KEY` | `FAILCT` |
| `GAS_UNITS_KEY` | `GASUN` |
| `ALERTS_KEY` | `ALERT` |

---

## Shortening map (legacy → current)

Historical longer prefixes that were shortened (see ADR 002):

### hunty-core

- `HUNT` → `HUNT`
- `CLUE` → `CLU`
- `PROG` → `PR`
- `PLRS` → `PL` (list) / kept as `PLRS` for entry keys
- `CLST` → `CLS` (list) / kept as `CLST` for entry keys
- `CNTR` → `CN`
- `CCNT` → `CC`
- `RWDMGR` → `R`
- `BAN` → `BA`
- `SUBMIT` → `S`
- `ADMIN` → `AD`
- `VIEW` → `V`
- `GVW` → `GV`
- `PAUSE_REG` → `PAUSE_RE`
- `PAUSE_ANS` → `PAUSE_A`
- `PAUSE_RWD` → `PAUSE_RW`

### nft-reward

- `NFT` → `NF`
- `CNTR` → `CN`
- `ONFC` → `ONFC`
- `HNFC` → `HN`
- `MAXS` → `MAXS` (or historically `MA`)
- `INIT` → `INIT` (or historically `I`)
- `ADMIN` → `ADMIN` (or historically `A` / `ADMN`)
- `MNTR` → `MNTR`
- `RWDMGR` → `RWDMGR`
- `NVER` → `NV`
- `THUNTS` → `TH`
- `TOWNRS` → `TO`
- `ALLNFTS` → `ALLNFT`
- `CVER` → `CV`
- operator / locker composites → `OPKEY` / `LOCKR`

### reward-manager

- `ADMIN` → `ADMI`
- `XLMTKN` → `X`
- `NFTADR` → `NFTA`
- `DPCAP` → `DPC`
- `DGRCAP` → `DGR`
- `DPDST` → `DPD`
- `DGDST` → `DGD`
- `DIST` → `DI`
- `DREC` → `DR`
- `POOL` → `POOL`
- `PCFG` → `PCFG`
- `PDEP` → `PDEP`
- `PDST` → `PDST`
- `TXLMDST` / `TXDST` → `TXDST`
- `HCORE` → `HCORE`
- `IN_DIST` → `IN_DIST`
- `PAUSED` → `PAUSE`
- `EMLOG` → `EMLOG`
- `ACNT` / audit → `AUDC` / `AUDL`
- `PNFT` → `PNFT`

---

## migration (`contracts/migration/src/lib.rs`)

| Constant | Symbol | Notes |
|---|---|---|
| `ROLLBACK_KEY` | `RBKVER` | rollback / previous version marker |
| `PROPOSAL_KEY` | `UPROP` | pending upgrade proposal |
| `TIMELOCK_KEY` | `UPTLK` | upgrade timelock |
| `HIST_COUNT_KEY` | `UPHCT` | upgrade history count |
| `UPGRADE_ADMIN_KEY` | `UPADM` | upgrade admin |

Also referenced from per-contract `migration.rs` modules via event symbols
(`UpgradeProposed`, `UpgradeExecuted`) — those are events, not storage keys.

---

## Migration notes

Changing a prefix leaves old persistent entries under the previous key. Prefer an
off-chain batched copy (dry-run → apply → verify → optional cleanup) over a
single on-chain migration that may hit execution limits.

When adding a new `*_KEY` constant:

1. Choose a prefix unique within the contract module.
2. Declare the constant next to the other keys in `storage.rs`.
3. Document it in this file (`docs/STORAGE_KEYS.md`) in the same PR.
4. Run `scripts/ci/check_storage_keys_doc.sh`.
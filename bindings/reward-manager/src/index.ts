import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




/**
 * Operation type for the pool audit log.
 * 
 * Values are explicit and append-only so adding operations does not renumber
 * records written by older deployments.
 */
export enum PoolOperation {
  Create = 0,
  Fund = 1,
  Distribute = 2,
  Withdraw = 3,
  Freeze = 4,
  Unfreeze = 5,
  Migrate = 6,
  Refund = 7,
  UpdateMinAmount = 8,
  SetTargetAmount = 9,
  SetDistributionInterval = 10,
  SetDistributionMode = 11,
  SetNftContract = 12,
  AddDelegate = 13,
  RemoveDelegate = 14,
  SetVestingPeriod = 15,
  SetTimeTiers = 16,
  SetRankTiers = 17,
}


/**
 * Read-only view of a player's vesting status for a specific hunt.
 * Returned by `get_vesting_status`.
 */
export interface VestingStatus {
  /**
 * XLM available to claim right now: `vested_amount - claimed_amount`.
 */
claimable_amount: i128;
  /**
 * Cumulative XLM already claimed.
 */
claimed_amount: i128;
  /**
 * True once `claimed_amount >= total_amount`.
 */
fully_vested: boolean;
  /**
 * Ledger timestamp when vesting began.
 */
start_time: u64;
  /**
 * Total XLM locked under this schedule.
 */
total_amount: i128;
  /**
 * XLM that has vested so far: `total_amount * min(elapsed / vesting_period_secs, 1)`.
 */
vested_amount: i128;
  /**
 * Full vesting duration in seconds.
 */
vesting_period_secs: u64;
}


/**
 * Pending NFT mint that failed and can be retried by the player or anyone
 * paying the transaction fee on behalf of the player.
 */
export interface PendingNftMint {
  /**
 * Frozen completion rank from hunty-core; preserved so a retry emits the
 * same rank that would have been recorded on the first attempt.
 */
completion_rank: u32;
  hunt_id: u64;
  nft_contract: string;
  nft_description: string;
  nft_hunt_title: string;
  nft_image_uri: string;
  nft_rarity: u32;
  nft_tier: u32;
  nft_title: string;
  player: string;
}


/**
 * A single entry in the pool audit log.
 */
export interface PoolAuditEntry {
  /**
 * Who triggered the operation.
 */
actor: string;
  /**
 * The XLM amount involved, if applicable.
 */
amount: Option<i128>;
  /**
 * Operation performed.
 */
operation: PoolOperation;
  /**
 * Timestamp (ledger time).
 */
timestamp: u64;
}

/**
 * How XLM rewards are calculated from the pool at distribution time.
 */
export enum DistributionMode {
  Fixed = 0,
  Proportional = 1,
}


/**
 * Record of a completed distribution for a specific pool.
 */
export interface PoolDistribution {
  nft_id: Option<u64>;
  player: string;
  timestamp: u64;
  xlm_amount: i128;
}

/**
 * Resolution outcome for a manually resolved failed distribution.
 * 
 * This enum tracks the final status of distributions that failed during
 * their initial execution and were later resolved by an administrator.
 * 
 * Related to issue #364: stuck-distribution resolution flow.
 */
export type ResolutionStatus = {tag: "Completed", values: void} | {tag: "Refunded", values: void};


/**
 * Configuration for a reward pool, set at creation time.
 * 
 * `time_based_tiers` is an optional list of (max_elapsed_seconds, xlm_amount)
 * pairs that define a conditional reward schedule based on how quickly a
 * player completes a hunt. `rank_based_tiers` is an optional list of exact
 * one-based completion ranks and their amounts; a matching rank takes
 * precedence over time and flat rewards. When both lists are empty the pool
 * behaves exactly as before. Tier lists can be updated after pool creation
 * and queried via `get_pool_config`.
 */
export interface RewardPoolConfig {
  /**
 * Unix timestamp after which claims are no longer allowed (0 = disabled).
 */
claim_deadline: u64;
  /**
 * Address of the hunt creator who owns this pool.
 * Anyone may fund the pool (see `fund_reward_pool`); the creator is the
 * only address authorized to manage its configuration and to trigger
 * `refund_pool`, which pays out the remaining balance pro rata across
 * every address that funded it.
 */
creator: string;
  /**
 * Addresses allowed to distribute rewards for this pool.
 * Only the creator can manage this list.
 */
delegates: Array<string>;
  /**
 * Distribution mode (Fixed or Proportional).
 */
distribution_mode: DistributionMode;
  /**
 * Whether distributions from this pool are temporarily frozen.
 * When `true`, `distribute_rewards` and other distribution functions
 * will reject calls with `RewardErrorCode::PoolFrozen`.
 */
frozen: boolean;
  /**
 * Address that most recently froze this pool, or `None` when the pool is
 * not frozen. Tracks whether the current freeze was issued by the pool
 * creator or the contract admin so that an admin-issued freeze can only be
 * lifted by the admin (#1077).
 */
frozen_by: Option<string>;
  /**
 * Minimum XLM amount per distribution. 0 means no minimum enforced.
 */
min_distribution_amount: i128;
  /**
 * Minimum seconds between distributions (0 = disabled).
 */
min_distribution_interval_secs: u64;
  /**
 * Optional NFT contract address for NFT-only or mixed reward pools.
 */
nft_contract: Option<string>;
  /**
 * Creator royalty basis points (0-10000) for NFT secondary market sales.
 * Only applied when minting reward NFTs from this pool.
 */
nft_royalty_bps: u32;
  /**
 * Whether reward NFTs minted from this pool are transferable.
 * If false, NFTs are soulbound to the initial recipient.
 */
nft_transferable: boolean;
  /**
 * Optional exact-rank reward tiers. A matching frozen completion rank
 * takes precedence over flat and time-based amounts.
 */
rank_based_tiers: Array<RankRewardTier>;
  /**
 * Target funding amount for progress tracking (0 = disabled).
 */
target_amount: i128;
  /**
 * Optional time-based reward tiers. When empty, the per-winner amount
 * is computed from `xlm_pool / max_winners` as before. When populated,
 * the appropriate tier's `xlm_amount` is selected at distribution time
 * based on the player's (completion_time - registration_time) elapsed.
 */
time_based_tiers: Array<TimeBasedRewardTier>;
  /**
 * Token address for the reward pool (e.g., XLM, USDC, or other SAC tokens).
 */
token_address: string;
  /**
 * Optional vesting period in seconds. When > 0, XLM rewards are not
 * transferred immediately at distribution time. Instead, a `VestingRecord`
 * is created and the player must call `claim_vested` to receive tokens
 * proportionally as time elapses. 0 means vesting is disabled (instant payout).
 */
vesting_period_secs: u64;
}


/**
 * Full status of a reward pool, returned by get_reward_pool().
 */
export interface RewardPoolStatus {
  /**
 * Current available balance for distributions.
 */
balance: i128;
  /**
 * Pool creator / only authorized funder.
 */
creator: string;
  /**
 * Whether distributions from this pool are temporarily frozen.
 */
frozen: boolean;
  /**
 * Address that most recently froze this pool, or `None` when not frozen.
 * See `RewardPoolConfig::frozen_by` (#1077).
 */
frozen_by: Option<string>;
  /**
 * Minimum XLM per distribution (0 = no minimum).
 */
min_distribution_amount: i128;
  /**
 * Cumulative total deposited into this pool across all fund calls.
 */
total_deposited: i128;
  /**
 * Cumulative total distributed from this pool.
 */
total_distributed: i128;
  /**
 * Cumulative total migrated out of this pool via `migrate_pool`.
 * Together with `total_distributed` and `total_refunded` these satisfy:
 * `total_deposited == balance + total_distributed + total_refunded + total_migrated_out`
 */
total_migrated_out: i128;
}


/**
 * Result of a pool validation check, returned by validate_pool().
 */
export interface ValidationResult {
  /**
 * Current pool balance at time of check.
 */
balance: i128;
  /**
 * Whether the pool has sufficient funds for the required amount
 * and the required amount meets the pool's minimum distribution size.
 */
is_valid: boolean;
  /**
 * Required amount that was checked against.
 */
required: i128;
}


/**
 * On-chain receipt / proof of a completed reward distribution.
 */
export interface DistributionProof {
  /**
 * XLM amount distributed (stroops).
 */
amount: i128;
  /**
 * SHA-256 over (pool_id, player, amount, timestamp).
 */
hash: Buffer;
  /**
 * Recipient of the distribution.
 */
player: string;
  /**
 * Pool / hunt identifier.
 */
pool_id: u64;
  /**
 * Ledger timestamp when the distribution was recorded.
 */
timestamp: u64;
}


/**
 * Status of a reward distribution for a specific hunt and player.
 */
export interface DistributionStatus {
  /**
 * Whether any reward has been distributed.
 */
distributed: boolean;
  /**
 * NFT ID if an NFT was minted.
 */
nft_id: Option<u64>;
  /**
 * Whether NFT minting failed during distribution (retry available).
 */
nft_mint_failed: boolean;
  /**
 * XLM amount distributed (0 if none).
 */
xlm_amount: i128;
}


/**
 * Comprehensive statistics for a reward pool, returned by get_pool_statistics().
 */
export interface RewardPoolStatistics {
  /**
 * Average XLM amount per distribution (0 if none).
 */
avg_distribution: i128;
  /**
 * Number of successful distributions made from this pool.
 */
distribution_count: u64;
  /**
 * Ledger timestamp of the most recent distribution (0 if none).
 */
last_distribution_timestamp: u64;
  /**
 * Total XLM distributed from the pool.
 */
total_distributed: i128;
  /**
 * Total XLM funded (deposited) into the pool.
 */
total_funded: i128;
}


/**
 * Statistical summary of distributions across a reward pool,
 * returned by get_distribution_analytics().
 */
export interface DistributionAnalytics {
  /**
 * Average (mean) XLM amount per distribution (stroops). 0 if count is 0.
 */
average: i128;
  /**
 * Number of distributions included in the analytics window.
 */
count: u64;
  /**
 * Maximum XLM amount in a single distribution (stroops). 0 if count is 0.
 */
max: i128;
  /**
 * Median XLM amount across distributions (stroops). 0 if count is 0.
 */
median: i128;
  /**
 * Minimum XLM amount in a single distribution (stroops). 0 if count is 0.
 */
min: i128;
  /**
 * Total XLM distributed in the analytics window (stroops).
 */
total: i128;
}


/**
 * Entry for batch distribution calls.
 */
export interface BatchDistributionEntry {
  hunt_id: u64;
  player_address: string;
  reward_config: RewardConfig;
}


/**
 * Paginated response for the audit log.
 */
export interface PoolAuditLogResponse {
  entries: Array<PoolAuditEntry>;
  total: u64;
}


/**
 * Log entry for emergency withdrawal record-keeping.
 */
export interface EmergencyWithdrawalLogEntry {
  amount: i128;
  hunt_id: u64;
  reason: string;
  timestamp: u64;
}

export const RewardErrorCode = {
  2001: {message:"NotInitialized"},
  2002: {message:"InsufficientPool"},
  2003: {message:"AlreadyDistributed"},
  2004: {message:"TransferFailed"},
  2005: {message:"InvalidAmount"},
  2006: {message:"InvalidConfig"},
  2007: {message:"NftMintFailed"},
  /**
   * Attempted to create a pool that already exists for this hunt_id.
   */
  2008: {message:"PoolAlreadyExists"},
  /**
   * Pool has not been created yet via create_reward_pool().
   */
  2009: {message:"PoolNotFound"},
  /**
   * Caller is not the pool creator and is not authorized to fund this pool.
   */
  2010: {message:"Unauthorized"},
  /**
   * Distribution amount is below the pool's minimum distribution threshold.
   */
  2011: {message:"BelowMinimumAmount"},
  /**
   * Contract initialization can only happen once.
   */
  2012: {message:"AlreadyInitialized"},
  /**
   * hunt_id does not exist in HuntyCore (validated via cross-contract call).
   */
  2013: {message:"HuntNotFound"},
  /**
   * A recursive distribution attempt was detected during an external XLM or NFT call.
   */
  2014: {message:"ReentrancyDetected"},
  /**
   * The tracked pool balance diverged from the actual XLM token balance.
   */
  2015: {message:"PoolBalanceDivergence"},
  /**
   * Replay attack detected: distribution nonce state inconsistency.
   */
  2016: {message:"ReplayDetected"},
  /**
   * Pool balance would exceed maximum allowed limit.
   */
  2017: {message:"PoolBalanceOverflow"},
  /**
   * Funding amount is below the minimum threshold (dust attack prevention).
   */
  2018: {message:"BelowMinimumFunding"},
  /**
   * Single funding amount exceeds the maximum allowed.
   */
  2019: {message:"ExceedsMaximumFunding"},
  /**
   * Daily distribution cap for a specific pool has been exceeded.
   */
  2020: {message:"DailyCapExceeded"},
  /**
   * Global daily distribution cap across all pools has been exceeded.
   */
  2021: {message:"GlobalDailyCapExceeded"},
  /**
   * Contract is paused and cannot perform this operation.
   */
  2022: {message:"ContractPaused"},
  /**
   * No pending failed NFT mint found for retry.
   */
  2023: {message:"NftMintPendingNotFound"},
  /**
   * No distribution record exists for the given hunt/player.
   */
  2024: {message:"DistributionNotFound"},
  /**
   * The source pool is not eligible for migration: its hunt is neither
   */
  2025: {message:"SourcePoolNotEligible"},
  /**
   * The destination pool does not exist (must be created first).
   */
  2026: {message:"DestinationPoolNotFound"},
  /**
   * Source and destination refer to the same hunt, or there is no balance
   */
  2027: {message:"InvalidMigration"},
  /**
   * Pool is frozen and distributions have been temporarily disabled.
   */
  2028: {message:"PoolFrozen"},
  /**
   * Distribution rate limit not yet elapsed (cooldown period active).
   */
  2029: {message:"DistributionRateLimited"},
  /**
   * Batch size exceeds maximum allowed limit.
   */
  2030: {message:"BatchTooLarge"},
  /**
   * Invalid score value provided.
   */
  2031: {message:"InvalidScore"},
  /**
   * Token contract validation failed.
   */
  2032: {message:"InvalidTokenContract"},
  /**
   * No vesting record exists for the given hunt/player pair.
   */
  2033: {message:"VestingNotStarted"},
  /**
   * Player has already claimed the full vested amount.
   */
  2034: {message:"VestingAlreadyClaimed"},
  /**
   * Nothing has vested yet (elapsed time is zero or vesting just started).
   */
  2035: {message:"NothingToVest"},
  /**
   * The pool does not have vesting configured (vesting_period_secs == 0).
   */
  2036: {message:"VestingNotConfigured"},
  /**
   * Pool funding is paused (issue #628). Distribution may still be running.
   */
  2037: {message:"FundingPaused"},
  /**
   * Reward distribution is paused (issue #628). Funding may still be open.
   */
  2038: {message:"DistributionPaused"},
  /**
   * The pool already has the maximum number of distinct tracked funders;
   */
  2039: {message:"TooManyFunders"},
  /**
   * The hunt is not in a terminal state (cancelled or ended), so its pool
   */
  2040: {message:"InvalidHuntStatus"},
  /**
   * The hunt is in a terminal state (cancelled or ended), so its pool
   */
  2041: {message:"HuntTerminal"}
}


export interface ContractHealth {
  active_alerts: u32;
  avg_gas_units: u64;
  failed_invocations: u64;
  failure_rate_bps: u32;
  total_invocations: u64;
}


export interface MigrationReport {
  dry_run: boolean;
  from_version: u32;
  message: string;
  steps_applied: u32;
  succeeded: boolean;
  to_version: u32;
}


export interface UpgradeProposal {
  effective_at: u64;
  proposed_at: u64;
  proposer: string;
  target_version: u32;
}

export const UpgradeAuthError = {
  1: {message:"Unauthorized"},
  2: {message:"NoProposal"},
  3: {message:"TimelockPending"},
  4: {message:"VersionMismatch"},
  5: {message:"InvalidTimelock"}
}


export interface UpgradeHistoryEntry {
  executed_at: u64;
  executor: string;
  from_version: u32;
  to_version: u32;
}


/**
 * Configuration for distributing rewards across the HuntyCore ↔ RewardManager boundary.
 */
export interface RewardConfig {
  /**
 * The player's finishing position for this hunt, set by hunty-core at
 * completion time and threaded here so the NFT can record an immutable rank.
 */
completion_rank: u32;
  nft_contract: Option<string>;
  nft_description: string;
  nft_hunt_title: string;
  nft_image_uri: string;
  nft_rarity: u32;
  nft_tier: u32;
  nft_title: string;
  xlm_amount: Option<i128>;
}


/**
 * One exact completion-rank reward tier.
 * 
 * Ranks are one-based: rank `1` is the first finisher. A rank tier applies
 * only to the exact rank it names; ranks without an entry use the normal
 * flat/time-based reward path. The list is kept in strictly increasing rank
 * order so configuration is deterministic and cheap to validate on-chain.
 */
export interface RankRewardTier {
  /**
 * One-based completion rank awarded by HuntyCore.
 */
rank: u32;
  /**
 * Token amount awarded to the player who finishes at this rank.
 */
xlm_amount: i128;
}


/**
 * One tier of a time-based reward schedule configured on a reward pool.
 * 
 * A tier defines an XLM amount that is granted to a player who completes the
 * hunt within `max_completion_secs` of registering. Tiers must be stored in
 * ascending order by `max_completion_secs` — i.e. a "faster" tier must
 * appear before a "slower" tier. The first tier for which
 * `max_completion_secs >= elapsed` is selected at distribution time; if the
 * elapsed time exceeds every configured tier, the last (slowest) tier's
 * amount is used as a fallback so the player still receives a reward.
 */
export interface TimeBasedRewardTier {
  /**
 * Inclusive upper bound on elapsed time (completion_time - registration_time)
 * in seconds. Must be strictly increasing across the tier list.
 */
max_completion_secs: u64;
  /**
 * XLM amount awarded to a player who qualifies for this tier.
 */
xlm_amount: i128;
}

export interface Client {
  /**
   * Construct and simulate a pause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Pauses the contract, preventing reward distributions and withdrawals.
   * Only the contract admin can call this. Emits a ContractPausedEvent.
   */
  pause: ({admin, reason}: {admin: string, reason: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Unpauses the contract, resuming normal operations.
   * Only the contract admin can call this.
   */
  unpause: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_paused transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether the contract is currently paused.
   */
  is_paused: (options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes the RewardManager with the XLM token contract address (SAC).
   * Must be called once before any reward distribution.
   * @deprecated Use constructor during deployment instead.
   */
  initialize: ({admin, xlm_token, hunty_core}: {admin: string, xlm_token: string, hunty_core: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a freeze_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Freezes a reward pool, preventing any further distributions.
   * 
   * Can be called by either the pool creator or the contract admin.
   * Records who issued the freeze in `RewardPoolConfig::frozen_by`; an
   * admin-issued freeze can only be lifted by the admin (see
   * `unfreeze_pool`, #1077).
   * Emits a `PoolFrozenEvent`.
   * 
   * # Arguments
   * * `caller` - The address calling freeze (must be pool creator or admin)
   * * `hunt_id` - The hunt whose pool to freeze
   * 
   * # Errors
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `Unauthorized` - Caller is neither the pool creator nor the contract admin
   */
  freeze_pool: ({caller, hunt_id}: {caller: string, hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refund_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Refunds the remaining pool balance for a hunt, paid out **pro rata**
   * across every address that funded it (see `fund_reward_pool`) in
   * proportion to each funder's share of total contributions — never
   * paying one funder's contribution to another party. A pool funded by a
   * single address (the common case) simply gets its whole balance back.
   * 
   * Can only be triggered by the pool creator, who must authorize the
   * call; the payout destinations are the tracked funders, not the caller.
   * Uses the token address specified when the pool was created. The hunt
   * must be in a terminal state (cancelled or ended) when HuntyCore is
   * configured — refunding an active hunt's pool out from under its
   * players is rejected.
   * 
   * **Important:** This is a destructive operation. Ensure all distributions are complete
   * before calling this function, as any remaining unclaimed rewards cannot be distributed
   * after the pool is refunded.
   * 
   * # Accounting
   * This function updates:
   * - Pool balance: Set to 0
   * - Total refunded: Incremented by the refund amount
   * - Audit l
   */
  refund_pool: ({creator, hunt_id}: {creator: string, hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Step two of a two-step admin key rotation.
   */
  accept_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_delegate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds a delegate allowed to distribute rewards for a pool.
   * Only the pool creator can manage delegates.
   */
  add_delegate: ({creator, hunt_id, delegate}: {creator: string, hunt_id: u64, delegate: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a claim_vested transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Claims the proportionally vested XLM reward for the caller.
   * 
   * The claimable amount is: `total_amount * min(elapsed / vesting_period_secs, 1) - claimed_amount`.
   * 
   * The player can call this any number of times over the vesting period.
   * Each call transfers whatever has newly vested since the last claim.
   * Once `claimed_amount == total_amount` the schedule is fully exhausted.
   * 
   * # Arguments
   * * `player` - The player claiming their vested reward
   * * `hunt_id` - The hunt whose vesting record to claim from
   * 
   * # Returns
   * The XLM amount (in stroops) transferred to the player.
   * 
   * # Errors
   * * `VestingNotStarted` - No vesting record exists for this (hunt_id, player)
   * * `VestingAlreadyClaimed` - Full vesting amount has already been claimed
   * * `NothingToVest` - Nothing has vested yet at the current timestamp
   * * `InsufficientPool` - Contract token balance is too low (should not normally occur)
   */
  claim_vested: ({player, hunt_id}: {player: string, hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a migrate_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Migrates the unused balance of an expired or cancelled hunt's pool into
   * an existing destination pool owned by the same creator.
   * 
   * This lets a creator recycle funds locked in a finished hunt into a fresh
   * hunt without withdrawing and re-depositing. The XLM never leaves this
   * contract; only the internal per-hunt balance accounting is re-keyed.
   * 
   * # Eligibility (acceptance criteria)
   * * The source pool's hunt must be **expired or cancelled** — verified via
   * a cross-contract call to the configured HuntyCore contract
   * (`is_hunt_expired_or_cancelled`). If HuntyCore is not configured, the
   * source cannot be shown eligible and migration is rejected.
   * * The **destination pool must already exist** (created via
   * `create_reward_pool`).
   * * **Both pools must have the same creator**, who must authorize the call.
   * * **Both pools must use the same token.**
   * 
   * # Accounting
   * After a successful migration the following identities hold:
   * 
   * **Source pool:**
   * `total_deposited == balance(0) + total_distributed + total_refunded + total_migrated_out`
   * 
   * **
   */
  migrate_pool: ({creator, source_hunt_id, dest_hunt_id}: {creator: string, source_hunt_id: u64, dest_hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a pause_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Blocks pool funding. Distribution is unaffected unless separately paused.
   */
  pause_funding: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a run_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  run_migration: ({admin, target_version, dry_run}: {admin: string, target_version: u32, dry_run: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MigrationReport>>>

  /**
   * Construct and simulate a unfreeze_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Unfreezes a reward pool, re-enabling distributions.
   * 
   * Can be called by either the pool creator or the contract admin, except
   * that a freeze issued by the admin may only be lifted by the admin
   * (#1077). Any freezer other than the pool creator was the admin at the
   * time of the freeze, so this restriction also survives an admin rotation.
   * A frozen pool with no recorded freezer (freeze state written before
   * `frozen_by` existed) is treated as an admin freeze and can only be
   * lifted by the admin. Clears `RewardPoolConfig::frozen_by`.
   * Emits a `PoolUnfrozenEvent`.
   * 
   * # Arguments
   * * `caller` - The address calling unfreeze (must be pool creator or admin)
   * * `hunt_id` - The hunt whose pool to unfreeze
   * 
   * # Errors
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `Unauthorized` - Caller is neither the pool creator nor the contract
   * admin, or the current freeze was issued by the admin and the caller is
   * not the admin
   */
  unfreeze_pool: ({caller, hunt_id}: {caller: string, hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a validate_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Validates whether a pool can cover a given distribution amount.
   * 
   * Checks that:
   * - The pool exists (was created via create_reward_pool)
   * - The required_amount is positive, except that `required_amount == 0` is
   * valid for pools with an NFT contract (NFT-only pools), which hold no
   * token balance by design (#1088)
   * - The pool balance >= required_amount
   * - The required_amount meets the pool's minimum distribution threshold (if set)
   * 
   * Returns a `ValidationResult` with balance details regardless of validity,
   * so callers can diagnose shortfalls without a separate query.
   */
  validate_pool: ({hunt_id, required_amount}: {hunt_id: u64, required_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<ValidationResult>>

  /**
   * Construct and simulate a is_pool_frozen transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether a reward pool is currently frozen.
   * Returns `false` if no pool exists for the given `hunt_id`.
   */
  is_pool_frozen: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a set_hunty_core transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the optional HuntyCore contract address used to validate hunt_id existence
   * in `create_reward_pool`. When set, pool creation will be rejected for unknown
   * hunt IDs. If not set, hunt_id is assumed caller-trusted.
   */
  set_hunty_core: ({admin, hunty_core}: {admin: string, hunty_core: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_pool_tiers transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates (or installs) the time-based reward tier schedule on an existing
   * reward pool, enabling conditional reward amounts based on player completion
   * time (acceptance criteria: "Define time-based reward tiers in pool config").
   * 
   * Tiers must be supplied in strictly ascending order of `max_completion_secs`
   * (i.e. faster tiers first), and every `xlm_amount` must be strictly positive.
   * Passing an empty `Vec` disables tier-based rewards so the pool reverts
   * to the flat `xlm_pool / max_winners` amount.
   * 
   * Only the pool creator is authorized to call this. The new tiers are
   * persisted immediately and become effective for any subsequent distribution
   * call. Already-distributed rewards are not affected.
   * 
   * # Arguments
   * * `creator` - The pool creator (must match the stored creator)
   * * `hunt_id` - The hunt whose pool config to update
   * * `time_based_tiers` - New tier list (strictly ascending by time, all amounts > 0;
   * an empty list disables tier-based rewards)
   * 
   * # Errors
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `Unauthorized` -
   */
  set_pool_tiers: ({creator, hunt_id, time_based_tiers}: {creator: string, hunt_id: u64, time_based_tiers: Array<TimeBasedRewardTier>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_pause_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Effective pause state as `(global, funding, distribution)`.
   * 
   * The two granular values are the *effective* ones, so they read `true`
   * whenever the global stop is engaged. Mirrors `HuntyCore::get_pause_state`.
   */
  get_pause_state: (options?: MethodOptions) => Promise<AssembledTransaction<readonly [boolean, boolean, boolean]>>

  /**
   * Construct and simulate a get_pool_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the full configuration of a reward pool, including its time
   * and exact-rank tier lists. `None` when no pool exists for the hunt.
   * 
   * This is the read path used by HuntyCore at completion time to resolve
   * rank- and time-based amounts without duplicating pool state.
   */
  get_pool_config: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<RewardPoolConfig>>>

  /**
   * Construct and simulate a get_reward_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the full status of a reward pool, including balance, totals, and configuration.
   * Returns None if no pool has been created for the given hunt_id.
   */
  get_reward_pool: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<RewardPoolStatus>>>

  /**
   * Construct and simulate a propose_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_upgrade: ({admin, target_version}: {admin: string, target_version: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<UpgradeProposal>>>

  /**
   * Construct and simulate a remove_delegate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes a delegate from a pool.
   * Only the pool creator can manage delegates.
   */
  remove_delegate: ({creator, hunt_id, delegate}: {creator: string, hunt_id: u64, delegate: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Resumes pool funding. Has no effect while the global pause is engaged.
   */
  unpause_funding: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a contract_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the on-chain version stored during initialize, or the compiled constant.
   */
  contract_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a distribute_batch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Distributes rewards to multiple players in a single atomic transaction.
   * 
   * Every entry in the batch is validated first (no state changes). If all
   * entries pass validation, all transfers are executed. If any single entry
   * fails validation, the entire batch is rejected with no state changes.
   * 
   * # Atomicity guarantee
   * 
   * The two-phase design (validate-all, execute-all) means callers get a
   * simple all-or-nothing contract:
   * - If the function returns `Ok(())`, every entry was processed.
   * - If it returns `Err(_)`, no tokens were moved and no distribution
   * records were created.
   * 
   * # Gas limit consideration
   * 
   * The batch size is capped at [`MAX_BATCH_SIZE`] (10 entries) to keep the
   * transaction within Soroban's per-transaction instruction budget even
   * when every entry performs both XLM and NFT operations.
   * 
   * # Arguments
   * * `distributions` - A `Vec` of `BatchDistributionEntry`, each containing
   * a `hunt_id`, `player_address`, and `reward_config`.
   * 
   * # Errors
   * * `InvalidConfig` - Batch is empty or an entry has an invalid config.
   * * `BatchTooLarge` 
   */
  distribute_batch: ({distributions}: {distributions: Array<BatchDistributionEntry>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a fund_reward_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Funds the reward pool for a specific hunt.
   * 
   * The pool must have been created via `create_reward_pool` first.
   * **Anyone may fund a pool** — this supports sponsorship (a brand funding
   * a community hunt, a DAO topping up a pool, several people pooling a
   * prize), not just the creator. Each funder must authorize the call
   * themselves; their contribution is tracked individually so that
   * `refund_pool` can later pay the remaining balance back out in
   * proportion to what each funder put in, and never hand one funder's
   * contribution to another party. See `docs/adr/006-reward-pool-sponsorship.md`.
   * 
   * Transfers tokens from the funder to this contract and records the balance.
   * Uses the token address specified when the pool was created.
   * 
   * # Validation
   * - Minimum funding: 1 XLM equivalent (10,000,000 base units) to prevent dust attacks
   * - Maximum single funding: 1 billion tokens to prevent overflow
   * - Pool balance limit: 1 billion tokens total to prevent overflow
   * - Rejects zero or negative amounts
   * - At most `MAX_FUNDERS_PER_POOL` distinct
   */
  fund_reward_pool: ({funder, hunt_id, amount}: {funder: string, hunt_id: u64, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_pool_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current reward pool balance for a hunt.
   */
  get_pool_balance: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_pool_funders transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the distinct addresses currently tracked as funders of a pool
   * (i.e. that have contributed and not yet been refunded), in the order
   * they first contributed. Empty if the pool has never been funded, has
   * been fully refunded, or has no sponsorship ledger (see `refund_pool`).
   */
  get_pool_funders: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_dist_cooldown transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Remaining seconds until the next distribution is allowed for this pool.
   * Returns 0 if no interval is configured or the cooldown has elapsed.
   */
  get_dist_cooldown: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_schema: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a propose_new_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Step one of a two-step admin key rotation.
   */
  propose_new_admin: ({admin, new_admin}: {admin: string, new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a admin_withdraw_all transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Explicitly withdraws the entire remaining balance from a reward pool.
   * 
   * This function provides an explicit, intentional way to drain a pool completely.
   * Unlike `admin_withdraw_unclaimed`, which handles partial withdrawals of unclaimed
   * amounts, this function is semantically clear: it empties the pool by name.
   * 
   * Withdrawal is only permitted after the hunt has ended (end_time passed) or been
   * cancelled. This prevents draining pools while a hunt is active and players may
   * still be mid-game. When HuntyCore is configured, the hunt status is verified.
   * 
   * # Arguments
   * * `admin` - The contract admin address (must match the stored admin)
   * * `hunt_id` - The hunt whose pool to drain completely
   * * `recipient` - The address that will receive the full pool balance
   * 
   * # Errors
   * * `NotInitialized` - Contract has not been initialized (no admin set)
   * * `Unauthorized` - Caller is not the contract admin
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `InvalidAmount` - Pool balance is zero (nothing to withdraw)
   * * `InvalidHuntStatus` - Hunt
   */
  admin_withdraw_all: ({admin, hunt_id, recipient}: {admin: string, hunt_id: u64, recipient: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a create_reward_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a reward pool for a specific hunt with a specified token.
   * 
   * Must be called before `fund_reward_pool`. Any address may fund the pool
   * after creation (see `fund_reward_pool`); the token contract must be
   * SAC-compatible.
   * 
   * # Arguments
   * * `creator` - The hunt creator who will own and fund the pool
   * * `hunt_id` - The hunt this pool is for
   * * `token_address` - Address of the SAC-compatible token contract (e.g., XLM, USDC)
   * * `min_distribution_amount` - Minimum token amount per distribution (0 = no minimum)
   * * `nft_royalty_bps` - Creator royalty basis points (0-10000) for secondary market sales
   * * `nft_transferable` - Whether reward NFTs from this pool are transferable
   * 
   * # Errors
   * * `PoolAlreadyExists` - A pool already exists for this hunt_id
   * * `InvalidAmount` - min_distribution_amount is negative
   * * `InvalidTokenContract` - token_address is not a valid SAC-compatible token
   * * `NotInitialized` - hunty_core has not been configured (set during initialize)
   * * `HuntNotFound` - hunt_id does not exist in HuntyCore
   */
  create_reward_pool: ({creator, hunt_id, token_address, min_distribution_amount, nft_royalty_bps, nft_transferable}: {creator: string, hunt_id: u64, token_address: string, min_distribution_amount: i128, nft_royalty_bps: u32, nft_transferable: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Legacy entrypoint retained for existing integrations. New contract
   * integrations should use `distribute_rewards_authorized`, which carries
   * and authenticates the calling contract explicitly.
   * 
   * Authorization is fail-closed: the caller must be an authorized
   * distributor (see `add_authorized_contract`). Unauthorized callers
   * receive `Unauthorized`.
   */
  distribute_rewards: ({caller, hunt_id, player_address, reward_config}: {caller: string, hunt_id: u64, player_address: string, reward_config: RewardConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a emergency_withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Emergency withdrawal: allows the admin to withdraw all funds from one or all
   * reward pools when the contract is paused (e.g. due to a critical vulnerability).
   * When `hunt_id` is 0, all pools with non-zero balances are drained.
   * When `all_pools` is true, iterates all hunts up to `max_hunt_id` and withdraws.
   * 
   * # Arguments
   * * `admin` - The contract admin address
   * * `hunt_id` - Specific hunt pool to drain (0 = all pools up to max_hunt_id)
   * * `recipient` - Address to receive the withdrawn funds
   * * `reason` - Reason for the emergency withdrawal (emitted in events)
   * * `max_hunt_id` - When hunt_id is 0, drains all pools from 1..=max_hunt_id
   * 
   * # Errors
   * * `NotInitialized` - Contract not initialized
   * * `Unauthorized` - Caller is not admin
   * * `ContractPaused` - Contract must be paused to call this
   */
  emergency_withdraw: ({admin, hunt_id, recipient, reason, max_hunt_id}: {admin: string, hunt_id: u64, recipient: string, reason: string, max_hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_emergency_logs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the emergency withdrawal log entries.
   */
  get_emergency_logs: (options?: MethodOptions) => Promise<AssembledTransaction<Array<EmergencyWithdrawalLogEntry>>>

  /**
   * Construct and simulate a get_pool_audit_log transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Exposes a paginated read query for the audit log of a given pool.
   */
  get_pool_audit_log: ({hunt_id, start_after, limit}: {hunt_id: u64, start_after: Option<u64>, limit: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<PoolAuditLogResponse>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_vesting_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current vesting status for a (hunt_id, player) pair.
   * 
   * Returns `None` when no vesting record exists (i.e. the pool either had
   * no vesting configured or the player has not completed that hunt yet).
   */
  get_vesting_status: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<VestingStatus>>>

  /**
   * Construct and simulate a pause_distribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Blocks reward distribution. Funding is unaffected unless separately paused.
   */
  pause_distribution: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a rollback_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rollback_migration: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MigrationReport>>>

  /**
   * Construct and simulate a set_daily_pool_cap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the daily distribution cap for a specific pool.
   * 
   * This limit controls the maximum amount of rewards that can be distributed from
   * a pool in a single day (24-hour rolling window). This is a live operational control
   * and should be validated to prevent silent misconfiguration.
   * 
   * # Arguments
   * * `admin` - The contract admin address (must match the stored admin)
   * * `hunt_id` - The hunt whose pool cap to set
   * * `cap` - The maximum amount to distribute per day. Must be non-negative.
   * A cap of 0 **disables all distributions** from this pool (the
   * distribution path rejects every attempt with `DailyCapExceeded`).
   * Use `freeze_pool` for a semantically richer freeze. A positive
   * value sets a rolling 24-hour distribution limit.
   * 
   * # Errors
   * * `NotInitialized` - Contract has not been initialized (no admin set)
   * * `Unauthorized` - Caller is not the contract admin
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `InvalidAmount` - Cap is negative (negative caps silently block distributions)
   */
  set_daily_pool_cap: ({admin, hunt_id, cap}: {admin: string, hunt_id: u64, cap: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_pool_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates the `min_distribution_amount` for an existing reward pool.
   * 
   * Only the pool creator is authorized to call this. Useful when a creator
   * has underfunded the pool and needs to lower the minimum so distributions
   * can proceed.
   * 
   * # Arguments
   * * `creator` - The pool creator (must match the stored creator)
   * * `hunt_id` - The hunt whose pool config to update
   * * `min_distribution_amount` - New minimum XLM per distribution (0 = no minimum)
   * 
   * # Errors
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `Unauthorized` - Caller is not the pool creator
   * * `InvalidAmount` - min_distribution_amount is negative
   */
  update_pool_config: ({creator, hunt_id, min_distribution_amount}: {creator: string, hunt_id: u64, min_distribution_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_pool_statistics transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns comprehensive statistics for a reward pool.
   * Returns None if no pool has been created for the given hunt_id.
   */
  get_pool_statistics: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<RewardPoolStatistics>>>

  /**
   * Construct and simulate a get_raw_pause_flags transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * The granular flags as stored, ignoring the global stop — lets an
   * operator see what will still be paused after `unpause()`.
   */
  get_raw_pause_flags: (options?: MethodOptions) => Promise<AssembledTransaction<readonly [boolean, boolean]>>

  /**
   * Construct and simulate a get_upgrade_history transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_upgrade_history: ({offset, limit}: {offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<UpgradeHistoryEntry>>>

  /**
   * Construct and simulate a set_pool_rank_tiers transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates (or installs) exact completion-rank reward tiers on an existing pool.
   * 
   * Ranks are one-based and the list must contain strictly increasing ranks
   * with strictly positive amounts. A matching rank is selected using the
   * immutable completion rank supplied by HuntyCore; ranks not present in
   * the list retain the existing flat/time-based behavior. Passing an empty
   * list disables rank-based rewards.
   * 
   * Only the pool creator may change this configuration. Changes affect
   * subsequent distributions and never rewrite an already-recorded payout.
   * 
   * # Errors
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `Unauthorized` - Caller is not the pool creator
   * * `InvalidConfig` - Tier list is longer than [`MAX_TIER_ENTRIES`], or
   * (when non-empty) is not strictly ascending with positive amounts
   */
  set_pool_rank_tiers: ({creator, hunt_id, rank_based_tiers}: {creator: string, hunt_id: u64, rank_based_tiers: Array<RankRewardTier>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a verify_distribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Verifies a distribution proof against the on-chain receipt.
   * 
   * Recomputes SHA-256(pool_id || player || amount || timestamp) and checks
   * it matches both the provided `hash` and the stored receipt (when present).
   */
  verify_distribution: ({pool_id, player, amount, timestamp, hash}: {pool_id: u64, player: string, amount: i128, timestamp: u64, hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_health_dashboard transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_health_dashboard: (options?: MethodOptions) => Promise<AssembledTransaction<ContractHealth>>

  /**
   * Construct and simulate a get_upgrade_proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_upgrade_proposal: (options?: MethodOptions) => Promise<AssembledTransaction<Option<UpgradeProposal>>>

  /**
   * Construct and simulate a get_upgrade_timelock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_upgrade_timelock: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a set_daily_global_cap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_daily_global_cap: ({admin, cap}: {admin: string, cap: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_upgrade_timelock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_upgrade_timelock: ({admin, delay_seconds}: {admin: string, delay_seconds: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause_distribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Resumes reward distribution. Has no effect while the global pause is engaged.
   */
  unpause_distribution: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_reward_distributed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether a reward has been distributed to a player for a hunt.
   */
  is_reward_distributed: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a retry_failed_nft_mint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Retries a failed NFT mint for a previously distributed reward.
   * 
   * When NFT minting fails during `distribute_rewards`, the failure is logged
   * and the pending mint data is stored. This function allows the player
   * (or anyone paying the transaction fee on behalf of the player) to
   * retry the failed NFT mint and update the distribution record. The
   * successfully minted NFT is always sent to the `player` address
   * recorded in the pending mint, regardless of who calls this function.
   * 
   * # Arguments
   * * `caller` - The address signing the transaction (any address; NFT is
   * still delivered to the `player` recorded in the pending mint)
   * * `hunt_id` - The hunt associated with the failed NFT mint
   * * `player` - The player who should receive the NFT
   * 
   * # Returns
   * The NFT ID of the successfully minted NFT
   * 
   * # Errors
   * * `NftMintPendingNotFound` - No pending failed NFT mint for this hunt/player
   * * `PoolNotFound` - No pool config exists for this hunt_id
   * * `NftMintFailed` - NFT mint attempt failed again
   */
  retry_failed_nft_mint: ({caller, hunt_id, player}: {caller: string, hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a set_distribution_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the distribution mode (Fixed or Proportional) for a pool.
   */
  set_distribution_mode: ({creator, hunt_id, mode}: {creator: string, hunt_id: u64, mode: DistributionMode}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_pool_nft_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets or updates the NFT contract address for an existing reward pool.
   * This allows pools to distribute NFTs alongside or instead of tokens.
   * 
   * # Arguments
   * * `creator` - The pool creator (must match the stored creator)
   * * `hunt_id` - The hunt whose pool config to update
   * * `nft_contract` - NFT contract address (or None to disable NFT rewards)
   * 
   * # Errors
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `Unauthorized` - Caller is not the pool creator
   */
  set_pool_nft_contract: ({creator, hunt_id, nft_contract}: {creator: string, hunt_id: u64, nft_contract: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_distribution_proof transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the on-chain distribution receipt/proof for a hunt/player pair.
   */
  get_distribution_proof: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<DistributionProof>>>

  /**
   * Construct and simulate a get_pool_distributions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a paginated list of distributions made from a specific reward pool.
   * 
   * # Arguments
   * * `hunt_id` - The hunt whose pool distributions to query
   * * `offset` - Starting index for pagination (0-based)
   * * `limit` - Maximum number of entries to return
   * 
   * # Returns
   * A Vec of PoolDistribution entries containing player addresses and distribution details.
   * Returns an empty Vec if the pool has no distributions or offset is beyond the list.
   */
  get_pool_distributions: ({hunt_id, offset, limit}: {hunt_id: u64, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<PoolDistribution>>>

  /**
   * Construct and simulate a list_pending_nft_mints transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a paginated list of all pending failed NFT mints across the
   * entire contract.
   * 
   * Each entry contains the full mint metadata (hunt, player, NFT contract,
   * rarity, etc.) so callers can identify which mints need to be retried.
   * 
   * # Arguments
   * * `offset` - Starting index for pagination (0-based)
   * * `limit` - Maximum number of entries to return
   * 
   * # Returns
   * A `Vec<PendingNftMint>` of pending mint entries, up to `limit` entries
   * starting from `offset`. Returns an empty `Vec` when `offset` is beyond
   * the end of the list or when no pending mints exist.
   */
  list_pending_nft_mints: ({offset, limit}: {offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<PendingNftMint>>>

  /**
   * Construct and simulate a set_pool_target_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the funding target used for top-up progress notifications.
   * `target_amount` of 0 disables percentage tracking (events report 0%).
   */
  set_pool_target_amount: ({creator, hunt_id, target_amount}: {creator: string, hunt_id: u64, target_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_authorized_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds a contract to the authorized callers list for `distribute_rewards`.
   * Only the contract admin can call this.
   */
  add_authorized_contract: ({admin, contract}: {admin: string, contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_proportional transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Distribute a proportional share of the pool based on player score.
   * 
   * Amount = floor((player_score / total_scores) * pool_balance).
   * Remainder stays in the pool. Enforces min_distribution_amount when set.
   * Requires the pool's distribution_mode to be Proportional (or will still
   * compute proportionally when called via this entry point).
   * 
   * Returns the XLM amount distributed.
   */
  distribute_proportional: ({hunt_id, player, player_score, total_scores}: {hunt_id: u64, player: string, player_score: u64, total_scores: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_distribution_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the distribution status for a hunt/player pair.
   */
  get_distribution_status: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<DistributionStatus>>

  /**
   * Construct and simulate a set_nft_reward_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the default NftReward contract address used for NFT distributions
   * when a per-call NFT contract is not provided.
   * Emits an NftContractSetEvent with the old and new contract addresses.
   */
  set_nft_reward_contract: ({admin, nft_contract}: {admin: string, nft_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_vesting_period_secs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the vesting period (in seconds) on an existing reward pool.
   * 
   * When `vesting_period_secs > 0`, subsequent `distribute_rewards` calls
   * will **not** transfer XLM immediately. Instead a `VestingRecord` is
   * stored and the player must call `claim_vested` to receive tokens
   * proportionally as time elapses after distribution.
   * 
   * Setting this to `0` disables vesting and reverts to instant payouts for
   * future distributions (already-pending vesting records are unaffected).
   * 
   * # Arguments
   * * `creator` - Pool owner (must match stored creator)
   * * `hunt_id` - The hunt whose pool to configure
   * * `vesting_period_secs` - Vesting duration in seconds (0 = disabled)
   * 
   * # Errors
   * * `PoolNotFound` - Pool does not exist
   * * `Unauthorized` - Caller is not the pool creator
   */
  set_vesting_period_secs: ({creator, hunt_id, vesting_period_secs}: {creator: string, hunt_id: u64, vesting_period_secs: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a admin_withdraw_unclaimed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Allows the admin to withdraw unclaimed (surplus) XLM remaining in a reward pool
   * after the hunt has ended and all winners have been determined.
   * 
   * This is needed when a hunt concludes with fewer winners than anticipated,
   * leaving unspent XLM locked in the pool. Only the contract admin may call this.
   * 
   * Withdrawal is only permitted after the hunt has ended (end_time passed) or been
   * cancelled. This prevents draining pools while a hunt is active and players may
   * still be mid-game. When HuntyCore is configured, the hunt status is verified.
   * 
   * # Arguments
   * * `admin` - The contract admin address (must match the stored admin)
   * * `hunt_id` - The hunt whose remaining pool balance to withdraw
   * * `recipient` - The address that will receive the withdrawn XLM
   * * `amount` - The amount to withdraw. Must be positive (> 0).
   * 
   * # Errors
   * * `NotInitialized` - Contract has not been initialized (no admin set)
   * * `Unauthorized` - Caller is not the contract admin
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `InvalidAmount` - Amount is <= 0,
   */
  admin_withdraw_unclaimed: ({admin, hunt_id, recipient, amount}: {admin: string, hunt_id: u64, recipient: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_rewards_legacy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Legacy entry point for XLM-only distribution.
   * Kept for backward compatibility with HuntyCore. For NFT or full config support use distribute_rewards.
   * 
   * Note: `nft_enabled` is ignored — NFT distribution requires metadata and a contract address
   * that are not available on this path. Use `distribute_rewards` with `RewardConfig` instead.
   * **DEPRECATED: Do not use for new integrations.**
   * 
   * This legacy distribution path is maintained only for backward compatibility.
   * All new integrations must use `distribute_rewards` instead.
   * 
   * This function wraps `distribute_rewards` and therefore inherits all the same
   * security constraints:
   * - Replays are rejected via the same nonce-based mechanism
   * - The ReentrancyGuard is acquired identically
   * - `min_distribution_amount` and daily caps are enforced
   * - Authorization is fail-closed: the immediate invoker must be an approved contract and the allowlist must not be empty
   * 
   * **Removal timeline:** This function is scheduled for removal in a future major release.
   * The exact deprecation timeline will
   */
  distribute_rewards_legacy: ({player, hunt_id, xlm_amount, nft_enabled}: {player: string, hunt_id: u64, xlm_amount: i128, nft_enabled: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_total_xlm_distributed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total XLM distributed across all hunts (protocol-level metric).
   */
  get_total_xlm_distributed: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a admin_resolve_distribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Manually resolves a distribution that failed mid-execution.
   * 
   * Allows the contract admin to mark a distribution as either `Completed`
   * or `Refunded` when the automatic distribution process could not finish
   * (e.g., XLM was sent but NFT mint failed). This is a bookkeeping-only
   * operation and does not move funds.
   * 
   * # Arguments
   * * `admin` - The contract admin address (must match the stored admin)
   * * `hunt_id` - The hunt whose distribution to resolve
   * * `player` - The player whose distribution to resolve
   * * `resolution` - Outcome: `ResolutionStatus::Completed` or `ResolutionStatus::Refunded`
   * 
   * # Errors
   * * `NotInitialized` - Contract has not been initialized (no admin set)
   * * `Unauthorized` - Caller is not the contract admin
   * * `DistributionNotFound` - No distribution record exists for this hunt/player
   */
  admin_resolve_distribution: ({admin, hunt_id, player, resolution}: {admin: string, hunt_id: u64, player: string, resolution: ResolutionStatus}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_distribution_analytics transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns distribution analytics (average, median, min, max) across a reward pool.
   * 
   * Supports optional time-range filtering via `start_time` and `end_time`
   * (ledger timestamps). Only distributions within `[start_time, end_time)`
   * are included when both bounds are provided; `None` means unbounded.
   * 
   * The computation is gas-bounded: at most [`MAX_ANALYTICS_ENTRIES`] (500)
   * distributions are processed. If the pool has more entries than this limit,
   * only the most recent entries (up to the limit) are analysed.
   * 
   * # Arguments
   * * `hunt_id` - The hunt whose pool analytics to query
   * * `start_time` - Optional lower bound (inclusive) ledger timestamp filter
   * * `end_time` - Optional upper bound (exclusive) ledger timestamp filter
   * 
   * # Returns
   * A `DistributionAnalytics` struct with count, total, average, median, min, max.
   * All fields are zero when the pool has no distributions or no entries match
   * the time filter.
   */
  get_distribution_analytics: ({hunt_id, start_time, end_time}: {hunt_id: u64, start_time: Option<u64>, end_time: Option<u64>}, options?: MethodOptions) => Promise<AssembledTransaction<DistributionAnalytics>>

  /**
   * Construct and simulate a remove_authorized_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes a contract from the authorized callers list.
   * Only the contract admin can call this.
   */
  remove_authorized_contract: ({admin, contract}: {admin: string, contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a create_reward_pool_with_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a reward pool for a specific hunt with a specified token.
   * 
   * Must be called before `fund_reward_pool`. Any address may fund the pool
   * after creation (see `fund_reward_pool`); the token contract must be
   * SAC-compatible.
   * 
   * For NFT-only pools (pools that distribute only NFTs without any token component),
   * set `min_distribution_amount` to 0 and provide an `nft_contract` address.
   * 
   * # Arguments
   * * `creator` - The hunt creator who will own and fund the pool
   * * `hunt_id` - The hunt this pool is for
   * * `token_address` - Address of the SAC-compatible token contract (e.g., XLM, USDC)
   * * `min_distribution_amount` - Minimum token amount per distribution (0 for NFT-only pools)
   * * `nft_contract` - Optional NFT contract address for NFT rewards
   * * `nft_royalty_bps` - Creator royalty basis points (0-10000) for secondary market sales
   * * `nft_transferable` - Whether reward NFTs from this pool are transferable
   * 
   * # Errors
   * * `PoolAlreadyExists` - A pool already exists for this hunt_id
   * * `InvalidAmount` - min_distribution_amount is negativ
   */
  create_reward_pool_with_nft: ({creator, hunt_id, token_address, min_distribution_amount, nft_contract, nft_royalty_bps, nft_transferable}: {creator: string, hunt_id: u64, token_address: string, min_distribution_amount: i128, nft_contract: Option<string>, nft_royalty_bps: u32, nft_transferable: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_batch_authorized transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Batch distribution entrypoint for an explicitly authenticated caller.
   */
  distribute_batch_authorized: ({caller, distributions}: {caller: string, distributions: Array<BatchDistributionEntry>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_min_distribution_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the minimum distribution amount configured for a hunt's reward pool.
   * Returns 0 if no pool has been created for the hunt.
   */
  get_min_distribution_amount: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_pool_distribution_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total count of distributions made from a specific reward pool.
   * 
   * # Arguments
   * * `hunt_id` - The hunt whose pool distribution count to query
   * 
   * # Returns
   * The total number of distributions for the pool.
   */
  get_pool_distribution_count: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_pool_funder_contribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns how much `funder` has contributed to a pool that has not yet
   * been refunded. 0 if they have never funded it or were already refunded.
   */
  get_pool_funder_contribution: ({hunt_id, funder}: {hunt_id: u64, funder: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a distribute_rewards_authorized transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Distribution entrypoint for an explicitly authenticated caller contract.
   */
  distribute_rewards_authorized: ({caller, hunt_id, player_address, reward_config}: {caller: string, hunt_id: u64, player_address: string, reward_config: RewardConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_min_distribution_interval transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the minimum seconds between distributions for a pool (0 disables).
   */
  set_min_distribution_interval: ({creator, hunt_id, min_distribution_interval_secs}: {creator: string, hunt_id: u64, min_distribution_interval_secs: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a check_nft_reward_compatibility transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns true if the given NftReward contract meets the minimum required version.
   * Returns false on any error (e.g. the address is not an nft-reward contract
   * or an old one without `contract_version`) instead of trapping.
   */
  check_nft_reward_compatibility: ({nft_reward_address}: {nft_reward_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, xlm_token, hunty_core}: {admin: string, xlm_token: string, hunty_core: string},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin, xlm_token, hunty_core}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAwAAAJhPcGVyYXRpb24gdHlwZSBmb3IgdGhlIHBvb2wgYXVkaXQgbG9nLgoKVmFsdWVzIGFyZSBleHBsaWNpdCBhbmQgYXBwZW5kLW9ubHkgc28gYWRkaW5nIG9wZXJhdGlvbnMgZG9lcyBub3QgcmVudW1iZXIKcmVjb3JkcyB3cml0dGVuIGJ5IG9sZGVyIGRlcGxveW1lbnRzLgAAAAAAAAANUG9vbE9wZXJhdGlvbgAAAAAAABIAAAAAAAAABkNyZWF0ZQAAAAAAAAAAAAAAAAAERnVuZAAAAAEAAAAAAAAACkRpc3RyaWJ1dGUAAAAAAAIAAAApRnVuZHMgd2VyZSB3aXRoZHJhd24gYnkgYW4gYWRtaW5pc3RyYXRvci4AAAAAAAAIV2l0aGRyYXcAAAADAAAAOFRoZSBwb29sIHdhcyBmcm96ZW4gYnkgaXRzIGNyZWF0b3Igb3IgdGhlIGFkbWluaXN0cmF0b3IuAAAABkZyZWV6ZQAAAAAABAAAADpUaGUgcG9vbCB3YXMgdW5mcm96ZW4gYnkgaXRzIGNyZWF0b3Igb3IgdGhlIGFkbWluaXN0cmF0b3IuAAAAAAAIVW5mcmVlemUAAAAFAAAAQVVudXNlZCBiYWxhbmNlIHdhcyBtaWdyYXRlZCBvdXQgdG8gKG9yIGludG8pIGFub3RoZXIgaHVudCdzIHBvb2wuAAAAAAAAB01pZ3JhdGUAAAAABgAAADBVbnVzZWQgYmFsYW5jZSB3YXMgcmVmdW5kZWQgdG8gdGhlIHBvb2wgY3JlYXRvci4AAAAGUmVmdW5kAAAAAAAHAAAATmB1cGRhdGVfcG9vbF9jb25maWdgIGNoYW5nZWQgYG1pbl9kaXN0cmlidXRpb25fYW1vdW50YCAoYGFtb3VudGAgPSBuZXcgdmFsdWUpLgAAAAAAD1VwZGF0ZU1pbkFtb3VudAAAAAAIAAAAS2BzZXRfcG9vbF90YXJnZXRfYW1vdW50YCBjaGFuZ2VkIHRoZSBmdW5kaW5nIHRhcmdldCAoYGFtb3VudGAgPSBuZXcgdmFsdWUpLgAAAAAPU2V0VGFyZ2V0QW1vdW50AAAAAAkAAABCYHNldF9taW5fZGlzdHJpYnV0aW9uX2ludGVydmFsYCBjaGFuZ2VkIHRoZSBkaXN0cmlidXRpb24gY29vbGRvd24uAAAAAAAXU2V0RGlzdHJpYnV0aW9uSW50ZXJ2YWwAAAAACgAAAEBgc2V0X2Rpc3RyaWJ1dGlvbl9tb2RlYCBzd2l0Y2hlZCBiZXR3ZWVuIEZpeGVkIGFuZCBQcm9wb3J0aW9uYWwuAAAAE1NldERpc3RyaWJ1dGlvbk1vZGUAAAAACwAAAD9gc2V0X3Bvb2xfbmZ0X2NvbnRyYWN0YCBzZXQgb3IgY2xlYXJlZCB0aGUgcG9vbCdzIE5GVCBjb250cmFjdC4AAAAADlNldE5mdENvbnRyYWN0AAAAAAAMAAAANmBhZGRfZGVsZWdhdGVgIGF1dGhvcmlzZWQgYSBuZXcgZGlzdHJpYnV0aW9uIGRlbGVnYXRlLgAAAAAAC0FkZERlbGVnYXRlAAAAAA0AAAAyYHJlbW92ZV9kZWxlZ2F0ZWAgcmV2b2tlZCBhIGRpc3RyaWJ1dGlvbiBkZWxlZ2F0ZS4AAAAAAA5SZW1vdmVEZWxlZ2F0ZQAAAAAADgAAADVgc2V0X3Zlc3RpbmdfcGVyaW9kX3NlY3NgIGNoYW5nZWQgdGhlIHZlc3RpbmcgcGVyaW9kLgAAAAAAABBTZXRWZXN0aW5nUGVyaW9kAAAADwAAADdgc2V0X3Bvb2xfdGllcnNgIHJlcGxhY2VkIHRoZSB0aW1lLWJhc2VkIHRpZXIgc2NoZWR1bGUuAAAAAAxTZXRUaW1lVGllcnMAAAAQAAAAPGBzZXRfcG9vbF9yYW5rX3RpZXJzYCByZXBsYWNlZCB0aGUgcmFuay1iYXNlZCB0aWVyIHNjaGVkdWxlLgAAAAxTZXRSYW5rVGllcnMAAAAR",
        "AAAAAQAAAGJSZWFkLW9ubHkgdmlldyBvZiBhIHBsYXllcidzIHZlc3Rpbmcgc3RhdHVzIGZvciBhIHNwZWNpZmljIGh1bnQuClJldHVybmVkIGJ5IGBnZXRfdmVzdGluZ19zdGF0dXNgLgAAAAAAAAAAAA1WZXN0aW5nU3RhdHVzAAAAAAAABwAAAENYTE0gYXZhaWxhYmxlIHRvIGNsYWltIHJpZ2h0IG5vdzogYHZlc3RlZF9hbW91bnQgLSBjbGFpbWVkX2Ftb3VudGAuAAAAABBjbGFpbWFibGVfYW1vdW50AAAACwAAAB9DdW11bGF0aXZlIFhMTSBhbHJlYWR5IGNsYWltZWQuAAAAAA5jbGFpbWVkX2Ftb3VudAAAAAAACwAAACtUcnVlIG9uY2UgYGNsYWltZWRfYW1vdW50ID49IHRvdGFsX2Ftb3VudGAuAAAAAAxmdWxseV92ZXN0ZWQAAAABAAAAJExlZGdlciB0aW1lc3RhbXAgd2hlbiB2ZXN0aW5nIGJlZ2FuLgAAAApzdGFydF90aW1lAAAAAAAGAAAAJVRvdGFsIFhMTSBsb2NrZWQgdW5kZXIgdGhpcyBzY2hlZHVsZS4AAAAAAAAMdG90YWxfYW1vdW50AAAACwAAAFNYTE0gdGhhdCBoYXMgdmVzdGVkIHNvIGZhcjogYHRvdGFsX2Ftb3VudCAqIG1pbihlbGFwc2VkIC8gdmVzdGluZ19wZXJpb2Rfc2VjcywgMSlgLgAAAAANdmVzdGVkX2Ftb3VudAAAAAAAAAsAAAAhRnVsbCB2ZXN0aW5nIGR1cmF0aW9uIGluIHNlY29uZHMuAAAAAAAAE3Zlc3RpbmdfcGVyaW9kX3NlY3MAAAAABg==",
        "AAAAAQAAAHtQZW5kaW5nIE5GVCBtaW50IHRoYXQgZmFpbGVkIGFuZCBjYW4gYmUgcmV0cmllZCBieSB0aGUgcGxheWVyIG9yIGFueW9uZQpwYXlpbmcgdGhlIHRyYW5zYWN0aW9uIGZlZSBvbiBiZWhhbGYgb2YgdGhlIHBsYXllci4AAAAAAAAAAA5QZW5kaW5nTmZ0TWludAAAAAAACgAAAIRGcm96ZW4gY29tcGxldGlvbiByYW5rIGZyb20gaHVudHktY29yZTsgcHJlc2VydmVkIHNvIGEgcmV0cnkgZW1pdHMgdGhlCnNhbWUgcmFuayB0aGF0IHdvdWxkIGhhdmUgYmVlbiByZWNvcmRlZCBvbiB0aGUgZmlyc3QgYXR0ZW1wdC4AAAAPY29tcGxldGlvbl9yYW5rAAAAAAQAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAMbmZ0X2NvbnRyYWN0AAAAEwAAAAAAAAAPbmZ0X2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAADm5mdF9odW50X3RpdGxlAAAAAAAQAAAAAAAAAA1uZnRfaW1hZ2VfdXJpAAAAAAAAEAAAAAAAAAAKbmZ0X3Jhcml0eQAAAAAABAAAAAAAAAAIbmZ0X3RpZXIAAAAEAAAAAAAAAAluZnRfdGl0bGUAAAAAAAAQAAAAAAAAAAZwbGF5ZXIAAAAAABM=",
        "AAAAAQAAACVBIHNpbmdsZSBlbnRyeSBpbiB0aGUgcG9vbCBhdWRpdCBsb2cuAAAAAAAAAAAAAA5Qb29sQXVkaXRFbnRyeQAAAAAABAAAABxXaG8gdHJpZ2dlcmVkIHRoZSBvcGVyYXRpb24uAAAABWFjdG9yAAAAAAAAEwAAACdUaGUgWExNIGFtb3VudCBpbnZvbHZlZCwgaWYgYXBwbGljYWJsZS4AAAAABmFtb3VudAAAAAAD6AAAAAsAAAAUT3BlcmF0aW9uIHBlcmZvcm1lZC4AAAAJb3BlcmF0aW9uAAAAAAAH0AAAAA1Qb29sT3BlcmF0aW9uAAAAAAAAGFRpbWVzdGFtcCAobGVkZ2VyIHRpbWUpLgAAAAl0aW1lc3RhbXAAAAAAAAAG",
        "AAAAAwAAAEJIb3cgWExNIHJld2FyZHMgYXJlIGNhbGN1bGF0ZWQgZnJvbSB0aGUgcG9vbCBhdCBkaXN0cmlidXRpb24gdGltZS4AAAAAAAAAAAAQRGlzdHJpYnV0aW9uTW9kZQAAAAIAAABARml4ZWQgYW1vdW50IHN1cHBsaWVkIGJ5IHRoZSBjYWxsZXIgKGBSZXdhcmRDb25maWcueGxtX2Ftb3VudGApLgAAAAVGaXhlZAAAAAAAAAAAAABCU2hhcmUgb2YgdGhlIHBvb2w6IGAocGxheWVyX3Njb3JlIC8gdG90YWxfc2NvcmVzKSAqIHBvb2xfYmFsYW5jZWAuAAAAAAAMUHJvcG9ydGlvbmFsAAAAAQ==",
        "AAAAAQAAADdSZWNvcmQgb2YgYSBjb21wbGV0ZWQgZGlzdHJpYnV0aW9uIGZvciBhIHNwZWNpZmljIHBvb2wuAAAAAAAAAAAQUG9vbERpc3RyaWJ1dGlvbgAAAAQAAAAAAAAABm5mdF9pZAAAAAAD6AAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABgAAAAAAAAAKeGxtX2Ftb3VudAAAAAAACw==",
        "AAAAAgAAAQdSZXNvbHV0aW9uIG91dGNvbWUgZm9yIGEgbWFudWFsbHkgcmVzb2x2ZWQgZmFpbGVkIGRpc3RyaWJ1dGlvbi4KClRoaXMgZW51bSB0cmFja3MgdGhlIGZpbmFsIHN0YXR1cyBvZiBkaXN0cmlidXRpb25zIHRoYXQgZmFpbGVkIGR1cmluZwp0aGVpciBpbml0aWFsIGV4ZWN1dGlvbiBhbmQgd2VyZSBsYXRlciByZXNvbHZlZCBieSBhbiBhZG1pbmlzdHJhdG9yLgoKUmVsYXRlZCB0byBpc3N1ZSAjMzY0OiBzdHVjay1kaXN0cmlidXRpb24gcmVzb2x1dGlvbiBmbG93LgAAAAAAAAAAEFJlc29sdXRpb25TdGF0dXMAAAACAAAAAAAAAEZUaGUgZGlzdHJpYnV0aW9uIHdhcyBzdWNjZXNzZnVsbHkgY29tcGxldGVkIGFmdGVyIG1hbnVhbCBpbnRlcnZlbnRpb24uAAAAAAAJQ29tcGxldGVkAAAAAAAAAAAAADhUaGUgZGlzdHJpYnV0aW9uIHdhcyByZWZ1bmRlZCB0byB0aGUgcG9vbCBhZnRlciBmYWlsaW5nLgAAAAhSZWZ1bmRlZA==",
        "AAAAAQAAAg1Db25maWd1cmF0aW9uIGZvciBhIHJld2FyZCBwb29sLCBzZXQgYXQgY3JlYXRpb24gdGltZS4KCmB0aW1lX2Jhc2VkX3RpZXJzYCBpcyBhbiBvcHRpb25hbCBsaXN0IG9mIChtYXhfZWxhcHNlZF9zZWNvbmRzLCB4bG1fYW1vdW50KQpwYWlycyB0aGF0IGRlZmluZSBhIGNvbmRpdGlvbmFsIHJld2FyZCBzY2hlZHVsZSBiYXNlZCBvbiBob3cgcXVpY2tseSBhCnBsYXllciBjb21wbGV0ZXMgYSBodW50LiBgcmFua19iYXNlZF90aWVyc2AgaXMgYW4gb3B0aW9uYWwgbGlzdCBvZiBleGFjdApvbmUtYmFzZWQgY29tcGxldGlvbiByYW5rcyBhbmQgdGhlaXIgYW1vdW50czsgYSBtYXRjaGluZyByYW5rIHRha2VzCnByZWNlZGVuY2Ugb3ZlciB0aW1lIGFuZCBmbGF0IHJld2FyZHMuIFdoZW4gYm90aCBsaXN0cyBhcmUgZW1wdHkgdGhlIHBvb2wKYmVoYXZlcyBleGFjdGx5IGFzIGJlZm9yZS4gVGllciBsaXN0cyBjYW4gYmUgdXBkYXRlZCBhZnRlciBwb29sIGNyZWF0aW9uCmFuZCBxdWVyaWVkIHZpYSBgZ2V0X3Bvb2xfY29uZmlnYC4AAAAAAAAAAAAAEFJld2FyZFBvb2xDb25maWcAAAAQAAAAR1VuaXggdGltZXN0YW1wIGFmdGVyIHdoaWNoIGNsYWltcyBhcmUgbm8gbG9uZ2VyIGFsbG93ZWQgKDAgPSBkaXNhYmxlZCkuAAAAAA5jbGFpbV9kZWFkbGluZQAAAAAABgAAARpBZGRyZXNzIG9mIHRoZSBodW50IGNyZWF0b3Igd2hvIG93bnMgdGhpcyBwb29sLgpBbnlvbmUgbWF5IGZ1bmQgdGhlIHBvb2wgKHNlZSBgZnVuZF9yZXdhcmRfcG9vbGApOyB0aGUgY3JlYXRvciBpcyB0aGUKb25seSBhZGRyZXNzIGF1dGhvcml6ZWQgdG8gbWFuYWdlIGl0cyBjb25maWd1cmF0aW9uIGFuZCB0byB0cmlnZ2VyCmByZWZ1bmRfcG9vbGAsIHdoaWNoIHBheXMgb3V0IHRoZSByZW1haW5pbmcgYmFsYW5jZSBwcm8gcmF0YSBhY3Jvc3MKZXZlcnkgYWRkcmVzcyB0aGF0IGZ1bmRlZCBpdC4AAAAAAAdjcmVhdG9yAAAAABMAAABdQWRkcmVzc2VzIGFsbG93ZWQgdG8gZGlzdHJpYnV0ZSByZXdhcmRzIGZvciB0aGlzIHBvb2wuCk9ubHkgdGhlIGNyZWF0b3IgY2FuIG1hbmFnZSB0aGlzIGxpc3QuAAAAAAAACWRlbGVnYXRlcwAAAAAAA+oAAAATAAAAKkRpc3RyaWJ1dGlvbiBtb2RlIChGaXhlZCBvciBQcm9wb3J0aW9uYWwpLgAAAAAAEWRpc3RyaWJ1dGlvbl9tb2RlAAAAAAAH0AAAABBEaXN0cmlidXRpb25Nb2RlAAAAtVdoZXRoZXIgZGlzdHJpYnV0aW9ucyBmcm9tIHRoaXMgcG9vbCBhcmUgdGVtcG9yYXJpbHkgZnJvemVuLgpXaGVuIGB0cnVlYCwgYGRpc3RyaWJ1dGVfcmV3YXJkc2AgYW5kIG90aGVyIGRpc3RyaWJ1dGlvbiBmdW5jdGlvbnMKd2lsbCByZWplY3QgY2FsbHMgd2l0aCBgUmV3YXJkRXJyb3JDb2RlOjpQb29sRnJvemVuYC4AAAAAAAAGZnJvemVuAAAAAAABAAAA8UFkZHJlc3MgdGhhdCBtb3N0IHJlY2VudGx5IGZyb3plIHRoaXMgcG9vbCwgb3IgYE5vbmVgIHdoZW4gdGhlIHBvb2wgaXMKbm90IGZyb3plbi4gVHJhY2tzIHdoZXRoZXIgdGhlIGN1cnJlbnQgZnJlZXplIHdhcyBpc3N1ZWQgYnkgdGhlIHBvb2wKY3JlYXRvciBvciB0aGUgY29udHJhY3QgYWRtaW4gc28gdGhhdCBhbiBhZG1pbi1pc3N1ZWQgZnJlZXplIGNhbiBvbmx5IGJlCmxpZnRlZCBieSB0aGUgYWRtaW4gKCMxMDc3KS4AAAAAAAAJZnJvemVuX2J5AAAAAAAD6AAAABMAAABBTWluaW11bSBYTE0gYW1vdW50IHBlciBkaXN0cmlidXRpb24uIDAgbWVhbnMgbm8gbWluaW11bSBlbmZvcmNlZC4AAAAAAAAXbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnQAAAAACwAAADVNaW5pbXVtIHNlY29uZHMgYmV0d2VlbiBkaXN0cmlidXRpb25zICgwID0gZGlzYWJsZWQpLgAAAAAAAB5taW5fZGlzdHJpYnV0aW9uX2ludGVydmFsX3NlY3MAAAAAAAYAAABBT3B0aW9uYWwgTkZUIGNvbnRyYWN0IGFkZHJlc3MgZm9yIE5GVC1vbmx5IG9yIG1peGVkIHJld2FyZCBwb29scy4AAAAAAAAMbmZ0X2NvbnRyYWN0AAAD6AAAABMAAAB8Q3JlYXRvciByb3lhbHR5IGJhc2lzIHBvaW50cyAoMC0xMDAwMCkgZm9yIE5GVCBzZWNvbmRhcnkgbWFya2V0IHNhbGVzLgpPbmx5IGFwcGxpZWQgd2hlbiBtaW50aW5nIHJld2FyZCBORlRzIGZyb20gdGhpcyBwb29sLgAAAA9uZnRfcm95YWx0eV9icHMAAAAABAAAAHJXaGV0aGVyIHJld2FyZCBORlRzIG1pbnRlZCBmcm9tIHRoaXMgcG9vbCBhcmUgdHJhbnNmZXJhYmxlLgpJZiBmYWxzZSwgTkZUcyBhcmUgc291bGJvdW5kIHRvIHRoZSBpbml0aWFsIHJlY2lwaWVudC4AAAAAABBuZnRfdHJhbnNmZXJhYmxlAAAAAQAAAHZPcHRpb25hbCBleGFjdC1yYW5rIHJld2FyZCB0aWVycy4gQSBtYXRjaGluZyBmcm96ZW4gY29tcGxldGlvbiByYW5rCnRha2VzIHByZWNlZGVuY2Ugb3ZlciBmbGF0IGFuZCB0aW1lLWJhc2VkIGFtb3VudHMuAAAAAAAQcmFua19iYXNlZF90aWVycwAAA+oAAAfQAAAADlJhbmtSZXdhcmRUaWVyAAAAAAA7VGFyZ2V0IGZ1bmRpbmcgYW1vdW50IGZvciBwcm9ncmVzcyB0cmFja2luZyAoMCA9IGRpc2FibGVkKS4AAAAADXRhcmdldF9hbW91bnQAAAAAAAALAAABEk9wdGlvbmFsIHRpbWUtYmFzZWQgcmV3YXJkIHRpZXJzLiBXaGVuIGVtcHR5LCB0aGUgcGVyLXdpbm5lciBhbW91bnQKaXMgY29tcHV0ZWQgZnJvbSBgeGxtX3Bvb2wgLyBtYXhfd2lubmVyc2AgYXMgYmVmb3JlLiBXaGVuIHBvcHVsYXRlZCwKdGhlIGFwcHJvcHJpYXRlIHRpZXIncyBgeGxtX2Ftb3VudGAgaXMgc2VsZWN0ZWQgYXQgZGlzdHJpYnV0aW9uIHRpbWUKYmFzZWQgb24gdGhlIHBsYXllcidzIChjb21wbGV0aW9uX3RpbWUgLSByZWdpc3RyYXRpb25fdGltZSkgZWxhcHNlZC4AAAAAABB0aW1lX2Jhc2VkX3RpZXJzAAAD6gAAB9AAAAATVGltZUJhc2VkUmV3YXJkVGllcgAAAABJVG9rZW4gYWRkcmVzcyBmb3IgdGhlIHJld2FyZCBwb29sIChlLmcuLCBYTE0sIFVTREMsIG9yIG90aGVyIFNBQyB0b2tlbnMpLgAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAR1PcHRpb25hbCB2ZXN0aW5nIHBlcmlvZCBpbiBzZWNvbmRzLiBXaGVuID4gMCwgWExNIHJld2FyZHMgYXJlIG5vdAp0cmFuc2ZlcnJlZCBpbW1lZGlhdGVseSBhdCBkaXN0cmlidXRpb24gdGltZS4gSW5zdGVhZCwgYSBgVmVzdGluZ1JlY29yZGAKaXMgY3JlYXRlZCBhbmQgdGhlIHBsYXllciBtdXN0IGNhbGwgYGNsYWltX3Zlc3RlZGAgdG8gcmVjZWl2ZSB0b2tlbnMKcHJvcG9ydGlvbmFsbHkgYXMgdGltZSBlbGFwc2VzLiAwIG1lYW5zIHZlc3RpbmcgaXMgZGlzYWJsZWQgKGluc3RhbnQgcGF5b3V0KS4AAAAAAAATdmVzdGluZ19wZXJpb2Rfc2VjcwAAAAAG",
        "AAAAAQAAADxGdWxsIHN0YXR1cyBvZiBhIHJld2FyZCBwb29sLCByZXR1cm5lZCBieSBnZXRfcmV3YXJkX3Bvb2woKS4AAAAAAAAAEFJld2FyZFBvb2xTdGF0dXMAAAAIAAAALEN1cnJlbnQgYXZhaWxhYmxlIGJhbGFuY2UgZm9yIGRpc3RyaWJ1dGlvbnMuAAAAB2JhbGFuY2UAAAAACwAAACZQb29sIGNyZWF0b3IgLyBvbmx5IGF1dGhvcml6ZWQgZnVuZGVyLgAAAAAAB2NyZWF0b3IAAAAAEwAAADxXaGV0aGVyIGRpc3RyaWJ1dGlvbnMgZnJvbSB0aGlzIHBvb2wgYXJlIHRlbXBvcmFyaWx5IGZyb3plbi4AAAAGZnJvemVuAAAAAAABAAAAcUFkZHJlc3MgdGhhdCBtb3N0IHJlY2VudGx5IGZyb3plIHRoaXMgcG9vbCwgb3IgYE5vbmVgIHdoZW4gbm90IGZyb3plbi4KU2VlIGBSZXdhcmRQb29sQ29uZmlnOjpmcm96ZW5fYnlgICgjMTA3NykuAAAAAAAACWZyb3plbl9ieQAAAAAAA+gAAAATAAAALk1pbmltdW0gWExNIHBlciBkaXN0cmlidXRpb24gKDAgPSBubyBtaW5pbXVtKS4AAAAAABdtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudAAAAAALAAAAQEN1bXVsYXRpdmUgdG90YWwgZGVwb3NpdGVkIGludG8gdGhpcyBwb29sIGFjcm9zcyBhbGwgZnVuZCBjYWxscy4AAAAPdG90YWxfZGVwb3NpdGVkAAAAAAsAAAAsQ3VtdWxhdGl2ZSB0b3RhbCBkaXN0cmlidXRlZCBmcm9tIHRoaXMgcG9vbC4AAAARdG90YWxfZGlzdHJpYnV0ZWQAAAAAAAALAAAA20N1bXVsYXRpdmUgdG90YWwgbWlncmF0ZWQgb3V0IG9mIHRoaXMgcG9vbCB2aWEgYG1pZ3JhdGVfcG9vbGAuClRvZ2V0aGVyIHdpdGggYHRvdGFsX2Rpc3RyaWJ1dGVkYCBhbmQgYHRvdGFsX3JlZnVuZGVkYCB0aGVzZSBzYXRpc2Z5OgpgdG90YWxfZGVwb3NpdGVkID09IGJhbGFuY2UgKyB0b3RhbF9kaXN0cmlidXRlZCArIHRvdGFsX3JlZnVuZGVkICsgdG90YWxfbWlncmF0ZWRfb3V0YAAAAAASdG90YWxfbWlncmF0ZWRfb3V0AAAAAAAL",
        "AAAAAQAAAD9SZXN1bHQgb2YgYSBwb29sIHZhbGlkYXRpb24gY2hlY2ssIHJldHVybmVkIGJ5IHZhbGlkYXRlX3Bvb2woKS4AAAAAAAAAABBWYWxpZGF0aW9uUmVzdWx0AAAAAwAAACZDdXJyZW50IHBvb2wgYmFsYW5jZSBhdCB0aW1lIG9mIGNoZWNrLgAAAAAAB2JhbGFuY2UAAAAACwAAAIFXaGV0aGVyIHRoZSBwb29sIGhhcyBzdWZmaWNpZW50IGZ1bmRzIGZvciB0aGUgcmVxdWlyZWQgYW1vdW50CmFuZCB0aGUgcmVxdWlyZWQgYW1vdW50IG1lZXRzIHRoZSBwb29sJ3MgbWluaW11bSBkaXN0cmlidXRpb24gc2l6ZS4AAAAAAAAIaXNfdmFsaWQAAAABAAAAKVJlcXVpcmVkIGFtb3VudCB0aGF0IHdhcyBjaGVja2VkIGFnYWluc3QuAAAAAAAACHJlcXVpcmVkAAAACw==",
        "AAAAAQAAADxPbi1jaGFpbiByZWNlaXB0IC8gcHJvb2Ygb2YgYSBjb21wbGV0ZWQgcmV3YXJkIGRpc3RyaWJ1dGlvbi4AAAAAAAAAEURpc3RyaWJ1dGlvblByb29mAAAAAAAABQAAACFYTE0gYW1vdW50IGRpc3RyaWJ1dGVkIChzdHJvb3BzKS4AAAAAAAAGYW1vdW50AAAAAAALAAAAMlNIQS0yNTYgb3ZlciAocG9vbF9pZCwgcGxheWVyLCBhbW91bnQsIHRpbWVzdGFtcCkuAAAAAAAEaGFzaAAAA+4AAAAgAAAAHlJlY2lwaWVudCBvZiB0aGUgZGlzdHJpYnV0aW9uLgAAAAAABnBsYXllcgAAAAAAEwAAABdQb29sIC8gaHVudCBpZGVudGlmaWVyLgAAAAAHcG9vbF9pZAAAAAAGAAAANExlZGdlciB0aW1lc3RhbXAgd2hlbiB0aGUgZGlzdHJpYnV0aW9uIHdhcyByZWNvcmRlZC4AAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAQAAAD9TdGF0dXMgb2YgYSByZXdhcmQgZGlzdHJpYnV0aW9uIGZvciBhIHNwZWNpZmljIGh1bnQgYW5kIHBsYXllci4AAAAAAAAAABJEaXN0cmlidXRpb25TdGF0dXMAAAAAAAQAAAAoV2hldGhlciBhbnkgcmV3YXJkIGhhcyBiZWVuIGRpc3RyaWJ1dGVkLgAAAAtkaXN0cmlidXRlZAAAAAABAAAAHE5GVCBJRCBpZiBhbiBORlQgd2FzIG1pbnRlZC4AAAAGbmZ0X2lkAAAAAAPoAAAABgAAAEFXaGV0aGVyIE5GVCBtaW50aW5nIGZhaWxlZCBkdXJpbmcgZGlzdHJpYnV0aW9uIChyZXRyeSBhdmFpbGFibGUpLgAAAAAAAA9uZnRfbWludF9mYWlsZWQAAAAAAQAAACNYTE0gYW1vdW50IGRpc3RyaWJ1dGVkICgwIGlmIG5vbmUpLgAAAAAKeGxtX2Ftb3VudAAAAAAACw==",
        "AAAAAQAAAE5Db21wcmVoZW5zaXZlIHN0YXRpc3RpY3MgZm9yIGEgcmV3YXJkIHBvb2wsIHJldHVybmVkIGJ5IGdldF9wb29sX3N0YXRpc3RpY3MoKS4AAAAAAAAAAAAUUmV3YXJkUG9vbFN0YXRpc3RpY3MAAAAFAAAAMEF2ZXJhZ2UgWExNIGFtb3VudCBwZXIgZGlzdHJpYnV0aW9uICgwIGlmIG5vbmUpLgAAABBhdmdfZGlzdHJpYnV0aW9uAAAACwAAADdOdW1iZXIgb2Ygc3VjY2Vzc2Z1bCBkaXN0cmlidXRpb25zIG1hZGUgZnJvbSB0aGlzIHBvb2wuAAAAABJkaXN0cmlidXRpb25fY291bnQAAAAAAAYAAAA9TGVkZ2VyIHRpbWVzdGFtcCBvZiB0aGUgbW9zdCByZWNlbnQgZGlzdHJpYnV0aW9uICgwIGlmIG5vbmUpLgAAAAAAABtsYXN0X2Rpc3RyaWJ1dGlvbl90aW1lc3RhbXAAAAAABgAAACRUb3RhbCBYTE0gZGlzdHJpYnV0ZWQgZnJvbSB0aGUgcG9vbC4AAAARdG90YWxfZGlzdHJpYnV0ZWQAAAAAAAALAAAAK1RvdGFsIFhMTSBmdW5kZWQgKGRlcG9zaXRlZCkgaW50byB0aGUgcG9vbC4AAAAADHRvdGFsX2Z1bmRlZAAAAAs=",
        "AAAAAQAAAGRTdGF0aXN0aWNhbCBzdW1tYXJ5IG9mIGRpc3RyaWJ1dGlvbnMgYWNyb3NzIGEgcmV3YXJkIHBvb2wsCnJldHVybmVkIGJ5IGdldF9kaXN0cmlidXRpb25fYW5hbHl0aWNzKCkuAAAAAAAAABVEaXN0cmlidXRpb25BbmFseXRpY3MAAAAAAAAGAAAARkF2ZXJhZ2UgKG1lYW4pIFhMTSBhbW91bnQgcGVyIGRpc3RyaWJ1dGlvbiAoc3Ryb29wcykuIDAgaWYgY291bnQgaXMgMC4AAAAAAAdhdmVyYWdlAAAAAAsAAAA5TnVtYmVyIG9mIGRpc3RyaWJ1dGlvbnMgaW5jbHVkZWQgaW4gdGhlIGFuYWx5dGljcyB3aW5kb3cuAAAAAAAABWNvdW50AAAAAAAABgAAAEdNYXhpbXVtIFhMTSBhbW91bnQgaW4gYSBzaW5nbGUgZGlzdHJpYnV0aW9uIChzdHJvb3BzKS4gMCBpZiBjb3VudCBpcyAwLgAAAAADbWF4AAAAAAsAAABCTWVkaWFuIFhMTSBhbW91bnQgYWNyb3NzIGRpc3RyaWJ1dGlvbnMgKHN0cm9vcHMpLiAwIGlmIGNvdW50IGlzIDAuAAAAAAAGbWVkaWFuAAAAAAALAAAAR01pbmltdW0gWExNIGFtb3VudCBpbiBhIHNpbmdsZSBkaXN0cmlidXRpb24gKHN0cm9vcHMpLiAwIGlmIGNvdW50IGlzIDAuAAAAAANtaW4AAAAACwAAADhUb3RhbCBYTE0gZGlzdHJpYnV0ZWQgaW4gdGhlIGFuYWx5dGljcyB3aW5kb3cgKHN0cm9vcHMpLgAAAAV0b3RhbAAAAAAAAAs=",
        "AAAAAQAAACNFbnRyeSBmb3IgYmF0Y2ggZGlzdHJpYnV0aW9uIGNhbGxzLgAAAAAAAAAAFkJhdGNoRGlzdHJpYnV0aW9uRW50cnkAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAOcGxheWVyX2FkZHJlc3MAAAAAABMAAAAAAAAADXJld2FyZF9jb25maWcAAAAAAAfQAAAADFJld2FyZENvbmZpZw==",
        "AAAAAAAAAIlQYXVzZXMgdGhlIGNvbnRyYWN0LCBwcmV2ZW50aW5nIHJld2FyZCBkaXN0cmlidXRpb25zIGFuZCB3aXRoZHJhd2Fscy4KT25seSB0aGUgY29udHJhY3QgYWRtaW4gY2FuIGNhbGwgdGhpcy4gRW1pdHMgYSBDb250cmFjdFBhdXNlZEV2ZW50LgAAAAAAAAVwYXVzZQAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAGcmVhc29uAAAAAAAQAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAFlVbnBhdXNlcyB0aGUgY29udHJhY3QsIHJlc3VtaW5nIG5vcm1hbCBvcGVyYXRpb25zLgpPbmx5IHRoZSBjb250cmFjdCBhZG1pbiBjYW4gY2FsbCB0aGlzLgAAAAAAAAd1bnBhdXNlAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAADFSZXR1cm5zIHdoZXRoZXIgdGhlIGNvbnRyYWN0IGlzIGN1cnJlbnRseSBwYXVzZWQuAAAAAAAACWlzX3BhdXNlZAAAAAAAAAAAAAABAAAAAQ==",
        "AAAAAAAAALNJbml0aWFsaXplcyB0aGUgUmV3YXJkTWFuYWdlciB3aXRoIHRoZSBYTE0gdG9rZW4gY29udHJhY3QgYWRkcmVzcyAoU0FDKS4KTXVzdCBiZSBjYWxsZWQgb25jZSBiZWZvcmUgYW55IHJld2FyZCBkaXN0cmlidXRpb24uCkBkZXByZWNhdGVkIFVzZSBjb25zdHJ1Y3RvciBkdXJpbmcgZGVwbG95bWVudCBpbnN0ZWFkLgAAAAAKaW5pdGlhbGl6ZQAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAl4bG1fdG9rZW4AAAAAAAATAAAAAAAAAApodW50eV9jb3JlAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAjhGcmVlemVzIGEgcmV3YXJkIHBvb2wsIHByZXZlbnRpbmcgYW55IGZ1cnRoZXIgZGlzdHJpYnV0aW9ucy4KCkNhbiBiZSBjYWxsZWQgYnkgZWl0aGVyIHRoZSBwb29sIGNyZWF0b3Igb3IgdGhlIGNvbnRyYWN0IGFkbWluLgpSZWNvcmRzIHdobyBpc3N1ZWQgdGhlIGZyZWV6ZSBpbiBgUmV3YXJkUG9vbENvbmZpZzo6ZnJvemVuX2J5YDsgYW4KYWRtaW4taXNzdWVkIGZyZWV6ZSBjYW4gb25seSBiZSBsaWZ0ZWQgYnkgdGhlIGFkbWluIChzZWUKYHVuZnJlZXplX3Bvb2xgLCAjMTA3NykuCkVtaXRzIGEgYFBvb2xGcm96ZW5FdmVudGAuCgojIEFyZ3VtZW50cwoqIGBjYWxsZXJgIC0gVGhlIGFkZHJlc3MgY2FsbGluZyBmcmVlemUgKG11c3QgYmUgcG9vbCBjcmVhdG9yIG9yIGFkbWluKQoqIGBodW50X2lkYCAtIFRoZSBodW50IHdob3NlIHBvb2wgdG8gZnJlZXplCgojIEVycm9ycwoqIGBQb29sTm90Rm91bmRgIC0gTm8gcG9vbCBleGlzdHMgZm9yIHRoaXMgaHVudF9pZAoqIGBVbmF1dGhvcml6ZWRgIC0gQ2FsbGVyIGlzIG5laXRoZXIgdGhlIHBvb2wgY3JlYXRvciBub3IgdGhlIGNvbnRyYWN0IGFkbWluAAAAC2ZyZWV6ZV9wb29sAAAAAAIAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAABABSZWZ1bmRzIHRoZSByZW1haW5pbmcgcG9vbCBiYWxhbmNlIGZvciBhIGh1bnQsIHBhaWQgb3V0ICoqcHJvIHJhdGEqKgphY3Jvc3MgZXZlcnkgYWRkcmVzcyB0aGF0IGZ1bmRlZCBpdCAoc2VlIGBmdW5kX3Jld2FyZF9wb29sYCkgaW4KcHJvcG9ydGlvbiB0byBlYWNoIGZ1bmRlcidzIHNoYXJlIG9mIHRvdGFsIGNvbnRyaWJ1dGlvbnMg4oCUIG5ldmVyCnBheWluZyBvbmUgZnVuZGVyJ3MgY29udHJpYnV0aW9uIHRvIGFub3RoZXIgcGFydHkuIEEgcG9vbCBmdW5kZWQgYnkgYQpzaW5nbGUgYWRkcmVzcyAodGhlIGNvbW1vbiBjYXNlKSBzaW1wbHkgZ2V0cyBpdHMgd2hvbGUgYmFsYW5jZSBiYWNrLgoKQ2FuIG9ubHkgYmUgdHJpZ2dlcmVkIGJ5IHRoZSBwb29sIGNyZWF0b3IsIHdobyBtdXN0IGF1dGhvcml6ZSB0aGUKY2FsbDsgdGhlIHBheW91dCBkZXN0aW5hdGlvbnMgYXJlIHRoZSB0cmFja2VkIGZ1bmRlcnMsIG5vdCB0aGUgY2FsbGVyLgpVc2VzIHRoZSB0b2tlbiBhZGRyZXNzIHNwZWNpZmllZCB3aGVuIHRoZSBwb29sIHdhcyBjcmVhdGVkLiBUaGUgaHVudAptdXN0IGJlIGluIGEgdGVybWluYWwgc3RhdGUgKGNhbmNlbGxlZCBvciBlbmRlZCkgd2hlbiBIdW50eUNvcmUgaXMKY29uZmlndXJlZCDigJQgcmVmdW5kaW5nIGFuIGFjdGl2ZSBodW50J3MgcG9vbCBvdXQgZnJvbSB1bmRlciBpdHMKcGxheWVycyBpcyByZWplY3RlZC4KCioqSW1wb3J0YW50OioqIFRoaXMgaXMgYSBkZXN0cnVjdGl2ZSBvcGVyYXRpb24uIEVuc3VyZSBhbGwgZGlzdHJpYnV0aW9ucyBhcmUgY29tcGxldGUKYmVmb3JlIGNhbGxpbmcgdGhpcyBmdW5jdGlvbiwgYXMgYW55IHJlbWFpbmluZyB1bmNsYWltZWQgcmV3YXJkcyBjYW5ub3QgYmUgZGlzdHJpYnV0ZWQKYWZ0ZXIgdGhlIHBvb2wgaXMgcmVmdW5kZWQuCgojIEFjY291bnRpbmcKVGhpcyBmdW5jdGlvbiB1cGRhdGVzOgotIFBvb2wgYmFsYW5jZTogU2V0IHRvIDAKLSBUb3RhbCByZWZ1bmRlZDogSW5jcmVtZW50ZWQgYnkgdGhlIHJlZnVuZCBhbW91bnQKLSBBdWRpdCBsAAAAC3JlZnVuZF9wb29sAAAAAAIAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAACpTdGVwIHR3byBvZiBhIHR3by1zdGVwIGFkbWluIGtleSByb3RhdGlvbi4AAAAAAAxhY2NlcHRfYWRtaW4AAAABAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAGVBZGRzIGEgZGVsZWdhdGUgYWxsb3dlZCB0byBkaXN0cmlidXRlIHJld2FyZHMgZm9yIGEgcG9vbC4KT25seSB0aGUgcG9vbCBjcmVhdG9yIGNhbiBtYW5hZ2UgZGVsZWdhdGVzLgAAAAAAAAxhZGRfZGVsZWdhdGUAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAIZGVsZWdhdGUAAAATAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAA2ZDbGFpbXMgdGhlIHByb3BvcnRpb25hbGx5IHZlc3RlZCBYTE0gcmV3YXJkIGZvciB0aGUgY2FsbGVyLgoKVGhlIGNsYWltYWJsZSBhbW91bnQgaXM6IGB0b3RhbF9hbW91bnQgKiBtaW4oZWxhcHNlZCAvIHZlc3RpbmdfcGVyaW9kX3NlY3MsIDEpIC0gY2xhaW1lZF9hbW91bnRgLgoKVGhlIHBsYXllciBjYW4gY2FsbCB0aGlzIGFueSBudW1iZXIgb2YgdGltZXMgb3ZlciB0aGUgdmVzdGluZyBwZXJpb2QuCkVhY2ggY2FsbCB0cmFuc2ZlcnMgd2hhdGV2ZXIgaGFzIG5ld2x5IHZlc3RlZCBzaW5jZSB0aGUgbGFzdCBjbGFpbS4KT25jZSBgY2xhaW1lZF9hbW91bnQgPT0gdG90YWxfYW1vdW50YCB0aGUgc2NoZWR1bGUgaXMgZnVsbHkgZXhoYXVzdGVkLgoKIyBBcmd1bWVudHMKKiBgcGxheWVyYCAtIFRoZSBwbGF5ZXIgY2xhaW1pbmcgdGhlaXIgdmVzdGVkIHJld2FyZAoqIGBodW50X2lkYCAtIFRoZSBodW50IHdob3NlIHZlc3RpbmcgcmVjb3JkIHRvIGNsYWltIGZyb20KCiMgUmV0dXJucwpUaGUgWExNIGFtb3VudCAoaW4gc3Ryb29wcykgdHJhbnNmZXJyZWQgdG8gdGhlIHBsYXllci4KCiMgRXJyb3JzCiogYFZlc3RpbmdOb3RTdGFydGVkYCAtIE5vIHZlc3RpbmcgcmVjb3JkIGV4aXN0cyBmb3IgdGhpcyAoaHVudF9pZCwgcGxheWVyKQoqIGBWZXN0aW5nQWxyZWFkeUNsYWltZWRgIC0gRnVsbCB2ZXN0aW5nIGFtb3VudCBoYXMgYWxyZWFkeSBiZWVuIGNsYWltZWQKKiBgTm90aGluZ1RvVmVzdGAgLSBOb3RoaW5nIGhhcyB2ZXN0ZWQgeWV0IGF0IHRoZSBjdXJyZW50IHRpbWVzdGFtcAoqIGBJbnN1ZmZpY2llbnRQb29sYCAtIENvbnRyYWN0IHRva2VuIGJhbGFuY2UgaXMgdG9vIGxvdyAoc2hvdWxkIG5vdCBub3JtYWxseSBvY2N1cikAAAAAAAxjbGFpbV92ZXN0ZWQAAAACAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAACwAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAABABNaWdyYXRlcyB0aGUgdW51c2VkIGJhbGFuY2Ugb2YgYW4gZXhwaXJlZCBvciBjYW5jZWxsZWQgaHVudCdzIHBvb2wgaW50bwphbiBleGlzdGluZyBkZXN0aW5hdGlvbiBwb29sIG93bmVkIGJ5IHRoZSBzYW1lIGNyZWF0b3IuCgpUaGlzIGxldHMgYSBjcmVhdG9yIHJlY3ljbGUgZnVuZHMgbG9ja2VkIGluIGEgZmluaXNoZWQgaHVudCBpbnRvIGEgZnJlc2gKaHVudCB3aXRob3V0IHdpdGhkcmF3aW5nIGFuZCByZS1kZXBvc2l0aW5nLiBUaGUgWExNIG5ldmVyIGxlYXZlcyB0aGlzCmNvbnRyYWN0OyBvbmx5IHRoZSBpbnRlcm5hbCBwZXItaHVudCBiYWxhbmNlIGFjY291bnRpbmcgaXMgcmUta2V5ZWQuCgojIEVsaWdpYmlsaXR5IChhY2NlcHRhbmNlIGNyaXRlcmlhKQoqIFRoZSBzb3VyY2UgcG9vbCdzIGh1bnQgbXVzdCBiZSAqKmV4cGlyZWQgb3IgY2FuY2VsbGVkKiog4oCUIHZlcmlmaWVkIHZpYQphIGNyb3NzLWNvbnRyYWN0IGNhbGwgdG8gdGhlIGNvbmZpZ3VyZWQgSHVudHlDb3JlIGNvbnRyYWN0CihgaXNfaHVudF9leHBpcmVkX29yX2NhbmNlbGxlZGApLiBJZiBIdW50eUNvcmUgaXMgbm90IGNvbmZpZ3VyZWQsIHRoZQpzb3VyY2UgY2Fubm90IGJlIHNob3duIGVsaWdpYmxlIGFuZCBtaWdyYXRpb24gaXMgcmVqZWN0ZWQuCiogVGhlICoqZGVzdGluYXRpb24gcG9vbCBtdXN0IGFscmVhZHkgZXhpc3QqKiAoY3JlYXRlZCB2aWEKYGNyZWF0ZV9yZXdhcmRfcG9vbGApLgoqICoqQm90aCBwb29scyBtdXN0IGhhdmUgdGhlIHNhbWUgY3JlYXRvcioqLCB3aG8gbXVzdCBhdXRob3JpemUgdGhlIGNhbGwuCiogKipCb3RoIHBvb2xzIG11c3QgdXNlIHRoZSBzYW1lIHRva2VuLioqCgojIEFjY291bnRpbmcKQWZ0ZXIgYSBzdWNjZXNzZnVsIG1pZ3JhdGlvbiB0aGUgZm9sbG93aW5nIGlkZW50aXRpZXMgaG9sZDoKCioqU291cmNlIHBvb2w6KioKYHRvdGFsX2RlcG9zaXRlZCA9PSBiYWxhbmNlKDApICsgdG90YWxfZGlzdHJpYnV0ZWQgKyB0b3RhbF9yZWZ1bmRlZCArIHRvdGFsX21pZ3JhdGVkX291dGAKCioqAAAADG1pZ3JhdGVfcG9vbAAAAAMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAOc291cmNlX2h1bnRfaWQAAAAAAAYAAAAAAAAADGRlc3RfaHVudF9pZAAAAAYAAAABAAAD6QAAAAsAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAHJDb25zdHJ1Y3RvciAtIHJ1bnMgYXRvbWljYWxseSBkdXJpbmcgZGVwbG95bWVudC4KUHJldmVudHMgZnJvbnQtcnVubmluZyBieSBpbml0aWFsaXppbmcgZHVyaW5nIGRlcGxveSB0cmFuc2FjdGlvbi4AAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAl4bG1fdG9rZW4AAAAAAAATAAAAAAAAAApodW50eV9jb3JlAAAAAAATAAAAAA==",
        "AAAAAAAAAElCbG9ja3MgcG9vbCBmdW5kaW5nLiBEaXN0cmlidXRpb24gaXMgdW5hZmZlY3RlZCB1bmxlc3Mgc2VwYXJhdGVseSBwYXVzZWQuAAAAAAAADXBhdXNlX2Z1bmRpbmcAAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAAAAAAANcnVuX21pZ3JhdGlvbgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAAAAAAAB2RyeV9ydW4AAAAAAQAAAAEAAAPpAAAH0AAAAA9NaWdyYXRpb25SZXBvcnQAAAAH0AAAABBVcGdyYWRlQXV0aEVycm9y",
        "AAAAAAAAA4lVbmZyZWV6ZXMgYSByZXdhcmQgcG9vbCwgcmUtZW5hYmxpbmcgZGlzdHJpYnV0aW9ucy4KCkNhbiBiZSBjYWxsZWQgYnkgZWl0aGVyIHRoZSBwb29sIGNyZWF0b3Igb3IgdGhlIGNvbnRyYWN0IGFkbWluLCBleGNlcHQKdGhhdCBhIGZyZWV6ZSBpc3N1ZWQgYnkgdGhlIGFkbWluIG1heSBvbmx5IGJlIGxpZnRlZCBieSB0aGUgYWRtaW4KKCMxMDc3KS4gQW55IGZyZWV6ZXIgb3RoZXIgdGhhbiB0aGUgcG9vbCBjcmVhdG9yIHdhcyB0aGUgYWRtaW4gYXQgdGhlCnRpbWUgb2YgdGhlIGZyZWV6ZSwgc28gdGhpcyByZXN0cmljdGlvbiBhbHNvIHN1cnZpdmVzIGFuIGFkbWluIHJvdGF0aW9uLgpBIGZyb3plbiBwb29sIHdpdGggbm8gcmVjb3JkZWQgZnJlZXplciAoZnJlZXplIHN0YXRlIHdyaXR0ZW4gYmVmb3JlCmBmcm96ZW5fYnlgIGV4aXN0ZWQpIGlzIHRyZWF0ZWQgYXMgYW4gYWRtaW4gZnJlZXplIGFuZCBjYW4gb25seSBiZQpsaWZ0ZWQgYnkgdGhlIGFkbWluLiBDbGVhcnMgYFJld2FyZFBvb2xDb25maWc6OmZyb3plbl9ieWAuCkVtaXRzIGEgYFBvb2xVbmZyb3plbkV2ZW50YC4KCiMgQXJndW1lbnRzCiogYGNhbGxlcmAgLSBUaGUgYWRkcmVzcyBjYWxsaW5nIHVuZnJlZXplIChtdXN0IGJlIHBvb2wgY3JlYXRvciBvciBhZG1pbikKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB3aG9zZSBwb29sIHRvIHVuZnJlZXplCgojIEVycm9ycwoqIGBQb29sTm90Rm91bmRgIC0gTm8gcG9vbCBleGlzdHMgZm9yIHRoaXMgaHVudF9pZAoqIGBVbmF1dGhvcml6ZWRgIC0gQ2FsbGVyIGlzIG5laXRoZXIgdGhlIHBvb2wgY3JlYXRvciBub3IgdGhlIGNvbnRyYWN0CmFkbWluLCBvciB0aGUgY3VycmVudCBmcmVlemUgd2FzIGlzc3VlZCBieSB0aGUgYWRtaW4gYW5kIHRoZSBjYWxsZXIgaXMKbm90IHRoZSBhZG1pbgAAAAAAAA11bmZyZWV6ZV9wb29sAAAAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAi9WYWxpZGF0ZXMgd2hldGhlciBhIHBvb2wgY2FuIGNvdmVyIGEgZ2l2ZW4gZGlzdHJpYnV0aW9uIGFtb3VudC4KCkNoZWNrcyB0aGF0OgotIFRoZSBwb29sIGV4aXN0cyAod2FzIGNyZWF0ZWQgdmlhIGNyZWF0ZV9yZXdhcmRfcG9vbCkKLSBUaGUgcmVxdWlyZWRfYW1vdW50IGlzIHBvc2l0aXZlLCBleGNlcHQgdGhhdCBgcmVxdWlyZWRfYW1vdW50ID09IDBgIGlzCnZhbGlkIGZvciBwb29scyB3aXRoIGFuIE5GVCBjb250cmFjdCAoTkZULW9ubHkgcG9vbHMpLCB3aGljaCBob2xkIG5vCnRva2VuIGJhbGFuY2UgYnkgZGVzaWduICgjMTA4OCkKLSBUaGUgcG9vbCBiYWxhbmNlID49IHJlcXVpcmVkX2Ftb3VudAotIFRoZSByZXF1aXJlZF9hbW91bnQgbWVldHMgdGhlIHBvb2wncyBtaW5pbXVtIGRpc3RyaWJ1dGlvbiB0aHJlc2hvbGQgKGlmIHNldCkKClJldHVybnMgYSBgVmFsaWRhdGlvblJlc3VsdGAgd2l0aCBiYWxhbmNlIGRldGFpbHMgcmVnYXJkbGVzcyBvZiB2YWxpZGl0eSwKc28gY2FsbGVycyBjYW4gZGlhZ25vc2Ugc2hvcnRmYWxscyB3aXRob3V0IGEgc2VwYXJhdGUgcXVlcnkuAAAAAA12YWxpZGF0ZV9wb29sAAAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAA9yZXF1aXJlZF9hbW91bnQAAAAACwAAAAEAAAfQAAAAEFZhbGlkYXRpb25SZXN1bHQ=",
        "AAAAAQAAACVQYWdpbmF0ZWQgcmVzcG9uc2UgZm9yIHRoZSBhdWRpdCBsb2cuAAAAAAAAAAAAABRQb29sQXVkaXRMb2dSZXNwb25zZQAAAAIAAAAAAAAAB2VudHJpZXMAAAAD6gAAB9AAAAAOUG9vbEF1ZGl0RW50cnkAAAAAAAAAAAAFdG90YWwAAAAAAAAG",
        "AAAAAAAAAG1SZXR1cm5zIHdoZXRoZXIgYSByZXdhcmQgcG9vbCBpcyBjdXJyZW50bHkgZnJvemVuLgpSZXR1cm5zIGBmYWxzZWAgaWYgbm8gcG9vbCBleGlzdHMgZm9yIHRoZSBnaXZlbiBgaHVudF9pZGAuAAAAAAAADmlzX3Bvb2xfZnJvemVuAAAAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAAAQ==",
        "AAAAAAAAANZTZXRzIHRoZSBvcHRpb25hbCBIdW50eUNvcmUgY29udHJhY3QgYWRkcmVzcyB1c2VkIHRvIHZhbGlkYXRlIGh1bnRfaWQgZXhpc3RlbmNlCmluIGBjcmVhdGVfcmV3YXJkX3Bvb2xgLiBXaGVuIHNldCwgcG9vbCBjcmVhdGlvbiB3aWxsIGJlIHJlamVjdGVkIGZvciB1bmtub3duCmh1bnQgSURzLiBJZiBub3Qgc2V0LCBodW50X2lkIGlzIGFzc3VtZWQgY2FsbGVyLXRydXN0ZWQuAAAAAAAOc2V0X2h1bnR5X2NvcmUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAKaHVudHlfY29yZQAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAABABVcGRhdGVzIChvciBpbnN0YWxscykgdGhlIHRpbWUtYmFzZWQgcmV3YXJkIHRpZXIgc2NoZWR1bGUgb24gYW4gZXhpc3RpbmcKcmV3YXJkIHBvb2wsIGVuYWJsaW5nIGNvbmRpdGlvbmFsIHJld2FyZCBhbW91bnRzIGJhc2VkIG9uIHBsYXllciBjb21wbGV0aW9uCnRpbWUgKGFjY2VwdGFuY2UgY3JpdGVyaWE6ICJEZWZpbmUgdGltZS1iYXNlZCByZXdhcmQgdGllcnMgaW4gcG9vbCBjb25maWciKS4KClRpZXJzIG11c3QgYmUgc3VwcGxpZWQgaW4gc3RyaWN0bHkgYXNjZW5kaW5nIG9yZGVyIG9mIGBtYXhfY29tcGxldGlvbl9zZWNzYAooaS5lLiBmYXN0ZXIgdGllcnMgZmlyc3QpLCBhbmQgZXZlcnkgYHhsbV9hbW91bnRgIG11c3QgYmUgc3RyaWN0bHkgcG9zaXRpdmUuClBhc3NpbmcgYW4gZW1wdHkgYFZlY2AgZGlzYWJsZXMgdGllci1iYXNlZCByZXdhcmRzIHNvIHRoZSBwb29sIHJldmVydHMKdG8gdGhlIGZsYXQgYHhsbV9wb29sIC8gbWF4X3dpbm5lcnNgIGFtb3VudC4KCk9ubHkgdGhlIHBvb2wgY3JlYXRvciBpcyBhdXRob3JpemVkIHRvIGNhbGwgdGhpcy4gVGhlIG5ldyB0aWVycyBhcmUKcGVyc2lzdGVkIGltbWVkaWF0ZWx5IGFuZCBiZWNvbWUgZWZmZWN0aXZlIGZvciBhbnkgc3Vic2VxdWVudCBkaXN0cmlidXRpb24KY2FsbC4gQWxyZWFkeS1kaXN0cmlidXRlZCByZXdhcmRzIGFyZSBub3QgYWZmZWN0ZWQuCgojIEFyZ3VtZW50cwoqIGBjcmVhdG9yYCAtIFRoZSBwb29sIGNyZWF0b3IgKG11c3QgbWF0Y2ggdGhlIHN0b3JlZCBjcmVhdG9yKQoqIGBodW50X2lkYCAtIFRoZSBodW50IHdob3NlIHBvb2wgY29uZmlnIHRvIHVwZGF0ZQoqIGB0aW1lX2Jhc2VkX3RpZXJzYCAtIE5ldyB0aWVyIGxpc3QgKHN0cmljdGx5IGFzY2VuZGluZyBieSB0aW1lLCBhbGwgYW1vdW50cyA+IDA7CmFuIGVtcHR5IGxpc3QgZGlzYWJsZXMgdGllci1iYXNlZCByZXdhcmRzKQoKIyBFcnJvcnMKKiBgUG9vbE5vdEZvdW5kYCAtIE5vIHBvb2wgZXhpc3RzIGZvciB0aGlzIGh1bnRfaWQKKiBgVW5hdXRob3JpemVkYCAtAAAADnNldF9wb29sX3RpZXJzAAAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAQdGltZV9iYXNlZF90aWVycwAAA+oAAAfQAAAAE1RpbWVCYXNlZFJld2FyZFRpZXIAAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAM1FZmZlY3RpdmUgcGF1c2Ugc3RhdGUgYXMgYChnbG9iYWwsIGZ1bmRpbmcsIGRpc3RyaWJ1dGlvbilgLgoKVGhlIHR3byBncmFudWxhciB2YWx1ZXMgYXJlIHRoZSAqZWZmZWN0aXZlKiBvbmVzLCBzbyB0aGV5IHJlYWQgYHRydWVgCndoZW5ldmVyIHRoZSBnbG9iYWwgc3RvcCBpcyBlbmdhZ2VkLiBNaXJyb3JzIGBIdW50eUNvcmU6OmdldF9wYXVzZV9zdGF0ZWAuAAAAAAAAD2dldF9wYXVzZV9zdGF0ZQAAAAAAAAAAAQAAA+0AAAADAAAAAQAAAAEAAAAB",
        "AAAAAAAAAQtSZXR1cm5zIHRoZSBmdWxsIGNvbmZpZ3VyYXRpb24gb2YgYSByZXdhcmQgcG9vbCwgaW5jbHVkaW5nIGl0cyB0aW1lCmFuZCBleGFjdC1yYW5rIHRpZXIgbGlzdHMuIGBOb25lYCB3aGVuIG5vIHBvb2wgZXhpc3RzIGZvciB0aGUgaHVudC4KClRoaXMgaXMgdGhlIHJlYWQgcGF0aCB1c2VkIGJ5IEh1bnR5Q29yZSBhdCBjb21wbGV0aW9uIHRpbWUgdG8gcmVzb2x2ZQpyYW5rLSBhbmQgdGltZS1iYXNlZCBhbW91bnRzIHdpdGhvdXQgZHVwbGljYXRpbmcgcG9vbCBzdGF0ZS4AAAAAD2dldF9wb29sX2NvbmZpZwAAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAD6AAAB9AAAAAQUmV3YXJkUG9vbENvbmZpZw==",
        "AAAAAAAAAJdSZXR1cm5zIHRoZSBmdWxsIHN0YXR1cyBvZiBhIHJld2FyZCBwb29sLCBpbmNsdWRpbmcgYmFsYW5jZSwgdG90YWxzLCBhbmQgY29uZmlndXJhdGlvbi4KUmV0dXJucyBOb25lIGlmIG5vIHBvb2wgaGFzIGJlZW4gY3JlYXRlZCBmb3IgdGhlIGdpdmVuIGh1bnRfaWQuAAAAAA9nZXRfcmV3YXJkX3Bvb2wAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAA+gAAAfQAAAAEFJld2FyZFBvb2xTdGF0dXM=",
        "AAAAAAAAAAAAAAAPcHJvcG9zZV91cGdyYWRlAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAABAAAD6QAAB9AAAAAPVXBncmFkZVByb3Bvc2FsAAAAB9AAAAAQVXBncmFkZUF1dGhFcnJvcg==",
        "AAAAAAAAAEtSZW1vdmVzIGEgZGVsZWdhdGUgZnJvbSBhIHBvb2wuCk9ubHkgdGhlIHBvb2wgY3JlYXRvciBjYW4gbWFuYWdlIGRlbGVnYXRlcy4AAAAAD3JlbW92ZV9kZWxlZ2F0ZQAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAIZGVsZWdhdGUAAAATAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAEZSZXN1bWVzIHBvb2wgZnVuZGluZy4gSGFzIG5vIGVmZmVjdCB3aGlsZSB0aGUgZ2xvYmFsIHBhdXNlIGlzIGVuZ2FnZWQuAAAAAAAPdW5wYXVzZV9mdW5kaW5nAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAAFBSZXR1cm5zIHRoZSBvbi1jaGFpbiB2ZXJzaW9uIHN0b3JlZCBkdXJpbmcgaW5pdGlhbGl6ZSwgb3IgdGhlIGNvbXBpbGVkIGNvbnN0YW50LgAAABBjb250cmFjdF92ZXJzaW9uAAAAAAAAAAEAAAAE",
        "AAAAAAAABABEaXN0cmlidXRlcyByZXdhcmRzIHRvIG11bHRpcGxlIHBsYXllcnMgaW4gYSBzaW5nbGUgYXRvbWljIHRyYW5zYWN0aW9uLgoKRXZlcnkgZW50cnkgaW4gdGhlIGJhdGNoIGlzIHZhbGlkYXRlZCBmaXJzdCAobm8gc3RhdGUgY2hhbmdlcykuIElmIGFsbAplbnRyaWVzIHBhc3MgdmFsaWRhdGlvbiwgYWxsIHRyYW5zZmVycyBhcmUgZXhlY3V0ZWQuIElmIGFueSBzaW5nbGUgZW50cnkKZmFpbHMgdmFsaWRhdGlvbiwgdGhlIGVudGlyZSBiYXRjaCBpcyByZWplY3RlZCB3aXRoIG5vIHN0YXRlIGNoYW5nZXMuCgojIEF0b21pY2l0eSBndWFyYW50ZWUKClRoZSB0d28tcGhhc2UgZGVzaWduICh2YWxpZGF0ZS1hbGwsIGV4ZWN1dGUtYWxsKSBtZWFucyBjYWxsZXJzIGdldCBhCnNpbXBsZSBhbGwtb3Itbm90aGluZyBjb250cmFjdDoKLSBJZiB0aGUgZnVuY3Rpb24gcmV0dXJucyBgT2soKCkpYCwgZXZlcnkgZW50cnkgd2FzIHByb2Nlc3NlZC4KLSBJZiBpdCByZXR1cm5zIGBFcnIoXylgLCBubyB0b2tlbnMgd2VyZSBtb3ZlZCBhbmQgbm8gZGlzdHJpYnV0aW9uCnJlY29yZHMgd2VyZSBjcmVhdGVkLgoKIyBHYXMgbGltaXQgY29uc2lkZXJhdGlvbgoKVGhlIGJhdGNoIHNpemUgaXMgY2FwcGVkIGF0IFtgTUFYX0JBVENIX1NJWkVgXSAoMTAgZW50cmllcykgdG8ga2VlcCB0aGUKdHJhbnNhY3Rpb24gd2l0aGluIFNvcm9iYW4ncyBwZXItdHJhbnNhY3Rpb24gaW5zdHJ1Y3Rpb24gYnVkZ2V0IGV2ZW4Kd2hlbiBldmVyeSBlbnRyeSBwZXJmb3JtcyBib3RoIFhMTSBhbmQgTkZUIG9wZXJhdGlvbnMuCgojIEFyZ3VtZW50cwoqIGBkaXN0cmlidXRpb25zYCAtIEEgYFZlY2Agb2YgYEJhdGNoRGlzdHJpYnV0aW9uRW50cnlgLCBlYWNoIGNvbnRhaW5pbmcKYSBgaHVudF9pZGAsIGBwbGF5ZXJfYWRkcmVzc2AsIGFuZCBgcmV3YXJkX2NvbmZpZ2AuCgojIEVycm9ycwoqIGBJbnZhbGlkQ29uZmlnYCAtIEJhdGNoIGlzIGVtcHR5IG9yIGFuIGVudHJ5IGhhcyBhbiBpbnZhbGlkIGNvbmZpZy4KKiBgQmF0Y2hUb29MYXJnZWAgAAAAEGRpc3RyaWJ1dGVfYmF0Y2gAAAABAAAAAAAAAA1kaXN0cmlidXRpb25zAAAAAAAD6gAAB9AAAAAWQmF0Y2hEaXN0cmlidXRpb25FbnRyeQAAAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAABABGdW5kcyB0aGUgcmV3YXJkIHBvb2wgZm9yIGEgc3BlY2lmaWMgaHVudC4KClRoZSBwb29sIG11c3QgaGF2ZSBiZWVuIGNyZWF0ZWQgdmlhIGBjcmVhdGVfcmV3YXJkX3Bvb2xgIGZpcnN0LgoqKkFueW9uZSBtYXkgZnVuZCBhIHBvb2wqKiDigJQgdGhpcyBzdXBwb3J0cyBzcG9uc29yc2hpcCAoYSBicmFuZCBmdW5kaW5nCmEgY29tbXVuaXR5IGh1bnQsIGEgREFPIHRvcHBpbmcgdXAgYSBwb29sLCBzZXZlcmFsIHBlb3BsZSBwb29saW5nIGEKcHJpemUpLCBub3QganVzdCB0aGUgY3JlYXRvci4gRWFjaCBmdW5kZXIgbXVzdCBhdXRob3JpemUgdGhlIGNhbGwKdGhlbXNlbHZlczsgdGhlaXIgY29udHJpYnV0aW9uIGlzIHRyYWNrZWQgaW5kaXZpZHVhbGx5IHNvIHRoYXQKYHJlZnVuZF9wb29sYCBjYW4gbGF0ZXIgcGF5IHRoZSByZW1haW5pbmcgYmFsYW5jZSBiYWNrIG91dCBpbgpwcm9wb3J0aW9uIHRvIHdoYXQgZWFjaCBmdW5kZXIgcHV0IGluLCBhbmQgbmV2ZXIgaGFuZCBvbmUgZnVuZGVyJ3MKY29udHJpYnV0aW9uIHRvIGFub3RoZXIgcGFydHkuIFNlZSBgZG9jcy9hZHIvMDA2LXJld2FyZC1wb29sLXNwb25zb3JzaGlwLm1kYC4KClRyYW5zZmVycyB0b2tlbnMgZnJvbSB0aGUgZnVuZGVyIHRvIHRoaXMgY29udHJhY3QgYW5kIHJlY29yZHMgdGhlIGJhbGFuY2UuClVzZXMgdGhlIHRva2VuIGFkZHJlc3Mgc3BlY2lmaWVkIHdoZW4gdGhlIHBvb2wgd2FzIGNyZWF0ZWQuCgojIFZhbGlkYXRpb24KLSBNaW5pbXVtIGZ1bmRpbmc6IDEgWExNIGVxdWl2YWxlbnQgKDEwLDAwMCwwMDAgYmFzZSB1bml0cykgdG8gcHJldmVudCBkdXN0IGF0dGFja3MKLSBNYXhpbXVtIHNpbmdsZSBmdW5kaW5nOiAxIGJpbGxpb24gdG9rZW5zIHRvIHByZXZlbnQgb3ZlcmZsb3cKLSBQb29sIGJhbGFuY2UgbGltaXQ6IDEgYmlsbGlvbiB0b2tlbnMgdG90YWwgdG8gcHJldmVudCBvdmVyZmxvdwotIFJlamVjdHMgemVybyBvciBuZWdhdGl2ZSBhbW91bnRzCi0gQXQgbW9zdCBgTUFYX0ZVTkRFUlNfUEVSX1BPT0xgIGRpc3RpbmN0AAAAEGZ1bmRfcmV3YXJkX3Bvb2wAAAADAAAAAAAAAAZmdW5kZXIAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAADNSZXR1cm5zIHRoZSBjdXJyZW50IHJld2FyZCBwb29sIGJhbGFuY2UgZm9yIGEgaHVudC4AAAAAEGdldF9wb29sX2JhbGFuY2UAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAACw==",
        "AAAAAAAAARZSZXR1cm5zIHRoZSBkaXN0aW5jdCBhZGRyZXNzZXMgY3VycmVudGx5IHRyYWNrZWQgYXMgZnVuZGVycyBvZiBhIHBvb2wKKGkuZS4gdGhhdCBoYXZlIGNvbnRyaWJ1dGVkIGFuZCBub3QgeWV0IGJlZW4gcmVmdW5kZWQpLCBpbiB0aGUgb3JkZXIKdGhleSBmaXJzdCBjb250cmlidXRlZC4gRW1wdHkgaWYgdGhlIHBvb2wgaGFzIG5ldmVyIGJlZW4gZnVuZGVkLCBoYXMKYmVlbiBmdWxseSByZWZ1bmRlZCwgb3IgaGFzIG5vIHNwb25zb3JzaGlwIGxlZGdlciAoc2VlIGByZWZ1bmRfcG9vbGApLgAAAAAAEGdldF9wb29sX2Z1bmRlcnMAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAD6gAAABM=",
        "AAAAAAAAAItSZW1haW5pbmcgc2Vjb25kcyB1bnRpbCB0aGUgbmV4dCBkaXN0cmlidXRpb24gaXMgYWxsb3dlZCBmb3IgdGhpcyBwb29sLgpSZXR1cm5zIDAgaWYgbm8gaW50ZXJ2YWwgaXMgY29uZmlndXJlZCBvciB0aGUgY29vbGRvd24gaGFzIGVsYXBzZWQuAAAAABFnZXRfZGlzdF9jb29sZG93bgAAAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAAG",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9zY2hlbWEAAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAACpTdGVwIG9uZSBvZiBhIHR3by1zdGVwIGFkbWluIGtleSByb3RhdGlvbi4AAAAAABFwcm9wb3NlX25ld19hZG1pbgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAABABFeHBsaWNpdGx5IHdpdGhkcmF3cyB0aGUgZW50aXJlIHJlbWFpbmluZyBiYWxhbmNlIGZyb20gYSByZXdhcmQgcG9vbC4KClRoaXMgZnVuY3Rpb24gcHJvdmlkZXMgYW4gZXhwbGljaXQsIGludGVudGlvbmFsIHdheSB0byBkcmFpbiBhIHBvb2wgY29tcGxldGVseS4KVW5saWtlIGBhZG1pbl93aXRoZHJhd191bmNsYWltZWRgLCB3aGljaCBoYW5kbGVzIHBhcnRpYWwgd2l0aGRyYXdhbHMgb2YgdW5jbGFpbWVkCmFtb3VudHMsIHRoaXMgZnVuY3Rpb24gaXMgc2VtYW50aWNhbGx5IGNsZWFyOiBpdCBlbXB0aWVzIHRoZSBwb29sIGJ5IG5hbWUuCgpXaXRoZHJhd2FsIGlzIG9ubHkgcGVybWl0dGVkIGFmdGVyIHRoZSBodW50IGhhcyBlbmRlZCAoZW5kX3RpbWUgcGFzc2VkKSBvciBiZWVuCmNhbmNlbGxlZC4gVGhpcyBwcmV2ZW50cyBkcmFpbmluZyBwb29scyB3aGlsZSBhIGh1bnQgaXMgYWN0aXZlIGFuZCBwbGF5ZXJzIG1heQpzdGlsbCBiZSBtaWQtZ2FtZS4gV2hlbiBIdW50eUNvcmUgaXMgY29uZmlndXJlZCwgdGhlIGh1bnQgc3RhdHVzIGlzIHZlcmlmaWVkLgoKIyBBcmd1bWVudHMKKiBgYWRtaW5gIC0gVGhlIGNvbnRyYWN0IGFkbWluIGFkZHJlc3MgKG11c3QgbWF0Y2ggdGhlIHN0b3JlZCBhZG1pbikKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB3aG9zZSBwb29sIHRvIGRyYWluIGNvbXBsZXRlbHkKKiBgcmVjaXBpZW50YCAtIFRoZSBhZGRyZXNzIHRoYXQgd2lsbCByZWNlaXZlIHRoZSBmdWxsIHBvb2wgYmFsYW5jZQoKIyBFcnJvcnMKKiBgTm90SW5pdGlhbGl6ZWRgIC0gQ29udHJhY3QgaGFzIG5vdCBiZWVuIGluaXRpYWxpemVkIChubyBhZG1pbiBzZXQpCiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90IHRoZSBjb250cmFjdCBhZG1pbgoqIGBQb29sTm90Rm91bmRgIC0gTm8gcG9vbCBleGlzdHMgZm9yIHRoaXMgaHVudF9pZAoqIGBJbnZhbGlkQW1vdW50YCAtIFBvb2wgYmFsYW5jZSBpcyB6ZXJvIChub3RoaW5nIHRvIHdpdGhkcmF3KQoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50AAAAEmFkbWluX3dpdGhkcmF3X2FsbAAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAACXJlY2lwaWVudAAAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAA/FDcmVhdGVzIGEgcmV3YXJkIHBvb2wgZm9yIGEgc3BlY2lmaWMgaHVudCB3aXRoIGEgc3BlY2lmaWVkIHRva2VuLgoKTXVzdCBiZSBjYWxsZWQgYmVmb3JlIGBmdW5kX3Jld2FyZF9wb29sYC4gQW55IGFkZHJlc3MgbWF5IGZ1bmQgdGhlIHBvb2wKYWZ0ZXIgY3JlYXRpb24gKHNlZSBgZnVuZF9yZXdhcmRfcG9vbGApOyB0aGUgdG9rZW4gY29udHJhY3QgbXVzdCBiZQpTQUMtY29tcGF0aWJsZS4KCiMgQXJndW1lbnRzCiogYGNyZWF0b3JgIC0gVGhlIGh1bnQgY3JlYXRvciB3aG8gd2lsbCBvd24gYW5kIGZ1bmQgdGhlIHBvb2wKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0aGlzIHBvb2wgaXMgZm9yCiogYHRva2VuX2FkZHJlc3NgIC0gQWRkcmVzcyBvZiB0aGUgU0FDLWNvbXBhdGlibGUgdG9rZW4gY29udHJhY3QgKGUuZy4sIFhMTSwgVVNEQykKKiBgbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnRgIC0gTWluaW11bSB0b2tlbiBhbW91bnQgcGVyIGRpc3RyaWJ1dGlvbiAoMCA9IG5vIG1pbmltdW0pCiogYG5mdF9yb3lhbHR5X2Jwc2AgLSBDcmVhdG9yIHJveWFsdHkgYmFzaXMgcG9pbnRzICgwLTEwMDAwKSBmb3Igc2Vjb25kYXJ5IG1hcmtldCBzYWxlcwoqIGBuZnRfdHJhbnNmZXJhYmxlYCAtIFdoZXRoZXIgcmV3YXJkIE5GVHMgZnJvbSB0aGlzIHBvb2wgYXJlIHRyYW5zZmVyYWJsZQoKIyBFcnJvcnMKKiBgUG9vbEFscmVhZHlFeGlzdHNgIC0gQSBwb29sIGFscmVhZHkgZXhpc3RzIGZvciB0aGlzIGh1bnRfaWQKKiBgSW52YWxpZEFtb3VudGAgLSBtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudCBpcyBuZWdhdGl2ZQoqIGBJbnZhbGlkVG9rZW5Db250cmFjdGAgLSB0b2tlbl9hZGRyZXNzIGlzIG5vdCBhIHZhbGlkIFNBQy1jb21wYXRpYmxlIHRva2VuCiogYE5vdEluaXRpYWxpemVkYCAtIGh1bnR5X2NvcmUgaGFzIG5vdCBiZWVuIGNvbmZpZ3VyZWQgKHNldCBkdXJpbmcgaW5pdGlhbGl6ZSkKKiBgSHVudE5vdEZvdW5kYCAtIGh1bnRfaWQgZG9lcyBub3QgZXhpc3QgaW4gSHVudHlDb3JlAAAAAAAAEmNyZWF0ZV9yZXdhcmRfcG9vbAAAAAAABgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAABdtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudAAAAAALAAAAAAAAAA9uZnRfcm95YWx0eV9icHMAAAAABAAAAAAAAAAQbmZ0X3RyYW5zZmVyYWJsZQAAAAEAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAVZMZWdhY3kgZW50cnlwb2ludCByZXRhaW5lZCBmb3IgZXhpc3RpbmcgaW50ZWdyYXRpb25zLiBOZXcgY29udHJhY3QKaW50ZWdyYXRpb25zIHNob3VsZCB1c2UgYGRpc3RyaWJ1dGVfcmV3YXJkc19hdXRob3JpemVkYCwgd2hpY2ggY2FycmllcwphbmQgYXV0aGVudGljYXRlcyB0aGUgY2FsbGluZyBjb250cmFjdCBleHBsaWNpdGx5LgoKQXV0aG9yaXphdGlvbiBpcyBmYWlsLWNsb3NlZDogdGhlIGNhbGxlciBtdXN0IGJlIGFuIGF1dGhvcml6ZWQKZGlzdHJpYnV0b3IgKHNlZSBgYWRkX2F1dGhvcml6ZWRfY29udHJhY3RgKS4gVW5hdXRob3JpemVkIGNhbGxlcnMKcmVjZWl2ZSBgVW5hdXRob3JpemVkYC4AAAAAABJkaXN0cmlidXRlX3Jld2FyZHMAAAAAAAQAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAA5wbGF5ZXJfYWRkcmVzcwAAAAAAEwAAAAAAAAANcmV3YXJkX2NvbmZpZwAAAAAAB9AAAAAMUmV3YXJkQ29uZmlnAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAxBFbWVyZ2VuY3kgd2l0aGRyYXdhbDogYWxsb3dzIHRoZSBhZG1pbiB0byB3aXRoZHJhdyBhbGwgZnVuZHMgZnJvbSBvbmUgb3IgYWxsCnJld2FyZCBwb29scyB3aGVuIHRoZSBjb250cmFjdCBpcyBwYXVzZWQgKGUuZy4gZHVlIHRvIGEgY3JpdGljYWwgdnVsbmVyYWJpbGl0eSkuCldoZW4gYGh1bnRfaWRgIGlzIDAsIGFsbCBwb29scyB3aXRoIG5vbi16ZXJvIGJhbGFuY2VzIGFyZSBkcmFpbmVkLgpXaGVuIGBhbGxfcG9vbHNgIGlzIHRydWUsIGl0ZXJhdGVzIGFsbCBodW50cyB1cCB0byBgbWF4X2h1bnRfaWRgIGFuZCB3aXRoZHJhd3MuCgojIEFyZ3VtZW50cwoqIGBhZG1pbmAgLSBUaGUgY29udHJhY3QgYWRtaW4gYWRkcmVzcwoqIGBodW50X2lkYCAtIFNwZWNpZmljIGh1bnQgcG9vbCB0byBkcmFpbiAoMCA9IGFsbCBwb29scyB1cCB0byBtYXhfaHVudF9pZCkKKiBgcmVjaXBpZW50YCAtIEFkZHJlc3MgdG8gcmVjZWl2ZSB0aGUgd2l0aGRyYXduIGZ1bmRzCiogYHJlYXNvbmAgLSBSZWFzb24gZm9yIHRoZSBlbWVyZ2VuY3kgd2l0aGRyYXdhbCAoZW1pdHRlZCBpbiBldmVudHMpCiogYG1heF9odW50X2lkYCAtIFdoZW4gaHVudF9pZCBpcyAwLCBkcmFpbnMgYWxsIHBvb2xzIGZyb20gMS4uPW1heF9odW50X2lkCgojIEVycm9ycwoqIGBOb3RJbml0aWFsaXplZGAgLSBDb250cmFjdCBub3QgaW5pdGlhbGl6ZWQKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgYWRtaW4KKiBgQ29udHJhY3RQYXVzZWRgIC0gQ29udHJhY3QgbXVzdCBiZSBwYXVzZWQgdG8gY2FsbCB0aGlzAAAAEmVtZXJnZW5jeV93aXRoZHJhdwAAAAAABQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAACXJlY2lwaWVudAAAAAAAABMAAAAAAAAABnJlYXNvbgAAAAAAEAAAAAAAAAALbWF4X2h1bnRfaWQAAAAABgAAAAEAAAPpAAAACwAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAAC1SZXR1cm5zIHRoZSBlbWVyZ2VuY3kgd2l0aGRyYXdhbCBsb2cgZW50cmllcy4AAAAAAAASZ2V0X2VtZXJnZW5jeV9sb2dzAAAAAAAAAAAAAQAAA+oAAAfQAAAAG0VtZXJnZW5jeVdpdGhkcmF3YWxMb2dFbnRyeQA=",
        "AAAAAAAAAEFFeHBvc2VzIGEgcGFnaW5hdGVkIHJlYWQgcXVlcnkgZm9yIHRoZSBhdWRpdCBsb2cgb2YgYSBnaXZlbiBwb29sLgAAAAAAABJnZXRfcG9vbF9hdWRpdF9sb2cAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAALc3RhcnRfYWZ0ZXIAAAAD6AAAAAYAAAAAAAAABWxpbWl0AAAAAAAD6AAAAAQAAAABAAAH0AAAABRQb29sQXVkaXRMb2dSZXNwb25zZQ==",
        "AAAAAAAAAAAAAAASZ2V0X3NjaGVtYV92ZXJzaW9uAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAM5SZXR1cm5zIHRoZSBjdXJyZW50IHZlc3Rpbmcgc3RhdHVzIGZvciBhIChodW50X2lkLCBwbGF5ZXIpIHBhaXIuCgpSZXR1cm5zIGBOb25lYCB3aGVuIG5vIHZlc3RpbmcgcmVjb3JkIGV4aXN0cyAoaS5lLiB0aGUgcG9vbCBlaXRoZXIgaGFkCm5vIHZlc3RpbmcgY29uZmlndXJlZCBvciB0aGUgcGxheWVyIGhhcyBub3QgY29tcGxldGVkIHRoYXQgaHVudCB5ZXQpLgAAAAAAEmdldF92ZXN0aW5nX3N0YXR1cwAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6AAAB9AAAAANVmVzdGluZ1N0YXR1cwAAAA==",
        "AAAAAAAAAEtCbG9ja3MgcmV3YXJkIGRpc3RyaWJ1dGlvbi4gRnVuZGluZyBpcyB1bmFmZmVjdGVkIHVubGVzcyBzZXBhcmF0ZWx5IHBhdXNlZC4AAAAAEnBhdXNlX2Rpc3RyaWJ1dGlvbgAAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAAAAAAAScm9sbGJhY2tfbWlncmF0aW9uAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AAAAB9AAAAAQVXBncmFkZUF1dGhFcnJvcg==",
        "AAAAAAAAA9VTZXRzIHRoZSBkYWlseSBkaXN0cmlidXRpb24gY2FwIGZvciBhIHNwZWNpZmljIHBvb2wuCgpUaGlzIGxpbWl0IGNvbnRyb2xzIHRoZSBtYXhpbXVtIGFtb3VudCBvZiByZXdhcmRzIHRoYXQgY2FuIGJlIGRpc3RyaWJ1dGVkIGZyb20KYSBwb29sIGluIGEgc2luZ2xlIGRheSAoMjQtaG91ciByb2xsaW5nIHdpbmRvdykuIFRoaXMgaXMgYSBsaXZlIG9wZXJhdGlvbmFsIGNvbnRyb2wKYW5kIHNob3VsZCBiZSB2YWxpZGF0ZWQgdG8gcHJldmVudCBzaWxlbnQgbWlzY29uZmlndXJhdGlvbi4KCiMgQXJndW1lbnRzCiogYGFkbWluYCAtIFRoZSBjb250cmFjdCBhZG1pbiBhZGRyZXNzIChtdXN0IG1hdGNoIHRoZSBzdG9yZWQgYWRtaW4pCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCBjYXAgdG8gc2V0CiogYGNhcGAgLSBUaGUgbWF4aW11bSBhbW91bnQgdG8gZGlzdHJpYnV0ZSBwZXIgZGF5LiBNdXN0IGJlIG5vbi1uZWdhdGl2ZS4KQSBjYXAgb2YgMCAqKmRpc2FibGVzIGFsbCBkaXN0cmlidXRpb25zKiogZnJvbSB0aGlzIHBvb2wgKHRoZQpkaXN0cmlidXRpb24gcGF0aCByZWplY3RzIGV2ZXJ5IGF0dGVtcHQgd2l0aCBgRGFpbHlDYXBFeGNlZWRlZGApLgpVc2UgYGZyZWV6ZV9wb29sYCBmb3IgYSBzZW1hbnRpY2FsbHkgcmljaGVyIGZyZWV6ZS4gQSBwb3NpdGl2ZQp2YWx1ZSBzZXRzIGEgcm9sbGluZyAyNC1ob3VyIGRpc3RyaWJ1dGlvbiBsaW1pdC4KCiMgRXJyb3JzCiogYE5vdEluaXRpYWxpemVkYCAtIENvbnRyYWN0IGhhcyBub3QgYmVlbiBpbml0aWFsaXplZCAobm8gYWRtaW4gc2V0KQoqIGBVbmF1dGhvcml6ZWRgIC0gQ2FsbGVyIGlzIG5vdCB0aGUgY29udHJhY3QgYWRtaW4KKiBgUG9vbE5vdEZvdW5kYCAtIE5vIHBvb2wgZXhpc3RzIGZvciB0aGlzIGh1bnRfaWQKKiBgSW52YWxpZEFtb3VudGAgLSBDYXAgaXMgbmVnYXRpdmUgKG5lZ2F0aXZlIGNhcHMgc2lsZW50bHkgYmxvY2sgZGlzdHJpYnV0aW9ucykAAAAAAAASc2V0X2RhaWx5X3Bvb2xfY2FwAAAAAAADAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAADY2FwAAAAAAsAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAldVcGRhdGVzIHRoZSBgbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnRgIGZvciBhbiBleGlzdGluZyByZXdhcmQgcG9vbC4KCk9ubHkgdGhlIHBvb2wgY3JlYXRvciBpcyBhdXRob3JpemVkIHRvIGNhbGwgdGhpcy4gVXNlZnVsIHdoZW4gYSBjcmVhdG9yCmhhcyB1bmRlcmZ1bmRlZCB0aGUgcG9vbCBhbmQgbmVlZHMgdG8gbG93ZXIgdGhlIG1pbmltdW0gc28gZGlzdHJpYnV0aW9ucwpjYW4gcHJvY2VlZC4KCiMgQXJndW1lbnRzCiogYGNyZWF0b3JgIC0gVGhlIHBvb2wgY3JlYXRvciAobXVzdCBtYXRjaCB0aGUgc3RvcmVkIGNyZWF0b3IpCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCBjb25maWcgdG8gdXBkYXRlCiogYG1pbl9kaXN0cmlidXRpb25fYW1vdW50YCAtIE5ldyBtaW5pbXVtIFhMTSBwZXIgZGlzdHJpYnV0aW9uICgwID0gbm8gbWluaW11bSkKCiMgRXJyb3JzCiogYFBvb2xOb3RGb3VuZGAgLSBObyBwb29sIGV4aXN0cyBmb3IgdGhpcyBodW50X2lkCiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90IHRoZSBwb29sIGNyZWF0b3IKKiBgSW52YWxpZEFtb3VudGAgLSBtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudCBpcyBuZWdhdGl2ZQAAAAASdXBkYXRlX3Bvb2xfY29uZmlnAAAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAXbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnQAAAAACwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAAHNSZXR1cm5zIGNvbXByZWhlbnNpdmUgc3RhdGlzdGljcyBmb3IgYSByZXdhcmQgcG9vbC4KUmV0dXJucyBOb25lIGlmIG5vIHBvb2wgaGFzIGJlZW4gY3JlYXRlZCBmb3IgdGhlIGdpdmVuIGh1bnRfaWQuAAAAABNnZXRfcG9vbF9zdGF0aXN0aWNzAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPoAAAH0AAAABRSZXdhcmRQb29sU3RhdGlzdGljcw==",
        "AAAAAAAAAHxUaGUgZ3JhbnVsYXIgZmxhZ3MgYXMgc3RvcmVkLCBpZ25vcmluZyB0aGUgZ2xvYmFsIHN0b3Ag4oCUIGxldHMgYW4Kb3BlcmF0b3Igc2VlIHdoYXQgd2lsbCBzdGlsbCBiZSBwYXVzZWQgYWZ0ZXIgYHVucGF1c2UoKWAuAAAAE2dldF9yYXdfcGF1c2VfZmxhZ3MAAAAAAAAAAAEAAAPtAAAAAgAAAAEAAAAB",
        "AAAAAAAAAAAAAAATZ2V0X3VwZ3JhZGVfaGlzdG9yeQAAAAACAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAH0AAAABNVcGdyYWRlSGlzdG9yeUVudHJ5AA==",
        "AAAAAAAAAw5VcGRhdGVzIChvciBpbnN0YWxscykgZXhhY3QgY29tcGxldGlvbi1yYW5rIHJld2FyZCB0aWVycyBvbiBhbiBleGlzdGluZyBwb29sLgoKUmFua3MgYXJlIG9uZS1iYXNlZCBhbmQgdGhlIGxpc3QgbXVzdCBjb250YWluIHN0cmljdGx5IGluY3JlYXNpbmcgcmFua3MKd2l0aCBzdHJpY3RseSBwb3NpdGl2ZSBhbW91bnRzLiBBIG1hdGNoaW5nIHJhbmsgaXMgc2VsZWN0ZWQgdXNpbmcgdGhlCmltbXV0YWJsZSBjb21wbGV0aW9uIHJhbmsgc3VwcGxpZWQgYnkgSHVudHlDb3JlOyByYW5rcyBub3QgcHJlc2VudCBpbgp0aGUgbGlzdCByZXRhaW4gdGhlIGV4aXN0aW5nIGZsYXQvdGltZS1iYXNlZCBiZWhhdmlvci4gUGFzc2luZyBhbiBlbXB0eQpsaXN0IGRpc2FibGVzIHJhbmstYmFzZWQgcmV3YXJkcy4KCk9ubHkgdGhlIHBvb2wgY3JlYXRvciBtYXkgY2hhbmdlIHRoaXMgY29uZmlndXJhdGlvbi4gQ2hhbmdlcyBhZmZlY3QKc3Vic2VxdWVudCBkaXN0cmlidXRpb25zIGFuZCBuZXZlciByZXdyaXRlIGFuIGFscmVhZHktcmVjb3JkZWQgcGF5b3V0LgoKIyBFcnJvcnMKKiBgUG9vbE5vdEZvdW5kYCAtIE5vIHBvb2wgZXhpc3RzIGZvciB0aGlzIGh1bnRfaWQKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIHBvb2wgY3JlYXRvcgoqIGBJbnZhbGlkQ29uZmlnYCAtIFRpZXIgbGlzdCBpcyBsb25nZXIgdGhhbiBbYE1BWF9USUVSX0VOVFJJRVNgXSwgb3IKKHdoZW4gbm9uLWVtcHR5KSBpcyBub3Qgc3RyaWN0bHkgYXNjZW5kaW5nIHdpdGggcG9zaXRpdmUgYW1vdW50cwAAAAAAE3NldF9wb29sX3JhbmtfdGllcnMAAAAAAwAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAEHJhbmtfYmFzZWRfdGllcnMAAAPqAAAH0AAAAA5SYW5rUmV3YXJkVGllcgAAAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAM9WZXJpZmllcyBhIGRpc3RyaWJ1dGlvbiBwcm9vZiBhZ2FpbnN0IHRoZSBvbi1jaGFpbiByZWNlaXB0LgoKUmVjb21wdXRlcyBTSEEtMjU2KHBvb2xfaWQgfHwgcGxheWVyIHx8IGFtb3VudCB8fCB0aW1lc3RhbXApIGFuZCBjaGVja3MKaXQgbWF0Y2hlcyBib3RoIHRoZSBwcm92aWRlZCBgaGFzaGAgYW5kIHRoZSBzdG9yZWQgcmVjZWlwdCAod2hlbiBwcmVzZW50KS4AAAAAE3ZlcmlmeV9kaXN0cmlidXRpb24AAAAABQAAAAAAAAAHcG9vbF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABgAAAAAAAAAEaGFzaAAAA+4AAAAgAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAUZ2V0X2hlYWx0aF9kYXNoYm9hcmQAAAAAAAAAAQAAB9AAAAAOQ29udHJhY3RIZWFsdGgAAA==",
        "AAAAAAAAAAAAAAAUZ2V0X3VwZ3JhZGVfcHJvcG9zYWwAAAAAAAAAAQAAA+gAAAfQAAAAD1VwZ3JhZGVQcm9wb3NhbAA=",
        "AAAAAAAAAAAAAAAUZ2V0X3VwZ3JhZGVfdGltZWxvY2sAAAAAAAAAAQAAAAY=",
        "AAAAAAAAAAAAAAAUc2V0X2RhaWx5X2dsb2JhbF9jYXAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAA2NhcAAAAAALAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAAAAAAAUc2V0X3VwZ3JhZGVfdGltZWxvY2sAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADWRlbGF5X3NlY29uZHMAAAAAAAAGAAAAAQAAA+kAAAACAAAH0AAAABBVcGdyYWRlQXV0aEVycm9y",
        "AAAAAAAAAE1SZXN1bWVzIHJld2FyZCBkaXN0cmlidXRpb24uIEhhcyBubyBlZmZlY3Qgd2hpbGUgdGhlIGdsb2JhbCBwYXVzZSBpcyBlbmdhZ2VkLgAAAAAAABR1bnBhdXNlX2Rpc3RyaWJ1dGlvbgAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAQAAADJMb2cgZW50cnkgZm9yIGVtZXJnZW5jeSB3aXRoZHJhd2FsIHJlY29yZC1rZWVwaW5nLgAAAAAAAAAAABtFbWVyZ2VuY3lXaXRoZHJhd2FsTG9nRW50cnkAAAAABAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnJlYXNvbgAAAAAAEAAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAAAAAEVSZXR1cm5zIHdoZXRoZXIgYSByZXdhcmQgaGFzIGJlZW4gZGlzdHJpYnV0ZWQgdG8gYSBwbGF5ZXIgZm9yIGEgaHVudC4AAAAAAAAVaXNfcmV3YXJkX2Rpc3RyaWJ1dGVkAAAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAA81SZXRyaWVzIGEgZmFpbGVkIE5GVCBtaW50IGZvciBhIHByZXZpb3VzbHkgZGlzdHJpYnV0ZWQgcmV3YXJkLgoKV2hlbiBORlQgbWludGluZyBmYWlscyBkdXJpbmcgYGRpc3RyaWJ1dGVfcmV3YXJkc2AsIHRoZSBmYWlsdXJlIGlzIGxvZ2dlZAphbmQgdGhlIHBlbmRpbmcgbWludCBkYXRhIGlzIHN0b3JlZC4gVGhpcyBmdW5jdGlvbiBhbGxvd3MgdGhlIHBsYXllcgoob3IgYW55b25lIHBheWluZyB0aGUgdHJhbnNhY3Rpb24gZmVlIG9uIGJlaGFsZiBvZiB0aGUgcGxheWVyKSB0bwpyZXRyeSB0aGUgZmFpbGVkIE5GVCBtaW50IGFuZCB1cGRhdGUgdGhlIGRpc3RyaWJ1dGlvbiByZWNvcmQuIFRoZQpzdWNjZXNzZnVsbHkgbWludGVkIE5GVCBpcyBhbHdheXMgc2VudCB0byB0aGUgYHBsYXllcmAgYWRkcmVzcwpyZWNvcmRlZCBpbiB0aGUgcGVuZGluZyBtaW50LCByZWdhcmRsZXNzIG9mIHdobyBjYWxscyB0aGlzIGZ1bmN0aW9uLgoKIyBBcmd1bWVudHMKKiBgY2FsbGVyYCAtIFRoZSBhZGRyZXNzIHNpZ25pbmcgdGhlIHRyYW5zYWN0aW9uIChhbnkgYWRkcmVzczsgTkZUIGlzCnN0aWxsIGRlbGl2ZXJlZCB0byB0aGUgYHBsYXllcmAgcmVjb3JkZWQgaW4gdGhlIHBlbmRpbmcgbWludCkKKiBgaHVudF9pZGAgLSBUaGUgaHVudCBhc3NvY2lhdGVkIHdpdGggdGhlIGZhaWxlZCBORlQgbWludAoqIGBwbGF5ZXJgIC0gVGhlIHBsYXllciB3aG8gc2hvdWxkIHJlY2VpdmUgdGhlIE5GVAoKIyBSZXR1cm5zClRoZSBORlQgSUQgb2YgdGhlIHN1Y2Nlc3NmdWxseSBtaW50ZWQgTkZUCgojIEVycm9ycwoqIGBOZnRNaW50UGVuZGluZ05vdEZvdW5kYCAtIE5vIHBlbmRpbmcgZmFpbGVkIE5GVCBtaW50IGZvciB0aGlzIGh1bnQvcGxheWVyCiogYFBvb2xOb3RGb3VuZGAgLSBObyBwb29sIGNvbmZpZyBleGlzdHMgZm9yIHRoaXMgaHVudF9pZAoqIGBOZnRNaW50RmFpbGVkYCAtIE5GVCBtaW50IGF0dGVtcHQgZmFpbGVkIGFnYWluAAAAAAAAFXJldHJ5X2ZhaWxlZF9uZnRfbWludAAAAAAAAAMAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAAAYAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAD5TZXRzIHRoZSBkaXN0cmlidXRpb24gbW9kZSAoRml4ZWQgb3IgUHJvcG9ydGlvbmFsKSBmb3IgYSBwb29sLgAAAAAAFXNldF9kaXN0cmlidXRpb25fbW9kZQAAAAAAAAMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAARtb2RlAAAH0AAAABBEaXN0cmlidXRpb25Nb2RlAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAcFTZXRzIG9yIHVwZGF0ZXMgdGhlIE5GVCBjb250cmFjdCBhZGRyZXNzIGZvciBhbiBleGlzdGluZyByZXdhcmQgcG9vbC4KVGhpcyBhbGxvd3MgcG9vbHMgdG8gZGlzdHJpYnV0ZSBORlRzIGFsb25nc2lkZSBvciBpbnN0ZWFkIG9mIHRva2Vucy4KCiMgQXJndW1lbnRzCiogYGNyZWF0b3JgIC0gVGhlIHBvb2wgY3JlYXRvciAobXVzdCBtYXRjaCB0aGUgc3RvcmVkIGNyZWF0b3IpCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCBjb25maWcgdG8gdXBkYXRlCiogYG5mdF9jb250cmFjdGAgLSBORlQgY29udHJhY3QgYWRkcmVzcyAob3IgTm9uZSB0byBkaXNhYmxlIE5GVCByZXdhcmRzKQoKIyBFcnJvcnMKKiBgUG9vbE5vdEZvdW5kYCAtIE5vIHBvb2wgZXhpc3RzIGZvciB0aGlzIGh1bnRfaWQKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIHBvb2wgY3JlYXRvcgAAAAAAABVzZXRfcG9vbF9uZnRfY29udHJhY3QAAAAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAMbmZ0X2NvbnRyYWN0AAAD6AAAABMAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAEdSZXR1cm5zIHRoZSBvbi1jaGFpbiBkaXN0cmlidXRpb24gcmVjZWlwdC9wcm9vZiBmb3IgYSBodW50L3BsYXllciBwYWlyLgAAAAAWZ2V0X2Rpc3RyaWJ1dGlvbl9wcm9vZgAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6AAAB9AAAAARRGlzdHJpYnV0aW9uUHJvb2YAAAA=",
        "AAAAAAAAAa1SZXR1cm5zIGEgcGFnaW5hdGVkIGxpc3Qgb2YgZGlzdHJpYnV0aW9ucyBtYWRlIGZyb20gYSBzcGVjaWZpYyByZXdhcmQgcG9vbC4KCiMgQXJndW1lbnRzCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCBkaXN0cmlidXRpb25zIHRvIHF1ZXJ5CiogYG9mZnNldGAgLSBTdGFydGluZyBpbmRleCBmb3IgcGFnaW5hdGlvbiAoMC1iYXNlZCkKKiBgbGltaXRgIC0gTWF4aW11bSBudW1iZXIgb2YgZW50cmllcyB0byByZXR1cm4KCiMgUmV0dXJucwpBIFZlYyBvZiBQb29sRGlzdHJpYnV0aW9uIGVudHJpZXMgY29udGFpbmluZyBwbGF5ZXIgYWRkcmVzc2VzIGFuZCBkaXN0cmlidXRpb24gZGV0YWlscy4KUmV0dXJucyBhbiBlbXB0eSBWZWMgaWYgdGhlIHBvb2wgaGFzIG5vIGRpc3RyaWJ1dGlvbnMgb3Igb2Zmc2V0IGlzIGJleW9uZCB0aGUgbGlzdC4AAAAAAAAWZ2V0X3Bvb2xfZGlzdHJpYnV0aW9ucwAAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAH0AAAABBQb29sRGlzdHJpYnV0aW9u",
        "AAAAAAAAAiJSZXR1cm5zIGEgcGFnaW5hdGVkIGxpc3Qgb2YgYWxsIHBlbmRpbmcgZmFpbGVkIE5GVCBtaW50cyBhY3Jvc3MgdGhlCmVudGlyZSBjb250cmFjdC4KCkVhY2ggZW50cnkgY29udGFpbnMgdGhlIGZ1bGwgbWludCBtZXRhZGF0YSAoaHVudCwgcGxheWVyLCBORlQgY29udHJhY3QsCnJhcml0eSwgZXRjLikgc28gY2FsbGVycyBjYW4gaWRlbnRpZnkgd2hpY2ggbWludHMgbmVlZCB0byBiZSByZXRyaWVkLgoKIyBBcmd1bWVudHMKKiBgb2Zmc2V0YCAtIFN0YXJ0aW5nIGluZGV4IGZvciBwYWdpbmF0aW9uICgwLWJhc2VkKQoqIGBsaW1pdGAgLSBNYXhpbXVtIG51bWJlciBvZiBlbnRyaWVzIHRvIHJldHVybgoKIyBSZXR1cm5zCkEgYFZlYzxQZW5kaW5nTmZ0TWludD5gIG9mIHBlbmRpbmcgbWludCBlbnRyaWVzLCB1cCB0byBgbGltaXRgIGVudHJpZXMKc3RhcnRpbmcgZnJvbSBgb2Zmc2V0YC4gUmV0dXJucyBhbiBlbXB0eSBgVmVjYCB3aGVuIGBvZmZzZXRgIGlzIGJleW9uZAp0aGUgZW5kIG9mIHRoZSBsaXN0IG9yIHdoZW4gbm8gcGVuZGluZyBtaW50cyBleGlzdC4AAAAAABZsaXN0X3BlbmRpbmdfbmZ0X21pbnRzAAAAAAACAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAH0AAAAA5QZW5kaW5nTmZ0TWludAAA",
        "AAAAAAAAAIVTZXRzIHRoZSBmdW5kaW5nIHRhcmdldCB1c2VkIGZvciB0b3AtdXAgcHJvZ3Jlc3Mgbm90aWZpY2F0aW9ucy4KYHRhcmdldF9hbW91bnRgIG9mIDAgZGlzYWJsZXMgcGVyY2VudGFnZSB0cmFja2luZyAoZXZlbnRzIHJlcG9ydCAwJSkuAAAAAAAAFnNldF9wb29sX3RhcmdldF9hbW91bnQAAAAAAAMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAA10YXJnZXRfYW1vdW50AAAAAAAACwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAAG9BZGRzIGEgY29udHJhY3QgdG8gdGhlIGF1dGhvcml6ZWQgY2FsbGVycyBsaXN0IGZvciBgZGlzdHJpYnV0ZV9yZXdhcmRzYC4KT25seSB0aGUgY29udHJhY3QgYWRtaW4gY2FuIGNhbGwgdGhpcy4AAAAAF2FkZF9hdXRob3JpemVkX2NvbnRyYWN0AAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIY29udHJhY3QAAAATAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAXBEaXN0cmlidXRlIGEgcHJvcG9ydGlvbmFsIHNoYXJlIG9mIHRoZSBwb29sIGJhc2VkIG9uIHBsYXllciBzY29yZS4KCkFtb3VudCA9IGZsb29yKChwbGF5ZXJfc2NvcmUgLyB0b3RhbF9zY29yZXMpICogcG9vbF9iYWxhbmNlKS4KUmVtYWluZGVyIHN0YXlzIGluIHRoZSBwb29sLiBFbmZvcmNlcyBtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudCB3aGVuIHNldC4KUmVxdWlyZXMgdGhlIHBvb2wncyBkaXN0cmlidXRpb25fbW9kZSB0byBiZSBQcm9wb3J0aW9uYWwgKG9yIHdpbGwgc3RpbGwKY29tcHV0ZSBwcm9wb3J0aW9uYWxseSB3aGVuIGNhbGxlZCB2aWEgdGhpcyBlbnRyeSBwb2ludCkuCgpSZXR1cm5zIHRoZSBYTE0gYW1vdW50IGRpc3RyaWJ1dGVkLgAAABdkaXN0cmlidXRlX3Byb3BvcnRpb25hbAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAMcGxheWVyX3Njb3JlAAAABgAAAAAAAAAMdG90YWxfc2NvcmVzAAAABgAAAAEAAAPpAAAACwAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAADdSZXR1cm5zIHRoZSBkaXN0cmlidXRpb24gc3RhdHVzIGZvciBhIGh1bnQvcGxheWVyIHBhaXIuAAAAABdnZXRfZGlzdHJpYnV0aW9uX3N0YXR1cwAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAEAAAfQAAAAEkRpc3RyaWJ1dGlvblN0YXR1cwAA",
        "AAAAAAAAALpTZXRzIHRoZSBkZWZhdWx0IE5mdFJld2FyZCBjb250cmFjdCBhZGRyZXNzIHVzZWQgZm9yIE5GVCBkaXN0cmlidXRpb25zCndoZW4gYSBwZXItY2FsbCBORlQgY29udHJhY3QgaXMgbm90IHByb3ZpZGVkLgpFbWl0cyBhbiBOZnRDb250cmFjdFNldEV2ZW50IHdpdGggdGhlIG9sZCBhbmQgbmV3IGNvbnRyYWN0IGFkZHJlc3Nlcy4AAAAAABdzZXRfbmZ0X3Jld2FyZF9jb250cmFjdAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADG5mdF9jb250cmFjdAAAABMAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAuhTZXRzIHRoZSB2ZXN0aW5nIHBlcmlvZCAoaW4gc2Vjb25kcykgb24gYW4gZXhpc3RpbmcgcmV3YXJkIHBvb2wuCgpXaGVuIGB2ZXN0aW5nX3BlcmlvZF9zZWNzID4gMGAsIHN1YnNlcXVlbnQgYGRpc3RyaWJ1dGVfcmV3YXJkc2AgY2FsbHMKd2lsbCAqKm5vdCoqIHRyYW5zZmVyIFhMTSBpbW1lZGlhdGVseS4gSW5zdGVhZCBhIGBWZXN0aW5nUmVjb3JkYCBpcwpzdG9yZWQgYW5kIHRoZSBwbGF5ZXIgbXVzdCBjYWxsIGBjbGFpbV92ZXN0ZWRgIHRvIHJlY2VpdmUgdG9rZW5zCnByb3BvcnRpb25hbGx5IGFzIHRpbWUgZWxhcHNlcyBhZnRlciBkaXN0cmlidXRpb24uCgpTZXR0aW5nIHRoaXMgdG8gYDBgIGRpc2FibGVzIHZlc3RpbmcgYW5kIHJldmVydHMgdG8gaW5zdGFudCBwYXlvdXRzIGZvcgpmdXR1cmUgZGlzdHJpYnV0aW9ucyAoYWxyZWFkeS1wZW5kaW5nIHZlc3RpbmcgcmVjb3JkcyBhcmUgdW5hZmZlY3RlZCkuCgojIEFyZ3VtZW50cwoqIGBjcmVhdG9yYCAtIFBvb2wgb3duZXIgKG11c3QgbWF0Y2ggc3RvcmVkIGNyZWF0b3IpCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCB0byBjb25maWd1cmUKKiBgdmVzdGluZ19wZXJpb2Rfc2Vjc2AgLSBWZXN0aW5nIGR1cmF0aW9uIGluIHNlY29uZHMgKDAgPSBkaXNhYmxlZCkKCiMgRXJyb3JzCiogYFBvb2xOb3RGb3VuZGAgLSBQb29sIGRvZXMgbm90IGV4aXN0CiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90IHRoZSBwb29sIGNyZWF0b3IAAAAXc2V0X3Zlc3RpbmdfcGVyaW9kX3NlY3MAAAAAAwAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAE3Zlc3RpbmdfcGVyaW9kX3NlY3MAAAAABgAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAABABBbGxvd3MgdGhlIGFkbWluIHRvIHdpdGhkcmF3IHVuY2xhaW1lZCAoc3VycGx1cykgWExNIHJlbWFpbmluZyBpbiBhIHJld2FyZCBwb29sCmFmdGVyIHRoZSBodW50IGhhcyBlbmRlZCBhbmQgYWxsIHdpbm5lcnMgaGF2ZSBiZWVuIGRldGVybWluZWQuCgpUaGlzIGlzIG5lZWRlZCB3aGVuIGEgaHVudCBjb25jbHVkZXMgd2l0aCBmZXdlciB3aW5uZXJzIHRoYW4gYW50aWNpcGF0ZWQsCmxlYXZpbmcgdW5zcGVudCBYTE0gbG9ja2VkIGluIHRoZSBwb29sLiBPbmx5IHRoZSBjb250cmFjdCBhZG1pbiBtYXkgY2FsbCB0aGlzLgoKV2l0aGRyYXdhbCBpcyBvbmx5IHBlcm1pdHRlZCBhZnRlciB0aGUgaHVudCBoYXMgZW5kZWQgKGVuZF90aW1lIHBhc3NlZCkgb3IgYmVlbgpjYW5jZWxsZWQuIFRoaXMgcHJldmVudHMgZHJhaW5pbmcgcG9vbHMgd2hpbGUgYSBodW50IGlzIGFjdGl2ZSBhbmQgcGxheWVycyBtYXkKc3RpbGwgYmUgbWlkLWdhbWUuIFdoZW4gSHVudHlDb3JlIGlzIGNvbmZpZ3VyZWQsIHRoZSBodW50IHN0YXR1cyBpcyB2ZXJpZmllZC4KCiMgQXJndW1lbnRzCiogYGFkbWluYCAtIFRoZSBjb250cmFjdCBhZG1pbiBhZGRyZXNzIChtdXN0IG1hdGNoIHRoZSBzdG9yZWQgYWRtaW4pCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcmVtYWluaW5nIHBvb2wgYmFsYW5jZSB0byB3aXRoZHJhdwoqIGByZWNpcGllbnRgIC0gVGhlIGFkZHJlc3MgdGhhdCB3aWxsIHJlY2VpdmUgdGhlIHdpdGhkcmF3biBYTE0KKiBgYW1vdW50YCAtIFRoZSBhbW91bnQgdG8gd2l0aGRyYXcuIE11c3QgYmUgcG9zaXRpdmUgKD4gMCkuCgojIEVycm9ycwoqIGBOb3RJbml0aWFsaXplZGAgLSBDb250cmFjdCBoYXMgbm90IGJlZW4gaW5pdGlhbGl6ZWQgKG5vIGFkbWluIHNldCkKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGNvbnRyYWN0IGFkbWluCiogYFBvb2xOb3RGb3VuZGAgLSBObyBwb29sIGV4aXN0cyBmb3IgdGhpcyBodW50X2lkCiogYEludmFsaWRBbW91bnRgIC0gQW1vdW50IGlzIDw9IDAsAAAAGGFkbWluX3dpdGhkcmF3X3VuY2xhaW1lZAAAAAQAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAlyZWNpcGllbnQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAABABMZWdhY3kgZW50cnkgcG9pbnQgZm9yIFhMTS1vbmx5IGRpc3RyaWJ1dGlvbi4KS2VwdCBmb3IgYmFja3dhcmQgY29tcGF0aWJpbGl0eSB3aXRoIEh1bnR5Q29yZS4gRm9yIE5GVCBvciBmdWxsIGNvbmZpZyBzdXBwb3J0IHVzZSBkaXN0cmlidXRlX3Jld2FyZHMuCgpOb3RlOiBgbmZ0X2VuYWJsZWRgIGlzIGlnbm9yZWQg4oCUIE5GVCBkaXN0cmlidXRpb24gcmVxdWlyZXMgbWV0YWRhdGEgYW5kIGEgY29udHJhY3QgYWRkcmVzcwp0aGF0IGFyZSBub3QgYXZhaWxhYmxlIG9uIHRoaXMgcGF0aC4gVXNlIGBkaXN0cmlidXRlX3Jld2FyZHNgIHdpdGggYFJld2FyZENvbmZpZ2AgaW5zdGVhZC4KKipERVBSRUNBVEVEOiBEbyBub3QgdXNlIGZvciBuZXcgaW50ZWdyYXRpb25zLioqCgpUaGlzIGxlZ2FjeSBkaXN0cmlidXRpb24gcGF0aCBpcyBtYWludGFpbmVkIG9ubHkgZm9yIGJhY2t3YXJkIGNvbXBhdGliaWxpdHkuCkFsbCBuZXcgaW50ZWdyYXRpb25zIG11c3QgdXNlIGBkaXN0cmlidXRlX3Jld2FyZHNgIGluc3RlYWQuCgpUaGlzIGZ1bmN0aW9uIHdyYXBzIGBkaXN0cmlidXRlX3Jld2FyZHNgIGFuZCB0aGVyZWZvcmUgaW5oZXJpdHMgYWxsIHRoZSBzYW1lCnNlY3VyaXR5IGNvbnN0cmFpbnRzOgotIFJlcGxheXMgYXJlIHJlamVjdGVkIHZpYSB0aGUgc2FtZSBub25jZS1iYXNlZCBtZWNoYW5pc20KLSBUaGUgUmVlbnRyYW5jeUd1YXJkIGlzIGFjcXVpcmVkIGlkZW50aWNhbGx5Ci0gYG1pbl9kaXN0cmlidXRpb25fYW1vdW50YCBhbmQgZGFpbHkgY2FwcyBhcmUgZW5mb3JjZWQKLSBBdXRob3JpemF0aW9uIGlzIGZhaWwtY2xvc2VkOiB0aGUgaW1tZWRpYXRlIGludm9rZXIgbXVzdCBiZSBhbiBhcHByb3ZlZCBjb250cmFjdCBhbmQgdGhlIGFsbG93bGlzdCBtdXN0IG5vdCBiZSBlbXB0eQoKKipSZW1vdmFsIHRpbWVsaW5lOioqIFRoaXMgZnVuY3Rpb24gaXMgc2NoZWR1bGVkIGZvciByZW1vdmFsIGluIGEgZnV0dXJlIG1ham9yIHJlbGVhc2UuClRoZSBleGFjdCBkZXByZWNhdGlvbiB0aW1lbGluZSB3aWxsAAAAGWRpc3RyaWJ1dGVfcmV3YXJkc19sZWdhY3kAAAAAAAAEAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAKeGxtX2Ftb3VudAAAAAAACwAAAAAAAAALbmZ0X2VuYWJsZWQAAAAAAQAAAAEAAAAB",
        "AAAAAAAAAEtSZXR1cm5zIHRoZSB0b3RhbCBYTE0gZGlzdHJpYnV0ZWQgYWNyb3NzIGFsbCBodW50cyAocHJvdG9jb2wtbGV2ZWwgbWV0cmljKS4AAAAAGWdldF90b3RhbF94bG1fZGlzdHJpYnV0ZWQAAAAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAAxlNYW51YWxseSByZXNvbHZlcyBhIGRpc3RyaWJ1dGlvbiB0aGF0IGZhaWxlZCBtaWQtZXhlY3V0aW9uLgoKQWxsb3dzIHRoZSBjb250cmFjdCBhZG1pbiB0byBtYXJrIGEgZGlzdHJpYnV0aW9uIGFzIGVpdGhlciBgQ29tcGxldGVkYApvciBgUmVmdW5kZWRgIHdoZW4gdGhlIGF1dG9tYXRpYyBkaXN0cmlidXRpb24gcHJvY2VzcyBjb3VsZCBub3QgZmluaXNoCihlLmcuLCBYTE0gd2FzIHNlbnQgYnV0IE5GVCBtaW50IGZhaWxlZCkuIFRoaXMgaXMgYSBib29ra2VlcGluZy1vbmx5Cm9wZXJhdGlvbiBhbmQgZG9lcyBub3QgbW92ZSBmdW5kcy4KCiMgQXJndW1lbnRzCiogYGFkbWluYCAtIFRoZSBjb250cmFjdCBhZG1pbiBhZGRyZXNzIChtdXN0IG1hdGNoIHRoZSBzdG9yZWQgYWRtaW4pCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgZGlzdHJpYnV0aW9uIHRvIHJlc29sdmUKKiBgcGxheWVyYCAtIFRoZSBwbGF5ZXIgd2hvc2UgZGlzdHJpYnV0aW9uIHRvIHJlc29sdmUKKiBgcmVzb2x1dGlvbmAgLSBPdXRjb21lOiBgUmVzb2x1dGlvblN0YXR1czo6Q29tcGxldGVkYCBvciBgUmVzb2x1dGlvblN0YXR1czo6UmVmdW5kZWRgCgojIEVycm9ycwoqIGBOb3RJbml0aWFsaXplZGAgLSBDb250cmFjdCBoYXMgbm90IGJlZW4gaW5pdGlhbGl6ZWQgKG5vIGFkbWluIHNldCkKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGNvbnRyYWN0IGFkbWluCiogYERpc3RyaWJ1dGlvbk5vdEZvdW5kYCAtIE5vIGRpc3RyaWJ1dGlvbiByZWNvcmQgZXhpc3RzIGZvciB0aGlzIGh1bnQvcGxheWVyAAAAAAAAGmFkbWluX3Jlc29sdmVfZGlzdHJpYnV0aW9uAAAAAAAEAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAApyZXNvbHV0aW9uAAAAAAfQAAAAEFJlc29sdXRpb25TdGF0dXMAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAA39SZXR1cm5zIGRpc3RyaWJ1dGlvbiBhbmFseXRpY3MgKGF2ZXJhZ2UsIG1lZGlhbiwgbWluLCBtYXgpIGFjcm9zcyBhIHJld2FyZCBwb29sLgoKU3VwcG9ydHMgb3B0aW9uYWwgdGltZS1yYW5nZSBmaWx0ZXJpbmcgdmlhIGBzdGFydF90aW1lYCBhbmQgYGVuZF90aW1lYAoobGVkZ2VyIHRpbWVzdGFtcHMpLiBPbmx5IGRpc3RyaWJ1dGlvbnMgd2l0aGluIGBbc3RhcnRfdGltZSwgZW5kX3RpbWUpYAphcmUgaW5jbHVkZWQgd2hlbiBib3RoIGJvdW5kcyBhcmUgcHJvdmlkZWQ7IGBOb25lYCBtZWFucyB1bmJvdW5kZWQuCgpUaGUgY29tcHV0YXRpb24gaXMgZ2FzLWJvdW5kZWQ6IGF0IG1vc3QgW2BNQVhfQU5BTFlUSUNTX0VOVFJJRVNgXSAoNTAwKQpkaXN0cmlidXRpb25zIGFyZSBwcm9jZXNzZWQuIElmIHRoZSBwb29sIGhhcyBtb3JlIGVudHJpZXMgdGhhbiB0aGlzIGxpbWl0LApvbmx5IHRoZSBtb3N0IHJlY2VudCBlbnRyaWVzICh1cCB0byB0aGUgbGltaXQpIGFyZSBhbmFseXNlZC4KCiMgQXJndW1lbnRzCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCBhbmFseXRpY3MgdG8gcXVlcnkKKiBgc3RhcnRfdGltZWAgLSBPcHRpb25hbCBsb3dlciBib3VuZCAoaW5jbHVzaXZlKSBsZWRnZXIgdGltZXN0YW1wIGZpbHRlcgoqIGBlbmRfdGltZWAgLSBPcHRpb25hbCB1cHBlciBib3VuZCAoZXhjbHVzaXZlKSBsZWRnZXIgdGltZXN0YW1wIGZpbHRlcgoKIyBSZXR1cm5zCkEgYERpc3RyaWJ1dGlvbkFuYWx5dGljc2Agc3RydWN0IHdpdGggY291bnQsIHRvdGFsLCBhdmVyYWdlLCBtZWRpYW4sIG1pbiwgbWF4LgpBbGwgZmllbGRzIGFyZSB6ZXJvIHdoZW4gdGhlIHBvb2wgaGFzIG5vIGRpc3RyaWJ1dGlvbnMgb3Igbm8gZW50cmllcyBtYXRjaAp0aGUgdGltZSBmaWx0ZXIuAAAAABpnZXRfZGlzdHJpYnV0aW9uX2FuYWx5dGljcwAAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAApzdGFydF90aW1lAAAAAAPoAAAABgAAAAAAAAAIZW5kX3RpbWUAAAPoAAAABgAAAAEAAAfQAAAAFURpc3RyaWJ1dGlvbkFuYWx5dGljcwAAAA==",
        "AAAAAAAAAFtSZW1vdmVzIGEgY29udHJhY3QgZnJvbSB0aGUgYXV0aG9yaXplZCBjYWxsZXJzIGxpc3QuCk9ubHkgdGhlIGNvbnRyYWN0IGFkbWluIGNhbiBjYWxsIHRoaXMuAAAAABpyZW1vdmVfYXV0aG9yaXplZF9jb250cmFjdAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAhjb250cmFjdAAAABMAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAABABDcmVhdGVzIGEgcmV3YXJkIHBvb2wgZm9yIGEgc3BlY2lmaWMgaHVudCB3aXRoIGEgc3BlY2lmaWVkIHRva2VuLgoKTXVzdCBiZSBjYWxsZWQgYmVmb3JlIGBmdW5kX3Jld2FyZF9wb29sYC4gQW55IGFkZHJlc3MgbWF5IGZ1bmQgdGhlIHBvb2wKYWZ0ZXIgY3JlYXRpb24gKHNlZSBgZnVuZF9yZXdhcmRfcG9vbGApOyB0aGUgdG9rZW4gY29udHJhY3QgbXVzdCBiZQpTQUMtY29tcGF0aWJsZS4KCkZvciBORlQtb25seSBwb29scyAocG9vbHMgdGhhdCBkaXN0cmlidXRlIG9ubHkgTkZUcyB3aXRob3V0IGFueSB0b2tlbiBjb21wb25lbnQpLApzZXQgYG1pbl9kaXN0cmlidXRpb25fYW1vdW50YCB0byAwIGFuZCBwcm92aWRlIGFuIGBuZnRfY29udHJhY3RgIGFkZHJlc3MuCgojIEFyZ3VtZW50cwoqIGBjcmVhdG9yYCAtIFRoZSBodW50IGNyZWF0b3Igd2hvIHdpbGwgb3duIGFuZCBmdW5kIHRoZSBwb29sCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgdGhpcyBwb29sIGlzIGZvcgoqIGB0b2tlbl9hZGRyZXNzYCAtIEFkZHJlc3Mgb2YgdGhlIFNBQy1jb21wYXRpYmxlIHRva2VuIGNvbnRyYWN0IChlLmcuLCBYTE0sIFVTREMpCiogYG1pbl9kaXN0cmlidXRpb25fYW1vdW50YCAtIE1pbmltdW0gdG9rZW4gYW1vdW50IHBlciBkaXN0cmlidXRpb24gKDAgZm9yIE5GVC1vbmx5IHBvb2xzKQoqIGBuZnRfY29udHJhY3RgIC0gT3B0aW9uYWwgTkZUIGNvbnRyYWN0IGFkZHJlc3MgZm9yIE5GVCByZXdhcmRzCiogYG5mdF9yb3lhbHR5X2Jwc2AgLSBDcmVhdG9yIHJveWFsdHkgYmFzaXMgcG9pbnRzICgwLTEwMDAwKSBmb3Igc2Vjb25kYXJ5IG1hcmtldCBzYWxlcwoqIGBuZnRfdHJhbnNmZXJhYmxlYCAtIFdoZXRoZXIgcmV3YXJkIE5GVHMgZnJvbSB0aGlzIHBvb2wgYXJlIHRyYW5zZmVyYWJsZQoKIyBFcnJvcnMKKiBgUG9vbEFscmVhZHlFeGlzdHNgIC0gQSBwb29sIGFscmVhZHkgZXhpc3RzIGZvciB0aGlzIGh1bnRfaWQKKiBgSW52YWxpZEFtb3VudGAgLSBtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudCBpcyBuZWdhdGl2AAAAG2NyZWF0ZV9yZXdhcmRfcG9vbF93aXRoX25mdAAAAAAHAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAAF21pbl9kaXN0cmlidXRpb25fYW1vdW50AAAAAAsAAAAAAAAADG5mdF9jb250cmFjdAAAA+gAAAATAAAAAAAAAA9uZnRfcm95YWx0eV9icHMAAAAABAAAAAAAAAAQbmZ0X3RyYW5zZmVyYWJsZQAAAAEAAAABAAAD6QAAAAIAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAEVCYXRjaCBkaXN0cmlidXRpb24gZW50cnlwb2ludCBmb3IgYW4gZXhwbGljaXRseSBhdXRoZW50aWNhdGVkIGNhbGxlci4AAAAAAAAbZGlzdHJpYnV0ZV9iYXRjaF9hdXRob3JpemVkAAAAAAIAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAANZGlzdHJpYnV0aW9ucwAAAAAAA+oAAAfQAAAAFkJhdGNoRGlzdHJpYnV0aW9uRW50cnkAAAAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAAIBSZXR1cm5zIHRoZSBtaW5pbXVtIGRpc3RyaWJ1dGlvbiBhbW91bnQgY29uZmlndXJlZCBmb3IgYSBodW50J3MgcmV3YXJkIHBvb2wuClJldHVybnMgMCBpZiBubyBwb29sIGhhcyBiZWVuIGNyZWF0ZWQgZm9yIHRoZSBodW50LgAAABtnZXRfbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnQAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAAAs=",
        "AAAAAAAAANBSZXR1cm5zIHRoZSB0b3RhbCBjb3VudCBvZiBkaXN0cmlidXRpb25zIG1hZGUgZnJvbSBhIHNwZWNpZmljIHJld2FyZCBwb29sLgoKIyBBcmd1bWVudHMKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB3aG9zZSBwb29sIGRpc3RyaWJ1dGlvbiBjb3VudCB0byBxdWVyeQoKIyBSZXR1cm5zClRoZSB0b3RhbCBudW1iZXIgb2YgZGlzdHJpYnV0aW9ucyBmb3IgdGhlIHBvb2wuAAAAG2dldF9wb29sX2Rpc3RyaWJ1dGlvbl9jb3VudAAAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAABg==",
        "AAAAAAAAAIxSZXR1cm5zIGhvdyBtdWNoIGBmdW5kZXJgIGhhcyBjb250cmlidXRlZCB0byBhIHBvb2wgdGhhdCBoYXMgbm90IHlldApiZWVuIHJlZnVuZGVkLiAwIGlmIHRoZXkgaGF2ZSBuZXZlciBmdW5kZWQgaXQgb3Igd2VyZSBhbHJlYWR5IHJlZnVuZGVkLgAAABxnZXRfcG9vbF9mdW5kZXJfY29udHJpYnV0aW9uAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZmdW5kZXIAAAAAABMAAAABAAAACw==",
        "AAAAAAAAAEhEaXN0cmlidXRpb24gZW50cnlwb2ludCBmb3IgYW4gZXhwbGljaXRseSBhdXRoZW50aWNhdGVkIGNhbGxlciBjb250cmFjdC4AAAAdZGlzdHJpYnV0ZV9yZXdhcmRzX2F1dGhvcml6ZWQAAAAAAAAEAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAOcGxheWVyX2FkZHJlc3MAAAAAABMAAAAAAAAADXJld2FyZF9jb25maWcAAAAAAAfQAAAADFJld2FyZENvbmZpZwAAAAEAAAPpAAAAAgAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAAEdTZXRzIHRoZSBtaW5pbXVtIHNlY29uZHMgYmV0d2VlbiBkaXN0cmlidXRpb25zIGZvciBhIHBvb2wgKDAgZGlzYWJsZXMpLgAAAAAdc2V0X21pbl9kaXN0cmlidXRpb25faW50ZXJ2YWwAAAAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAebWluX2Rpc3RyaWJ1dGlvbl9pbnRlcnZhbF9zZWNzAAAAAAAGAAAAAQAAA+kAAAACAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAANpSZXR1cm5zIHRydWUgaWYgdGhlIGdpdmVuIE5mdFJld2FyZCBjb250cmFjdCBtZWV0cyB0aGUgbWluaW11bSByZXF1aXJlZCB2ZXJzaW9uLgpSZXR1cm5zIGZhbHNlIG9uIGFueSBlcnJvciAoZS5nLiB0aGUgYWRkcmVzcyBpcyBub3QgYW4gbmZ0LXJld2FyZCBjb250cmFjdApvciBhbiBvbGQgb25lIHdpdGhvdXQgYGNvbnRyYWN0X3ZlcnNpb25gKSBpbnN0ZWFkIG9mIHRyYXBwaW5nLgAAAAAAHmNoZWNrX25mdF9yZXdhcmRfY29tcGF0aWJpbGl0eQAAAAAAAQAAAAAAAAASbmZ0X3Jld2FyZF9hZGRyZXNzAAAAAAATAAAAAQAAAAE=",
        "AAAABAAAAAAAAAAAAAAAD1Jld2FyZEVycm9yQ29kZQAAAAApAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAH0QAAAAAAAAAQSW5zdWZmaWNpZW50UG9vbAAAB9IAAAAAAAAAEkFscmVhZHlEaXN0cmlidXRlZAAAAAAH0wAAAAAAAAAOVHJhbnNmZXJGYWlsZWQAAAAAB9QAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAAfVAAAAAAAAAA1JbnZhbGlkQ29uZmlnAAAAAAAH1gAAAAAAAAANTmZ0TWludEZhaWxlZAAAAAAAB9cAAABAQXR0ZW1wdGVkIHRvIGNyZWF0ZSBhIHBvb2wgdGhhdCBhbHJlYWR5IGV4aXN0cyBmb3IgdGhpcyBodW50X2lkLgAAABFQb29sQWxyZWFkeUV4aXN0cwAAAAAAB9gAAAA3UG9vbCBoYXMgbm90IGJlZW4gY3JlYXRlZCB5ZXQgdmlhIGNyZWF0ZV9yZXdhcmRfcG9vbCgpLgAAAAAMUG9vbE5vdEZvdW5kAAAH2QAAAEdDYWxsZXIgaXMgbm90IHRoZSBwb29sIGNyZWF0b3IgYW5kIGlzIG5vdCBhdXRob3JpemVkIHRvIGZ1bmQgdGhpcyBwb29sLgAAAAAMVW5hdXRob3JpemVkAAAH2gAAAEdEaXN0cmlidXRpb24gYW1vdW50IGlzIGJlbG93IHRoZSBwb29sJ3MgbWluaW11bSBkaXN0cmlidXRpb24gdGhyZXNob2xkLgAAAAASQmVsb3dNaW5pbXVtQW1vdW50AAAAAAfbAAAALUNvbnRyYWN0IGluaXRpYWxpemF0aW9uIGNhbiBvbmx5IGhhcHBlbiBvbmNlLgAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAB9wAAABIaHVudF9pZCBkb2VzIG5vdCBleGlzdCBpbiBIdW50eUNvcmUgKHZhbGlkYXRlZCB2aWEgY3Jvc3MtY29udHJhY3QgY2FsbCkuAAAADEh1bnROb3RGb3VuZAAAB90AAABRQSByZWN1cnNpdmUgZGlzdHJpYnV0aW9uIGF0dGVtcHQgd2FzIGRldGVjdGVkIGR1cmluZyBhbiBleHRlcm5hbCBYTE0gb3IgTkZUIGNhbGwuAAAAAAAAElJlZW50cmFuY3lEZXRlY3RlZAAAAAAH3gAAAERUaGUgdHJhY2tlZCBwb29sIGJhbGFuY2UgZGl2ZXJnZWQgZnJvbSB0aGUgYWN0dWFsIFhMTSB0b2tlbiBiYWxhbmNlLgAAABVQb29sQmFsYW5jZURpdmVyZ2VuY2UAAAAAAAffAAAAP1JlcGxheSBhdHRhY2sgZGV0ZWN0ZWQ6IGRpc3RyaWJ1dGlvbiBub25jZSBzdGF0ZSBpbmNvbnNpc3RlbmN5LgAAAAAOUmVwbGF5RGV0ZWN0ZWQAAAAAB+AAAAAwUG9vbCBiYWxhbmNlIHdvdWxkIGV4Y2VlZCBtYXhpbXVtIGFsbG93ZWQgbGltaXQuAAAAE1Bvb2xCYWxhbmNlT3ZlcmZsb3cAAAAH4QAAAEdGdW5kaW5nIGFtb3VudCBpcyBiZWxvdyB0aGUgbWluaW11bSB0aHJlc2hvbGQgKGR1c3QgYXR0YWNrIHByZXZlbnRpb24pLgAAAAATQmVsb3dNaW5pbXVtRnVuZGluZwAAAAfiAAAAMlNpbmdsZSBmdW5kaW5nIGFtb3VudCBleGNlZWRzIHRoZSBtYXhpbXVtIGFsbG93ZWQuAAAAAAAVRXhjZWVkc01heGltdW1GdW5kaW5nAAAAAAAH4wAAAD1EYWlseSBkaXN0cmlidXRpb24gY2FwIGZvciBhIHNwZWNpZmljIHBvb2wgaGFzIGJlZW4gZXhjZWVkZWQuAAAAAAAAEERhaWx5Q2FwRXhjZWVkZWQAAAfkAAAAQUdsb2JhbCBkYWlseSBkaXN0cmlidXRpb24gY2FwIGFjcm9zcyBhbGwgcG9vbHMgaGFzIGJlZW4gZXhjZWVkZWQuAAAAAAAAFkdsb2JhbERhaWx5Q2FwRXhjZWVkZWQAAAAAB+UAAAA1Q29udHJhY3QgaXMgcGF1c2VkIGFuZCBjYW5ub3QgcGVyZm9ybSB0aGlzIG9wZXJhdGlvbi4AAAAAAAAOQ29udHJhY3RQYXVzZWQAAAAAB+YAAAArTm8gcGVuZGluZyBmYWlsZWQgTkZUIG1pbnQgZm91bmQgZm9yIHJldHJ5LgAAAAAWTmZ0TWludFBlbmRpbmdOb3RGb3VuZAAAAAAH5wAAADhObyBkaXN0cmlidXRpb24gcmVjb3JkIGV4aXN0cyBmb3IgdGhlIGdpdmVuIGh1bnQvcGxheWVyLgAAABREaXN0cmlidXRpb25Ob3RGb3VuZAAAB+gAAABCVGhlIHNvdXJjZSBwb29sIGlzIG5vdCBlbGlnaWJsZSBmb3IgbWlncmF0aW9uOiBpdHMgaHVudCBpcyBuZWl0aGVyAAAAAAAVU291cmNlUG9vbE5vdEVsaWdpYmxlAAAAAAAH6QAAADxUaGUgZGVzdGluYXRpb24gcG9vbCBkb2VzIG5vdCBleGlzdCAobXVzdCBiZSBjcmVhdGVkIGZpcnN0KS4AAAAXRGVzdGluYXRpb25Qb29sTm90Rm91bmQAAAAH6gAAAEVTb3VyY2UgYW5kIGRlc3RpbmF0aW9uIHJlZmVyIHRvIHRoZSBzYW1lIGh1bnQsIG9yIHRoZXJlIGlzIG5vIGJhbGFuY2UAAAAAAAAQSW52YWxpZE1pZ3JhdGlvbgAAB+sAAABAUG9vbCBpcyBmcm96ZW4gYW5kIGRpc3RyaWJ1dGlvbnMgaGF2ZSBiZWVuIHRlbXBvcmFyaWx5IGRpc2FibGVkLgAAAApQb29sRnJvemVuAAAAAAfsAAAAQURpc3RyaWJ1dGlvbiByYXRlIGxpbWl0IG5vdCB5ZXQgZWxhcHNlZCAoY29vbGRvd24gcGVyaW9kIGFjdGl2ZSkuAAAAAAAAF0Rpc3RyaWJ1dGlvblJhdGVMaW1pdGVkAAAAB+0AAAApQmF0Y2ggc2l6ZSBleGNlZWRzIG1heGltdW0gYWxsb3dlZCBsaW1pdC4AAAAAAAANQmF0Y2hUb29MYXJnZQAAAAAAB+4AAAAdSW52YWxpZCBzY29yZSB2YWx1ZSBwcm92aWRlZC4AAAAAAAAMSW52YWxpZFNjb3JlAAAH7wAAACFUb2tlbiBjb250cmFjdCB2YWxpZGF0aW9uIGZhaWxlZC4AAAAAAAAUSW52YWxpZFRva2VuQ29udHJhY3QAAAfwAAAAOE5vIHZlc3RpbmcgcmVjb3JkIGV4aXN0cyBmb3IgdGhlIGdpdmVuIGh1bnQvcGxheWVyIHBhaXIuAAAAEVZlc3RpbmdOb3RTdGFydGVkAAAAAAAH8QAAADJQbGF5ZXIgaGFzIGFscmVhZHkgY2xhaW1lZCB0aGUgZnVsbCB2ZXN0ZWQgYW1vdW50LgAAAAAAFVZlc3RpbmdBbHJlYWR5Q2xhaW1lZAAAAAAAB/IAAABGTm90aGluZyBoYXMgdmVzdGVkIHlldCAoZWxhcHNlZCB0aW1lIGlzIHplcm8gb3IgdmVzdGluZyBqdXN0IHN0YXJ0ZWQpLgAAAAAADU5vdGhpbmdUb1Zlc3QAAAAAAAfzAAAARVRoZSBwb29sIGRvZXMgbm90IGhhdmUgdmVzdGluZyBjb25maWd1cmVkICh2ZXN0aW5nX3BlcmlvZF9zZWNzID09IDApLgAAAAAAABRWZXN0aW5nTm90Q29uZmlndXJlZAAAB/QAAABHUG9vbCBmdW5kaW5nIGlzIHBhdXNlZCAoaXNzdWUgIzYyOCkuIERpc3RyaWJ1dGlvbiBtYXkgc3RpbGwgYmUgcnVubmluZy4AAAAADUZ1bmRpbmdQYXVzZWQAAAAAAAf1AAAARlJld2FyZCBkaXN0cmlidXRpb24gaXMgcGF1c2VkIChpc3N1ZSAjNjI4KS4gRnVuZGluZyBtYXkgc3RpbGwgYmUgb3Blbi4AAAAAABJEaXN0cmlidXRpb25QYXVzZWQAAAAAB/YAAABEVGhlIHBvb2wgYWxyZWFkeSBoYXMgdGhlIG1heGltdW0gbnVtYmVyIG9mIGRpc3RpbmN0IHRyYWNrZWQgZnVuZGVyczsAAAAOVG9vTWFueUZ1bmRlcnMAAAAAB/cAAABFVGhlIGh1bnQgaXMgbm90IGluIGEgdGVybWluYWwgc3RhdGUgKGNhbmNlbGxlZCBvciBlbmRlZCksIHNvIGl0cyBwb29sAAAAAAAAEUludmFsaWRIdW50U3RhdHVzAAAAAAAH+AAAAEFUaGUgaHVudCBpcyBpbiBhIHRlcm1pbmFsIHN0YXRlIChjYW5jZWxsZWQgb3IgZW5kZWQpLCBzbyBpdHMgcG9vbAAAAAAAAAxIdW50VGVybWluYWwAAAf5",
        "AAAAAQAAAAAAAAAAAAAADkNvbnRyYWN0SGVhbHRoAAAAAAAFAAAAAAAAAA1hY3RpdmVfYWxlcnRzAAAAAAAABAAAAAAAAAANYXZnX2dhc191bml0cwAAAAAAAAYAAAAAAAAAEmZhaWxlZF9pbnZvY2F0aW9ucwAAAAAABgAAAAAAAAAQZmFpbHVyZV9yYXRlX2JwcwAAAAQAAAAAAAAAEXRvdGFsX2ludm9jYXRpb25zAAAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAD1VwZ3JhZGVQcm9wb3NhbAAAAAAEAAAAAAAAAAxlZmZlY3RpdmVfYXQAAAAGAAAAAAAAAAtwcm9wb3NlZF9hdAAAAAAGAAAAAAAAAAhwcm9wb3NlcgAAABMAAAAAAAAADnRhcmdldF92ZXJzaW9uAAAAAAAE",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVBdXRoRXJyb3IAAAAFAAAAAAAAAAxVbmF1dGhvcml6ZWQAAAABAAAAAAAAAApOb1Byb3Bvc2FsAAAAAAACAAAAAAAAAA9UaW1lbG9ja1BlbmRpbmcAAAAAAwAAAAAAAAAPVmVyc2lvbk1pc21hdGNoAAAAAAQAAAAAAAAAD0ludmFsaWRUaW1lbG9jawAAAAAF",
        "AAAAAQAAAAAAAAAAAAAAE1VwZ3JhZGVIaXN0b3J5RW50cnkAAAAABAAAAAAAAAALZXhlY3V0ZWRfYXQAAAAABgAAAAAAAAAIZXhlY3V0b3IAAAATAAAAAAAAAAxmcm9tX3ZlcnNpb24AAAAEAAAAAAAAAAp0b192ZXJzaW9uAAAAAAAE",
        "AAAAAQAAAFdDb25maWd1cmF0aW9uIGZvciBkaXN0cmlidXRpbmcgcmV3YXJkcyBhY3Jvc3MgdGhlIEh1bnR5Q29yZSDihpQgUmV3YXJkTWFuYWdlciBib3VuZGFyeS4AAAAAAAAAAAxSZXdhcmRDb25maWcAAAAJAAAAjlRoZSBwbGF5ZXIncyBmaW5pc2hpbmcgcG9zaXRpb24gZm9yIHRoaXMgaHVudCwgc2V0IGJ5IGh1bnR5LWNvcmUgYXQKY29tcGxldGlvbiB0aW1lIGFuZCB0aHJlYWRlZCBoZXJlIHNvIHRoZSBORlQgY2FuIHJlY29yZCBhbiBpbW11dGFibGUgcmFuay4AAAAAAA9jb21wbGV0aW9uX3JhbmsAAAAABAAAAAAAAAAMbmZ0X2NvbnRyYWN0AAAD6AAAABMAAAAAAAAAD25mdF9kZXNjcmlwdGlvbgAAAAAQAAAAAAAAAA5uZnRfaHVudF90aXRsZQAAAAAAEAAAAAAAAAANbmZ0X2ltYWdlX3VyaQAAAAAAABAAAAAAAAAACm5mdF9yYXJpdHkAAAAAAAQAAAAAAAAACG5mdF90aWVyAAAABAAAAAAAAAAJbmZ0X3RpdGxlAAAAAAAAEAAAAAAAAAAKeGxtX2Ftb3VudAAAAAAD6AAAAAs=",
        "AAAAAQAAAUlPbmUgZXhhY3QgY29tcGxldGlvbi1yYW5rIHJld2FyZCB0aWVyLgoKUmFua3MgYXJlIG9uZS1iYXNlZDogcmFuayBgMWAgaXMgdGhlIGZpcnN0IGZpbmlzaGVyLiBBIHJhbmsgdGllciBhcHBsaWVzCm9ubHkgdG8gdGhlIGV4YWN0IHJhbmsgaXQgbmFtZXM7IHJhbmtzIHdpdGhvdXQgYW4gZW50cnkgdXNlIHRoZSBub3JtYWwKZmxhdC90aW1lLWJhc2VkIHJld2FyZCBwYXRoLiBUaGUgbGlzdCBpcyBrZXB0IGluIHN0cmljdGx5IGluY3JlYXNpbmcgcmFuawpvcmRlciBzbyBjb25maWd1cmF0aW9uIGlzIGRldGVybWluaXN0aWMgYW5kIGNoZWFwIHRvIHZhbGlkYXRlIG9uLWNoYWluLgAAAAAAAAAAAAAOUmFua1Jld2FyZFRpZXIAAAAAAAIAAAAvT25lLWJhc2VkIGNvbXBsZXRpb24gcmFuayBhd2FyZGVkIGJ5IEh1bnR5Q29yZS4AAAAABHJhbmsAAAAEAAAAPVRva2VuIGFtb3VudCBhd2FyZGVkIHRvIHRoZSBwbGF5ZXIgd2hvIGZpbmlzaGVzIGF0IHRoaXMgcmFuay4AAAAAAAAKeGxtX2Ftb3VudAAAAAAACw==",
        "AAAAAQAAAi5PbmUgdGllciBvZiBhIHRpbWUtYmFzZWQgcmV3YXJkIHNjaGVkdWxlIGNvbmZpZ3VyZWQgb24gYSByZXdhcmQgcG9vbC4KCkEgdGllciBkZWZpbmVzIGFuIFhMTSBhbW91bnQgdGhhdCBpcyBncmFudGVkIHRvIGEgcGxheWVyIHdobyBjb21wbGV0ZXMgdGhlCmh1bnQgd2l0aGluIGBtYXhfY29tcGxldGlvbl9zZWNzYCBvZiByZWdpc3RlcmluZy4gVGllcnMgbXVzdCBiZSBzdG9yZWQgaW4KYXNjZW5kaW5nIG9yZGVyIGJ5IGBtYXhfY29tcGxldGlvbl9zZWNzYCDigJQgaS5lLiBhICJmYXN0ZXIiIHRpZXIgbXVzdAphcHBlYXIgYmVmb3JlIGEgInNsb3dlciIgdGllci4gVGhlIGZpcnN0IHRpZXIgZm9yIHdoaWNoCmBtYXhfY29tcGxldGlvbl9zZWNzID49IGVsYXBzZWRgIGlzIHNlbGVjdGVkIGF0IGRpc3RyaWJ1dGlvbiB0aW1lOyBpZiB0aGUKZWxhcHNlZCB0aW1lIGV4Y2VlZHMgZXZlcnkgY29uZmlndXJlZCB0aWVyLCB0aGUgbGFzdCAoc2xvd2VzdCkgdGllcidzCmFtb3VudCBpcyB1c2VkIGFzIGEgZmFsbGJhY2sgc28gdGhlIHBsYXllciBzdGlsbCByZWNlaXZlcyBhIHJld2FyZC4AAAAAAAAAAAATVGltZUJhc2VkUmV3YXJkVGllcgAAAAACAAAAiUluY2x1c2l2ZSB1cHBlciBib3VuZCBvbiBlbGFwc2VkIHRpbWUgKGNvbXBsZXRpb25fdGltZSAtIHJlZ2lzdHJhdGlvbl90aW1lKQppbiBzZWNvbmRzLiBNdXN0IGJlIHN0cmljdGx5IGluY3JlYXNpbmcgYWNyb3NzIHRoZSB0aWVyIGxpc3QuAAAAAAAAE21heF9jb21wbGV0aW9uX3NlY3MAAAAABgAAADtYTE0gYW1vdW50IGF3YXJkZWQgdG8gYSBwbGF5ZXIgd2hvIHF1YWxpZmllcyBmb3IgdGhpcyB0aWVyLgAAAAAKeGxtX2Ftb3VudAAAAAAACw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    pause: this.txFromJSON<Result<void>>,
        unpause: this.txFromJSON<Result<void>>,
        is_paused: this.txFromJSON<boolean>,
        initialize: this.txFromJSON<Result<void>>,
        freeze_pool: this.txFromJSON<Result<void>>,
        refund_pool: this.txFromJSON<Result<void>>,
        accept_admin: this.txFromJSON<Result<void>>,
        add_delegate: this.txFromJSON<Result<void>>,
        claim_vested: this.txFromJSON<Result<i128>>,
        migrate_pool: this.txFromJSON<Result<i128>>,
        pause_funding: this.txFromJSON<Result<void>>,
        run_migration: this.txFromJSON<Result<MigrationReport>>,
        unfreeze_pool: this.txFromJSON<Result<void>>,
        validate_pool: this.txFromJSON<ValidationResult>,
        is_pool_frozen: this.txFromJSON<boolean>,
        set_hunty_core: this.txFromJSON<Result<void>>,
        set_pool_tiers: this.txFromJSON<Result<void>>,
        get_pause_state: this.txFromJSON<readonly [boolean, boolean, boolean]>,
        get_pool_config: this.txFromJSON<Option<RewardPoolConfig>>,
        get_reward_pool: this.txFromJSON<Option<RewardPoolStatus>>,
        propose_upgrade: this.txFromJSON<Result<UpgradeProposal>>,
        remove_delegate: this.txFromJSON<Result<void>>,
        unpause_funding: this.txFromJSON<Result<void>>,
        contract_version: this.txFromJSON<u32>,
        distribute_batch: this.txFromJSON<Result<void>>,
        fund_reward_pool: this.txFromJSON<Result<void>>,
        get_pool_balance: this.txFromJSON<i128>,
        get_pool_funders: this.txFromJSON<Array<string>>,
        get_dist_cooldown: this.txFromJSON<u64>,
        initialize_schema: this.txFromJSON<null>,
        propose_new_admin: this.txFromJSON<Result<void>>,
        admin_withdraw_all: this.txFromJSON<Result<void>>,
        create_reward_pool: this.txFromJSON<Result<void>>,
        distribute_rewards: this.txFromJSON<Result<void>>,
        emergency_withdraw: this.txFromJSON<Result<i128>>,
        get_emergency_logs: this.txFromJSON<Array<EmergencyWithdrawalLogEntry>>,
        get_pool_audit_log: this.txFromJSON<PoolAuditLogResponse>,
        get_schema_version: this.txFromJSON<u32>,
        get_vesting_status: this.txFromJSON<Option<VestingStatus>>,
        pause_distribution: this.txFromJSON<Result<void>>,
        rollback_migration: this.txFromJSON<Result<MigrationReport>>,
        set_daily_pool_cap: this.txFromJSON<Result<void>>,
        update_pool_config: this.txFromJSON<Result<void>>,
        get_pool_statistics: this.txFromJSON<Option<RewardPoolStatistics>>,
        get_raw_pause_flags: this.txFromJSON<readonly [boolean, boolean]>,
        get_upgrade_history: this.txFromJSON<Array<UpgradeHistoryEntry>>,
        set_pool_rank_tiers: this.txFromJSON<Result<void>>,
        verify_distribution: this.txFromJSON<boolean>,
        get_health_dashboard: this.txFromJSON<ContractHealth>,
        get_upgrade_proposal: this.txFromJSON<Option<UpgradeProposal>>,
        get_upgrade_timelock: this.txFromJSON<u64>,
        set_daily_global_cap: this.txFromJSON<Result<void>>,
        set_upgrade_timelock: this.txFromJSON<Result<void>>,
        unpause_distribution: this.txFromJSON<Result<void>>,
        is_reward_distributed: this.txFromJSON<boolean>,
        retry_failed_nft_mint: this.txFromJSON<Result<u64>>,
        set_distribution_mode: this.txFromJSON<Result<void>>,
        set_pool_nft_contract: this.txFromJSON<Result<void>>,
        get_distribution_proof: this.txFromJSON<Option<DistributionProof>>,
        get_pool_distributions: this.txFromJSON<Array<PoolDistribution>>,
        list_pending_nft_mints: this.txFromJSON<Array<PendingNftMint>>,
        set_pool_target_amount: this.txFromJSON<Result<void>>,
        add_authorized_contract: this.txFromJSON<Result<void>>,
        distribute_proportional: this.txFromJSON<Result<i128>>,
        get_distribution_status: this.txFromJSON<DistributionStatus>,
        set_nft_reward_contract: this.txFromJSON<Result<void>>,
        set_vesting_period_secs: this.txFromJSON<Result<void>>,
        admin_withdraw_unclaimed: this.txFromJSON<Result<void>>,
        distribute_rewards_legacy: this.txFromJSON<boolean>,
        get_total_xlm_distributed: this.txFromJSON<i128>,
        admin_resolve_distribution: this.txFromJSON<Result<void>>,
        get_distribution_analytics: this.txFromJSON<DistributionAnalytics>,
        remove_authorized_contract: this.txFromJSON<Result<void>>,
        create_reward_pool_with_nft: this.txFromJSON<Result<void>>,
        distribute_batch_authorized: this.txFromJSON<Result<void>>,
        get_min_distribution_amount: this.txFromJSON<i128>,
        get_pool_distribution_count: this.txFromJSON<u64>,
        get_pool_funder_contribution: this.txFromJSON<i128>,
        distribute_rewards_authorized: this.txFromJSON<Result<void>>,
        set_min_distribution_interval: this.txFromJSON<Result<void>>,
        check_nft_reward_compatibility: this.txFromJSON<boolean>
  }
}
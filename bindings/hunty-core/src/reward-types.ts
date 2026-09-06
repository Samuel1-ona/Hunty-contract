import type {
  u32,
  u64,
  i128,
  Option,
} from "@stellar/stellar-sdk/contract";

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


export interface UpgradeHistoryEntry {
  executed_at: u64;
  executor: string;
  from_version: u32;
  to_version: u32;
}


export interface UpgradeExecutedEvent {
  executed_at: u64;
  executor: string;
  from_version: u32;
  to_version: u32;
}


export interface UpgradeProposedEvent {
  effective_at: u64;
  proposed_at: u64;
  proposer: string;
  target_version: u32;
}


/**
 * Reason a tier or tier list failed validation.
 */
export type TierError = {tag: "NonPositiveAmount", values: void} | {tag: "NotStrictlyAscending", values: void} | {tag: "Empty", values: void};


/**
 * Configuration for distributing rewards across the HuntyCore ↔ RewardManager boundary.
 * @alias CrossContractRewardConfig — distinct from the hunt-embedded RewardConfig in types.ts
 */
export interface CrossContractRewardConfig {
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
 * How rewards are calculated from the pool at distribution time.
 */
export enum DistributionMode {
  Fixed = 0,
  Proportional = 1,
}


/**
 * Mirror of the RewardManager's per-hunt pool configuration. Callers such as
 * HuntyCore use this to deserialize `get_pool_config` cross-contract results.
 * Field names and order must stay in sync with the RewardManager's struct so
 * the XDR encodings match.
 */
export interface RewardPoolConfig {
  /**
 * Unix timestamp after which claims are no longer allowed (0 = disabled).
 */
claim_deadline: u64;
  /**
 * Address of the hunt creator who owns this pool.
 */
creator: string;
  /**
 * Addresses allowed to distribute rewards for this pool.
 */
delegates: Array<string>;
  /**
 * Distribution mode (Fixed or Proportional).
 */
distribution_mode: DistributionMode;
  /**
 * Whether distributions from this pool are temporarily frozen.
 */
frozen: boolean;
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
 * Target funding amount for progress tracking (0 = disabled).
 */
target_amount: i128;
  /**
 * Optional time-based reward tiers. When empty, the per-winner amount
 * is computed from `xlm_pool / max_winners`.
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

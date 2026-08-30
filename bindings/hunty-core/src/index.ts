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





export interface HealthAlert {
  alert_type: string;
  count: u32;
  last_ledger: u64;
}


export interface ContractHealth {
  active_alerts: u32;
  avg_gas_units: u64;
  failed_invocations: u64;
  failure_rate_bps: u32;
  total_invocations: u64;
}


/**
 * Stored clue with SHA256 answer hash. The hash is never exposed via get_clue/list_clues or events.
 */
export interface Clue {
  answer_hashes: Array<Buffer>;
  clue_id: u32;
  difficulty: u32;
  hint: Option<string>;
  hint_penalty_points: u32;
  is_required: boolean;
  points: u32;
  question: string;
  weight: u32;
}


export interface Hunt {
  activated_at: u64;
  /**
 * When true, players may claim their partial score after the hunt ends.
 */
allow_partial_scoring: boolean;
  /**
 * Minimum seconds a player must wait between attempts on the same clue.
 */
attempt_cooldown_secs: u32;
  categories: Array<string>;
  completed_count: u32;
  created_at: u64;
  creator: string;
  /**
 * Default point value applied to clues with 0 points. Clue-level points override this.
 */
default_points: u32;
  description: string;
  difficulty_override: Option<u32>;
  difficulty_rating: u32;
  end_time: u64;
  hunt_id: u64;
  /**
 * SHA256 hash (salted with hunt_id) of the invite code, if configured.
 */
invite_code_hash: Option<Buffer>;
  /**
 * When true, only players with a valid invite code may register.
 */
is_private: boolean;
  max_attempts_per_clue: u32;
  /**
 * Maximum number of players allowed to register. 0 = unlimited.
 */
max_players: u32;
  max_submissions_per_minute: u32;
  /**
 * Registration cutoff timestamp. 0 = no deadline (registration open while active).
 */
registration_deadline: u64;
  /**
 * Dynamically recalculated on every `get_hunt` read; not meaningful when read from a raw struct literal.
 */
remaining_slots: u32;
  required_clues: u32;
  reward_config: RewardConfig;
  start_multiplier_bps: u32;
  start_time: u64;
  status: HuntStatus;
  /**
 * When true, players may form teams and share clue progress.
 */
team_mode: boolean;
  time_bonus_decay_secs: Option<u64>;
  time_bonus_min_bps: Option<u32>;
  time_bonus_start_bps: Option<u32>;
  title: string;
  total_clues: u32;
}


/**
 * A team competing in a team-mode hunt.
 */
export interface Team {
  hunt_id: u64;
  leader: string;
  members: Array<string>;
  name: string;
  team_id: u32;
}


/**
 * Clue info returned by get_clue/list_clues. Excludes answer hash.
 */
export interface ClueInfo {
  clue_id: u32;
  difficulty: u32;
  hint_available: boolean;
  hint_penalty_points: u32;
  is_required: boolean;
  points: u32;
  question: string;
  weight: u32;
}


export interface Location {
  latitude: i64;
  longitude: i64;
  radius: u32;
}


export interface HuntCache {
  creator: string;
  end_time: u64;
  hunt_id: u64;
  max_winners: u32;
  required_clues: u32;
  start_time: u64;
  status: HuntStatus;
  total_clues: u32;
}

export type HuntStatus = {tag: "Draft", values: void} | {tag: "Active", values: void} | {tag: "Completed", values: void} | {tag: "Cancelled", values: void} | {tag: "Paused", values: void} | {tag: "EmergencyStopped", values: void} | {tag: "Archived", values: void};


export interface RewardConfig {
  claimed_count: u32;
  max_winners: u32;
  nft_contract: Option<string>;
  nft_enabled: boolean;
  nft_rarity: u32;
  nft_tier: u32;
  xlm_pool: i128;
}


/**
 * Shared progress for a team: clues completed by any member and the combined score.
 */
export interface TeamProgress {
  completed_clues: Array<u32>;
  total_score: u32;
}


/**
 * Input payload for adding multiple clues in one contract invocation.
 */
export interface BatchClueInput {
  answer: string;
  /**
 * Difficulty tier (1-5, 1 = easiest, 5 = hardest). Points earned = points * difficulty.
 */
difficulty: u32;
  is_required: boolean;
  points: u32;
  question: string;
}


/**
 * Emitted when a clue is added. Does not expose the answer hash.
 */
export interface ClueAddedEvent {
  clue_id: u32;
  creator: string;
  /**
 * Difficulty tier (1-5, 1 = easiest, 5 = hardest).
 */
difficulty: u32;
  hunt_id: u64;
  is_required: boolean;
  points: u32;
  question: string;
  /**
 * Weight multiplier (default 1).
 */
weight: u32;
}


/**
 * Aggregate statistics for a hunt (read-only query result).
 */
export interface HuntStatistics {
  average_score: u32;
  completed_count: u32;
  completion_rate_percent: u32;
  total_players: u32;
  total_score_sum: u64;
}


export interface LeaderboardRow {
  completed_at: u64;
  index: u32;
  is_completed: boolean;
  player: string;
  score: u32;
}


/**
 * Public view of player progress, with `player` and `hunt_id` reconstructed from the key.
 */
export interface PlayerProgress {
  clue_last_attempts: Map<u32, u64>;
  completed_at: u64;
  completed_clue_index: Map<u32, boolean>;
  completed_clues: Array<u32>;
  hinted_clues: Array<u32>;
  hunt_id: u64;
  is_completed: boolean;
  player: string;
  recent_submissions: Array<u64>;
  required_completed_count: u32;
  reward_claimed: boolean;
  started_at: u64;
  total_score: u32;
}


/**
 * Emitted when a hunt is cloned.
 */
export interface HuntClonedEvent {
  creator: string;
  new_hunt_id: u64;
  original_hunt_id: u64;
}


/**
 * Emitted when a creator force-closes a hunt early (marks it Completed) while
 * preserving player scores and any already-distributed rewards. `rewarded_players`
 * is the number of completed players who received a final reward as part of closing.
 */
export interface HuntClosedEvent {
  closed_at: u64;
  hunt_id: u64;
  rewarded_players: u32;
}


export interface RateLimitStatus {
  cooldown_seconds: u64;
  creations_today: u32;
  daily_limit: u32;
}


export interface TimeBonusConfig {
  decay_duration_secs: u64;
  min_multiplier_bps: u32;
  start_multiplier_bps: u32;
}


export interface HuntCreatedEvent {
  creator: string;
  hunt_id: u64;
}


/**
 * Leaderboard entry for a single player in a hunt (read-only query result).
 */
export interface LeaderboardEntry {
  completed_at: u64;
  is_completed: boolean;
  player: string;
  rank: u32;
  score: u32;
}


export interface TeamCreatedEvent {
  hunt_id: u64;
  leader: string;
  name: string;
  team_id: u32;
}


export interface HuntArchivedEvent {
  hunt_id: u64;
}


/**
 * Wrapper returned by `get_hunt_leaderboard` that includes truncation
 * information so callers can tell when the visible entries are incomplete.
 */
export interface LeaderboardResult {
  entries: Array<LeaderboardEntry>;
  total_players: u32;
  truncated: boolean;
}


export interface LeaderboardWindow {
  entries: Array<LeaderboardRow>;
  finished: boolean;
  next_index: u32;
  queried_at: u64;
}


export interface PlayerBannedEvent {
  hunt_id: u64;
  player: string;
}


export interface ClueCompletedEvent {
  clue_id: u32;
  hunt_id: u64;
  player: string;
  points_earned: u32;
}


export interface HuntActivatedEvent {
  activated_at: u64;
  hunt_id: u64;
}


export interface HuntCancelledEvent {
  hunt_id: u64;
}


export interface HuntCompletedEvent {
  completion_rank: u32;
  completion_time: u64;
  hunt_id: u64;
  player: string;
  total_score: u32;
}


export interface RewardClaimedEvent {
  hunt_id: u64;
  nft_awarded: boolean;
  player: string;
  xlm_amount: i128;
}


export interface PlayerUnbannedEvent {
  hunt_id: u64;
  player: string;
}


export interface AnswerIncorrectEvent {
  clue_id: u32;
  hunt_id: u64;
  player: string;
  timestamp: u64;
}


export interface HuntDeactivatedEvent {
  hunt_id: u64;
}


export interface HuntReactivatedEvent {
  activated_at: u64;
  hunt_id: u64;
}


/**
 * Internal compact storage representation of player progress.
 * Does not store `player` or `hunt_id` — those are already the storage key.
 * 
 * ## Compact encoding
 * - Timestamps are delta-encoded as `u32` offsets from the hunt's `activated_at`,
 * saving 4 bytes each vs full `u64` UNIX timestamps. The max delta (~136 years)
 * far exceeds any realistic hunt duration.
 * - Boolean fields (`is_completed`, `reward_claimed`) are packed into `flags`.
 */
export interface StoredPlayerProgress {
  clue_last_attempts: Map<u32, u64>;
  /**
 * Seconds elapsed from player registration to hunt completion, or 0 if not completed.
 * Reconstruct absolute: `activated_at + started_at_delta + completed_at_delta`.
 */
completed_at_delta: u32;
  completed_clues: Array<u32>;
  /**
 * Bit flags for boolean fields to reduce storage footprint.
 * BIT0 (1): is_completed
 * BIT1 (2): reward_claimed
 * BIT2–BIT31: reserved for future use
 */
flags: u32;
  hinted_clues: Array<u32>;
  recent_submissions: Array<u64>;
  required_completed_count: u32;
  /**
 * Seconds elapsed from hunt `activated_at` to player registration.
 * Reconstruct absolute: `activated_at + started_at_delta`.
 */
started_at_delta: u32;
  total_score: u32;
}


/**
 * Team leaderboard entry (read-only query result), ranked by shared team score.
 */
export interface TeamLeaderboardEntry {
  member_count: u32;
  name: string;
  rank: u32;
  score: u32;
  team_id: u32;
}


export interface ClueAliasesAddedEvent {
  aliases_count: u32;
  clue_id: u32;
  creator: string;
  hunt_id: u64;
}


/**
 * Stored top-N leaderboard entry maintained incrementally on score changes.
 */
export interface LeaderboardIndexEntry {
  completed_at: u64;
  is_completed: boolean;
  player: string;
  score: u32;
}


/**
 * Emitted when a player registers for an active hunt.
 */
export interface PlayerRegisteredEvent {
  hunt_id: u64;
  player: string;
}


export interface RewardManagerSetEvent {
  new_address: string;
  old_address: Option<string>;
}


export interface TeamMemberJoinedEvent {
  hunt_id: u64;
  player: string;
  team_id: u32;
}


export interface HuntStatusChangedEvent {
  changed_at: u64;
  hunt_id: u64;
  new_status: HuntStatus;
  old_status: HuntStatus;
}


/**
 * Emitted when a hunt creator clears the invite code, pausing new registrations.
 */
export interface InviteCodeRevokedEvent {
  creator: string;
  hunt_id: u64;
}


export interface RewardClaimFailedEvent {
  error_code: u32;
  hunt_id: u64;
  player: string;
}


export interface CreatorBlacklistedEvent {
  admin: string;
  creator: string;
}


/**
 * Emitted when a hunt creator generates or updates the invite code for a private hunt.
 * The invite code itself is never emitted or stored — only its hash.
 */
export interface InviteCodeGeneratedEvent {
  creator: string;
  hunt_id: u64;
}


export interface PartialScoreClaimedEvent {
  clues_completed: u32;
  hunt_id: u64;
  partial_score: u32;
  player: string;
}


export interface HuntDescriptionUpdatedEvent {
  creator: string;
  description: string;
  hunt_id: u64;
}


export interface RegistrationDeadlineSetEvent {
  hunt_id: u64;
  registration_deadline: u64;
}


/**
 * Emitted when a player successfully registers using an invite code.
 */
export interface PlayerRegisteredWithInviteEvent {
  hunt_id: u64;
  player: string;
}


export interface CreatorRemovedFromBlacklistEvent {
  admin: string;
  creator: string;
}

export const HuntErrorCode = {
  1001: {message:"HuntNotFound"},
  1002: {message:"ClueNotFound"},
  1003: {message:"InvalidHuntStatus"},
  1004: {message:"PlayerNotRegistered"},
  1005: {message:"ClueAlreadyCompleted"},
  1006: {message:"InvalidAnswer"},
  1007: {message:"HuntNotActive"},
  1008: {message:"Unauthorized"},
  1009: {message:"InsufficientRewardPool"},
  1010: {message:"DuplicateRegistration"},
  1011: {message:"InvalidTitle"},
  1012: {message:"InvalidDescription"},
  1013: {message:"InvalidAddress"},
  1014: {message:"TooManyClues"},
  1015: {message:"InvalidQuestion"},
  1016: {message:"RefundFailed"},
  1017: {message:"NoCluesAdded"},
  1018: {message:"HuntNotCompleted"},
  1019: {message:"RewardAlreadyClaimed"},
  /**
   * A RewardManager cross-contract call failed. The originating contract's
   * error code (range 2001–2999) is published in the `reward_distribution_failed`
   * diagnostic event emitted immediately before this error is returned.
   */
  1020: {message:"RewardDistributionFailed"},
  1021: {message:"NoRewardsConfigured"},
  1022: {message:"DuplicateSubmission"},
  1023: {message:"SubmissionExpired"},
  1024: {message:"BannedPlayer"},
  1025: {message:"NoRequiredClues"},
  1026: {message:"RateLimitExceeded"},
  1027: {message:"ScoreOverflow"},
  1028: {message:"RegistrationsPaused"},
  1029: {message:"AnswersPaused"},
  1030: {message:"RewardsPaused"},
  1031: {message:"HuntEndTimeInPast"},
  1032: {message:"NoPendingAdmin"},
  1033: {message:"PendingAdminMismatch"},
  1034: {message:"InvalidRarity"},
  1035: {message:"InvalidTimeBonusConfig"},
  1036: {message:"AddressBlacklisted"},
  1037: {message:"ContractPaused"},
  1038: {message:"InvalidMaxAttempts"},
  1039: {message:"InvalidWeight"},
  1040: {message:"HintNotAvailable"},
  1041: {message:"HintAlreadyUnlocked"},
  1042: {message:"InsufficientScore"},
  1043: {message:"TooManyCategories"},
  1044: {message:"InvalidCategory"},
  1045: {message:"InvalidDifficulty"},
  1046: {message:"CorruptPlayerProgress"},
  1047: {message:"HuntNotStarted"},
  1048: {message:"AdminAlreadyProposed"},
  1049: {message:"InvalidPoints"},
  1050: {message:"HuntFull"}
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
 */
export interface RewardConfig {
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

export const RewardErrorCode = {
  2001: {message:"NotInitialized"},
  2002: {message:"InsufficientPool"},
  2003: {message:"AlreadyDistributed"},
  2004: {message:"TransferFailed"},
  2005: {message:"InvalidAmount"},
  2006: {message:"InvalidConfig"},
  2007: {message:"NftMintFailed"},
  2008: {message:"PoolAlreadyExists"},
  2009: {message:"PoolNotFound"},
  2010: {message:"Unauthorized"},
  2011: {message:"BelowMinimumAmount"},
  2012: {message:"AlreadyInitialized"},
  2013: {message:"HuntNotFound"},
  2014: {message:"ReentrancyDetected"},
  2015: {message:"PoolBalanceDivergence"},
  2016: {message:"ReplayDetected"},
  2017: {message:"PoolBalanceOverflow"},
  2018: {message:"BelowMinimumFunding"},
  2019: {message:"ExceedsMaximumFunding"},
  2020: {message:"DailyCapExceeded"},
  2021: {message:"GlobalDailyCapExceeded"},
  2022: {message:"ContractPaused"},
  2023: {message:"NftMintPendingNotFound"},
  2024: {message:"DistributionNotFound"},
  2025: {message:"SourcePoolNotEligible"},
  2026: {message:"DestinationPoolNotFound"},
  2027: {message:"InvalidMigration"},
  2028: {message:"PoolFrozen"},
  2029: {message:"DistributionRateLimited"},
  2030: {message:"BatchTooLarge"},
  2031: {message:"InvalidScore"},
  2032: {message:"InvalidTokenContract"},
  2033: {message:"VestingNotStarted"},
  2034: {message:"VestingAlreadyClaimed"},
  2035: {message:"NothingToVest"},
  2036: {message:"VestingNotConfigured"},
  2037: {message:"FundingPaused"},
  2038: {message:"DistributionPaused"},
  2039: {message:"TooManyFunders"},
  2040: {message:"InvalidHuntStatus"}
}

export interface Client {
  /**
   * Construct and simulate a add_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds a clue to a hunt. Only the hunt creator can add clues.
   * Answers are hashed with SHA256 before storage; the hash is never exposed.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to add the clue to
   * * `question` - The clue question text (max 2000 chars, non-empty)
   * * `answer` - Plain-text answer; normalized (trimmed, lowercased) then hashed
   * * `points` - Points awarded for solving this clue
   * * `is_required` - Whether this clue must be solved to complete the hunt
   * 
   * # Returns
   * The sequential clue ID assigned within the hunt
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `InvalidHuntStatus` - Hunt is not in Draft
   * * `Unauthorized` - Caller is not the hunt creator
   * * `TooManyClues` - Hunt already has max clues
   * * `InvalidQuestion` - Question empty or too long
   * * `InvalidAnswer` - Answer empty or too long
   */
  add_clue: ({hunt_id, question, answer, points, is_required, difficulty, weight}: {hunt_id: u64, question: string, answer: string, points: u32, is_required: boolean, difficulty: Option<u32>, weight: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a get_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns clue information for a hunt/clue. Does not expose the answer hash.
   */
  get_clue: ({hunt_id, clue_id}: {hunt_id: u64, clue_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<ClueInfo>>>

  /**
   * Construct and simulate a add_clues transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds multiple clues to a draft hunt in one invocation. Only the hunt creator can add clues.
   * 
   * The batch is validated against the per-hunt clue cap before writing any new clues,
   * so a request that would exceed the limit fails without partially adding clues.
   */
  add_clues: ({hunt_id, clues}: {hunt_id: u64, clues: Array<BatchClueInput>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<u32>>>>

  /**
   * Construct and simulate a clone_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a new draft hunt by copying clues from an existing completed hunt.
   * 
   * The template hunt must already be completed. The copied hunt starts as a fresh
   * draft with a new hunt ID, creator, title, and description, but reuses the
   * template's clue questions, hashes, points, and required flags.
   * Clones an existing hunt into a new draft.
   * The caller must be the original hunt creator.
   * All clues are duplicated with new clue IDs.
   * Returns the new hunt ID.
   */
  clone_hunt: ({template_hunt_id, caller}: {template_hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a close_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Force-closes (ends early) an in-progress hunt on behalf of its creator.
   * 
   * Unlike [`cancel_hunt`], closing preserves all player scores and any
   * rewards already collected: it marks the hunt `Completed` and triggers a
   * final reward distribution for every player who has completed the hunt but
   * not yet claimed. Players who have not completed the hunt keep their
   * progress and are simply not rewarded. Any unspent reward-pool balance is
   * left intact (a creator can refund it separately via [`cancel_hunt`] flows
   * only while a hunt is still cancellable — see project docs).
   * 
   * Only the creator may close a hunt, and only while it is `Active` or
   * `Paused`. Closing a `Draft`, `Completed`, `Cancelled`, `EmergencyStopped`,
   * or `Archived` hunt is rejected with `InvalidHuntStatus`.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to close
   * * `caller` - The creator (must authorize the call via require_auth)
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `Unauthorized` - Caller is not
   */
  close_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a list_clues transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns paginated clues for a hunt. Answer hashes are not exposed.
   */
  list_clues: ({hunt_id, offset, limit}: {hunt_id: u64, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<ClueInfo>>>

  /**
   * Construct and simulate a list_hunts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a list of all hunts (paginated).
   */
  list_hunts: ({offset, limit}: {offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<Hunt>>>

  /**
   * Construct and simulate a cancel_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a create_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a new scavenger hunt with the provided metadata.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `creator` - The address of the hunt creator (typically use env.invoker() from the caller)
   * * `title` - The title of the hunt (max 200 characters)
   * * `description` - The description of the hunt (max 2000 characters)
   * * `start_time` - Optional start timestamp. When set, players cannot register
   * or submit answers until the ledger timestamp reaches this value. 0 means
   * no start time restriction (immediately playable once activated).
   * * `end_time` - Optional end timestamp (0 means no end time restriction)
   * 
   * # Returns
   * The unique hunt ID of the newly created hunt
   * 
   * # Errors
   * * `InvalidTitle` - If title is empty or exceeds maximum length
   * * `InvalidDescription` - If description exceeds maximum length
   * * `InvalidAddress` - If creator address is invalid
   */
  create_hunt: ({creator, title, description, start_time, end_time, max_submissions_per_minute, start_multiplier_bps, default_points}: {creator: string, title: string, description: string, start_time: Option<u64>, end_time: Option<u64>, max_submissions_per_minute: u32, start_multiplier_bps: Option<u32>, default_points: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a set_max_attempts_per_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates the maximum number of attempts allowed per clue and attempt cooldown duration for a draft hunt. Only the hunt creator or co-creator can update it.
   */
  set_max_attempts_per_clue: ({hunt_id, caller, max_attempts_per_clue, attempt_cooldown_secs}: {hunt_id: u64, caller: string, max_attempts_per_clue: u32, attempt_cooldown_secs: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Step two of a two-step admin key rotation.
   * 
   * The proposed new admin accepts the role, completing the rotation. Only the
   * address stored by `propose_new_admin` may accept, so a wrong proposal cannot
   * silently take over the contract.
   */
  accept_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a archive_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  archive_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_view_only transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_view_only: ({hunt_id, address}: {hunt_id: u64, address: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a request_hint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Unlocks a clue hint for a registered player and deducts the clue's hint penalty.
   */
  request_hint: ({hunt_id, clue_id, player}: {hunt_id: u64, clue_id: u32, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a search_hunts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Searches hunts by partial title match over a caller-bounded hunt-id window.
   */
  search_hunts: ({title_substring, offset, limit, scan_limit}: {title_substring: string, offset: u32, limit: u32, scan_limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<Hunt>>>

  /**
   * Construct and simulate a activate_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  activate_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a complete_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Completes a hunt for a player and distributes rewards.
   * 
   * This function verifies that the player has completed all required clues,
   * then distributes rewards via the RewardManager contract (if configured)
   * and updates the player's reward status.
   * 
   * Reward amounts can be either flat (`xlm_pool / max_winners`) or
   * time-based (configured via `RewardManager::set_pool_tiers`), in which
   * case the amount depends on `completion_at - started_at` for the
   * completing player.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt ID
   * * `player` - The player claiming completion/rewards
   * 
   * # Returns
   * `Ok(())` on successful reward claim
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `InvalidHuntStatus` - Hunt is not Active (e.g. already Completed or Cancelled)
   * * `PlayerNotRegistered` - Player is not registered
   * * `HuntNotCompleted` - Player hasn't completed all required clues
   * * `RewardAlreadyClaimed` - Player already claimed their reward
   * * `NoRewardsConfigured` - No rewards set up for this hunt
   * * `InsufficientRewardPool
   */
  complete_hunt: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunt_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_hunt_info: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Hunt>>>

  /**
   * Construct and simulate a pause_answers transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pause_answers: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a pause_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pause_rewards: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a run_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  run_migration: ({admin, target_version, dry_run}: {admin: string, target_version: u32, dry_run: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MigrationReport>>>

  /**
   * Construct and simulate a set_clue_hint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets or clears the optional hint for a draft clue.
   */
  set_clue_hint: ({hunt_id, clue_id, caller, hint, hint_penalty_points}: {hunt_id: u64, clue_id: u32, caller: string, hint: Option<string>, hint_penalty_points: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a submit_answer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  submit_answer: ({hunt_id, clue_id, player, answer, submission_nonce, submitted_at}: {hunt_id: u64, clue_id: u32, player: string, answer: string, submission_nonce: u64, submitted_at: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_co_creator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_co_creator: ({hunt_id, creator, new_co_creator}: {hunt_id: u64, creator: string, new_co_creator: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunt_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total number of hunts created (read-only).
   */
  get_hunt_count: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a is_blacklisted transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns true if the given address is blacklisted.
   */
  is_blacklisted: ({creator}: {creator: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a pause_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Pauses all player operations (registrations, answers, rewards) globally.
   */
  pause_contract: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a preview_answer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Verifies a candidate answer for a registered player with authorization and rate limiting.
   */
  preview_answer: ({hunt_id, clue_id, player, answer}: {hunt_id: u64, clue_id: u32, player: string, answer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

  /**
   * Construct and simulate a deactivate_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deactivate_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_co_creators transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_co_creators: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_pause_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pause_state: (options?: MethodOptions) => Promise<AssembledTransaction<readonly [boolean, boolean, boolean]>>

  /**
   * Construct and simulate a register_player transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Registers a player for an active hunt. The caller must pass their address and authorize;
   * only that identity can register themselves. Initializes player progress and prevents
   * duplicate registrations. Registration is only allowed while the hunt is active and
   * (if set) before end_time.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to register for
   * * `player` - The address of the player (must authorize the call via require_auth)
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `InvalidHuntStatus` - Hunt is not in Active status
   * * `HuntNotActive` - Hunt has ended (past end_time)
   * * `DuplicateRegistration` - Player is already registered for this hunt
   */
  register_player: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_max_players transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the maximum players for a hunt. Only the hunt creator can set it, and only in Draft status.
   */
  set_max_players: ({hunt_id, caller, max_players}: {hunt_id: u64, caller: string, max_players: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause_answers transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause_answers: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause_rewards: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_clue_aliases transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds alternative acceptable answers to an existing clue (synonyms).
   * Only the hunt creator can add aliases, and only while the hunt is in Draft status.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt containing the clue
   * * `clue_id` - The existing clue to add aliases to
   * * `answers` - Alternative answers that should also be accepted
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `InvalidHuntStatus` - Hunt is not in Draft
   * * `Unauthorized` - Caller is not the hunt creator
   * * `ClueNotFound` - Clue does not exist
   * * `InvalidAnswer` - Any answer is empty or exceeds max length
   */
  add_clue_aliases: ({hunt_id, clue_id, answers}: {hunt_id: u64, clue_id: u32, answers: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the contract admin once. Subsequent calls require current admin auth via set_admin.
   */
  initialize_admin: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_hunt_privacy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets whether a hunt is private (invite-only).
   * 
   * Only the hunt creator can call this, and only while the hunt is in Draft status.
   * When making a hunt private, an invite code must already be configured via
   * `generate_invite_code` before the hunt can be activated.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to update privacy for
   * * `creator` - The hunt creator (must authorize the call)
   * * `is_private` - Whether the hunt should be invite-only
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `Unauthorized` - Caller is not the hunt creator
   * * `InvalidHuntStatus` - Hunt is not in Draft status
   */
  set_hunt_privacy: ({hunt_id, creator, is_private}: {hunt_id: u64, creator: string, is_private: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Resumes all player operations.
   */
  unpause_contract: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a blacklist_creator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Blacklists a creator address, preventing them from creating new hunts.
   * Caller must be the admin.
   */
  blacklist_creator: ({admin, creator}: {admin: string, creator: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunt_end_time transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Exposes the end time of a hunt.
   */
  get_hunt_end_time: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_schema: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a propose_new_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Step one of a two-step admin key rotation.
   * 
   * The current admin proposes a new admin. The change is NOT applied until the
   * proposed address calls `accept_admin`, which prevents accidental lockout: a
   * typo in `propose_new_admin` can simply be overwritten or ignored, and the
   * current admin never loses access until the new admin actively accepts.
   */
  propose_new_admin: ({admin, new_admin}: {admin: string, new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_co_creator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_co_creator: ({hunt_id, creator, co_creator_to_remove}: {hunt_id: u64, creator: string, co_creator_to_remove: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_view_only_list transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_view_only_list: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a is_contract_paused transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether the global contract pause is active.
   */
  is_contract_paused: (options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a revoke_invite_code transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Clears the invite code for a private hunt, effectively pausing new registrations.
   * The hunt creator can generate a new code later via `generate_invite_code`.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to revoke the invite code for
   * * `creator` - The hunt creator (must authorize the call)
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `Unauthorized` - Caller is not the hunt creator
   * * `InvalidHuntStatus` - Hunt is not in Draft status
   */
  revoke_invite_code: ({hunt_id, creator}: {hunt_id: u64, creator: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a rollback_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rollback_migration: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MigrationReport>>>

  /**
   * Construct and simulate a set_reward_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the RewardManager contract address for cross-contract reward distribution.
   */
  set_reward_manager: ({admin, reward_manager}: {admin: string, reward_manager: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_completed_clues transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the list of clue IDs that the player has completed for a hunt (read-only).
   * Useful for UI to show progress. Returns empty vec if player is not registered.
   */
  get_completed_clues: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u32>>>

  /**
   * Construct and simulate a get_hunt_statistics transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns aggregate statistics for a hunt (read-only): total players, completion rate, average score.
   * Returns error if hunt does not exist.
   */
  get_hunt_statistics: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<HuntStatistics>>>

  /**
   * Construct and simulate a get_player_progress transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns player progress for a hunt (read-only).
   * Includes completed clues, score, and completion status.
   * Returns error if player is not registered.
   */
  get_player_progress: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<PlayerProgress>>>

  /**
   * Construct and simulate a is_global_view_only transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_global_view_only: ({address}: {address: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a pause_registrations transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pause_registrations: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_hunt_categories transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates categories for a draft hunt. At most five categories are allowed.
   */
  set_hunt_categories: ({hunt_id, caller, categories}: {hunt_id: u64, caller: string, categories: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_global_view_only transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_global_view_only: ({admin, viewer}: {admin: string, viewer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_view_only_access transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_view_only_access: ({hunt_id, creator, viewer}: {hunt_id: u64, creator: string, viewer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a generate_invite_code transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Generates or updates the invite code for a private hunt.
   * 
   * The invite code is hashed with SHA256 (using hunt_id as salt) and only the hash
   * is stored on-chain. The plain-text code is never persisted or emitted in events.
   * Calling this function overwrites any previously set invite code.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to generate an invite code for
   * * `creator` - The hunt creator (must authorize the call)
   * * `invite_code` - The plain-text invite code to hash and store
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `Unauthorized` - Caller is not the hunt creator
   * * `InvalidHuntStatus` - Hunt is not in Draft status
   */
  generate_invite_code: ({hunt_id, creator, invite_code}: {hunt_id: u64, creator: string, invite_code: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_health_dashboard transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_health_dashboard: (options?: MethodOptions) => Promise<AssembledTransaction<ContractHealth>>

  /**
   * Construct and simulate a get_hunt_leaderboard transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns ranked players for a hunt with pagination support (read-only).
   * Sorted by score descending, then by completion time ascending (earlier = better).
   * Limit is capped at 20 to control gas. Returns error if hunt does not exist.
   */
  get_hunt_leaderboard: ({hunt_id, limit}: {hunt_id: u64, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<LeaderboardResult>>>

  /**
   * Construct and simulate a list_clues_paginated transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a paginated slice of clues for a hunt. Useful for large hunts to bound gas.
   * Page is 0-indexed. Max page_size is capped at MAX_BATCH_SIZE (50).
   * Estimated gas: O(page_size) ~5_000 gas per clue + 10_000 overhead.
   */
  list_clues_paginated: ({hunt_id, page, page_size}: {hunt_id: u64, page: u32, page_size: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<ClueInfo>>>

  /**
   * Construct and simulate a register_with_invite transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Registers a player for a private hunt using a valid invite code.
   * 
   * The provided invite code is hashed (with hunt_id as salt) and compared against
   * the stored `invite_code_hash`. If they match, the player is registered.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The private hunt to register for
   * * `player` - The address of the player (must authorize the call via require_auth)
   * * `invite_code` - The plain-text invite code to validate
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `InvalidHuntStatus` - Hunt is not in Active status, is not private (use
   * `register_player` instead), or has no invite code configured
   * * `InvalidAnswer` - The provided invite code is empty or does not match
   * * `DuplicateRegistration` - Player is already registered for this hunt
   */
  register_with_invite: ({hunt_id, player, invite_code}: {hunt_id: u64, player: string, invite_code: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunts_by_category transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns hunts whose categories include the exact category string.
   */
  get_hunts_by_category: ({category, offset, limit, scan_limit}: {category: string, offset: u32, limit: u32, scan_limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<Hunt>>>

  /**
   * Construct and simulate a remove_from_blacklist transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes a creator from the blacklist, restoring their ability to create hunts.
   * Caller must be the admin.
   */
  remove_from_blacklist: ({admin, creator}: {admin: string, creator: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_time_bonus_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_time_bonus_config: ({hunt_id, caller, time_bonus_config}: {hunt_id: u64, caller: string, time_bonus_config: Option<TimeBonusConfig>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause_registrations transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause_registrations: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_global_view_only transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_global_view_only: ({admin, viewer}: {admin: string, viewer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_view_only_access transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_view_only_access: ({hunt_id, creator, viewer}: {hunt_id: u64, creator: string, viewer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a submit_answer_with_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Variant of `submit_answer` which accepts a precomputed SHA256 answer hash.
   * This avoids on-chain normalization and hashing when the client supplies
   * the correctly computed `answer_hash = SHA256(hunt_id || clue_id || normalized_answer)`.
   * Use this from off-chain callers that can perform normalization+hashing cheaply.
   */
  submit_answer_with_hash: ({hunt_id, clue_id, player, answer_hash, submission_nonce, submitted_at}: {hunt_id: u64, clue_id: u32, player: string, answer_hash: Buffer, submission_nonce: u64, submitted_at: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_hunt_description transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates a hunt's description. Only the hunt creator can update it, and it can be updated for any hunt status.
   */
  update_hunt_description: ({hunt_id, caller, description}: {hunt_id: u64, caller: string, description: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_global_view_only_list transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_global_view_only_list: (options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_hunt_leaderboard_window transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Scans a bounded window of registered players for a hunt and returns
   * their compact rows. This method enables clients to page through all
   * registered players in multiple calls (bounded by `MAX_LEADERBOARD_SCAN_SIZE`)
   * and merge results off-chain to build a full leaderboard without a single
   * large on-chain scan.
   */
  get_hunt_leaderboard_window: ({hunt_id, start_index, window_size}: {hunt_id: u64, start_index: u32, window_size: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<LeaderboardWindow>>>

  /**
   * Construct and simulate a set_hunt_difficulty_override transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets or clears a manual hunt difficulty override. Without an override,
   * the rating is the average clue difficulty.
   */
  set_hunt_difficulty_override: ({hunt_id, caller, difficulty_override}: {hunt_id: u64, caller: string, difficulty_override: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
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
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAz5BZGRzIGEgY2x1ZSB0byBhIGh1bnQuIE9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gYWRkIGNsdWVzLgpBbnN3ZXJzIGFyZSBoYXNoZWQgd2l0aCBTSEEyNTYgYmVmb3JlIHN0b3JhZ2U7IHRoZSBoYXNoIGlzIG5ldmVyIGV4cG9zZWQuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBhZGQgdGhlIGNsdWUgdG8KKiBgcXVlc3Rpb25gIC0gVGhlIGNsdWUgcXVlc3Rpb24gdGV4dCAobWF4IDIwMDAgY2hhcnMsIG5vbi1lbXB0eSkKKiBgYW5zd2VyYCAtIFBsYWluLXRleHQgYW5zd2VyOyBub3JtYWxpemVkICh0cmltbWVkLCBsb3dlcmNhc2VkKSB0aGVuIGhhc2hlZAoqIGBwb2ludHNgIC0gUG9pbnRzIGF3YXJkZWQgZm9yIHNvbHZpbmcgdGhpcyBjbHVlCiogYGlzX3JlcXVpcmVkYCAtIFdoZXRoZXIgdGhpcyBjbHVlIG11c3QgYmUgc29sdmVkIHRvIGNvbXBsZXRlIHRoZSBodW50CgojIFJldHVybnMKVGhlIHNlcXVlbnRpYWwgY2x1ZSBJRCBhc3NpZ25lZCB3aXRoaW4gdGhlIGh1bnQKCiMgRXJyb3JzCiogYEh1bnROb3RGb3VuZGAgLSBIdW50IGRvZXMgbm90IGV4aXN0CiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IGluIERyYWZ0CiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90IHRoZSBodW50IGNyZWF0b3IKKiBgVG9vTWFueUNsdWVzYCAtIEh1bnQgYWxyZWFkeSBoYXMgbWF4IGNsdWVzCiogYEludmFsaWRRdWVzdGlvbmAgLSBRdWVzdGlvbiBlbXB0eSBvciB0b28gbG9uZwoqIGBJbnZhbGlkQW5zd2VyYCAtIEFuc3dlciBlbXB0eSBvciB0b28gbG9uZwAAAAAACGFkZF9jbHVlAAAABwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAhxdWVzdGlvbgAAABAAAAAAAAAABmFuc3dlcgAAAAAAEAAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAtpc19yZXF1aXJlZAAAAAABAAAAAAAAAApkaWZmaWN1bHR5AAAAAAPoAAAABAAAAAAAAAAGd2VpZ2h0AAAAAAPoAAAABAAAAAEAAAPpAAAABAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAEpSZXR1cm5zIGNsdWUgaW5mb3JtYXRpb24gZm9yIGEgaHVudC9jbHVlLiBEb2VzIG5vdCBleHBvc2UgdGhlIGFuc3dlciBoYXNoLgAAAAAACGdldF9jbHVlAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjbHVlX2lkAAAAAAQAAAABAAAD6QAAB9AAAAAIQ2x1ZUluZm8AAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAP5BZGRzIG11bHRpcGxlIGNsdWVzIHRvIGEgZHJhZnQgaHVudCBpbiBvbmUgaW52b2NhdGlvbi4gT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiBhZGQgY2x1ZXMuCgpUaGUgYmF0Y2ggaXMgdmFsaWRhdGVkIGFnYWluc3QgdGhlIHBlci1odW50IGNsdWUgY2FwIGJlZm9yZSB3cml0aW5nIGFueSBuZXcgY2x1ZXMsCnNvIGEgcmVxdWVzdCB0aGF0IHdvdWxkIGV4Y2VlZCB0aGUgbGltaXQgZmFpbHMgd2l0aG91dCBwYXJ0aWFsbHkgYWRkaW5nIGNsdWVzLgAAAAAACWFkZF9jbHVlcwAAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAFY2x1ZXMAAAAAAAPqAAAH0AAAAA5CYXRjaENsdWVJbnB1dAAAAAAAAQAAA+kAAAPqAAAABAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAcBDcmVhdGVzIGEgbmV3IGRyYWZ0IGh1bnQgYnkgY29weWluZyBjbHVlcyBmcm9tIGFuIGV4aXN0aW5nIGNvbXBsZXRlZCBodW50LgoKVGhlIHRlbXBsYXRlIGh1bnQgbXVzdCBhbHJlYWR5IGJlIGNvbXBsZXRlZC4gVGhlIGNvcGllZCBodW50IHN0YXJ0cyBhcyBhIGZyZXNoCmRyYWZ0IHdpdGggYSBuZXcgaHVudCBJRCwgY3JlYXRvciwgdGl0bGUsIGFuZCBkZXNjcmlwdGlvbiwgYnV0IHJldXNlcyB0aGUKdGVtcGxhdGUncyBjbHVlIHF1ZXN0aW9ucywgaGFzaGVzLCBwb2ludHMsIGFuZCByZXF1aXJlZCBmbGFncy4KQ2xvbmVzIGFuIGV4aXN0aW5nIGh1bnQgaW50byBhIG5ldyBkcmFmdC4KVGhlIGNhbGxlciBtdXN0IGJlIHRoZSBvcmlnaW5hbCBodW50IGNyZWF0b3IuCkFsbCBjbHVlcyBhcmUgZHVwbGljYXRlZCB3aXRoIG5ldyBjbHVlIElEcy4KUmV0dXJucyB0aGUgbmV3IGh1bnQgSUQuAAAACmNsb25lX2h1bnQAAAAAAAIAAAAAAAAAEHRlbXBsYXRlX2h1bnRfaWQAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAAAYAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAABABGb3JjZS1jbG9zZXMgKGVuZHMgZWFybHkpIGFuIGluLXByb2dyZXNzIGh1bnQgb24gYmVoYWxmIG9mIGl0cyBjcmVhdG9yLgoKVW5saWtlIFtgY2FuY2VsX2h1bnRgXSwgY2xvc2luZyBwcmVzZXJ2ZXMgYWxsIHBsYXllciBzY29yZXMgYW5kIGFueQpyZXdhcmRzIGFscmVhZHkgY29sbGVjdGVkOiBpdCBtYXJrcyB0aGUgaHVudCBgQ29tcGxldGVkYCBhbmQgdHJpZ2dlcnMgYQpmaW5hbCByZXdhcmQgZGlzdHJpYnV0aW9uIGZvciBldmVyeSBwbGF5ZXIgd2hvIGhhcyBjb21wbGV0ZWQgdGhlIGh1bnQgYnV0Cm5vdCB5ZXQgY2xhaW1lZC4gUGxheWVycyB3aG8gaGF2ZSBub3QgY29tcGxldGVkIHRoZSBodW50IGtlZXAgdGhlaXIKcHJvZ3Jlc3MgYW5kIGFyZSBzaW1wbHkgbm90IHJld2FyZGVkLiBBbnkgdW5zcGVudCByZXdhcmQtcG9vbCBiYWxhbmNlIGlzCmxlZnQgaW50YWN0IChhIGNyZWF0b3IgY2FuIHJlZnVuZCBpdCBzZXBhcmF0ZWx5IHZpYSBbYGNhbmNlbF9odW50YF0gZmxvd3MKb25seSB3aGlsZSBhIGh1bnQgaXMgc3RpbGwgY2FuY2VsbGFibGUg4oCUIHNlZSBwcm9qZWN0IGRvY3MpLgoKT25seSB0aGUgY3JlYXRvciBtYXkgY2xvc2UgYSBodW50LCBhbmQgb25seSB3aGlsZSBpdCBpcyBgQWN0aXZlYCBvcgpgUGF1c2VkYC4gQ2xvc2luZyBhIGBEcmFmdGAsIGBDb21wbGV0ZWRgLCBgQ2FuY2VsbGVkYCwgYEVtZXJnZW5jeVN0b3BwZWRgLApvciBgQXJjaGl2ZWRgIGh1bnQgaXMgcmVqZWN0ZWQgd2l0aCBgSW52YWxpZEh1bnRTdGF0dXNgLgoKIyBBcmd1bWVudHMKKiBgZW52YCAtIFRoZSBTb3JvYmFuIGVudmlyb25tZW50CiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgdG8gY2xvc2UKKiBgY2FsbGVyYCAtIFRoZSBjcmVhdG9yIChtdXN0IGF1dGhvcml6ZSB0aGUgY2FsbCB2aWEgcmVxdWlyZV9hdXRoKQoKIyBSZXR1cm5zCmBPaygoKSlgIG9uIHN1Y2Nlc3MKCiMgRXJyb3JzCiogYEh1bnROb3RGb3VuZGAgLSBIdW50IGRvZXMgbm90IGV4aXN0CiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90AAAACmNsb3NlX2h1bnQAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAEJSZXR1cm5zIHBhZ2luYXRlZCBjbHVlcyBmb3IgYSBodW50LiBBbnN3ZXIgaGFzaGVzIGFyZSBub3QgZXhwb3NlZC4AAAAAAApsaXN0X2NsdWVzAAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+oAAAfQAAAACENsdWVJbmZv",
        "AAAAAAAAAChSZXR1cm5zIGEgbGlzdCBvZiBhbGwgaHVudHMgKHBhZ2luYXRlZCkuAAAACmxpc3RfaHVudHMAAAAAAAIAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+oAAAfQAAAABEh1bnQ=",
        "AAAAAAAAAAAAAAALY2FuY2VsX2h1bnQAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAA1BDcmVhdGVzIGEgbmV3IHNjYXZlbmdlciBodW50IHdpdGggdGhlIHByb3ZpZGVkIG1ldGFkYXRhLgoKIyBBcmd1bWVudHMKKiBgZW52YCAtIFRoZSBTb3JvYmFuIGVudmlyb25tZW50CiogYGNyZWF0b3JgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGh1bnQgY3JlYXRvciAodHlwaWNhbGx5IHVzZSBlbnYuaW52b2tlcigpIGZyb20gdGhlIGNhbGxlcikKKiBgdGl0bGVgIC0gVGhlIHRpdGxlIG9mIHRoZSBodW50IChtYXggMjAwIGNoYXJhY3RlcnMpCiogYGRlc2NyaXB0aW9uYCAtIFRoZSBkZXNjcmlwdGlvbiBvZiB0aGUgaHVudCAobWF4IDIwMDAgY2hhcmFjdGVycykKKiBgc3RhcnRfdGltZWAgLSBPcHRpb25hbCBzdGFydCB0aW1lc3RhbXAuIFdoZW4gc2V0LCBwbGF5ZXJzIGNhbm5vdCByZWdpc3RlcgpvciBzdWJtaXQgYW5zd2VycyB1bnRpbCB0aGUgbGVkZ2VyIHRpbWVzdGFtcCByZWFjaGVzIHRoaXMgdmFsdWUuIDAgbWVhbnMKbm8gc3RhcnQgdGltZSByZXN0cmljdGlvbiAoaW1tZWRpYXRlbHkgcGxheWFibGUgb25jZSBhY3RpdmF0ZWQpLgoqIGBlbmRfdGltZWAgLSBPcHRpb25hbCBlbmQgdGltZXN0YW1wICgwIG1lYW5zIG5vIGVuZCB0aW1lIHJlc3RyaWN0aW9uKQoKIyBSZXR1cm5zClRoZSB1bmlxdWUgaHVudCBJRCBvZiB0aGUgbmV3bHkgY3JlYXRlZCBodW50CgojIEVycm9ycwoqIGBJbnZhbGlkVGl0bGVgIC0gSWYgdGl0bGUgaXMgZW1wdHkgb3IgZXhjZWVkcyBtYXhpbXVtIGxlbmd0aAoqIGBJbnZhbGlkRGVzY3JpcHRpb25gIC0gSWYgZGVzY3JpcHRpb24gZXhjZWVkcyBtYXhpbXVtIGxlbmd0aAoqIGBJbnZhbGlkQWRkcmVzc2AgLSBJZiBjcmVhdG9yIGFkZHJlc3MgaXMgaW52YWxpZAAAAAtjcmVhdGVfaHVudAAAAAAIAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAABXRpdGxlAAAAAAAAEAAAAAAAAAALZGVzY3JpcHRpb24AAAAAEAAAAAAAAAAKc3RhcnRfdGltZQAAAAAD6AAAAAYAAAAAAAAACGVuZF90aW1lAAAD6AAAAAYAAAAAAAAAGm1heF9zdWJtaXNzaW9uc19wZXJfbWludXRlAAAAAAAEAAAAAAAAABRzdGFydF9tdWx0aXBsaWVyX2JwcwAAA+gAAAAEAAAAAAAAAA5kZWZhdWx0X3BvaW50cwAAAAAD6AAAAAQAAAABAAAD6QAAAAYAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAFJVcGRhdGVzIGEgZHJhZnQgaHVudCdzIHRpdGxlIGFuZCBkZXNjcmlwdGlvbi4gT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiB1cGRhdGUgaXQuAAAAAAALdXBkYXRlX2h1bnQAAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAFW1heF9hdHRlbXB0c19wZXJfY2x1ZQAAAAAAAAQAAAAAAAAAFWF0dGVtcHRfY29vbGRvd25fc2VjcwAAAAAAAAQAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAORTdGVwIHR3byBvZiBhIHR3by1zdGVwIGFkbWluIGtleSByb3RhdGlvbi4KClRoZSBwcm9wb3NlZCBuZXcgYWRtaW4gYWNjZXB0cyB0aGUgcm9sZSwgY29tcGxldGluZyB0aGUgcm90YXRpb24uIE9ubHkgdGhlCmFkZHJlc3Mgc3RvcmVkIGJ5IGBwcm9wb3NlX25ld19hZG1pbmAgbWF5IGFjY2VwdCwgc28gYSB3cm9uZyBwcm9wb3NhbCBjYW5ub3QKc2lsZW50bHkgdGFrZSBvdmVyIHRoZSBjb250cmFjdC4AAAAMYWNjZXB0X2FkbWluAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAMYXJjaGl2ZV9odW50AAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAMaXNfdmlld19vbmx5AAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdhZGRyZXNzAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAFBVbmxvY2tzIGEgY2x1ZSBoaW50IGZvciBhIHJlZ2lzdGVyZWQgcGxheWVyIGFuZCBkZWR1Y3RzIHRoZSBjbHVlJ3MgaGludCBwZW5hbHR5LgAAAAxyZXF1ZXN0X2hpbnQAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+kAAAAQAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAEtTZWFyY2hlcyBodW50cyBieSBwYXJ0aWFsIHRpdGxlIG1hdGNoIG92ZXIgYSBjYWxsZXItYm91bmRlZCBodW50LWlkIHdpbmRvdy4AAAAADHNlYXJjaF9odW50cwAAAAQAAAAAAAAAD3RpdGxlX3N1YnN0cmluZwAAAAAQAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAAAAAAKc2Nhbl9saW1pdAAAAAAABAAAAAEAAAPqAAAH0AAAAARIdW50",
        "AAAAAAAAAAAAAAANYWN0aXZhdGVfaHVudAAAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAABABDb21wbGV0ZXMgYSBodW50IGZvciBhIHBsYXllciBhbmQgZGlzdHJpYnV0ZXMgcmV3YXJkcy4KClRoaXMgZnVuY3Rpb24gdmVyaWZpZXMgdGhhdCB0aGUgcGxheWVyIGhhcyBjb21wbGV0ZWQgYWxsIHJlcXVpcmVkIGNsdWVzLAp0aGVuIGRpc3RyaWJ1dGVzIHJld2FyZHMgdmlhIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IChpZiBjb25maWd1cmVkKQphbmQgdXBkYXRlcyB0aGUgcGxheWVyJ3MgcmV3YXJkIHN0YXR1cy4KClJld2FyZCBhbW91bnRzIGNhbiBiZSBlaXRoZXIgZmxhdCAoYHhsbV9wb29sIC8gbWF4X3dpbm5lcnNgKSBvcgp0aW1lLWJhc2VkIChjb25maWd1cmVkIHZpYSBgUmV3YXJkTWFuYWdlcjo6c2V0X3Bvb2xfdGllcnNgKSwgaW4gd2hpY2gKY2FzZSB0aGUgYW1vdW50IGRlcGVuZHMgb24gYGNvbXBsZXRpb25fYXQgLSBzdGFydGVkX2F0YCBmb3IgdGhlCmNvbXBsZXRpbmcgcGxheWVyLgoKIyBBcmd1bWVudHMKKiBgZW52YCAtIFRoZSBTb3JvYmFuIGVudmlyb25tZW50CiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgSUQKKiBgcGxheWVyYCAtIFRoZSBwbGF5ZXIgY2xhaW1pbmcgY29tcGxldGlvbi9yZXdhcmRzCgojIFJldHVybnMKYE9rKCgpKWAgb24gc3VjY2Vzc2Z1bCByZXdhcmQgY2xhaW0KCiMgRXJyb3JzCiogYEh1bnROb3RGb3VuZGAgLSBIdW50IGRvZXMgbm90IGV4aXN0CiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IEFjdGl2ZSAoZS5nLiBhbHJlYWR5IENvbXBsZXRlZCBvciBDYW5jZWxsZWQpCiogYFBsYXllck5vdFJlZ2lzdGVyZWRgIC0gUGxheWVyIGlzIG5vdCByZWdpc3RlcmVkCiogYEh1bnROb3RDb21wbGV0ZWRgIC0gUGxheWVyIGhhc24ndCBjb21wbGV0ZWQgYWxsIHJlcXVpcmVkIGNsdWVzCiogYFJld2FyZEFscmVhZHlDbGFpbWVkYCAtIFBsYXllciBhbHJlYWR5IGNsYWltZWQgdGhlaXIgcmV3YXJkCiogYE5vUmV3YXJkc0NvbmZpZ3VyZWRgIC0gTm8gcmV3YXJkcyBzZXQgdXAgZm9yIHRoaXMgaHVudAoqIGBJbnN1ZmZpY2llbnRSZXdhcmRQb29sAAAADWNvbXBsZXRlX2h1bnQAAAAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAANZ2V0X2h1bnRfaW5mbwAAAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAH0AAAAARIdW50AAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAANcGF1c2VfYW5zd2VycwAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAANcGF1c2VfcmV3YXJkcwAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAANcnVuX21pZ3JhdGlvbgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAAAAAAAB2RyeV9ydW4AAAAAAQAAAAEAAAPpAAAH0AAAAA9NaWdyYXRpb25SZXBvcnQAAAAH0AAAABBVcGdyYWRlQXV0aEVycm9y",
        "AAAAAAAAADJTZXRzIG9yIGNsZWFycyB0aGUgb3B0aW9uYWwgaGludCBmb3IgYSBkcmFmdCBjbHVlLgAAAAAADXNldF9jbHVlX2hpbnQAAAAAAAAFAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAARoaW50AAAD6AAAABAAAAAAAAAAE2hpbnRfcGVuYWx0eV9wb2ludHMAAAAABAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAANc3VibWl0X2Fuc3dlcgAAAAAAAAYAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAABmFuc3dlcgAAAAAAEAAAAAAAAAAQc3VibWlzc2lvbl9ub25jZQAAAAYAAAAAAAAADHN1Ym1pdHRlZF9hdAAAAAYAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAOYWRkX2NvX2NyZWF0b3IAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAA5uZXdfY29fY3JlYXRvcgAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAADZSZXR1cm5zIHRoZSB0b3RhbCBudW1iZXIgb2YgaHVudHMgY3JlYXRlZCAocmVhZC1vbmx5KS4AAAAAAA5nZXRfaHVudF9jb3VudAAAAAAAAAAAAAEAAAAG",
        "AAAAAAAAADFSZXR1cm5zIHRydWUgaWYgdGhlIGdpdmVuIGFkZHJlc3MgaXMgYmxhY2tsaXN0ZWQuAAAAAAAADmlzX2JsYWNrbGlzdGVkAAAAAAABAAAAAAAAAAdjcmVhdG9yAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAEhQYXVzZXMgYWxsIHBsYXllciBvcGVyYXRpb25zIChyZWdpc3RyYXRpb25zLCBhbnN3ZXJzLCByZXdhcmRzKSBnbG9iYWxseS4AAAAOcGF1c2VfY29udHJhY3QAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAEpWZXJpZmllcyBhIGNhbmRpZGF0ZSBhbnN3ZXIgd2l0aG91dCByZWNvcmRpbmcgcHJvZ3Jlc3Mgb3IgZW1pdHRpbmcgZXZlbnRzLgAAAAAADnByZXZpZXdfYW5zd2VyAAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAAZhbnN3ZXIAAAAAABAAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAPZGVhY3RpdmF0ZV9odW50AAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAPZ2V0X2NvX2NyZWF0b3JzAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAPZ2V0X3BhdXNlX3N0YXRlAAAAAAAAAAABAAAD7QAAAAMAAAABAAAAAQAAAAE=",
        "AAAAAAAAAsFSZWdpc3RlcnMgYSBwbGF5ZXIgZm9yIGFuIGFjdGl2ZSBodW50LiBUaGUgY2FsbGVyIG11c3QgcGFzcyB0aGVpciBhZGRyZXNzIGFuZCBhdXRob3JpemU7Cm9ubHkgdGhhdCBpZGVudGl0eSBjYW4gcmVnaXN0ZXIgdGhlbXNlbHZlcy4gSW5pdGlhbGl6ZXMgcGxheWVyIHByb2dyZXNzIGFuZCBwcmV2ZW50cwpkdXBsaWNhdGUgcmVnaXN0cmF0aW9ucy4gUmVnaXN0cmF0aW9uIGlzIG9ubHkgYWxsb3dlZCB3aGlsZSB0aGUgaHVudCBpcyBhY3RpdmUgYW5kCihpZiBzZXQpIGJlZm9yZSBlbmRfdGltZS4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHJlZ2lzdGVyIGZvcgoqIGBwbGF5ZXJgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIHBsYXllciAobXVzdCBhdXRob3JpemUgdGhlIGNhbGwgdmlhIHJlcXVpcmVfYXV0aCkKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzCgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkb2VzIG5vdCBleGlzdAoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBBY3RpdmUgc3RhdHVzCiogYEh1bnROb3RBY3RpdmVgIC0gSHVudCBoYXMgZW5kZWQgKHBhc3QgZW5kX3RpbWUpCiogYER1cGxpY2F0ZVJlZ2lzdHJhdGlvbmAgLSBQbGF5ZXIgaXMgYWxyZWFkeSByZWdpc3RlcmVkIGZvciB0aGlzIGh1bnQAAAAAAAAPcmVnaXN0ZXJfcGxheWVyAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAGBTZXRzIHRoZSBtYXhpbXVtIHBsYXllcnMgZm9yIGEgaHVudC4gT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiBzZXQgaXQsIGFuZCBvbmx5IGluIERyYWZ0IHN0YXR1cy4AAAAPc2V0X21heF9wbGF5ZXJzAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAttYXhfcGxheWVycwAAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAPdW5wYXVzZV9hbnN3ZXJzAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAPdW5wYXVzZV9yZXdhcmRzAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAlZBZGRzIGFsdGVybmF0aXZlIGFjY2VwdGFibGUgYW5zd2VycyB0byBhbiBleGlzdGluZyBjbHVlIChzeW5vbnltcykuCk9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gYWRkIGFsaWFzZXMsIGFuZCBvbmx5IHdoaWxlIHRoZSBodW50IGlzIGluIERyYWZ0IHN0YXR1cy4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IGNvbnRhaW5pbmcgdGhlIGNsdWUKKiBgY2x1ZV9pZGAgLSBUaGUgZXhpc3RpbmcgY2x1ZSB0byBhZGQgYWxpYXNlcyB0bwoqIGBhbnN3ZXJzYCAtIEFsdGVybmF0aXZlIGFuc3dlcnMgdGhhdCBzaG91bGQgYWxzbyBiZSBhY2NlcHRlZAoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgSW52YWxpZEh1bnRTdGF0dXNgIC0gSHVudCBpcyBub3QgaW4gRHJhZnQKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvcgoqIGBDbHVlTm90Rm91bmRgIC0gQ2x1ZSBkb2VzIG5vdCBleGlzdAoqIGBJbnZhbGlkQW5zd2VyYCAtIEFueSBhbnN3ZXIgaXMgZW1wdHkgb3IgZXhjZWVkcyBtYXggbGVuZ3RoAAAAAAAQYWRkX2NsdWVfYWxpYXNlcwAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdhbnN3ZXJzAAAAA+oAAAAQAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAFhTZXRzIHRoZSBjb250cmFjdCBhZG1pbiBvbmNlLiBTdWJzZXF1ZW50IGNhbGxzIHJlcXVpcmUgY3VycmVudCBhZG1pbiBhdXRoIHZpYSBzZXRfYWRtaW4uAAAAEGluaXRpYWxpemVfYWRtaW4AAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAoVTZXRzIHdoZXRoZXIgYSBodW50IGlzIHByaXZhdGUgKGludml0ZS1vbmx5KS4KCk9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gY2FsbCB0aGlzLCBhbmQgb25seSB3aGlsZSB0aGUgaHVudCBpcyBpbiBEcmFmdCBzdGF0dXMuCldoZW4gbWFraW5nIGEgaHVudCBwcml2YXRlLCBhbiBpbnZpdGUgY29kZSBtdXN0IGFscmVhZHkgYmUgY29uZmlndXJlZCB2aWEKYGdlbmVyYXRlX2ludml0ZV9jb2RlYCBiZWZvcmUgdGhlIGh1bnQgY2FuIGJlIGFjdGl2YXRlZC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHVwZGF0ZSBwcml2YWN5IGZvcgoqIGBjcmVhdG9yYCAtIFRoZSBodW50IGNyZWF0b3IgKG11c3QgYXV0aG9yaXplIHRoZSBjYWxsKQoqIGBpc19wcml2YXRlYCAtIFdoZXRoZXIgdGhlIGh1bnQgc2hvdWxkIGJlIGludml0ZS1vbmx5CgojIFJldHVybnMKYE9rKCgpKWAgb24gc3VjY2VzcwoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvcgoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBEcmFmdCBzdGF0dXMAAAAAAAAQc2V0X2h1bnRfcHJpdmFjeQAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAppc19wcml2YXRlAAAAAAABAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAB5SZXN1bWVzIGFsbCBwbGF5ZXIgb3BlcmF0aW9ucy4AAAAAABB1bnBhdXNlX2NvbnRyYWN0AAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAGBCbGFja2xpc3RzIGEgY3JlYXRvciBhZGRyZXNzLCBwcmV2ZW50aW5nIHRoZW0gZnJvbSBjcmVhdGluZyBuZXcgaHVudHMuCkNhbGxlciBtdXN0IGJlIHRoZSBhZG1pbi4AAAARYmxhY2tsaXN0X2NyZWF0b3IAAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAB9FeHBvc2VzIHRoZSBlbmQgdGltZSBvZiBhIGh1bnQuAAAAABFnZXRfaHVudF9lbmRfdGltZQAAAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAABgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9zY2hlbWEAAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAVRTdGVwIG9uZSBvZiBhIHR3by1zdGVwIGFkbWluIGtleSByb3RhdGlvbi4KClRoZSBjdXJyZW50IGFkbWluIHByb3Bvc2VzIGEgbmV3IGFkbWluLiBUaGUgY2hhbmdlIGlzIE5PVCBhcHBsaWVkIHVudGlsIHRoZQpwcm9wb3NlZCBhZGRyZXNzIGNhbGxzIGBhY2NlcHRfYWRtaW5gLCB3aGljaCBwcmV2ZW50cyBhY2NpZGVudGFsIGxvY2tvdXQ6IGEKdHlwbyBpbiBgcHJvcG9zZV9uZXdfYWRtaW5gIGNhbiBzaW1wbHkgYmUgb3ZlcndyaXR0ZW4gb3IgaWdub3JlZCwgYW5kIHRoZQpjdXJyZW50IGFkbWluIG5ldmVyIGxvc2VzIGFjY2VzcyB1bnRpbCB0aGUgbmV3IGFkbWluIGFjdGl2ZWx5IGFjY2VwdHMuAAAAEXByb3Bvc2VfbmV3X2FkbWluAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAARcmVtb3ZlX2NvX2NyZWF0b3IAAAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAUY29fY3JlYXRvcl90b19yZW1vdmUAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X3NjaGVtYV92ZXJzaW9uAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAASZ2V0X3ZpZXdfb25seV9saXN0AAAAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAD6gAAABM=",
        "AAAAAAAAADRSZXR1cm5zIHdoZXRoZXIgdGhlIGdsb2JhbCBjb250cmFjdCBwYXVzZSBpcyBhY3RpdmUuAAAAEmlzX2NvbnRyYWN0X3BhdXNlZAAAAAAAAAAAAAEAAAAB",
        "AAAAAAAAAe9DbGVhcnMgdGhlIGludml0ZSBjb2RlIGZvciBhIHByaXZhdGUgaHVudCwgZWZmZWN0aXZlbHkgcGF1c2luZyBuZXcgcmVnaXN0cmF0aW9ucy4KVGhlIGh1bnQgY3JlYXRvciBjYW4gZ2VuZXJhdGUgYSBuZXcgY29kZSBsYXRlciB2aWEgYGdlbmVyYXRlX2ludml0ZV9jb2RlYC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHJldm9rZSB0aGUgaW52aXRlIGNvZGUgZm9yCiogYGNyZWF0b3JgIC0gVGhlIGh1bnQgY3JlYXRvciAobXVzdCBhdXRob3JpemUgdGhlIGNhbGwpCgojIFJldHVybnMKYE9rKCgpKWAgb24gc3VjY2VzcwoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvcgoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBEcmFmdCBzdGF0dXMAAAAAEnJldm9rZV9pbnZpdGVfY29kZQAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAScm9sbGJhY2tfbWlncmF0aW9uAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AAAAB9AAAAAQVXBncmFkZUF1dGhFcnJvcg==",
        "AAAAAAAAAE9TZXRzIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IGFkZHJlc3MgZm9yIGNyb3NzLWNvbnRyYWN0IHJld2FyZCBkaXN0cmlidXRpb24uAAAAABJzZXRfcmV3YXJkX21hbmFnZXIAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOcmV3YXJkX21hbmFnZXIAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAKFSZXR1cm5zIHRoZSBsaXN0IG9mIGNsdWUgSURzIHRoYXQgdGhlIHBsYXllciBoYXMgY29tcGxldGVkIGZvciBhIGh1bnQgKHJlYWQtb25seSkuClVzZWZ1bCBmb3IgVUkgdG8gc2hvdyBwcm9ncmVzcy4gUmV0dXJucyBlbXB0eSB2ZWMgaWYgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkLgAAAAAAABNnZXRfY29tcGxldGVkX2NsdWVzAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+oAAAAE",
        "AAAAAAAAAIlSZXR1cm5zIGFnZ3JlZ2F0ZSBzdGF0aXN0aWNzIGZvciBhIGh1bnQgKHJlYWQtb25seSk6IHRvdGFsIHBsYXllcnMsIGNvbXBsZXRpb24gcmF0ZSwgYXZlcmFnZSBzY29yZS4KUmV0dXJucyBlcnJvciBpZiBodW50IGRvZXMgbm90IGV4aXN0LgAAAAAAABNnZXRfaHVudF9zdGF0aXN0aWNzAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAH0AAAAA5IdW50U3RhdGlzdGljcwAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAJJSZXR1cm5zIHBsYXllciBwcm9ncmVzcyBmb3IgYSBodW50IChyZWFkLW9ubHkpLgpJbmNsdWRlcyBjb21wbGV0ZWQgY2x1ZXMsIHNjb3JlLCBhbmQgY29tcGxldGlvbiBzdGF0dXMuClJldHVybnMgZXJyb3IgaWYgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkLgAAAAAAE2dldF9wbGF5ZXJfcHJvZ3Jlc3MAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAB9AAAAAOUGxheWVyUHJvZ3Jlc3MAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAATaXNfZ2xvYmFsX3ZpZXdfb25seQAAAAABAAAAAAAAAAdhZGRyZXNzAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAATcGF1c2VfcmVnaXN0cmF0aW9ucwAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAElVcGRhdGVzIGNhdGVnb3JpZXMgZm9yIGEgZHJhZnQgaHVudC4gQXQgbW9zdCBmaXZlIGNhdGVnb3JpZXMgYXJlIGFsbG93ZWQuAAAAAAAAE3NldF9odW50X2NhdGVnb3JpZXMAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAACmNhdGVnb3JpZXMAAAAAA+oAAAAQAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAUYWRkX2dsb2JhbF92aWV3X29ubHkAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABnZpZXdlcgAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAUYWRkX3ZpZXdfb25seV9hY2Nlc3MAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAGdmlld2VyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAq5HZW5lcmF0ZXMgb3IgdXBkYXRlcyB0aGUgaW52aXRlIGNvZGUgZm9yIGEgcHJpdmF0ZSBodW50LgoKVGhlIGludml0ZSBjb2RlIGlzIGhhc2hlZCB3aXRoIFNIQTI1NiAodXNpbmcgaHVudF9pZCBhcyBzYWx0KSBhbmQgb25seSB0aGUgaGFzaAppcyBzdG9yZWQgb24tY2hhaW4uIFRoZSBwbGFpbi10ZXh0IGNvZGUgaXMgbmV2ZXIgcGVyc2lzdGVkIG9yIGVtaXR0ZWQgaW4gZXZlbnRzLgpDYWxsaW5nIHRoaXMgZnVuY3Rpb24gb3ZlcndyaXRlcyBhbnkgcHJldmlvdXNseSBzZXQgaW52aXRlIGNvZGUuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBnZW5lcmF0ZSBhbiBpbnZpdGUgY29kZSBmb3IKKiBgY3JlYXRvcmAgLSBUaGUgaHVudCBjcmVhdG9yIChtdXN0IGF1dGhvcml6ZSB0aGUgY2FsbCkKKiBgaW52aXRlX2NvZGVgIC0gVGhlIHBsYWluLXRleHQgaW52aXRlIGNvZGUgdG8gaGFzaCBhbmQgc3RvcmUKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzCgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkb2VzIG5vdCBleGlzdAoqIGBVbmF1dGhvcml6ZWRgIC0gQ2FsbGVyIGlzIG5vdCB0aGUgaHVudCBjcmVhdG9yCiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IGluIERyYWZ0IHN0YXR1cwAAAAAAFGdlbmVyYXRlX2ludml0ZV9jb2RlAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAC2ludml0ZV9jb2RlAAAAABAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAUZ2V0X2hlYWx0aF9kYXNoYm9hcmQAAAAAAAAAAQAAB9AAAAAOQ29udHJhY3RIZWFsdGgAAA==",
        "AAAAAAAAAORSZXR1cm5zIHJhbmtlZCBwbGF5ZXJzIGZvciBhIGh1bnQgd2l0aCBwYWdpbmF0aW9uIHN1cHBvcnQgKHJlYWQtb25seSkuClNvcnRlZCBieSBzY29yZSBkZXNjZW5kaW5nLCB0aGVuIGJ5IGNvbXBsZXRpb24gdGltZSBhc2NlbmRpbmcgKGVhcmxpZXIgPSBiZXR0ZXIpLgpMaW1pdCBpcyBjYXBwZWQgYXQgMjAgdG8gY29udHJvbCBnYXMuIFJldHVybnMgZXJyb3IgaWYgaHVudCBkb2VzIG5vdCBleGlzdC4AAAAUZ2V0X2h1bnRfbGVhZGVyYm9hcmQAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPpAAAH0AAAABFMZWFkZXJib2FyZFJlc3VsdAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAANlSZXR1cm5zIGEgcGFnaW5hdGVkIHNsaWNlIG9mIGNsdWVzIGZvciBhIGh1bnQuIFVzZWZ1bCBmb3IgbGFyZ2UgaHVudHMgdG8gYm91bmQgZ2FzLgpQYWdlIGlzIDAtaW5kZXhlZC4gTWF4IHBhZ2Vfc2l6ZSBpcyBjYXBwZWQgYXQgTUFYX0JBVENIX1NJWkUgKDUwKS4KRXN0aW1hdGVkIGdhczogTyhwYWdlX3NpemUpIH41XzAwMCBnYXMgcGVyIGNsdWUgKyAxMF8wMDAgb3ZlcmhlYWQuAAAAAAAAFGxpc3RfY2x1ZXNfcGFnaW5hdGVkAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAARwYWdlAAAABAAAAAAAAAAJcGFnZV9zaXplAAAAAAAABAAAAAEAAAPqAAAH0AAAAAhDbHVlSW5mbw==",
        "AAAAAAAAAydSZWdpc3RlcnMgYSBwbGF5ZXIgZm9yIGEgcHJpdmF0ZSBodW50IHVzaW5nIGEgdmFsaWQgaW52aXRlIGNvZGUuCgpUaGUgcHJvdmlkZWQgaW52aXRlIGNvZGUgaXMgaGFzaGVkICh3aXRoIGh1bnRfaWQgYXMgc2FsdCkgYW5kIGNvbXBhcmVkIGFnYWluc3QKdGhlIHN0b3JlZCBgaW52aXRlX2NvZGVfaGFzaGAuIElmIHRoZXkgbWF0Y2gsIHRoZSBwbGF5ZXIgaXMgcmVnaXN0ZXJlZC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBwcml2YXRlIGh1bnQgdG8gcmVnaXN0ZXIgZm9yCiogYHBsYXllcmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgcGxheWVyIChtdXN0IGF1dGhvcml6ZSB0aGUgY2FsbCB2aWEgcmVxdWlyZV9hdXRoKQoqIGBpbnZpdGVfY29kZWAgLSBUaGUgcGxhaW4tdGV4dCBpbnZpdGUgY29kZSB0byB2YWxpZGF0ZQoKIyBSZXR1cm5zCmBPaygoKSlgIG9uIHN1Y2Nlc3MKCiMgRXJyb3JzCiogYEh1bnROb3RGb3VuZGAgLSBIdW50IGRvZXMgbm90IGV4aXN0CiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IGluIEFjdGl2ZSBzdGF0dXMsIGlzIG5vdCBwcml2YXRlICh1c2UKYHJlZ2lzdGVyX3BsYXllcmAgaW5zdGVhZCksIG9yIGhhcyBubyBpbnZpdGUgY29kZSBjb25maWd1cmVkCiogYEludmFsaWRBbnN3ZXJgIC0gVGhlIHByb3ZpZGVkIGludml0ZSBjb2RlIGlzIGVtcHR5IG9yIGRvZXMgbm90IG1hdGNoCiogYER1cGxpY2F0ZVJlZ2lzdHJhdGlvbmAgLSBQbGF5ZXIgaXMgYWxyZWFkeSByZWdpc3RlcmVkIGZvciB0aGlzIGh1bnQAAAAAFHJlZ2lzdGVyX3dpdGhfaW52aXRlAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAAC2ludml0ZV9jb2RlAAAAABAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAEFSZXR1cm5zIGh1bnRzIHdob3NlIGNhdGVnb3JpZXMgaW5jbHVkZSB0aGUgZXhhY3QgY2F0ZWdvcnkgc3RyaW5nLgAAAAAAABVnZXRfaHVudHNfYnlfY2F0ZWdvcnkAAAAAAAAEAAAAAAAAAAhjYXRlZ29yeQAAABAAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAAAAAApzY2FuX2xpbWl0AAAAAAAEAAAAAQAAA+oAAAfQAAAABEh1bnQ=",
        "AAAAAAAAAGhSZW1vdmVzIGEgY3JlYXRvciBmcm9tIHRoZSBibGFja2xpc3QsIHJlc3RvcmluZyB0aGVpciBhYmlsaXR5IHRvIGNyZWF0ZSBodW50cy4KQ2FsbGVyIG11c3QgYmUgdGhlIGFkbWluLgAAABVyZW1vdmVfZnJvbV9ibGFja2xpc3QAAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAVc2V0X3RpbWVfYm9udXNfY29uZmlnAAAAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAEXRpbWVfYm9udXNfY29uZmlnAAAAAAAD6AAAB9AAAAAPVGltZUJvbnVzQ29uZmlnAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAVdW5wYXVzZV9yZWdpc3RyYXRpb25zAAAAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAXcmVtb3ZlX2dsb2JhbF92aWV3X29ubHkAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZ2aWV3ZXIAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAXcmVtb3ZlX3ZpZXdfb25seV9hY2Nlc3MAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAABnZpZXdlcgAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAATpWYXJpYW50IG9mIGBzdWJtaXRfYW5zd2VyYCB3aGljaCBhY2NlcHRzIGEgcHJlY29tcHV0ZWQgU0hBMjU2IGFuc3dlciBoYXNoLgpUaGlzIGF2b2lkcyBvbi1jaGFpbiBub3JtYWxpemF0aW9uIGFuZCBoYXNoaW5nIHdoZW4gdGhlIGNsaWVudCBzdXBwbGllcwp0aGUgY29ycmVjdGx5IGNvbXB1dGVkIGBhbnN3ZXJfaGFzaCA9IFNIQTI1NihodW50X2lkIHx8IGNsdWVfaWQgfHwgbm9ybWFsaXplZF9hbnN3ZXIpYC4KVXNlIHRoaXMgZnJvbSBvZmYtY2hhaW4gY2FsbGVycyB0aGF0IGNhbiBwZXJmb3JtIG5vcm1hbGl6YXRpb24raGFzaGluZyBjaGVhcGx5LgAAAAAAF3N1Ym1pdF9hbnN3ZXJfd2l0aF9oYXNoAAAAAAYAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAAC2Fuc3dlcl9oYXNoAAAAA+4AAAAgAAAAAAAAABBzdWJtaXNzaW9uX25vbmNlAAAABgAAAAAAAAAMc3VibWl0dGVkX2F0AAAABgAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAG1VcGRhdGVzIGEgaHVudCdzIGRlc2NyaXB0aW9uLiBPbmx5IHRoZSBodW50IGNyZWF0b3IgY2FuIHVwZGF0ZSBpdCwgYW5kIGl0IGNhbiBiZSB1cGRhdGVkIGZvciBhbnkgaHVudCBzdGF0dXMuAAAAAAAAF3VwZGF0ZV9odW50X2Rlc2NyaXB0aW9uAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAtkZXNjcmlwdGlvbgAAAAAQAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAZZ2V0X2dsb2JhbF92aWV3X29ubHlfbGlzdAAAAAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAATNTY2FucyBhIGJvdW5kZWQgd2luZG93IG9mIHJlZ2lzdGVyZWQgcGxheWVycyBmb3IgYSBodW50IGFuZCByZXR1cm5zCnRoZWlyIGNvbXBhY3Qgcm93cy4gVGhpcyBtZXRob2QgZW5hYmxlcyBjbGllbnRzIHRvIHBhZ2UgdGhyb3VnaCBhbGwKcmVnaXN0ZXJlZCBwbGF5ZXJzIGluIG11bHRpcGxlIGNhbGxzIChib3VuZGVkIGJ5IGBNQVhfTEVBREVSQk9BUkRfU0NBTl9TSVpFYCkKYW5kIG1lcmdlIHJlc3VsdHMgb2ZmLWNoYWluIHRvIGJ1aWxkIGEgZnVsbCBsZWFkZXJib2FyZCB3aXRob3V0IGEgc2luZ2xlCmxhcmdlIG9uLWNoYWluIHNjYW4uAAAAABtnZXRfaHVudF9sZWFkZXJib2FyZF93aW5kb3cAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAtzdGFydF9pbmRleAAAAAAEAAAAAAAAAAt3aW5kb3dfc2l6ZQAAAAAEAAAAAQAAA+kAAAfQAAAAEUxlYWRlcmJvYXJkV2luZG93AAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAHFTZXRzIG9yIGNsZWFycyBhIG1hbnVhbCBodW50IGRpZmZpY3VsdHkgb3ZlcnJpZGUuIFdpdGhvdXQgYW4gb3ZlcnJpZGUsCnRoZSByYXRpbmcgaXMgdGhlIGF2ZXJhZ2UgY2x1ZSBkaWZmaWN1bHR5LgAAAAAAABxzZXRfaHVudF9kaWZmaWN1bHR5X292ZXJyaWRlAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAE2RpZmZpY3VsdHlfb3ZlcnJpZGUAAAAD6AAAAAQAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAQAAAAAAAAAAAAAAC0hlYWx0aEFsZXJ0AAAAAAMAAAAAAAAACmFsZXJ0X3R5cGUAAAAAABAAAAAAAAAABWNvdW50AAAAAAAABAAAAAAAAAALbGFzdF9sZWRnZXIAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADkNvbnRyYWN0SGVhbHRoAAAAAAAFAAAAAAAAAA1hY3RpdmVfYWxlcnRzAAAAAAAABAAAAAAAAAANYXZnX2dhc191bml0cwAAAAAAAAYAAAAAAAAAEmZhaWxlZF9pbnZvY2F0aW9ucwAAAAAABgAAAAAAAAAQZmFpbHVyZV9yYXRlX2JwcwAAAAQAAAAAAAAAEXRvdGFsX2ludm9jYXRpb25zAAAAAAAABg==",
        "AAAAAQAAAGFTdG9yZWQgY2x1ZSB3aXRoIFNIQTI1NiBhbnN3ZXIgaGFzaC4gVGhlIGhhc2ggaXMgbmV2ZXIgZXhwb3NlZCB2aWEgZ2V0X2NsdWUvbGlzdF9jbHVlcyBvciBldmVudHMuAAAAAAAAAAAAAARDbHVlAAAACQAAAAAAAAANYW5zd2VyX2hhc2hlcwAAAAAAA+oAAAPuAAAAIAAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAApkaWZmaWN1bHR5AAAAAAAEAAAAAAAAAARoaW50AAAD6AAAABAAAAAAAAAAE2hpbnRfcGVuYWx0eV9wb2ludHMAAAAABAAAAAAAAAALaXNfcmVxdWlyZWQAAAAAAQAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAhxdWVzdGlvbgAAABAAAAAAAAAABndlaWdodAAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAABEh1bnQAAAAfAAAAAAAAAAxhY3RpdmF0ZWRfYXQAAAAGAAAARVdoZW4gdHJ1ZSwgcGxheWVycyBtYXkgY2xhaW0gdGhlaXIgcGFydGlhbCBzY29yZSBhZnRlciB0aGUgaHVudCBlbmRzLgAAAAAAABVhbGxvd19wYXJ0aWFsX3Njb3JpbmcAAAAAAAABAAAARU1pbmltdW0gc2Vjb25kcyBhIHBsYXllciBtdXN0IHdhaXQgYmV0d2VlbiBhdHRlbXB0cyBvbiB0aGUgc2FtZSBjbHVlLgAAAAAAABVhdHRlbXB0X2Nvb2xkb3duX3NlY3MAAAAAAAAEAAAAAAAAAApjYXRlZ29yaWVzAAAAAAPqAAAAEAAAAAAAAAAPY29tcGxldGVkX2NvdW50AAAAAAQAAAAAAAAACmNyZWF0ZWRfYXQAAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAFREZWZhdWx0IHBvaW50IHZhbHVlIGFwcGxpZWQgdG8gY2x1ZXMgd2l0aCAwIHBvaW50cy4gQ2x1ZS1sZXZlbCBwb2ludHMgb3ZlcnJpZGUgdGhpcy4AAAAOZGVmYXVsdF9wb2ludHMAAAAAAAQAAAAAAAAAC2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAAE2RpZmZpY3VsdHlfb3ZlcnJpZGUAAAAD6AAAAAQAAAAAAAAAEWRpZmZpY3VsdHlfcmF0aW5nAAAAAAAABAAAAAAAAAAIZW5kX3RpbWUAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAABEU0hBMjU2IGhhc2ggKHNhbHRlZCB3aXRoIGh1bnRfaWQpIG9mIHRoZSBpbnZpdGUgY29kZSwgaWYgY29uZmlndXJlZC4AAAAQaW52aXRlX2NvZGVfaGFzaAAAA+gAAAPuAAAAIAAAAD5XaGVuIHRydWUsIG9ubHkgcGxheWVycyB3aXRoIGEgdmFsaWQgaW52aXRlIGNvZGUgbWF5IHJlZ2lzdGVyLgAAAAAACmlzX3ByaXZhdGUAAAAAAAEAAAAAAAAAFW1heF9hdHRlbXB0c19wZXJfY2x1ZQAAAAAAAAQAAAA9TWF4aW11bSBudW1iZXIgb2YgcGxheWVycyBhbGxvd2VkIHRvIHJlZ2lzdGVyLiAwID0gdW5saW1pdGVkLgAAAAAAAAttYXhfcGxheWVycwAAAAAEAAAAAAAAABptYXhfc3VibWlzc2lvbnNfcGVyX21pbnV0ZQAAAAAABAAAAFBSZWdpc3RyYXRpb24gY3V0b2ZmIHRpbWVzdGFtcC4gMCA9IG5vIGRlYWRsaW5lIChyZWdpc3RyYXRpb24gb3BlbiB3aGlsZSBhY3RpdmUpLgAAABVyZWdpc3RyYXRpb25fZGVhZGxpbmUAAAAAAAAGAAAAZkR5bmFtaWNhbGx5IHJlY2FsY3VsYXRlZCBvbiBldmVyeSBgZ2V0X2h1bnRgIHJlYWQ7IG5vdCBtZWFuaW5nZnVsIHdoZW4gcmVhZCBmcm9tIGEgcmF3IHN0cnVjdCBsaXRlcmFsLgAAAAAAD3JlbWFpbmluZ19zbG90cwAAAAAEAAAAAAAAAA5yZXF1aXJlZF9jbHVlcwAAAAAABAAAAAAAAAANcmV3YXJkX2NvbmZpZwAAAAAAB9AAAAAMUmV3YXJkQ29uZmlnAAAAAAAAABRzdGFydF9tdWx0aXBsaWVyX2JwcwAAAAQAAAAAAAAACnN0YXJ0X3RpbWUAAAAAAAYAAAAAAAAABnN0YXR1cwAAAAAH0AAAAApIdW50U3RhdHVzAAAAAAA6V2hlbiB0cnVlLCBwbGF5ZXJzIG1heSBmb3JtIHRlYW1zIGFuZCBzaGFyZSBjbHVlIHByb2dyZXNzLgAAAAAACXRlYW1fbW9kZQAAAAAAAAEAAAAAAAAAFXRpbWVfYm9udXNfZGVjYXlfc2VjcwAAAAAAA+gAAAAGAAAAAAAAABJ0aW1lX2JvbnVzX21pbl9icHMAAAAAA+gAAAAEAAAAAAAAABR0aW1lX2JvbnVzX3N0YXJ0X2JwcwAAA+gAAAAEAAAAAAAAAAV0aXRsZQAAAAAAABAAAAAAAAAAC3RvdGFsX2NsdWVzAAAAAAQ=",
        "AAAAAQAAACVBIHRlYW0gY29tcGV0aW5nIGluIGEgdGVhbS1tb2RlIGh1bnQuAAAAAAAAAAAAAARUZWFtAAAABQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZsZWFkZXIAAAAAABMAAAAAAAAAB21lbWJlcnMAAAAD6gAAABMAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAd0ZWFtX2lkAAAAAAQ=",
        "AAAAAQAAAEBDbHVlIGluZm8gcmV0dXJuZWQgYnkgZ2V0X2NsdWUvbGlzdF9jbHVlcy4gRXhjbHVkZXMgYW5zd2VyIGhhc2guAAAAAAAAAAhDbHVlSW5mbwAAAAgAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAKZGlmZmljdWx0eQAAAAAABAAAAAAAAAAOaGludF9hdmFpbGFibGUAAAAAAAEAAAAAAAAAE2hpbnRfcGVuYWx0eV9wb2ludHMAAAAABAAAAAAAAAALaXNfcmVxdWlyZWQAAAAAAQAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAhxdWVzdGlvbgAAABAAAAAAAAAABndlaWdodAAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAACExvY2F0aW9uAAAAAwAAAAAAAAAIbGF0aXR1ZGUAAAAHAAAAAAAAAAlsb25naXR1ZGUAAAAAAAAHAAAAAAAAAAZyYWRpdXMAAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAACUh1bnRDYWNoZQAAAAAAAAgAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAIZW5kX3RpbWUAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAC21heF93aW5uZXJzAAAAAAQAAAAAAAAADnJlcXVpcmVkX2NsdWVzAAAAAAAEAAAAAAAAAApzdGFydF90aW1lAAAAAAAGAAAAAAAAAAZzdGF0dXMAAAAAB9AAAAAKSHVudFN0YXR1cwAAAAAAAAAAAAt0b3RhbF9jbHVlcwAAAAAE",
        "AAAAAgAAAAAAAAAAAAAACkh1bnRTdGF0dXMAAAAAAAcAAAAAAAAAAAAAAAVEcmFmdAAAAAAAAAAAAAAAAAAABkFjdGl2ZQAAAAAAAAAAAAAAAAAJQ29tcGxldGVkAAAAAAAAAAAAAAAAAAAJQ2FuY2VsbGVkAAAAAAAAAAAAAAAAAAAGUGF1c2VkAAAAAAAAAAAAAAAAABBFbWVyZ2VuY3lTdG9wcGVkAAAAAAAAAAAAAAAIQXJjaGl2ZWQ=",
        "AAAAAQAAAAAAAAAAAAAADFJld2FyZENvbmZpZwAAAAcAAAAAAAAADWNsYWltZWRfY291bnQAAAAAAAAEAAAAAAAAAAttYXhfd2lubmVycwAAAAAEAAAAAAAAAAxuZnRfY29udHJhY3QAAAPoAAAAEwAAAAAAAAALbmZ0X2VuYWJsZWQAAAAAAQAAAAAAAAAKbmZ0X3Jhcml0eQAAAAAABAAAAAAAAAAIbmZ0X3RpZXIAAAAEAAAAAAAAAAh4bG1fcG9vbAAAAAs=",
        "AAAAAQAAAFFTaGFyZWQgcHJvZ3Jlc3MgZm9yIGEgdGVhbTogY2x1ZXMgY29tcGxldGVkIGJ5IGFueSBtZW1iZXIgYW5kIHRoZSBjb21iaW5lZCBzY29yZS4AAAAAAAAAAAAADFRlYW1Qcm9ncmVzcwAAAAIAAAAAAAAAD2NvbXBsZXRlZF9jbHVlcwAAAAPqAAAABAAAAAAAAAALdG90YWxfc2NvcmUAAAAABA==",
        "AAAAAQAAAENJbnB1dCBwYXlsb2FkIGZvciBhZGRpbmcgbXVsdGlwbGUgY2x1ZXMgaW4gb25lIGNvbnRyYWN0IGludm9jYXRpb24uAAAAAAAAAAAOQmF0Y2hDbHVlSW5wdXQAAAAAAAUAAAAAAAAABmFuc3dlcgAAAAAAEAAAAEJEaWZmaWN1bHR5IG11bHRpcGxpZXIgKDEtMTApLiBQb2ludHMgZWFybmVkID0gcG9pbnRzICogZGlmZmljdWx0eS4AAAAAAApkaWZmaWN1bHR5AAAAAAAEAAAAAAAAAAtpc19yZXF1aXJlZAAAAAABAAAAAAAAAAZwb2ludHMAAAAAAAQAAAAAAAAACHF1ZXN0aW9uAAAAEA==",
        "AAAAAQAAAD5FbWl0dGVkIHdoZW4gYSBjbHVlIGlzIGFkZGVkLiBEb2VzIG5vdCBleHBvc2UgdGhlIGFuc3dlciBoYXNoLgAAAAAAAAAAAA5DbHVlQWRkZWRFdmVudAAAAAAACAAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAdRGlmZmljdWx0eSBtdWx0aXBsaWVyICgxLTEwKS4AAAAAAAAKZGlmZmljdWx0eQAAAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAtpc19yZXF1aXJlZAAAAAABAAAAAAAAAAZwb2ludHMAAAAAAAQAAAAAAAAACHF1ZXN0aW9uAAAAEAAAAB5XZWlnaHQgbXVsdGlwbGllciAoZGVmYXVsdCAxKS4AAAAAAAZ3ZWlnaHQAAAAAAAQ=",
        "AAAAAQAAADlBZ2dyZWdhdGUgc3RhdGlzdGljcyBmb3IgYSBodW50IChyZWFkLW9ubHkgcXVlcnkgcmVzdWx0KS4AAAAAAAAAAAAADkh1bnRTdGF0aXN0aWNzAAAAAAAFAAAAAAAAAA1hdmVyYWdlX3Njb3JlAAAAAAAABAAAAAAAAAAPY29tcGxldGVkX2NvdW50AAAAAAQAAAAAAAAAF2NvbXBsZXRpb25fcmF0ZV9wZXJjZW50AAAAAAQAAAAAAAAADXRvdGFsX3BsYXllcnMAAAAAAAAEAAAAAAAAAA90b3RhbF9zY29yZV9zdW0AAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADkxlYWRlcmJvYXJkUm93AAAAAAAFAAAAAAAAAAxjb21wbGV0ZWRfYXQAAAAGAAAAAAAAAAVpbmRleAAAAAAAAAQAAAAAAAAADGlzX2NvbXBsZXRlZAAAAAEAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAFc2NvcmUAAAAAAAAE",
        "AAAAAQAAAFdQdWJsaWMgdmlldyBvZiBwbGF5ZXIgcHJvZ3Jlc3MsIHdpdGggYHBsYXllcmAgYW5kIGBodW50X2lkYCByZWNvbnN0cnVjdGVkIGZyb20gdGhlIGtleS4AAAAAAAAAAA5QbGF5ZXJQcm9ncmVzcwAAAAAADQAAAAAAAAASY2x1ZV9sYXN0X2F0dGVtcHRzAAAAAAPsAAAABAAAAAYAAAAAAAAADGNvbXBsZXRlZF9hdAAAAAYAAAAAAAAAFGNvbXBsZXRlZF9jbHVlX2luZGV4AAAD7AAAAAQAAAABAAAAAAAAAA9jb21wbGV0ZWRfY2x1ZXMAAAAD6gAAAAQAAAAAAAAADGhpbnRlZF9jbHVlcwAAA+oAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAADGlzX2NvbXBsZXRlZAAAAAEAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAScmVjZW50X3N1Ym1pc3Npb25zAAAAAAPqAAAABgAAAAAAAAAYcmVxdWlyZWRfY29tcGxldGVkX2NvdW50AAAABAAAAAAAAAAOcmV3YXJkX2NsYWltZWQAAAAAAAEAAAAAAAAACnN0YXJ0ZWRfYXQAAAAAAAYAAAAAAAAAC3RvdGFsX3Njb3JlAAAAAAQ=",
        "AAAAAQAAAB5FbWl0dGVkIHdoZW4gYSBodW50IGlzIGNsb25lZC4AAAAAAAAAAAAPSHVudENsb25lZEV2ZW50AAAAAAMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAALbmV3X2h1bnRfaWQAAAAABgAAAAAAAAAQb3JpZ2luYWxfaHVudF9pZAAAAAY=",
        "AAAAAQAAAO9FbWl0dGVkIHdoZW4gYSBjcmVhdG9yIGZvcmNlLWNsb3NlcyBhIGh1bnQgZWFybHkgKG1hcmtzIGl0IENvbXBsZXRlZCkgd2hpbGUKcHJlc2VydmluZyBwbGF5ZXIgc2NvcmVzIGFuZCBhbnkgYWxyZWFkeS1kaXN0cmlidXRlZCByZXdhcmRzLiBgcmV3YXJkZWRfcGxheWVyc2AKaXMgdGhlIG51bWJlciBvZiBjb21wbGV0ZWQgcGxheWVycyB3aG8gcmVjZWl2ZWQgYSBmaW5hbCByZXdhcmQgYXMgcGFydCBvZiBjbG9zaW5nLgAAAAAAAAAAD0h1bnRDbG9zZWRFdmVudAAAAAADAAAAAAAAAAljbG9zZWRfYXQAAAAAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAEHJld2FyZGVkX3BsYXllcnMAAAAE",
        "AAAAAQAAAAAAAAAAAAAAD1JhdGVMaW1pdFN0YXR1cwAAAAADAAAAAAAAABBjb29sZG93bl9zZWNvbmRzAAAABgAAAAAAAAAPY3JlYXRpb25zX3RvZGF5AAAAAAQAAAAAAAAAC2RhaWx5X2xpbWl0AAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAD1RpbWVCb251c0NvbmZpZwAAAAADAAAAAAAAABNkZWNheV9kdXJhdGlvbl9zZWNzAAAAAAYAAAAAAAAAEm1pbl9tdWx0aXBsaWVyX2JwcwAAAAAABAAAAAAAAAAUc3RhcnRfbXVsdGlwbGllcl9icHMAAAAE",
        "AAAAAQAAAAAAAAAAAAAAEEh1bnRDcmVhdGVkRXZlbnQAAAACAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABg==",
        "AAAAAQAAAElMZWFkZXJib2FyZCBlbnRyeSBmb3IgYSBzaW5nbGUgcGxheWVyIGluIGEgaHVudCAocmVhZC1vbmx5IHF1ZXJ5IHJlc3VsdCkuAAAAAAAAAAAAABBMZWFkZXJib2FyZEVudHJ5AAAABQAAAAAAAAAMY29tcGxldGVkX2F0AAAABgAAAAAAAAAMaXNfY29tcGxldGVkAAAAAQAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAARyYW5rAAAABAAAAAAAAAAFc2NvcmUAAAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAEFRlYW1DcmVhdGVkRXZlbnQAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABmxlYWRlcgAAAAAAEwAAAAAAAAAEbmFtZQAAABAAAAAAAAAAB3RlYW1faWQAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAEUh1bnRBcmNoaXZlZEV2ZW50AAAAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAIxXcmFwcGVyIHJldHVybmVkIGJ5IGBnZXRfaHVudF9sZWFkZXJib2FyZGAgdGhhdCBpbmNsdWRlcyB0cnVuY2F0aW9uCmluZm9ybWF0aW9uIHNvIGNhbGxlcnMgY2FuIHRlbGwgd2hlbiB0aGUgdmlzaWJsZSBlbnRyaWVzIGFyZSBpbmNvbXBsZXRlLgAAAAAAAAARTGVhZGVyYm9hcmRSZXN1bHQAAAAAAAADAAAAAAAAAAdlbnRyaWVzAAAAA+oAAAfQAAAAEExlYWRlcmJvYXJkRW50cnkAAAAAAAAADXRvdGFsX3BsYXllcnMAAAAAAAAEAAAAAAAAAAl0cnVuY2F0ZWQAAAAAAAAB",
        "AAAAAQAAAAAAAAAAAAAAEUxlYWRlcmJvYXJkV2luZG93AAAAAAAABAAAAAAAAAAHZW50cmllcwAAAAPqAAAH0AAAAA5MZWFkZXJib2FyZFJvdwAAAAAAAAAAAAhmaW5pc2hlZAAAAAEAAAAAAAAACm5leHRfaW5kZXgAAAAAAAQAAAAAAAAACnF1ZXJpZWRfYXQAAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAAEVBsYXllckJhbm5lZEV2ZW50AAAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAEkNsdWVDb21wbGV0ZWRFdmVudAAAAAAABAAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAANcG9pbnRzX2Vhcm5lZAAAAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAEkh1bnRBY3RpdmF0ZWRFdmVudAAAAAAAAgAAAAAAAAAMYWN0aXZhdGVkX2F0AAAABgAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAEkh1bnRDYW5jZWxsZWRFdmVudAAAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAEkh1bnRDb21wbGV0ZWRFdmVudAAAAAAABQAAAAAAAAAPY29tcGxldGlvbl9yYW5rAAAAAAQAAAAAAAAAD2NvbXBsZXRpb25fdGltZQAAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAALdG90YWxfc2NvcmUAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAElJld2FyZENsYWltZWRFdmVudAAAAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAtuZnRfYXdhcmRlZAAAAAABAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAACnhsbV9hbW91bnQAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAE1BsYXllclVuYmFubmVkRXZlbnQAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAFEFuc3dlckluY29ycmVjdEV2ZW50AAAABAAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAFEh1bnREZWFjdGl2YXRlZEV2ZW50AAAAAQAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAFEh1bnRSZWFjdGl2YXRlZEV2ZW50AAAAAgAAAAAAAAAMYWN0aXZhdGVkX2F0AAAABgAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAbBJbnRlcm5hbCBjb21wYWN0IHN0b3JhZ2UgcmVwcmVzZW50YXRpb24gb2YgcGxheWVyIHByb2dyZXNzLgpEb2VzIG5vdCBzdG9yZSBgcGxheWVyYCBvciBgaHVudF9pZGAg4oCUIHRob3NlIGFyZSBhbHJlYWR5IHRoZSBzdG9yYWdlIGtleS4KCiMjIENvbXBhY3QgZW5jb2RpbmcKLSBUaW1lc3RhbXBzIGFyZSBkZWx0YS1lbmNvZGVkIGFzIGB1MzJgIG9mZnNldHMgZnJvbSB0aGUgaHVudCdzIGBhY3RpdmF0ZWRfYXRgLApzYXZpbmcgNCBieXRlcyBlYWNoIHZzIGZ1bGwgYHU2NGAgVU5JWCB0aW1lc3RhbXBzLiBUaGUgbWF4IGRlbHRhICh+MTM2IHllYXJzKQpmYXIgZXhjZWVkcyBhbnkgcmVhbGlzdGljIGh1bnQgZHVyYXRpb24uCi0gQm9vbGVhbiBmaWVsZHMgKGBpc19jb21wbGV0ZWRgLCBgcmV3YXJkX2NsYWltZWRgKSBhcmUgcGFja2VkIGludG8gYGZsYWdzYC4AAAAAAAAAFFN0b3JlZFBsYXllclByb2dyZXNzAAAACQAAAAAAAAASY2x1ZV9sYXN0X2F0dGVtcHRzAAAAAAPsAAAABAAAAAYAAAChU2Vjb25kcyBlbGFwc2VkIGZyb20gcGxheWVyIHJlZ2lzdHJhdGlvbiB0byBodW50IGNvbXBsZXRpb24sIG9yIDAgaWYgbm90IGNvbXBsZXRlZC4KUmVjb25zdHJ1Y3QgYWJzb2x1dGU6IGBhY3RpdmF0ZWRfYXQgKyBzdGFydGVkX2F0X2RlbHRhICsgY29tcGxldGVkX2F0X2RlbHRhYC4AAAAAAAASY29tcGxldGVkX2F0X2RlbHRhAAAAAAAEAAAAAAAAAA9jb21wbGV0ZWRfY2x1ZXMAAAAD6gAAAAQAAACPQml0IGZsYWdzIGZvciBib29sZWFuIGZpZWxkcyB0byByZWR1Y2Ugc3RvcmFnZSBmb290cHJpbnQuCkJJVDAgKDEpOiBpc19jb21wbGV0ZWQKQklUMSAoMik6IHJld2FyZF9jbGFpbWVkCkJJVDLigJNCSVQzMTogcmVzZXJ2ZWQgZm9yIGZ1dHVyZSB1c2UAAAAABWZsYWdzAAAAAAAABAAAAAAAAAAMaGludGVkX2NsdWVzAAAD6gAAAAQAAAAAAAAAEnJlY2VudF9zdWJtaXNzaW9ucwAAAAAD6gAAAAYAAAAAAAAAGHJlcXVpcmVkX2NvbXBsZXRlZF9jb3VudAAAAAQAAAB5U2Vjb25kcyBlbGFwc2VkIGZyb20gaHVudCBgYWN0aXZhdGVkX2F0YCB0byBwbGF5ZXIgcmVnaXN0cmF0aW9uLgpSZWNvbnN0cnVjdCBhYnNvbHV0ZTogYGFjdGl2YXRlZF9hdCArIHN0YXJ0ZWRfYXRfZGVsdGFgLgAAAAAAABBzdGFydGVkX2F0X2RlbHRhAAAABAAAAAAAAAALdG90YWxfc2NvcmUAAAAABA==",
        "AAAAAQAAAE1UZWFtIGxlYWRlcmJvYXJkIGVudHJ5IChyZWFkLW9ubHkgcXVlcnkgcmVzdWx0KSwgcmFua2VkIGJ5IHNoYXJlZCB0ZWFtIHNjb3JlLgAAAAAAAAAAAAAUVGVhbUxlYWRlcmJvYXJkRW50cnkAAAAFAAAAAAAAAAxtZW1iZXJfY291bnQAAAAEAAAAAAAAAARuYW1lAAAAEAAAAAAAAAAEcmFuawAAAAQAAAAAAAAABXNjb3JlAAAAAAAABAAAAAAAAAAHdGVhbV9pZAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAFUNsdWVBbGlhc2VzQWRkZWRFdmVudAAAAAAAAAQAAAAAAAAADWFsaWFzZXNfY291bnQAAAAAAAAEAAAAAAAAAAdjbHVlX2lkAAAAAAQAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAElTdG9yZWQgdG9wLU4gbGVhZGVyYm9hcmQgZW50cnkgbWFpbnRhaW5lZCBpbmNyZW1lbnRhbGx5IG9uIHNjb3JlIGNoYW5nZXMuAAAAAAAAAAAAABVMZWFkZXJib2FyZEluZGV4RW50cnkAAAAAAAAEAAAAAAAAAAxjb21wbGV0ZWRfYXQAAAAGAAAAAAAAAAxpc19jb21wbGV0ZWQAAAABAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAABXNjb3JlAAAAAAAABA==",
        "AAAAAQAAADNFbWl0dGVkIHdoZW4gYSBwbGF5ZXIgcmVnaXN0ZXJzIGZvciBhbiBhY3RpdmUgaHVudC4AAAAAAAAAABVQbGF5ZXJSZWdpc3RlcmVkRXZlbnQAAAAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAAFVJld2FyZE1hbmFnZXJTZXRFdmVudAAAAAAAAAIAAAAAAAAAC25ld19hZGRyZXNzAAAAABMAAAAAAAAAC29sZF9hZGRyZXNzAAAAA+gAAAAT",
        "AAAAAQAAAAAAAAAAAAAAFVRlYW1NZW1iZXJKb2luZWRFdmVudAAAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAAd0ZWFtX2lkAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAFkh1bnRTdGF0dXNDaGFuZ2VkRXZlbnQAAAAAAAQAAAAAAAAACmNoYW5nZWRfYXQAAAAAAAYAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAKbmV3X3N0YXR1cwAAAAAH0AAAAApIdW50U3RhdHVzAAAAAAAAAAAACm9sZF9zdGF0dXMAAAAAB9AAAAAKSHVudFN0YXR1cwAA",
        "AAAAAQAAAE5FbWl0dGVkIHdoZW4gYSBodW50IGNyZWF0b3IgY2xlYXJzIHRoZSBpbnZpdGUgY29kZSwgcGF1c2luZyBuZXcgcmVnaXN0cmF0aW9ucy4AAAAAAAAAAAAWSW52aXRlQ29kZVJldm9rZWRFdmVudAAAAAAAAgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAdodW50X2lkAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAAFlJld2FyZENsYWltRmFpbGVkRXZlbnQAAAAAAAMAAAAAAAAACmVycm9yX2NvZGUAAAAAAAQAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAAF0NyZWF0b3JCbGFja2xpc3RlZEV2ZW50AAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAHY3JlYXRvcgAAAAAT",
        "AAAAAQAAAJlFbWl0dGVkIHdoZW4gYSBodW50IGNyZWF0b3IgZ2VuZXJhdGVzIG9yIHVwZGF0ZXMgdGhlIGludml0ZSBjb2RlIGZvciBhIHByaXZhdGUgaHVudC4KVGhlIGludml0ZSBjb2RlIGl0c2VsZiBpcyBuZXZlciBlbWl0dGVkIG9yIHN0b3JlZCDigJQgb25seSBpdHMgaGFzaC4AAAAAAAAAAAAAGEludml0ZUNvZGVHZW5lcmF0ZWRFdmVudAAAAAIAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAGFBhcnRpYWxTY29yZUNsYWltZWRFdmVudAAAAAQAAAAAAAAAD2NsdWVzX2NvbXBsZXRlZAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAADXBhcnRpYWxfc2NvcmUAAAAAAAAEAAAAAAAAAAZwbGF5ZXIAAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAG0h1bnREZXNjcmlwdGlvblVwZGF0ZWRFdmVudAAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAC2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAAB2h1bnRfaWQAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAHFJlZ2lzdHJhdGlvbkRlYWRsaW5lU2V0RXZlbnQAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAFXJlZ2lzdHJhdGlvbl9kZWFkbGluZQAAAAAAAAY=",
        "AAAAAQAAAEJFbWl0dGVkIHdoZW4gYSBwbGF5ZXIgc3VjY2Vzc2Z1bGx5IHJlZ2lzdGVycyB1c2luZyBhbiBpbnZpdGUgY29kZS4AAAAAAAAAAAAfUGxheWVyUmVnaXN0ZXJlZFdpdGhJbnZpdGVFdmVudAAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAAIENyZWF0b3JSZW1vdmVkRnJvbUJsYWNrbGlzdEV2ZW50AAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdjcmVhdG9yAAAAABM=",
        "AAAABAAAAAAAAAAAAAAADUh1bnRFcnJvckNvZGUAAAAAAAAyAAAAAAAAAAxIdW50Tm90Rm91bmQAAAABAAAAAAAAAAxDbHVlTm90Rm91bmQAAAACAAAAAAAAABFJbnZhbGlkSHVudFN0YXR1cwAAAAAAAAMAAAAAAAAAE1BsYXllck5vdFJlZ2lzdGVyZWQAAAAABAAAAAAAAAAUQ2x1ZUFscmVhZHlDb21wbGV0ZWQAAAAFAAAAAAAAAA1JbnZhbGlkQW5zd2VyAAAAAAAABgAAAAAAAAANSHVudE5vdEFjdGl2ZQAAAAAAAAcAAAAAAAAADFVuYXV0aG9yaXplZAAAAAgAAAAAAAAAFkluc3VmZmljaWVudFJld2FyZFBvb2wAAAAAAAkAAAAAAAAAFUR1cGxpY2F0ZVJlZ2lzdHJhdGlvbgAAAAAAAAoAAAAAAAAADEludmFsaWRUaXRsZQAAAAsAAAAAAAAAEkludmFsaWREZXNjcmlwdGlvbgAAAAAADAAAAAAAAAAOSW52YWxpZEFkZHJlc3MAAAAAAA0AAAAAAAAADFRvb01hbnlDbHVlcwAAAA4AAAAAAAAAD0ludmFsaWRRdWVzdGlvbgAAAAAPAAAAAAAAAAxSZWZ1bmRGYWlsZWQAAAAQAAAAAAAAAAxOb0NsdWVzQWRkZWQAAAARAAAAAAAAABBIdW50Tm90Q29tcGxldGVkAAAAEgAAAAAAAAAUUmV3YXJkQWxyZWFkeUNsYWltZWQAAAATAAAAAAAAABhSZXdhcmREaXN0cmlidXRpb25GYWlsZWQAAAAUAAAAAAAAABNOb1Jld2FyZHNDb25maWd1cmVkAAAAABUAAAAAAAAAE0R1cGxpY2F0ZVN1Ym1pc3Npb24AAAAAFgAAAAAAAAARU3VibWlzc2lvbkV4cGlyZWQAAAAAAAAXAAAAAAAAAAxCYW5uZWRQbGF5ZXIAAAAYAAAAAAAAAA9Ob1JlcXVpcmVkQ2x1ZXMAAAAAGQAAAAAAAAARUmF0ZUxpbWl0RXhjZWVkZWQAAAAAAAAaAAAAAAAAAA1TY29yZU92ZXJmbG93AAAAAAAAGwAAAAAAAAATUmVnaXN0cmF0aW9uc1BhdXNlZAAAAAAcAAAAAAAAAA1BbnN3ZXJzUGF1c2VkAAAAAAAAHQAAAAAAAAANUmV3YXJkc1BhdXNlZAAAAAAAAB4AAAAAAAAAEUh1bnRFbmRUaW1lSW5QYXN0AAAAAAAAHwAAAAAAAAAOTm9QZW5kaW5nQWRtaW4AAAAAACAAAAAAAAAAFFBlbmRpbmdBZG1pbk1pc21hdGNoAAAAIQAAAAAAAAANSW52YWxpZFJhcml0eQAAAAAAACIAAAAAAAAAFkludmFsaWRUaW1lQm9udXNDb25maWcAAAAAACMAAAAAAAAAEkFkZHJlc3NCbGFja2xpc3RlZAAAAAAAJAAAAAAAAAAOQ29udHJhY3RQYXVzZWQAAAAAACUAAAAAAAAAEkludmFsaWRNYXhBdHRlbXB0cwAAAAAAJgAAAAAAAAANSW52YWxpZFdlaWdodAAAAAAAACcAAAAAAAAAEEhpbnROb3RBdmFpbGFibGUAAAAoAAAAAAAAABNIaW50QWxyZWFkeVVubG9ja2VkAAAAACkAAAAAAAAAEUluc3VmZmljaWVudFNjb3JlAAAAAAAAKgAAAAAAAAARVG9vTWFueUNhdGVnb3JpZXMAAAAAAAArAAAAAAAAAA9JbnZhbGlkQ2F0ZWdvcnkAAAAALAAAAAAAAAARSW52YWxpZERpZmZpY3VsdHkAAAAAAAAtAAAAAAAAABVDb3JydXB0UGxheWVyUHJvZ3Jlc3MAAAAAAAAuAAAAAAAAAA5IdW50Tm90U3RhcnRlZAAAAAAALwAAAAAAAAAUQWRtaW5BbHJlYWR5UHJvcG9zZWQAAAAwAAAAAAAAAA1JbnZhbGlkUG9pbnRzAAAAAAAAMQAAAAAAAAAISHVudEZ1bGwAAAAy",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAD1VwZ3JhZGVQcm9wb3NhbAAAAAAEAAAAAAAAAAxlZmZlY3RpdmVfYXQAAAAGAAAAAAAAAAtwcm9wb3NlZF9hdAAAAAAGAAAAAAAAAAhwcm9wb3NlcgAAABMAAAAAAAAADnRhcmdldF92ZXJzaW9uAAAAAAAE",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVBdXRoRXJyb3IAAAAFAAAAAAAAAAxVbmF1dGhvcml6ZWQAAAABAAAAAAAAAApOb1Byb3Bvc2FsAAAAAAACAAAAAAAAAA9UaW1lbG9ja1BlbmRpbmcAAAAAAwAAAAAAAAAPVmVyc2lvbk1pc21hdGNoAAAAAAQAAAAAAAAAD0ludmFsaWRUaW1lbG9jawAAAAAF",
        "AAAAAQAAAAAAAAAAAAAAE1VwZ3JhZGVIaXN0b3J5RW50cnkAAAAABAAAAAAAAAALZXhlY3V0ZWRfYXQAAAAABgAAAAAAAAAIZXhlY3V0b3IAAAATAAAAAAAAAAxmcm9tX3ZlcnNpb24AAAAEAAAAAAAAAAp0b192ZXJzaW9uAAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAFFVwZ3JhZGVFeGVjdXRlZEV2ZW50AAAABAAAAAAAAAALZXhlY3V0ZWRfYXQAAAAABgAAAAAAAAAIZXhlY3V0b3IAAAATAAAAAAAAAAxmcm9tX3ZlcnNpb24AAAAEAAAAAAAAAAp0b192ZXJzaW9uAAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAFFVwZ3JhZGVQcm9wb3NlZEV2ZW50AAAABAAAAAAAAAAMZWZmZWN0aXZlX2F0AAAABgAAAAAAAAALcHJvcG9zZWRfYXQAAAAABgAAAAAAAAAIcHJvcG9zZXIAAAATAAAAAAAAAA50YXJnZXRfdmVyc2lvbgAAAAAABA==",
        "AAAAAgAAAC1SZWFzb24gYSB0aWVyIG9yIHRpZXIgbGlzdCBmYWlsZWQgdmFsaWRhdGlvbi4AAAAAAAAAAAAACVRpZXJFcnJvcgAAAAAAAAMAAAAAAAAAK0EgdGllcidzIGB4bG1fYW1vdW50YCB3YXMgemVybyBvciBuZWdhdGl2ZS4AAAAAEU5vblBvc2l0aXZlQW1vdW50AAAAAAAAAAAAAIpUd28gYWRqYWNlbnQgdGllcnMgc2hhcmUgdGhlIHNhbWUgYG1heF9jb21wbGV0aW9uX3NlY3NgIG9yIGFwcGVhciBvdXQgb2YKYXNjZW5kaW5nIG9yZGVyLiBUaWVycyBtdXN0IGJlIHN0cmljdGx5IGluY3JlYXNpbmcgaW4gdGltZSBib3VuZC4AAAAAABROb3RTdHJpY3RseUFzY2VuZGluZwAAAAAAAAA7VGhlIHRpZXIgbGlzdCBpcyBlbXB0eSB3aGVuIGF0IGxlYXN0IG9uZSB0aWVyIHdhcyByZXF1aXJlZC4AAAAABUVtcHR5AAAA",
        "AAAAAQAAAFdDb25maWd1cmF0aW9uIGZvciBkaXN0cmlidXRpbmcgcmV3YXJkcyBhY3Jvc3MgdGhlIEh1bnR5Q29yZSDihpQgUmV3YXJkTWFuYWdlciBib3VuZGFyeS4AAAAAAAAAAAxSZXdhcmRDb25maWcAAAAIAAAAAAAAAAxuZnRfY29udHJhY3QAAAPoAAAAEwAAAAAAAAAPbmZ0X2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAADm5mdF9odW50X3RpdGxlAAAAAAAQAAAAAAAAAA1uZnRfaW1hZ2VfdXJpAAAAAAAAEAAAAAAAAAAKbmZ0X3Jhcml0eQAAAAAABAAAAAAAAAAIbmZ0X3RpZXIAAAAEAAAAAAAAAAluZnRfdGl0bGUAAAAAAAAQAAAAAAAAAAp4bG1fYW1vdW50AAAAAAPoAAAACw==",
        "AAAAAwAAAD5Ib3cgcmV3YXJkcyBhcmUgY2FsY3VsYXRlZCBmcm9tIHRoZSBwb29sIGF0IGRpc3RyaWJ1dGlvbiB0aW1lLgAAAAAAAAAAABBEaXN0cmlidXRpb25Nb2RlAAAAAgAAACRGaXhlZCBhbW91bnQgc3VwcGxpZWQgYnkgdGhlIGNhbGxlci4AAAAFRml4ZWQAAAAAAAAAAAAAKFNoYXJlIG9mIHRoZSBwb29sIGJhc2VkIG9uIHBsYXllciBzY29yZS4AAAAMUHJvcG9ydGlvbmFsAAAAAQ==",
        "AAAAAQAAAPpNaXJyb3Igb2YgdGhlIFJld2FyZE1hbmFnZXIncyBwZXItaHVudCBwb29sIGNvbmZpZ3VyYXRpb24uIENhbGxlcnMgc3VjaCBhcwpIdW50eUNvcmUgdXNlIHRoaXMgdG8gZGVzZXJpYWxpemUgYGdldF9wb29sX2NvbmZpZ2AgY3Jvc3MtY29udHJhY3QgcmVzdWx0cy4KRmllbGQgbmFtZXMgYW5kIG9yZGVyIG11c3Qgc3RheSBpbiBzeW5jIHdpdGggdGhlIFJld2FyZE1hbmFnZXIncyBzdHJ1Y3Qgc28KdGhlIFhEUiBlbmNvZGluZ3MgbWF0Y2guAAAAAAAAAAAAEFJld2FyZFBvb2xDb25maWcAAAAMAAAAR1VuaXggdGltZXN0YW1wIGFmdGVyIHdoaWNoIGNsYWltcyBhcmUgbm8gbG9uZ2VyIGFsbG93ZWQgKDAgPSBkaXNhYmxlZCkuAAAAAA5jbGFpbV9kZWFkbGluZQAAAAAABgAAAC9BZGRyZXNzIG9mIHRoZSBodW50IGNyZWF0b3Igd2hvIG93bnMgdGhpcyBwb29sLgAAAAAHY3JlYXRvcgAAAAATAAAANkFkZHJlc3NlcyBhbGxvd2VkIHRvIGRpc3RyaWJ1dGUgcmV3YXJkcyBmb3IgdGhpcyBwb29sLgAAAAAACWRlbGVnYXRlcwAAAAAAA+oAAAATAAAAKkRpc3RyaWJ1dGlvbiBtb2RlIChGaXhlZCBvciBQcm9wb3J0aW9uYWwpLgAAAAAAEWRpc3RyaWJ1dGlvbl9tb2RlAAAAAAAH0AAAABBEaXN0cmlidXRpb25Nb2RlAAAAPFdoZXRoZXIgZGlzdHJpYnV0aW9ucyBmcm9tIHRoaXMgcG9vbCBhcmUgdGVtcG9yYXJpbHkgZnJvemVuLgAAAAZmcm96ZW4AAAAAAAEAAABBTWluaW11bSBYTE0gYW1vdW50IHBlciBkaXN0cmlidXRpb24uIDAgbWVhbnMgbm8gbWluaW11bSBlbmZvcmNlZC4AAAAAAAAXbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnQAAAAACwAAADVNaW5pbXVtIHNlY29uZHMgYmV0d2VlbiBkaXN0cmlidXRpb25zICgwID0gZGlzYWJsZWQpLgAAAAAAAB5taW5fZGlzdHJpYnV0aW9uX2ludGVydmFsX3NlY3MAAAAAAAYAAABBT3B0aW9uYWwgTkZUIGNvbnRyYWN0IGFkZHJlc3MgZm9yIE5GVC1vbmx5IG9yIG1peGVkIHJld2FyZCBwb29scy4AAAAAAAAMbmZ0X2NvbnRyYWN0AAAD6AAAABMAAAA7VGFyZ2V0IGZ1bmRpbmcgYW1vdW50IGZvciBwcm9ncmVzcyB0cmFja2luZyAoMCA9IGRpc2FibGVkKS4AAAAADXRhcmdldF9hbW91bnQAAAAAAAALAAAAbk9wdGlvbmFsIHRpbWUtYmFzZWQgcmV3YXJkIHRpZXJzLiBXaGVuIGVtcHR5LCB0aGUgcGVyLXdpbm5lciBhbW91bnQKaXMgY29tcHV0ZWQgZnJvbSBgeGxtX3Bvb2wgLyBtYXhfd2lubmVyc2AuAAAAAAAQdGltZV9iYXNlZF90aWVycwAAA+oAAAfQAAAAE1RpbWVCYXNlZFJld2FyZFRpZXIAAAAASVRva2VuIGFkZHJlc3MgZm9yIHRoZSByZXdhcmQgcG9vbCAoZS5nLiwgWExNLCBVU0RDLCBvciBvdGhlciBTQUMgdG9rZW5zKS4AAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAEdT3B0aW9uYWwgdmVzdGluZyBwZXJpb2QgaW4gc2Vjb25kcy4gV2hlbiA+IDAsIFhMTSByZXdhcmRzIGFyZSBub3QKdHJhbnNmZXJyZWQgaW1tZWRpYXRlbHkgYXQgZGlzdHJpYnV0aW9uIHRpbWUuIEluc3RlYWQsIGEgYFZlc3RpbmdSZWNvcmRgCmlzIGNyZWF0ZWQgYW5kIHRoZSBwbGF5ZXIgbXVzdCBjYWxsIGBjbGFpbV92ZXN0ZWRgIHRvIHJlY2VpdmUgdG9rZW5zCnByb3BvcnRpb25hbGx5IGFzIHRpbWUgZWxhcHNlcy4gMCBtZWFucyB2ZXN0aW5nIGlzIGRpc2FibGVkIChpbnN0YW50IHBheW91dCkuAAAAAAAAE3Zlc3RpbmdfcGVyaW9kX3NlY3MAAAAABg==",
        "AAAAAQAAAi5PbmUgdGllciBvZiBhIHRpbWUtYmFzZWQgcmV3YXJkIHNjaGVkdWxlIGNvbmZpZ3VyZWQgb24gYSByZXdhcmQgcG9vbC4KCkEgdGllciBkZWZpbmVzIGFuIFhMTSBhbW91bnQgdGhhdCBpcyBncmFudGVkIHRvIGEgcGxheWVyIHdobyBjb21wbGV0ZXMgdGhlCmh1bnQgd2l0aGluIGBtYXhfY29tcGxldGlvbl9zZWNzYCBvZiByZWdpc3RlcmluZy4gVGllcnMgbXVzdCBiZSBzdG9yZWQgaW4KYXNjZW5kaW5nIG9yZGVyIGJ5IGBtYXhfY29tcGxldGlvbl9zZWNzYCDigJQgaS5lLiBhICJmYXN0ZXIiIHRpZXIgbXVzdAphcHBlYXIgYmVmb3JlIGEgInNsb3dlciIgdGllci4gVGhlIGZpcnN0IHRpZXIgZm9yIHdoaWNoCmBtYXhfY29tcGxldGlvbl9zZWNzID49IGVsYXBzZWRgIGlzIHNlbGVjdGVkIGF0IGRpc3RyaWJ1dGlvbiB0aW1lOyBpZiB0aGUKZWxhcHNlZCB0aW1lIGV4Y2VlZHMgZXZlcnkgY29uZmlndXJlZCB0aWVyLCB0aGUgbGFzdCAoc2xvd2VzdCkgdGllcidzCmFtb3VudCBpcyB1c2VkIGFzIGEgZmFsbGJhY2sgc28gdGhlIHBsYXllciBzdGlsbCByZWNlaXZlcyBhIHJld2FyZC4AAAAAAAAAAAATVGltZUJhc2VkUmV3YXJkVGllcgAAAAACAAAAiUluY2x1c2l2ZSB1cHBlciBib3VuZCBvbiBlbGFwc2VkIHRpbWUgKGNvbXBsZXRpb25fdGltZSAtIHJlZ2lzdHJhdGlvbl90aW1lKQppbiBzZWNvbmRzLiBNdXN0IGJlIHN0cmljdGx5IGluY3JlYXNpbmcgYWNyb3NzIHRoZSB0aWVyIGxpc3QuAAAAAAAAE21heF9jb21wbGV0aW9uX3NlY3MAAAAABgAAADtYTE0gYW1vdW50IGF3YXJkZWQgdG8gYSBwbGF5ZXIgd2hvIHF1YWxpZmllcyBmb3IgdGhpcyB0aWVyLgAAAAAKeGxtX2Ftb3VudAAAAAAACw==",
        "AAAABAAAAAAAAAAAAAAAD1Jld2FyZEVycm9yQ29kZQAAAAAWAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAAAQAAAAAAAAAQSW5zdWZmaWNpZW50UG9vbAAAAAIAAAAAAAAAEkFscmVhZHlEaXN0cmlidXRlZAAAAAAAAwAAAAAAAAAOVHJhbnNmZXJGYWlsZWQAAAAAAAQAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAAAFAAAAAAAAAA1JbnZhbGlkQ29uZmlnAAAAAAAABgAAAAAAAAANTmZ0TWludEZhaWxlZAAAAAAAAAcAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAAACAAAAAAAAAAMUG9vbE5vdEZvdW5kAAAACQAAAAAAAAAMVW5hdXRob3JpemVkAAAACgAAAAAAAAASQmVsb3dNaW5pbXVtQW1vdW50AAAAAAALAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAwAAAAAAAAADEh1bnROb3RGb3VuZAAAAA0AAABRQSByZWN1cnNpdmUgZGlzdHJpYnV0aW9uIGF0dGVtcHQgd2FzIGRldGVjdGVkIGR1cmluZyBhbiBleHRlcm5hbCBYTE0gb3IgTkZUIGNhbGwuAAAAAAAAElJlZW50cmFuY3lEZXRlY3RlZAAAAAAADgAAAERUaGUgdHJhY2tlZCBwb29sIGJhbGFuY2UgZGl2ZXJnZWQgZnJvbSB0aGUgYWN0dWFsIFhMTSB0b2tlbiBiYWxhbmNlLgAAABVQb29sQmFsYW5jZURpdmVyZ2VuY2UAAAAAAAAPAAAAWlBvb2wgYmFsYW5jZSB3b3VsZCBvdmVyZmxvdyBpZiB0aGlzIGZ1bmRpbmcgYW1vdW50IGlzIGFkZGVkIChwb29sIGJhbGFuY2UgbGltaXQgZXhjZWVkZWQpLgAAAAAAE1Bvb2xCYWxhbmNlT3ZlcmZsb3cAAAAAEAAAAEZGdW5kaW5nIGFtb3VudCBpcyBiZWxvdyB0aGUgbWluaW11bSByZXF1aXJlZCAoZHVzdCBhdHRhY2sgcHJldmVudGlvbikuAAAAAAATQmVsb3dNaW5pbXVtRnVuZGluZwAAAAARAAAAOEZ1bmRpbmcgYW1vdW50IGV4Y2VlZHMgdGhlIG1heGltdW0gc2luZ2xlIGZ1bmRpbmcgbGltaXQuAAAAFUV4Y2VlZHNNYXhpbXVtRnVuZGluZwAAAAAAABIAAAA9RGFpbHkgZGlzdHJpYnV0aW9uIGNhcCBmb3IgYSBzcGVjaWZpYyBwb29sIGhhcyBiZWVuIGV4Y2VlZGVkLgAAAAAAABBEYWlseUNhcEV4Y2VlZGVkAAAAEwAAADBHbG9iYWwgZGFpbHkgZGlzdHJpYnV0aW9uIGNhcCBoYXMgYmVlbiBleGNlZWRlZC4AAAAWR2xvYmFsRGFpbHlDYXBFeGNlZWRlZAAAAAAAFAAAADFDb250cmFjdCBpcyBwYXVzZWQgYW5kIGNhbm5vdCBwZXJmb3JtIG9wZXJhdGlvbnMuAAAAAAAADkNvbnRyYWN0UGF1c2VkAAAAAAAVAAAAHEVtZXJnZW5jeSB3aXRoZHJhd2FsIGZhaWxlZC4AAAAZRW1lcmdlbmN5V2l0aGRyYXdhbEZhaWxlZAAAAAAAABY=" ]),
      options
    )
  }
  public readonly fromJSON = {
    add_clue: this.txFromJSON<Result<u32>>,
        get_clue: this.txFromJSON<Result<ClueInfo>>,
        add_clues: this.txFromJSON<Result<Array<u32>>>,
        clone_hunt: this.txFromJSON<Result<u64>>,
        close_hunt: this.txFromJSON<Result<void>>,
        list_clues: this.txFromJSON<Array<ClueInfo>>,
        list_hunts: this.txFromJSON<Array<Hunt>>,
        cancel_hunt: this.txFromJSON<Result<void>>,
        create_hunt: this.txFromJSON<Result<u64>>,
        set_max_attempts_per_clue: this.txFromJSON<Result<void>>,
        accept_admin: this.txFromJSON<Result<void>>,
        archive_hunt: this.txFromJSON<Result<void>>,
        is_view_only: this.txFromJSON<boolean>,
        request_hint: this.txFromJSON<Result<string>>,
        search_hunts: this.txFromJSON<Array<Hunt>>,
        activate_hunt: this.txFromJSON<Result<void>>,
        complete_hunt: this.txFromJSON<Result<void>>,
        get_hunt_info: this.txFromJSON<Result<Hunt>>,
        pause_answers: this.txFromJSON<Result<void>>,
        pause_rewards: this.txFromJSON<Result<void>>,
        run_migration: this.txFromJSON<Result<MigrationReport>>,
        set_clue_hint: this.txFromJSON<Result<void>>,
        submit_answer: this.txFromJSON<Result<void>>,
        add_co_creator: this.txFromJSON<Result<void>>,
        get_hunt_count: this.txFromJSON<u64>,
        is_blacklisted: this.txFromJSON<boolean>,
        pause_contract: this.txFromJSON<Result<void>>,
        preview_answer: this.txFromJSON<Result<boolean>>,
        deactivate_hunt: this.txFromJSON<Result<void>>,
        get_co_creators: this.txFromJSON<Array<string>>,
        get_pause_state: this.txFromJSON<readonly [boolean, boolean, boolean]>,
        register_player: this.txFromJSON<Result<void>>,
        set_max_players: this.txFromJSON<Result<void>>,
        unpause_answers: this.txFromJSON<Result<void>>,
        unpause_rewards: this.txFromJSON<Result<void>>,
        add_clue_aliases: this.txFromJSON<Result<void>>,
        initialize_admin: this.txFromJSON<Result<void>>,
        set_hunt_privacy: this.txFromJSON<Result<void>>,
        unpause_contract: this.txFromJSON<Result<void>>,
        blacklist_creator: this.txFromJSON<Result<void>>,
        get_hunt_end_time: this.txFromJSON<Result<u64>>,
        initialize_schema: this.txFromJSON<null>,
        propose_new_admin: this.txFromJSON<Result<void>>,
        remove_co_creator: this.txFromJSON<Result<void>>,
        get_schema_version: this.txFromJSON<u32>,
        get_view_only_list: this.txFromJSON<Array<string>>,
        is_contract_paused: this.txFromJSON<boolean>,
        revoke_invite_code: this.txFromJSON<Result<void>>,
        rollback_migration: this.txFromJSON<Result<MigrationReport>>,
        set_reward_manager: this.txFromJSON<Result<void>>,
        get_completed_clues: this.txFromJSON<Array<u32>>,
        get_hunt_statistics: this.txFromJSON<Result<HuntStatistics>>,
        get_player_progress: this.txFromJSON<Result<PlayerProgress>>,
        is_global_view_only: this.txFromJSON<boolean>,
        pause_registrations: this.txFromJSON<Result<void>>,
        set_hunt_categories: this.txFromJSON<Result<void>>,
        add_global_view_only: this.txFromJSON<Result<void>>,
        add_view_only_access: this.txFromJSON<Result<void>>,
        generate_invite_code: this.txFromJSON<Result<void>>,
        get_health_dashboard: this.txFromJSON<ContractHealth>,
        get_hunt_leaderboard: this.txFromJSON<Result<LeaderboardResult>>,
        list_clues_paginated: this.txFromJSON<Array<ClueInfo>>,
        register_with_invite: this.txFromJSON<Result<void>>,
        get_hunts_by_category: this.txFromJSON<Array<Hunt>>,
        remove_from_blacklist: this.txFromJSON<Result<void>>,
        set_time_bonus_config: this.txFromJSON<Result<void>>,
        unpause_registrations: this.txFromJSON<Result<void>>,
        remove_global_view_only: this.txFromJSON<Result<void>>,
        remove_view_only_access: this.txFromJSON<Result<void>>,
        submit_answer_with_hash: this.txFromJSON<Result<void>>,
        update_hunt_description: this.txFromJSON<Result<void>>,
        get_global_view_only_list: this.txFromJSON<Array<string>>,
        get_hunt_leaderboard_window: this.txFromJSON<Result<LeaderboardWindow>>,
        set_hunt_difficulty_override: this.txFromJSON<Result<void>>
  }
}
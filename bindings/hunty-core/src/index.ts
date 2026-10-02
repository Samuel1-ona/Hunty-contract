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
 * SHA256 hash of the invite code (salted with hunt_id). None if no invite code is set.
 */
invite_code_hash: Option<Buffer>;
  /**
 * When true, only players with a valid invite code may register.
 */
is_private: boolean;
  /**
 * Controls who can view the hunt's leaderboard. Defaults to Public.
 */
leaderboard_visibility: LeaderboardVisibility;
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


/**
 * Result of a `gc_hunt` sweep (issue #446).
 * 
 * Counts are split by storage tier because the two are charged and expire
 * differently on Soroban: instance entries share the contract's own TTL, while
 * persistent entries each carry their own. An operator reclaiming space needs
 * to see which tier actually shrank.
 */
export interface GcReport {
  /**
 * Clues whose per-hunt entries were swept.
 */
clues_swept: u32;
  hunt_id: u64;
  /**
 * Entries removed from instance storage.
 */
instance_removed: u32;
  /**
 * Entries removed from persistent storage.
 */
persistent_removed: u32;
  /**
 * Players whose per-hunt entries were swept.
 */
players_swept: u32;
  /**
 * Teams whose per-hunt entries were swept.
 */
teams_swept: u32;
  /**
 * `persistent_removed + instance_removed`.
 */
total_removed: u32;
}

export enum HuntStatus {
  Draft = 0,
  Active = 1,
  Completed = 2,
  Cancelled = 3,
  Paused = 4,
  EmergencyStopped = 5,
  Archived = 6,
}


export interface RewardConfig {
  claimed_count: u32;
  max_winners: u32;
  nft_contract: Option<string>;
  nft_enabled: boolean;
  nft_image_uri: Option<string>;
  nft_rarity: u32;
  nft_tier: u32;
  xlm_pool: i128;
}


/**
 * Input payload for adding multiple clues in one contract invocation.
 */
export interface BatchClueInput {
  answer: string;
  /**
 * Difficulty tier (1-5, 1 = easiest, 5 = hardest).
 * Difficulty multiplies the clue's points: points earned = points * difficulty.
 */
difficulty: u32;
  is_required: boolean;
  points: u32;
  question: string;
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
  /**
 * The player's finishing position among all completions for this hunt,
 * frozen at the moment `is_completed` was set to `true`.  Zero means the
 * player has not yet completed the hunt.
 */
completion_rank: u32;
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

/**
 * Controls who can view the leaderboard for a hunt.
 */
export type LeaderboardVisibility = {tag: "Public", values: void} | {tag: "RegisteredOnly", values: void} | {tag: "CreatorOnly", values: void};

export const HuntErrorCode = {
  1: {message:"HuntNotFound"},
  2: {message:"ClueNotFound"},
  3: {message:"InvalidHuntStatus"},
  4: {message:"PlayerNotRegistered"},
  5: {message:"ClueAlreadyCompleted"},
  6: {message:"InvalidAnswer"},
  7: {message:"HuntNotActive"},
  8: {message:"Unauthorized"},
  9: {message:"InsufficientRewardPool"},
  10: {message:"DuplicateRegistration"},
  11: {message:"InvalidTitle"},
  12: {message:"InvalidDescription"},
  13: {message:"InvalidAddress"},
  14: {message:"TooManyClues"},
  15: {message:"InvalidQuestion"},
  16: {message:"RefundFailed"},
  17: {message:"NoCluesAdded"},
  18: {message:"HuntNotCompleted"},
  19: {message:"RewardAlreadyClaimed"},
  20: {message:"RewardDistributionFailed"},
  21: {message:"NoRewardsConfigured"},
  22: {message:"DuplicateSubmission"},
  23: {message:"SubmissionExpired"},
  24: {message:"BannedPlayer"},
  25: {message:"NoRequiredClues"},
  26: {message:"RateLimitExceeded"},
  27: {message:"ScoreOverflow"},
  28: {message:"RegistrationsPaused"},
  29: {message:"AnswersPaused"},
  30: {message:"RewardsPaused"},
  31: {message:"HuntEndTimeInPast"},
  32: {message:"NoPendingAdmin"},
  33: {message:"PendingAdminMismatch"},
  34: {message:"InvalidRarity"},
  35: {message:"InvalidTimeBonusConfig"},
  36: {message:"AddressBlacklisted"},
  37: {message:"ContractPaused"},
  38: {message:"InvalidMaxAttempts"},
  39: {message:"InvalidWeight"},
  40: {message:"HintNotAvailable"},
  41: {message:"HintAlreadyUnlocked"},
  42: {message:"InsufficientScore"},
  43: {message:"TooManyCategories"},
  44: {message:"InvalidCategory"},
  45: {message:"InvalidDifficulty"},
  46: {message:"CorruptPlayerProgress"},
  47: {message:"HuntNotStarted"},
  48: {message:"AdminAlreadyProposed"},
  49: {message:"InvalidPoints"},
  50: {message:"HuntFull"},
  51: {message:"LeaderboardVisibilityUnauthorized"},
  52: {message:"InviteCodeRequired"},
  53: {message:"TooManyAliases"}
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


export interface MigrationReport {
  dry_run: boolean;
  from_version: u32;
  message: string;
  steps_applied: u32;
  succeeded: boolean;
  to_version: u32;
}

export const UpgradeAuthError = {
  1: {message:"Unauthorized"},
  2: {message:"NoProposal"},
  3: {message:"TimelockPending"},
  4: {message:"VersionMismatch"},
  5: {message:"InvalidTimelock"}
}

export interface Client {
  /**
   * Construct and simulate a gc_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Reclaims the storage of a cancelled or archived hunt (issue #446).
   * 
   * A cancelled hunt keeps every clue, player-progress, team, leaderboard
   * and bookkeeping entry it ever wrote. Nothing referenced those entries
   * any more, but nothing removed them either, so they sat in persistent
   * storage paying rent until their TTL lapsed.
   * 
   * Only `Cancelled` and `Archived` hunts may be collected — those are the
   * two terminal states. Anything else is rejected with `InvalidHuntStatus`,
   * because collecting a live hunt would destroy player progress.
   * 
   * The sweep is **idempotent**: running it twice reports zero the second
   * time rather than failing, so an interrupted call is safe to retry.
   * 
   * # Authorization
   * The hunt creator or the contract admin.
   * 
   * # Returns
   * A [`GcReport`] describing what was reclaimed.
   */
  gc_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<GcReport>>>

  /**
   * Construct and simulate a add_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds a clue to a hunt. Only the hunt creator can add clues.
   * Answers are hashed with SHA256 before storage. The ledger is public, so this is not a
   * secrecy guarantee; answer verification remains on-chain through plaintext submissions.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to add the clue to
   * * `question` - The clue question text (max 2000 chars, non-empty)
   * * `answer` - Plain-text answer; normalized (trimmed, lowercased) then hashed
   * * `points` - Points awarded for solving this clue (must be within 1..=10_000)
   * * `is_required` - Whether this clue must be solved to complete the hunt
   * * `difficulty` - Optional difficulty tier (defaults to 1) used as a multiplier on
   * the clue's points. Valid scale is 1..=5, where 1 is easiest and 5 is hardest.
   * * `weight` - Optional weight multiplier (defaults to 1)
   * 
   * # Returns
   * The sequential clue ID assigned within the hunt
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `InvalidHuntStatus` - Hunt is not in Draft
   * * `Unauthorized` - Caller is not the hun
   */
  add_clue: ({hunt_id, question, answer, points, is_required, difficulty, weight}: {hunt_id: u64, question: string, answer: string, points: u32, is_required: boolean, difficulty: Option<u32>, weight: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a get_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns clue information for a hunt/clue. Does not expose the answer hash.
   * 
   * Questions are only returned once the hunt is `Active` and the ledger
   * timestamp has reached `start_time` (when set). Before that, callers
   * receive [`HuntErrorCode::HuntNotActive`] so questions cannot be read
   * ahead of registration and solved offline to game time-based scoring
   * and reward tiers.
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
   * Construct and simulate a ban_player transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Bans a player from participating in a hunt.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to ban the player from
   * * `caller` - The hunt creator or the contract admin
   * * `player` - The player to ban
   */
  ban_player: ({hunt_id, caller, player}: {hunt_id: u64, caller: string, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a clone_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a new draft hunt by copying clues from an existing completed hunt.
   * 
   * Backwards-compatible wrapper: older callers can still clone a completed hunt, but
   * secure rehashing requires a caller-supplied answer list. The explicit
   * `clone_hunt_with_answers` entry point preserves answer isolation for cloned clues.
   */
  clone_hunt: ({template_hunt_id, caller}: {template_hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a close_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Force-closes (ends early) an in-progress hunt on behalf of its creator.
   * 
   * Unlike [`cancel_hunt`], closing preserves all player scores and any
   * rewards already collected: it marks the hunt `Completed` and triggers a
   * final reward distribution for eligible players who have completed the
   * hunt but have not yet claimed. Players who have not completed the hunt,
   * or whose frozen completion rank is outside `max_winners`, keep their
   * progress and are simply not rewarded. Any unspent reward-pool balance is
   * left intact. [`cancel_hunt`] is rejected once any player has completed
   * (use this method instead to pay winners).
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
   * * `HuntNotFound` - Hunt d
   */
  close_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a list_clues transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns paginated clues for a hunt. Answer hashes are not exposed.
   * A `limit` of `0` defaults to `DEFAULT_PAGE_SIZE`.
   */
  list_clues: ({hunt_id, offset, limit}: {hunt_id: u64, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<ClueInfo>>>

  /**
   * Construct and simulate a list_hunts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a list of all hunts (paginated).
   * A `limit` of `0` defaults to `DEFAULT_PAGE_SIZE`.
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
   * * `start_time` - Optional start timestamp (0 or None means no start time restriction).
   * When set, players cannot register or submit answers until the ledger timestamp
   * reaches this value. Must be strictly less than `end_time` if `end_time` is also set.
   * * `end_time` - Optional end timestamp (0 or None means no end time restriction)
   * * `max_submissions_per_minute` - Maximum number of submissions allowed per
   * minute per player. [`UNLIMITED_SUBMISSIONS_PER_MINUTE`] (0) means no limit.
   * 
   * # Returns
   * The unique hunt ID of the newly created hunt
   * 
   * # Errors
   * * `InvalidTitle` - If title is empty or exceeds maximum length
   * * `InvalidDescription` - If description exceeds maximum length
   * * `InvalidAddress` - If creator
   */
  create_hunt: ({creator, title, description, start_time, end_time, max_submissions_per_minute, start_multiplier_bps, default_points}: {creator: string, title: string, description: string, start_time: Option<u64>, end_time: Option<u64>, max_submissions_per_minute: u32, start_multiplier_bps: Option<u32>, default_points: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

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
   * Construct and simulate a unban_player transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Unbans a player from a hunt.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to unban the player from
   * * `caller` - The hunt creator or the contract admin
   * * `player` - The player to unban
   */
  unban_player: ({hunt_id, caller, player}: {hunt_id: u64, caller: string, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
   * Reward amounts can be flat (`xlm_pool / max_winners`), time-based
   * (configured via `RewardManager::set_pool_tiers`), or exact-rank based
   * (configured via `RewardManager::set_pool_rank_tiers`). Rank-based
   * amounts use the completion rank frozen by HuntyCore.
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
   * * `InvalidHuntStatus` - Hunt is not Active or Paused (e.g. Completed or Cancelled)
   * * `PlayerNotRegistered` - Player is not registered
   * * `HuntNotCompleted` - Player hasn't completed all required clues
   * * `RewardAlreadyClaimed` - Player already claimed their reward
   * * `NoRewardsConfigured` - No rewards set up
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
   * Construct and simulate a set_team_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Enables or disables team features for a draft hunt.
   * Only the hunt creator can call this, and only while the hunt is in Draft status.
   */
  set_team_mode: ({hunt_id, creator, team_mode}: {hunt_id: u64, creator: string, team_mode: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a submit_answer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Verifies a submitted answer, recording the attempt either way.
   * 
   * # Returns
   * `Ok(true)` when the answer is correct, `Ok(false)` when it is wrong.
   * 
   * An incorrect answer is reported as `Ok(false)` rather than
   * `Err(InvalidAnswer)` so that the failed attempt, the per-clue cooldown
   * timestamp and the consumed submission nonce are committed instead of
   * rolled back. See `finalize_answer_submission`.
   */
  submit_answer: ({hunt_id, clue_id, player, answer, submission_nonce, submitted_at}: {hunt_id: u64, clue_id: u32, player: string, answer: string, submission_nonce: u64, submitted_at: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

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
   * 
   * Unlike `submit_answer`, `preview_answer` does not mark the clue as completed, award points,
   * or emit clue completion events. It still requires player authorization and enforces the
   * same per-minute rate limit, per-clue attempt cap, and attempt cooldown.
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
   * * `TooManyAliases` - Adding the aliases would exceed `MAX_ALIASES_PER_CLUE`
   */
  add_clue_aliases: ({hunt_id, clue_id, answers}: {hunt_id: u64, clue_id: u32, answers: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the contract admin once. Subsequent calls require current admin auth via set_admin.
   */
  initialize_admin: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_hunt_terminal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether the hunt is in a terminal state.
   * 
   * A hunt is terminal once it can no longer accept new play or be
   * reactivated: `Completed`, `Cancelled`, or `Archived`. This view is
   * consumed by the reward manager to decide whether a pool may be
   * refunded to its creator.
   */
  is_hunt_terminal: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

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
   * Construct and simulate a get_active_alerts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_active_alerts: (options?: MethodOptions) => Promise<AssembledTransaction<Array<HealthAlert>>>

  /**
   * Construct and simulate a get_hunt_end_time transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Exposes the end time of a hunt.
   */
  get_hunt_end_time: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_schema: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

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
   * Construct and simulate a set_reward_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the reward configuration for a hunt.
   * Only the hunt creator (or a co-creator) may do this, and only while the
   * hunt is still in `Draft` — reward parameters must not be mutable once
   * players can register (#1012).
   * Sets nft_image_uri to a placeholder when nft_enabled is true.
   */
  set_reward_config: ({hunt_id, max_winners, xlm_pool, nft_enabled, nft_contract, caller}: {hunt_id: u64, max_winners: u32, xlm_pool: i128, nft_enabled: boolean, nft_contract: Option<string>, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_view_only_list transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_view_only_list: ({hunt_id, offset, limit}: {hunt_id: u64, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

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
   * 
   * Thin backwards-compatible wrapper: returns at most `MAX_CLUES_PER_HUNT`
   * entries, since `add_clue` / `add_clues_batch` bound a hunt's clue set by
   * that same constant. Prefer `get_completed_clues_paginated` for new callers.
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
   * * `InvalidAnswer` - Invite code is empty or exceeds 256 bytes
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
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to query
   * * `limit` - Maximum entries to return (capped at `MAX_LEADERBOARD_SIZE`)
   */
  get_hunt_leaderboard: ({hunt_id, limit}: {hunt_id: u64, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<LeaderboardResult>>>

  /**
   * Construct and simulate a list_clues_paginated transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a paginated slice of clues for a hunt. Useful for large hunts to bound gas.
   * Page is 0-indexed. Max page_size is capped at MAX_BATCH_SIZE (50).
   * A `page_size` of `0` defaults to `DEFAULT_PAGE_SIZE`.
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
   * * `InvalidAnswer` - The invite code is empty, exceeds 256 bytes, or does not match
   * * `DuplicateRegistration` - Player is already registered for this hunt
   */
  register_with_invite: ({hunt_id, player, invite_code}: {hunt_id: u64, player: string, invite_code: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_rate_limit_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Bootstrap or transfer the rate-limit admin role.
   * 
   * The first call sets the admin with no prior-admin check. Subsequent
   * calls require `caller` to already be the stored admin.
   */
  set_rate_limit_admin: ({caller, new_admin}: {caller: string, new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
   * Construct and simulate a set_creator_hunt_limit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Admin-only: override the daily hunt-creation limit for a specific creator.
   * 
   * Pass `limit = 0` to remove an existing override, falling back to the
   * contract-wide default.
   */
  set_creator_hunt_limit: ({caller, creator, limit}: {caller: string, creator: string, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a clone_hunt_with_answers transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Secure clone path that rehashes cloned clues against the new hunt/clue context.
   * The creator must supply the plaintext answers for each clue in the template.
   */
  clone_hunt_with_answers: ({template_hunt_id, caller, answers}: {template_hunt_id: u64, caller: string, answers: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

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
   * Variant of `submit_answer` that accepts a precomputed SHA256 answer hash.
   * 
   * Shares the incorrect-answer semantics of `submit_answer`: a wrong answer
   * returns `Ok(false)` and commits the failed attempt.
   */
  submit_answer_with_hash: ({hunt_id, clue_id, player, answer_hash, submission_nonce, submitted_at}: {hunt_id: u64, clue_id: u32, player: string, answer_hash: Buffer, submission_nonce: u64, submitted_at: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

  /**
   * Construct and simulate a update_hunt_description transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates a hunt's description. Only the hunt creator can update it, and it can be updated for any hunt status.
   */
  update_hunt_description: ({hunt_id, caller, description}: {hunt_id: u64, caller: string, description: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_global_view_only_list transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_global_view_only_list: ({offset, limit}: {offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a set_allow_partial_scoring transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Enables or disables partial-score claims for a draft hunt.
   * Only the hunt creator can call this, and only while the hunt is in Draft status.
   */
  set_allow_partial_scoring: ({hunt_id, creator, allow_partial_scoring}: {hunt_id: u64, creator: string, allow_partial_scoring: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_max_attempts_per_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates the maximum number of attempts allowed per clue and attempt cooldown duration for a draft hunt.
   * Only the hunt creator or co-creator can update it.
   */
  set_max_attempts_per_clue: ({hunt_id, caller, max_attempts_per_clue, attempt_cooldown_secs}: {hunt_id: u64, caller: string, max_attempts_per_clue: u32, attempt_cooldown_secs: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_registration_deadline transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the registration cutoff timestamp for a draft hunt. A value of 0 disables the cutoff.
   * Only the hunt creator can call this, and only while the hunt is in Draft status.
   */
  set_registration_deadline: ({hunt_id, creator, registration_deadline}: {hunt_id: u64, creator: string, registration_deadline: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunt_storage_footprint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Reports how much storage a hunt currently occupies, without removing
   * anything. Read-only, so it needs no authorization — hunt existence and
   * size are already public via `get_hunt_info`.
   */
  get_hunt_storage_footprint: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<GcReport>>

  /**
   * Construct and simulate a get_hunt_leaderboard_window transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Scans a bounded window of registered players for a hunt and returns
   * their compact rows. This method enables clients to page through all
   * registered players in multiple calls (bounded by `MAX_LEADERBOARD_SCAN_SIZE`)
   * and merge results off-chain to build a full leaderboard without a single
   * large on-chain scan. Only the requested registration slice is read, so
   * the cost of a page depends on `window_size`, not on how many players the
   * hunt has. This read path is public; the `_caller` argument is
   * accepted for forward compatibility and is currently ignored.
   */
  get_hunt_leaderboard_window: ({hunt_id, start_index, window_size, caller}: {hunt_id: u64, start_index: u32, window_size: u32, caller: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<LeaderboardWindow>>>

  /**
   * Construct and simulate a is_hunt_expired_or_cancelled transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether the hunt is expired or cancelled.
   * 
   * A hunt is considered expired when it has an `end_time` set and the
   * current ledger timestamp is at or past that end time. A hunt is
   * cancelled when its status is `Cancelled`. This view is consumed by the
   * reward manager to decide whether a pool may be migrated to a new hunt.
   */
  is_hunt_expired_or_cancelled: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

  /**
   * Construct and simulate a set_hunt_difficulty_override transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets or clears a manual hunt difficulty override. Without an override,
   * the rating is the average clue difficulty.
   * 
   * Only the hunt creator or a co-creator can change the override, and only
   * while the hunt is in Draft status.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt to configure
   * * `caller` - The creator or co-creator making the change
   * * `difficulty_override` - `Some(value)` to set, `None` to clear
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `Unauthorized` - Caller is not the hunt creator or a co-creator
   * * `InvalidHuntStatus` - Hunt is not in Draft
   * * `InvalidDifficulty` - Override is outside the allowed tier scale
   * 
   * # Events
   * * `HuntDifficultyOverrideSet` - Emitted with the hunt id, caller, and
   * the new override value
   */
  set_hunt_difficulty_override: ({hunt_id, caller, difficulty_override}: {hunt_id: u64, caller: string, difficulty_override: Option<u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_completed_clues_paginated transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Paginated variant of `get_completed_clues` (read-only).
   * `offset` is 0-indexed; `limit` is capped at `MAX_BATCH_SIZE`, matching
   * `list_clues`. Returns an empty vec if the player is not registered or the
   * offset is past the end of the completed set.
   */
  get_completed_clues_paginated: ({hunt_id, player, offset, limit}: {hunt_id: u64, player: string, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u32>>>

  /**
   * Construct and simulate a get_creator_rate_limit_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Query the current quota status for a creator.
   * 
   * Returns how many hunts the creator has created today, their effective
   * daily limit, and the cooldown seconds until the next day begins (0 when
   * the limit has not been reached).
   */
  get_creator_rate_limit_status: ({creator}: {creator: string}, options?: MethodOptions) => Promise<AssembledTransaction<RateLimitStatus>>

  /**
   * Construct and simulate a set_default_hunt_creation_limit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Admin-only: update the contract-wide default daily hunt-creation limit.
   * 
   * This is the fallback used for any creator that has no per-creator
   * override. The initial value is [`rate_limit::DEFAULT_HUNT_CREATION_LIMIT`].
   */
  set_default_hunt_creation_limit: ({caller, limit}: {caller: string, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
      new ContractSpec([ "AAAAAAAAAw1SZWNsYWltcyB0aGUgc3RvcmFnZSBvZiBhIGNhbmNlbGxlZCBvciBhcmNoaXZlZCBodW50IChpc3N1ZSAjNDQ2KS4KCkEgY2FuY2VsbGVkIGh1bnQga2VlcHMgZXZlcnkgY2x1ZSwgcGxheWVyLXByb2dyZXNzLCB0ZWFtLCBsZWFkZXJib2FyZAphbmQgYm9va2tlZXBpbmcgZW50cnkgaXQgZXZlciB3cm90ZS4gTm90aGluZyByZWZlcmVuY2VkIHRob3NlIGVudHJpZXMKYW55IG1vcmUsIGJ1dCBub3RoaW5nIHJlbW92ZWQgdGhlbSBlaXRoZXIsIHNvIHRoZXkgc2F0IGluIHBlcnNpc3RlbnQKc3RvcmFnZSBwYXlpbmcgcmVudCB1bnRpbCB0aGVpciBUVEwgbGFwc2VkLgoKT25seSBgQ2FuY2VsbGVkYCBhbmQgYEFyY2hpdmVkYCBodW50cyBtYXkgYmUgY29sbGVjdGVkIOKAlCB0aG9zZSBhcmUgdGhlCnR3byB0ZXJtaW5hbCBzdGF0ZXMuIEFueXRoaW5nIGVsc2UgaXMgcmVqZWN0ZWQgd2l0aCBgSW52YWxpZEh1bnRTdGF0dXNgLApiZWNhdXNlIGNvbGxlY3RpbmcgYSBsaXZlIGh1bnQgd291bGQgZGVzdHJveSBwbGF5ZXIgcHJvZ3Jlc3MuCgpUaGUgc3dlZXAgaXMgKippZGVtcG90ZW50Kio6IHJ1bm5pbmcgaXQgdHdpY2UgcmVwb3J0cyB6ZXJvIHRoZSBzZWNvbmQKdGltZSByYXRoZXIgdGhhbiBmYWlsaW5nLCBzbyBhbiBpbnRlcnJ1cHRlZCBjYWxsIGlzIHNhZmUgdG8gcmV0cnkuCgojIEF1dGhvcml6YXRpb24KVGhlIGh1bnQgY3JlYXRvciBvciB0aGUgY29udHJhY3QgYWRtaW4uCgojIFJldHVybnMKQSBbYEdjUmVwb3J0YF0gZGVzY3JpYmluZyB3aGF0IHdhcyByZWNsYWltZWQuAAAAAAAAB2djX2h1bnQAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAB9AAAAAIR2NSZXBvcnQAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAABABBZGRzIGEgY2x1ZSB0byBhIGh1bnQuIE9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gYWRkIGNsdWVzLgpBbnN3ZXJzIGFyZSBoYXNoZWQgd2l0aCBTSEEyNTYgYmVmb3JlIHN0b3JhZ2UuIFRoZSBsZWRnZXIgaXMgcHVibGljLCBzbyB0aGlzIGlzIG5vdCBhCnNlY3JlY3kgZ3VhcmFudGVlOyBhbnN3ZXIgdmVyaWZpY2F0aW9uIHJlbWFpbnMgb24tY2hhaW4gdGhyb3VnaCBwbGFpbnRleHQgc3VibWlzc2lvbnMuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBhZGQgdGhlIGNsdWUgdG8KKiBgcXVlc3Rpb25gIC0gVGhlIGNsdWUgcXVlc3Rpb24gdGV4dCAobWF4IDIwMDAgY2hhcnMsIG5vbi1lbXB0eSkKKiBgYW5zd2VyYCAtIFBsYWluLXRleHQgYW5zd2VyOyBub3JtYWxpemVkICh0cmltbWVkLCBsb3dlcmNhc2VkKSB0aGVuIGhhc2hlZAoqIGBwb2ludHNgIC0gUG9pbnRzIGF3YXJkZWQgZm9yIHNvbHZpbmcgdGhpcyBjbHVlIChtdXN0IGJlIHdpdGhpbiAxLi49MTBfMDAwKQoqIGBpc19yZXF1aXJlZGAgLSBXaGV0aGVyIHRoaXMgY2x1ZSBtdXN0IGJlIHNvbHZlZCB0byBjb21wbGV0ZSB0aGUgaHVudAoqIGBkaWZmaWN1bHR5YCAtIE9wdGlvbmFsIGRpZmZpY3VsdHkgdGllciAoZGVmYXVsdHMgdG8gMSkgdXNlZCBhcyBhIG11bHRpcGxpZXIgb24KdGhlIGNsdWUncyBwb2ludHMuIFZhbGlkIHNjYWxlIGlzIDEuLj01LCB3aGVyZSAxIGlzIGVhc2llc3QgYW5kIDUgaXMgaGFyZGVzdC4KKiBgd2VpZ2h0YCAtIE9wdGlvbmFsIHdlaWdodCBtdWx0aXBsaWVyIChkZWZhdWx0cyB0byAxKQoKIyBSZXR1cm5zClRoZSBzZXF1ZW50aWFsIGNsdWUgSUQgYXNzaWduZWQgd2l0aGluIHRoZSBodW50CgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkb2VzIG5vdCBleGlzdAoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBEcmFmdAoqIGBVbmF1dGhvcml6ZWRgIC0gQ2FsbGVyIGlzIG5vdCB0aGUgaHVuAAAACGFkZF9jbHVlAAAABwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAhxdWVzdGlvbgAAABAAAAAAAAAABmFuc3dlcgAAAAAAEAAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAtpc19yZXF1aXJlZAAAAAABAAAAAAAAAApkaWZmaWN1bHR5AAAAAAPoAAAABAAAAAAAAAAGd2VpZ2h0AAAAAAPoAAAABAAAAAEAAAPpAAAABAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAW9SZXR1cm5zIGNsdWUgaW5mb3JtYXRpb24gZm9yIGEgaHVudC9jbHVlLiBEb2VzIG5vdCBleHBvc2UgdGhlIGFuc3dlciBoYXNoLgoKUXVlc3Rpb25zIGFyZSBvbmx5IHJldHVybmVkIG9uY2UgdGhlIGh1bnQgaXMgYEFjdGl2ZWAgYW5kIHRoZSBsZWRnZXIKdGltZXN0YW1wIGhhcyByZWFjaGVkIGBzdGFydF90aW1lYCAod2hlbiBzZXQpLiBCZWZvcmUgdGhhdCwgY2FsbGVycwpyZWNlaXZlIFtgSHVudEVycm9yQ29kZTo6SHVudE5vdEFjdGl2ZWBdIHNvIHF1ZXN0aW9ucyBjYW5ub3QgYmUgcmVhZAphaGVhZCBvZiByZWdpc3RyYXRpb24gYW5kIHNvbHZlZCBvZmZsaW5lIHRvIGdhbWUgdGltZS1iYXNlZCBzY29yaW5nCmFuZCByZXdhcmQgdGllcnMuAAAAAAhnZXRfY2x1ZQAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAQAAA+kAAAfQAAAACENsdWVJbmZvAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAP5BZGRzIG11bHRpcGxlIGNsdWVzIHRvIGEgZHJhZnQgaHVudCBpbiBvbmUgaW52b2NhdGlvbi4gT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiBhZGQgY2x1ZXMuCgpUaGUgYmF0Y2ggaXMgdmFsaWRhdGVkIGFnYWluc3QgdGhlIHBlci1odW50IGNsdWUgY2FwIGJlZm9yZSB3cml0aW5nIGFueSBuZXcgY2x1ZXMsCnNvIGEgcmVxdWVzdCB0aGF0IHdvdWxkIGV4Y2VlZCB0aGUgbGltaXQgZmFpbHMgd2l0aG91dCBwYXJ0aWFsbHkgYWRkaW5nIGNsdWVzLgAAAAAACWFkZF9jbHVlcwAAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAFY2x1ZXMAAAAAAAPqAAAH0AAAAA5CYXRjaENsdWVJbnB1dAAAAAAAAQAAA+kAAAPqAAAABAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAANtCYW5zIGEgcGxheWVyIGZyb20gcGFydGljaXBhdGluZyBpbiBhIGh1bnQuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBiYW4gdGhlIHBsYXllciBmcm9tCiogYGNhbGxlcmAgLSBUaGUgaHVudCBjcmVhdG9yIG9yIHRoZSBjb250cmFjdCBhZG1pbgoqIGBwbGF5ZXJgIC0gVGhlIHBsYXllciB0byBiYW4AAAAACmJhbl9wbGF5ZXIAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAATZDcmVhdGVzIGEgbmV3IGRyYWZ0IGh1bnQgYnkgY29weWluZyBjbHVlcyBmcm9tIGFuIGV4aXN0aW5nIGNvbXBsZXRlZCBodW50LgoKQmFja3dhcmRzLWNvbXBhdGlibGUgd3JhcHBlcjogb2xkZXIgY2FsbGVycyBjYW4gc3RpbGwgY2xvbmUgYSBjb21wbGV0ZWQgaHVudCwgYnV0CnNlY3VyZSByZWhhc2hpbmcgcmVxdWlyZXMgYSBjYWxsZXItc3VwcGxpZWQgYW5zd2VyIGxpc3QuIFRoZSBleHBsaWNpdApgY2xvbmVfaHVudF93aXRoX2Fuc3dlcnNgIGVudHJ5IHBvaW50IHByZXNlcnZlcyBhbnN3ZXIgaXNvbGF0aW9uIGZvciBjbG9uZWQgY2x1ZXMuAAAAAAAKY2xvbmVfaHVudAAAAAAAAgAAAAAAAAAQdGVtcGxhdGVfaHVudF9pZAAAAAYAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAEAAAPpAAAABgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAABABGb3JjZS1jbG9zZXMgKGVuZHMgZWFybHkpIGFuIGluLXByb2dyZXNzIGh1bnQgb24gYmVoYWxmIG9mIGl0cyBjcmVhdG9yLgoKVW5saWtlIFtgY2FuY2VsX2h1bnRgXSwgY2xvc2luZyBwcmVzZXJ2ZXMgYWxsIHBsYXllciBzY29yZXMgYW5kIGFueQpyZXdhcmRzIGFscmVhZHkgY29sbGVjdGVkOiBpdCBtYXJrcyB0aGUgaHVudCBgQ29tcGxldGVkYCBhbmQgdHJpZ2dlcnMgYQpmaW5hbCByZXdhcmQgZGlzdHJpYnV0aW9uIGZvciBlbGlnaWJsZSBwbGF5ZXJzIHdobyBoYXZlIGNvbXBsZXRlZCB0aGUKaHVudCBidXQgaGF2ZSBub3QgeWV0IGNsYWltZWQuIFBsYXllcnMgd2hvIGhhdmUgbm90IGNvbXBsZXRlZCB0aGUgaHVudCwKb3Igd2hvc2UgZnJvemVuIGNvbXBsZXRpb24gcmFuayBpcyBvdXRzaWRlIGBtYXhfd2lubmVyc2AsIGtlZXAgdGhlaXIKcHJvZ3Jlc3MgYW5kIGFyZSBzaW1wbHkgbm90IHJld2FyZGVkLiBBbnkgdW5zcGVudCByZXdhcmQtcG9vbCBiYWxhbmNlIGlzCmxlZnQgaW50YWN0LiBbYGNhbmNlbF9odW50YF0gaXMgcmVqZWN0ZWQgb25jZSBhbnkgcGxheWVyIGhhcyBjb21wbGV0ZWQKKHVzZSB0aGlzIG1ldGhvZCBpbnN0ZWFkIHRvIHBheSB3aW5uZXJzKS4KCk9ubHkgdGhlIGNyZWF0b3IgbWF5IGNsb3NlIGEgaHVudCwgYW5kIG9ubHkgd2hpbGUgaXQgaXMgYEFjdGl2ZWAgb3IKYFBhdXNlZGAuIENsb3NpbmcgYSBgRHJhZnRgLCBgQ29tcGxldGVkYCwgYENhbmNlbGxlZGAsIGBFbWVyZ2VuY3lTdG9wcGVkYCwKb3IgYEFyY2hpdmVkYCBodW50IGlzIHJlamVjdGVkIHdpdGggYEludmFsaWRIdW50U3RhdHVzYC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIGNsb3NlCiogYGNhbGxlcmAgLSBUaGUgY3JlYXRvciAobXVzdCBhdXRob3JpemUgdGhlIGNhbGwgdmlhIHJlcXVpcmVfYXV0aCkKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzCgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkAAAACmNsb3NlX2h1bnQAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAHRSZXR1cm5zIHBhZ2luYXRlZCBjbHVlcyBmb3IgYSBodW50LiBBbnN3ZXIgaGFzaGVzIGFyZSBub3QgZXhwb3NlZC4KQSBgbGltaXRgIG9mIGAwYCBkZWZhdWx0cyB0byBgREVGQVVMVF9QQUdFX1NJWkVgLgAAAApsaXN0X2NsdWVzAAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+oAAAfQAAAACENsdWVJbmZv",
        "AAAAAAAAAFpSZXR1cm5zIGEgbGlzdCBvZiBhbGwgaHVudHMgKHBhZ2luYXRlZCkuCkEgYGxpbWl0YCBvZiBgMGAgZGVmYXVsdHMgdG8gYERFRkFVTFRfUEFHRV9TSVpFYC4AAAAAAApsaXN0X2h1bnRzAAAAAAACAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAH0AAAAARIdW50",
        "AAAAAAAAAAAAAAALY2FuY2VsX2h1bnQAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAABABDcmVhdGVzIGEgbmV3IHNjYXZlbmdlciBodW50IHdpdGggdGhlIHByb3ZpZGVkIG1ldGFkYXRhLgoKIyBBcmd1bWVudHMKKiBgZW52YCAtIFRoZSBTb3JvYmFuIGVudmlyb25tZW50CiogYGNyZWF0b3JgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGh1bnQgY3JlYXRvciAodHlwaWNhbGx5IHVzZSBlbnYuaW52b2tlcigpIGZyb20gdGhlIGNhbGxlcikKKiBgdGl0bGVgIC0gVGhlIHRpdGxlIG9mIHRoZSBodW50IChtYXggMjAwIGNoYXJhY3RlcnMpCiogYGRlc2NyaXB0aW9uYCAtIFRoZSBkZXNjcmlwdGlvbiBvZiB0aGUgaHVudCAobWF4IDIwMDAgY2hhcmFjdGVycykKKiBgc3RhcnRfdGltZWAgLSBPcHRpb25hbCBzdGFydCB0aW1lc3RhbXAgKDAgb3IgTm9uZSBtZWFucyBubyBzdGFydCB0aW1lIHJlc3RyaWN0aW9uKS4KV2hlbiBzZXQsIHBsYXllcnMgY2Fubm90IHJlZ2lzdGVyIG9yIHN1Ym1pdCBhbnN3ZXJzIHVudGlsIHRoZSBsZWRnZXIgdGltZXN0YW1wCnJlYWNoZXMgdGhpcyB2YWx1ZS4gTXVzdCBiZSBzdHJpY3RseSBsZXNzIHRoYW4gYGVuZF90aW1lYCBpZiBgZW5kX3RpbWVgIGlzIGFsc28gc2V0LgoqIGBlbmRfdGltZWAgLSBPcHRpb25hbCBlbmQgdGltZXN0YW1wICgwIG9yIE5vbmUgbWVhbnMgbm8gZW5kIHRpbWUgcmVzdHJpY3Rpb24pCiogYG1heF9zdWJtaXNzaW9uc19wZXJfbWludXRlYCAtIE1heGltdW0gbnVtYmVyIG9mIHN1Ym1pc3Npb25zIGFsbG93ZWQgcGVyCm1pbnV0ZSBwZXIgcGxheWVyLiBbYFVOTElNSVRFRF9TVUJNSVNTSU9OU19QRVJfTUlOVVRFYF0gKDApIG1lYW5zIG5vIGxpbWl0LgoKIyBSZXR1cm5zClRoZSB1bmlxdWUgaHVudCBJRCBvZiB0aGUgbmV3bHkgY3JlYXRlZCBodW50CgojIEVycm9ycwoqIGBJbnZhbGlkVGl0bGVgIC0gSWYgdGl0bGUgaXMgZW1wdHkgb3IgZXhjZWVkcyBtYXhpbXVtIGxlbmd0aAoqIGBJbnZhbGlkRGVzY3JpcHRpb25gIC0gSWYgZGVzY3JpcHRpb24gZXhjZWVkcyBtYXhpbXVtIGxlbmd0aAoqIGBJbnZhbGlkQWRkcmVzc2AgLSBJZiBjcmVhdG9yAAAAC2NyZWF0ZV9odW50AAAAAAgAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAFdGl0bGUAAAAAAAAQAAAAAAAAAAtkZXNjcmlwdGlvbgAAAAAQAAAAAAAAAApzdGFydF90aW1lAAAAAAPoAAAABgAAAAAAAAAIZW5kX3RpbWUAAAPoAAAABgAAAAAAAAAabWF4X3N1Ym1pc3Npb25zX3Blcl9taW51dGUAAAAAAAQAAAAAAAAAFHN0YXJ0X211bHRpcGxpZXJfYnBzAAAD6AAAAAQAAAAAAAAADmRlZmF1bHRfcG9pbnRzAAAAAAPoAAAABAAAAAEAAAPpAAAABgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAORTdGVwIHR3byBvZiBhIHR3by1zdGVwIGFkbWluIGtleSByb3RhdGlvbi4KClRoZSBwcm9wb3NlZCBuZXcgYWRtaW4gYWNjZXB0cyB0aGUgcm9sZSwgY29tcGxldGluZyB0aGUgcm90YXRpb24uIE9ubHkgdGhlCmFkZHJlc3Mgc3RvcmVkIGJ5IGBwcm9wb3NlX25ld19hZG1pbmAgbWF5IGFjY2VwdCwgc28gYSB3cm9uZyBwcm9wb3NhbCBjYW5ub3QKc2lsZW50bHkgdGFrZSBvdmVyIHRoZSBjb250cmFjdC4AAAAMYWNjZXB0X2FkbWluAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAMYXJjaGl2ZV9odW50AAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAMaXNfdmlld19vbmx5AAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdhZGRyZXNzAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAFBVbmxvY2tzIGEgY2x1ZSBoaW50IGZvciBhIHJlZ2lzdGVyZWQgcGxheWVyIGFuZCBkZWR1Y3RzIHRoZSBjbHVlJ3MgaGludCBwZW5hbHR5LgAAAAxyZXF1ZXN0X2hpbnQAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+kAAAAQAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAEtTZWFyY2hlcyBodW50cyBieSBwYXJ0aWFsIHRpdGxlIG1hdGNoIG92ZXIgYSBjYWxsZXItYm91bmRlZCBodW50LWlkIHdpbmRvdy4AAAAADHNlYXJjaF9odW50cwAAAAQAAAAAAAAAD3RpdGxlX3N1YnN0cmluZwAAAAAQAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAAAAAAKc2Nhbl9saW1pdAAAAAAABAAAAAEAAAPqAAAH0AAAAARIdW50",
        "AAAAAAAAANBVbmJhbnMgYSBwbGF5ZXIgZnJvbSBhIGh1bnQuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byB1bmJhbiB0aGUgcGxheWVyIGZyb20KKiBgY2FsbGVyYCAtIFRoZSBodW50IGNyZWF0b3Igb3IgdGhlIGNvbnRyYWN0IGFkbWluCiogYHBsYXllcmAgLSBUaGUgcGxheWVyIHRvIHVuYmFuAAAADHVuYmFuX3BsYXllcgAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAANYWN0aXZhdGVfaHVudAAAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAABABDb21wbGV0ZXMgYSBodW50IGZvciBhIHBsYXllciBhbmQgZGlzdHJpYnV0ZXMgcmV3YXJkcy4KClRoaXMgZnVuY3Rpb24gdmVyaWZpZXMgdGhhdCB0aGUgcGxheWVyIGhhcyBjb21wbGV0ZWQgYWxsIHJlcXVpcmVkIGNsdWVzLAp0aGVuIGRpc3RyaWJ1dGVzIHJld2FyZHMgdmlhIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IChpZiBjb25maWd1cmVkKQphbmQgdXBkYXRlcyB0aGUgcGxheWVyJ3MgcmV3YXJkIHN0YXR1cy4KClJld2FyZCBhbW91bnRzIGNhbiBiZSBmbGF0IChgeGxtX3Bvb2wgLyBtYXhfd2lubmVyc2ApLCB0aW1lLWJhc2VkCihjb25maWd1cmVkIHZpYSBgUmV3YXJkTWFuYWdlcjo6c2V0X3Bvb2xfdGllcnNgKSwgb3IgZXhhY3QtcmFuayBiYXNlZAooY29uZmlndXJlZCB2aWEgYFJld2FyZE1hbmFnZXI6OnNldF9wb29sX3JhbmtfdGllcnNgKS4gUmFuay1iYXNlZAphbW91bnRzIHVzZSB0aGUgY29tcGxldGlvbiByYW5rIGZyb3plbiBieSBIdW50eUNvcmUuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCBJRAoqIGBwbGF5ZXJgIC0gVGhlIHBsYXllciBjbGFpbWluZyBjb21wbGV0aW9uL3Jld2FyZHMKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzZnVsIHJld2FyZCBjbGFpbQoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgSW52YWxpZEh1bnRTdGF0dXNgIC0gSHVudCBpcyBub3QgQWN0aXZlIG9yIFBhdXNlZCAoZS5nLiBDb21wbGV0ZWQgb3IgQ2FuY2VsbGVkKQoqIGBQbGF5ZXJOb3RSZWdpc3RlcmVkYCAtIFBsYXllciBpcyBub3QgcmVnaXN0ZXJlZAoqIGBIdW50Tm90Q29tcGxldGVkYCAtIFBsYXllciBoYXNuJ3QgY29tcGxldGVkIGFsbCByZXF1aXJlZCBjbHVlcwoqIGBSZXdhcmRBbHJlYWR5Q2xhaW1lZGAgLSBQbGF5ZXIgYWxyZWFkeSBjbGFpbWVkIHRoZWlyIHJld2FyZAoqIGBOb1Jld2FyZHNDb25maWd1cmVkYCAtIE5vIHJld2FyZHMgc2V0IHVwAAAADWNvbXBsZXRlX2h1bnQAAAAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAANZ2V0X2h1bnRfaW5mbwAAAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAH0AAAAARIdW50AAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAANcGF1c2VfYW5zd2VycwAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAANcGF1c2VfcmV3YXJkcwAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAANcnVuX21pZ3JhdGlvbgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAAAAAAAB2RyeV9ydW4AAAAAAQAAAAEAAAPpAAAH0AAAAA9NaWdyYXRpb25SZXBvcnQAAAAH0AAAABBVcGdyYWRlQXV0aEVycm9y",
        "AAAAAAAAADJTZXRzIG9yIGNsZWFycyB0aGUgb3B0aW9uYWwgaGludCBmb3IgYSBkcmFmdCBjbHVlLgAAAAAADXNldF9jbHVlX2hpbnQAAAAAAAAFAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAARoaW50AAAD6AAAABAAAAAAAAAAE2hpbnRfcGVuYWx0eV9wb2ludHMAAAAABAAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAIRFbmFibGVzIG9yIGRpc2FibGVzIHRlYW0gZmVhdHVyZXMgZm9yIGEgZHJhZnQgaHVudC4KT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiBjYWxsIHRoaXMsIGFuZCBvbmx5IHdoaWxlIHRoZSBodW50IGlzIGluIERyYWZ0IHN0YXR1cy4AAAANc2V0X3RlYW1fbW9kZQAAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAl0ZWFtX21vZGUAAAAAAAABAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAYVWZXJpZmllcyBhIHN1Ym1pdHRlZCBhbnN3ZXIsIHJlY29yZGluZyB0aGUgYXR0ZW1wdCBlaXRoZXIgd2F5LgoKIyBSZXR1cm5zCmBPayh0cnVlKWAgd2hlbiB0aGUgYW5zd2VyIGlzIGNvcnJlY3QsIGBPayhmYWxzZSlgIHdoZW4gaXQgaXMgd3JvbmcuCgpBbiBpbmNvcnJlY3QgYW5zd2VyIGlzIHJlcG9ydGVkIGFzIGBPayhmYWxzZSlgIHJhdGhlciB0aGFuCmBFcnIoSW52YWxpZEFuc3dlcilgIHNvIHRoYXQgdGhlIGZhaWxlZCBhdHRlbXB0LCB0aGUgcGVyLWNsdWUgY29vbGRvd24KdGltZXN0YW1wIGFuZCB0aGUgY29uc3VtZWQgc3VibWlzc2lvbiBub25jZSBhcmUgY29tbWl0dGVkIGluc3RlYWQgb2YKcm9sbGVkIGJhY2suIFNlZSBgZmluYWxpemVfYW5zd2VyX3N1Ym1pc3Npb25gLgAAAAAAAA1zdWJtaXRfYW5zd2VyAAAAAAAABgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjbHVlX2lkAAAAAAQAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAGYW5zd2VyAAAAAAAQAAAAAAAAABBzdWJtaXNzaW9uX25vbmNlAAAABgAAAAAAAAAMc3VibWl0dGVkX2F0AAAABgAAAAEAAAPpAAAAAQAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAOYWRkX2NvX2NyZWF0b3IAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAA5uZXdfY29fY3JlYXRvcgAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAADZSZXR1cm5zIHRoZSB0b3RhbCBudW1iZXIgb2YgaHVudHMgY3JlYXRlZCAocmVhZC1vbmx5KS4AAAAAAA5nZXRfaHVudF9jb3VudAAAAAAAAAAAAAEAAAAG",
        "AAAAAAAAADFSZXR1cm5zIHRydWUgaWYgdGhlIGdpdmVuIGFkZHJlc3MgaXMgYmxhY2tsaXN0ZWQuAAAAAAAADmlzX2JsYWNrbGlzdGVkAAAAAAABAAAAAAAAAAdjcmVhdG9yAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAEhQYXVzZXMgYWxsIHBsYXllciBvcGVyYXRpb25zIChyZWdpc3RyYXRpb25zLCBhbnN3ZXJzLCByZXdhcmRzKSBnbG9iYWxseS4AAAAOcGF1c2VfY29udHJhY3QAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAVZWZXJpZmllcyBhIGNhbmRpZGF0ZSBhbnN3ZXIgZm9yIGEgcmVnaXN0ZXJlZCBwbGF5ZXIgd2l0aCBhdXRob3JpemF0aW9uIGFuZCByYXRlIGxpbWl0aW5nLgoKVW5saWtlIGBzdWJtaXRfYW5zd2VyYCwgYHByZXZpZXdfYW5zd2VyYCBkb2VzIG5vdCBtYXJrIHRoZSBjbHVlIGFzIGNvbXBsZXRlZCwgYXdhcmQgcG9pbnRzLApvciBlbWl0IGNsdWUgY29tcGxldGlvbiBldmVudHMuIEl0IHN0aWxsIHJlcXVpcmVzIHBsYXllciBhdXRob3JpemF0aW9uIGFuZCBlbmZvcmNlcyB0aGUKc2FtZSBwZXItbWludXRlIHJhdGUgbGltaXQsIHBlci1jbHVlIGF0dGVtcHQgY2FwLCBhbmQgYXR0ZW1wdCBjb29sZG93bi4AAAAAAA5wcmV2aWV3X2Fuc3dlcgAAAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjbHVlX2lkAAAAAAQAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAGYW5zd2VyAAAAAAAQAAAAAQAAA+kAAAABAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAPZGVhY3RpdmF0ZV9odW50AAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAPZ2V0X2NvX2NyZWF0b3JzAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAPZ2V0X3BhdXNlX3N0YXRlAAAAAAAAAAABAAAD7QAAAAMAAAABAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAPcmVnaXN0ZXJfcGxheWVyAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAGBTZXRzIHRoZSBtYXhpbXVtIHBsYXllcnMgZm9yIGEgaHVudC4gT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiBzZXQgaXQsIGFuZCBvbmx5IGluIERyYWZ0IHN0YXR1cy4AAAAPc2V0X21heF9wbGF5ZXJzAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAttYXhfcGxheWVycwAAAAAEAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAPdW5wYXVzZV9hbnN3ZXJzAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAPdW5wYXVzZV9yZXdhcmRzAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAqJBZGRzIGFsdGVybmF0aXZlIGFjY2VwdGFibGUgYW5zd2VycyB0byBhbiBleGlzdGluZyBjbHVlIChzeW5vbnltcykuCk9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gYWRkIGFsaWFzZXMsIGFuZCBvbmx5IHdoaWxlIHRoZSBodW50IGlzIGluIERyYWZ0IHN0YXR1cy4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IGNvbnRhaW5pbmcgdGhlIGNsdWUKKiBgY2x1ZV9pZGAgLSBUaGUgZXhpc3RpbmcgY2x1ZSB0byBhZGQgYWxpYXNlcyB0bwoqIGBhbnN3ZXJzYCAtIEFsdGVybmF0aXZlIGFuc3dlcnMgdGhhdCBzaG91bGQgYWxzbyBiZSBhY2NlcHRlZAoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgSW52YWxpZEh1bnRTdGF0dXNgIC0gSHVudCBpcyBub3QgaW4gRHJhZnQKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvcgoqIGBDbHVlTm90Rm91bmRgIC0gQ2x1ZSBkb2VzIG5vdCBleGlzdAoqIGBJbnZhbGlkQW5zd2VyYCAtIEFueSBhbnN3ZXIgaXMgZW1wdHkgb3IgZXhjZWVkcyBtYXggbGVuZ3RoCiogYFRvb01hbnlBbGlhc2VzYCAtIEFkZGluZyB0aGUgYWxpYXNlcyB3b3VsZCBleGNlZWQgYE1BWF9BTElBU0VTX1BFUl9DTFVFYAAAAAAAEGFkZF9jbHVlX2FsaWFzZXMAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAHYW5zd2VycwAAAAPqAAAAEAAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAFhTZXRzIHRoZSBjb250cmFjdCBhZG1pbiBvbmNlLiBTdWJzZXF1ZW50IGNhbGxzIHJlcXVpcmUgY3VycmVudCBhZG1pbiBhdXRoIHZpYSBzZXRfYWRtaW4uAAAAEGluaXRpYWxpemVfYWRtaW4AAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAQtSZXR1cm5zIHdoZXRoZXIgdGhlIGh1bnQgaXMgaW4gYSB0ZXJtaW5hbCBzdGF0ZS4KCkEgaHVudCBpcyB0ZXJtaW5hbCBvbmNlIGl0IGNhbiBubyBsb25nZXIgYWNjZXB0IG5ldyBwbGF5IG9yIGJlCnJlYWN0aXZhdGVkOiBgQ29tcGxldGVkYCwgYENhbmNlbGxlZGAsIG9yIGBBcmNoaXZlZGAuIFRoaXMgdmlldyBpcwpjb25zdW1lZCBieSB0aGUgcmV3YXJkIG1hbmFnZXIgdG8gZGVjaWRlIHdoZXRoZXIgYSBwb29sIG1heSBiZQpyZWZ1bmRlZCB0byBpdHMgY3JlYXRvci4AAAAAEGlzX2h1bnRfdGVybWluYWwAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAD6QAAAAEAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAoVTZXRzIHdoZXRoZXIgYSBodW50IGlzIHByaXZhdGUgKGludml0ZS1vbmx5KS4KCk9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gY2FsbCB0aGlzLCBhbmQgb25seSB3aGlsZSB0aGUgaHVudCBpcyBpbiBEcmFmdCBzdGF0dXMuCldoZW4gbWFraW5nIGEgaHVudCBwcml2YXRlLCBhbiBpbnZpdGUgY29kZSBtdXN0IGFscmVhZHkgYmUgY29uZmlndXJlZCB2aWEKYGdlbmVyYXRlX2ludml0ZV9jb2RlYCBiZWZvcmUgdGhlIGh1bnQgY2FuIGJlIGFjdGl2YXRlZC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHVwZGF0ZSBwcml2YWN5IGZvcgoqIGBjcmVhdG9yYCAtIFRoZSBodW50IGNyZWF0b3IgKG11c3QgYXV0aG9yaXplIHRoZSBjYWxsKQoqIGBpc19wcml2YXRlYCAtIFdoZXRoZXIgdGhlIGh1bnQgc2hvdWxkIGJlIGludml0ZS1vbmx5CgojIFJldHVybnMKYE9rKCgpKWAgb24gc3VjY2VzcwoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvcgoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBEcmFmdCBzdGF0dXMAAAAAAAAQc2V0X2h1bnRfcHJpdmFjeQAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAppc19wcml2YXRlAAAAAAABAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAB5SZXN1bWVzIGFsbCBwbGF5ZXIgb3BlcmF0aW9ucy4AAAAAABB1bnBhdXNlX2NvbnRyYWN0AAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAGBCbGFja2xpc3RzIGEgY3JlYXRvciBhZGRyZXNzLCBwcmV2ZW50aW5nIHRoZW0gZnJvbSBjcmVhdGluZyBuZXcgaHVudHMuCkNhbGxlciBtdXN0IGJlIHRoZSBhZG1pbi4AAAARYmxhY2tsaXN0X2NyZWF0b3IAAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAARZ2V0X2FjdGl2ZV9hbGVydHMAAAAAAAAAAAAAAQAAA+oAAAfQAAAAC0hlYWx0aEFsZXJ0AA==",
        "AAAAAAAAAB9FeHBvc2VzIHRoZSBlbmQgdGltZSBvZiBhIGh1bnQuAAAAABFnZXRfaHVudF9lbmRfdGltZQAAAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAABgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9zY2hlbWEAAAAAAAAAAAAAAA==",
        "AAAAAAAAAVRTdGVwIG9uZSBvZiBhIHR3by1zdGVwIGFkbWluIGtleSByb3RhdGlvbi4KClRoZSBjdXJyZW50IGFkbWluIHByb3Bvc2VzIGEgbmV3IGFkbWluLiBUaGUgY2hhbmdlIGlzIE5PVCBhcHBsaWVkIHVudGlsIHRoZQpwcm9wb3NlZCBhZGRyZXNzIGNhbGxzIGBhY2NlcHRfYWRtaW5gLCB3aGljaCBwcmV2ZW50cyBhY2NpZGVudGFsIGxvY2tvdXQ6IGEKdHlwbyBpbiBgcHJvcG9zZV9uZXdfYWRtaW5gIGNhbiBzaW1wbHkgYmUgb3ZlcndyaXR0ZW4gb3IgaWdub3JlZCwgYW5kIHRoZQpjdXJyZW50IGFkbWluIG5ldmVyIGxvc2VzIGFjY2VzcyB1bnRpbCB0aGUgbmV3IGFkbWluIGFjdGl2ZWx5IGFjY2VwdHMuAAAAEXByb3Bvc2VfbmV3X2FkbWluAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAARcmVtb3ZlX2NvX2NyZWF0b3IAAAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAUY29fY3JlYXRvcl90b19yZW1vdmUAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAARVTZXRzIHRoZSByZXdhcmQgY29uZmlndXJhdGlvbiBmb3IgYSBodW50LgpPbmx5IHRoZSBodW50IGNyZWF0b3IgKG9yIGEgY28tY3JlYXRvcikgbWF5IGRvIHRoaXMsIGFuZCBvbmx5IHdoaWxlIHRoZQpodW50IGlzIHN0aWxsIGluIGBEcmFmdGAg4oCUIHJld2FyZCBwYXJhbWV0ZXJzIG11c3Qgbm90IGJlIG11dGFibGUgb25jZQpwbGF5ZXJzIGNhbiByZWdpc3RlciAoIzEwMTIpLgpTZXRzIG5mdF9pbWFnZV91cmkgdG8gYSBwbGFjZWhvbGRlciB3aGVuIG5mdF9lbmFibGVkIGlzIHRydWUuAAAAAAAAEXNldF9yZXdhcmRfY29uZmlnAAAAAAAABgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAttYXhfd2lubmVycwAAAAAEAAAAAAAAAAh4bG1fcG9vbAAAAAsAAAAAAAAAC25mdF9lbmFibGVkAAAAAAEAAAAAAAAADG5mdF9jb250cmFjdAAAA+gAAAATAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAASZ2V0X3NjaGVtYV92ZXJzaW9uAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAASZ2V0X3ZpZXdfb25seV9saXN0AAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+oAAAAT",
        "AAAAAAAAADRSZXR1cm5zIHdoZXRoZXIgdGhlIGdsb2JhbCBjb250cmFjdCBwYXVzZSBpcyBhY3RpdmUuAAAAEmlzX2NvbnRyYWN0X3BhdXNlZAAAAAAAAAAAAAEAAAAB",
        "AAAAAAAAAe9DbGVhcnMgdGhlIGludml0ZSBjb2RlIGZvciBhIHByaXZhdGUgaHVudCwgZWZmZWN0aXZlbHkgcGF1c2luZyBuZXcgcmVnaXN0cmF0aW9ucy4KVGhlIGh1bnQgY3JlYXRvciBjYW4gZ2VuZXJhdGUgYSBuZXcgY29kZSBsYXRlciB2aWEgYGdlbmVyYXRlX2ludml0ZV9jb2RlYC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHJldm9rZSB0aGUgaW52aXRlIGNvZGUgZm9yCiogYGNyZWF0b3JgIC0gVGhlIGh1bnQgY3JlYXRvciAobXVzdCBhdXRob3JpemUgdGhlIGNhbGwpCgojIFJldHVybnMKYE9rKCgpKWAgb24gc3VjY2VzcwoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvcgoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBEcmFmdCBzdGF0dXMAAAAAEnJldm9rZV9pbnZpdGVfY29kZQAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAScm9sbGJhY2tfbWlncmF0aW9uAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AAAAB9AAAAAQVXBncmFkZUF1dGhFcnJvcg==",
        "AAAAAAAAAE9TZXRzIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IGFkZHJlc3MgZm9yIGNyb3NzLWNvbnRyYWN0IHJld2FyZCBkaXN0cmlidXRpb24uAAAAABJzZXRfcmV3YXJkX21hbmFnZXIAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOcmV3YXJkX21hbmFnZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAX9SZXR1cm5zIHRoZSBsaXN0IG9mIGNsdWUgSURzIHRoYXQgdGhlIHBsYXllciBoYXMgY29tcGxldGVkIGZvciBhIGh1bnQgKHJlYWQtb25seSkuClVzZWZ1bCBmb3IgVUkgdG8gc2hvdyBwcm9ncmVzcy4gUmV0dXJucyBlbXB0eSB2ZWMgaWYgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkLgoKVGhpbiBiYWNrd2FyZHMtY29tcGF0aWJsZSB3cmFwcGVyOiByZXR1cm5zIGF0IG1vc3QgYE1BWF9DTFVFU19QRVJfSFVOVGAKZW50cmllcywgc2luY2UgYGFkZF9jbHVlYCAvIGBhZGRfY2x1ZXNfYmF0Y2hgIGJvdW5kIGEgaHVudCdzIGNsdWUgc2V0IGJ5CnRoYXQgc2FtZSBjb25zdGFudC4gUHJlZmVyIGBnZXRfY29tcGxldGVkX2NsdWVzX3BhZ2luYXRlZGAgZm9yIG5ldyBjYWxsZXJzLgAAAAATZ2V0X2NvbXBsZXRlZF9jbHVlcwAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAEAAAPqAAAABA==",
        "AAAAAAAAAIlSZXR1cm5zIGFnZ3JlZ2F0ZSBzdGF0aXN0aWNzIGZvciBhIGh1bnQgKHJlYWQtb25seSk6IHRvdGFsIHBsYXllcnMsIGNvbXBsZXRpb24gcmF0ZSwgYXZlcmFnZSBzY29yZS4KUmV0dXJucyBlcnJvciBpZiBodW50IGRvZXMgbm90IGV4aXN0LgAAAAAAABNnZXRfaHVudF9zdGF0aXN0aWNzAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAH0AAAAA5IdW50U3RhdGlzdGljcwAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAJJSZXR1cm5zIHBsYXllciBwcm9ncmVzcyBmb3IgYSBodW50IChyZWFkLW9ubHkpLgpJbmNsdWRlcyBjb21wbGV0ZWQgY2x1ZXMsIHNjb3JlLCBhbmQgY29tcGxldGlvbiBzdGF0dXMuClJldHVybnMgZXJyb3IgaWYgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkLgAAAAAAE2dldF9wbGF5ZXJfcHJvZ3Jlc3MAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAB9AAAAAOUGxheWVyUHJvZ3Jlc3MAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAATaXNfZ2xvYmFsX3ZpZXdfb25seQAAAAABAAAAAAAAAAdhZGRyZXNzAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAATcGF1c2VfcmVnaXN0cmF0aW9ucwAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAElVcGRhdGVzIGNhdGVnb3JpZXMgZm9yIGEgZHJhZnQgaHVudC4gQXQgbW9zdCBmaXZlIGNhdGVnb3JpZXMgYXJlIGFsbG93ZWQuAAAAAAAAE3NldF9odW50X2NhdGVnb3JpZXMAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAACmNhdGVnb3JpZXMAAAAAA+oAAAAQAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAUYWRkX2dsb2JhbF92aWV3X29ubHkAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABnZpZXdlcgAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAUYWRkX3ZpZXdfb25seV9hY2Nlc3MAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAGdmlld2VyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAuxHZW5lcmF0ZXMgb3IgdXBkYXRlcyB0aGUgaW52aXRlIGNvZGUgZm9yIGEgcHJpdmF0ZSBodW50LgoKVGhlIGludml0ZSBjb2RlIGlzIGhhc2hlZCB3aXRoIFNIQTI1NiAodXNpbmcgaHVudF9pZCBhcyBzYWx0KSBhbmQgb25seSB0aGUgaGFzaAppcyBzdG9yZWQgb24tY2hhaW4uIFRoZSBwbGFpbi10ZXh0IGNvZGUgaXMgbmV2ZXIgcGVyc2lzdGVkIG9yIGVtaXR0ZWQgaW4gZXZlbnRzLgpDYWxsaW5nIHRoaXMgZnVuY3Rpb24gb3ZlcndyaXRlcyBhbnkgcHJldmlvdXNseSBzZXQgaW52aXRlIGNvZGUuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBnZW5lcmF0ZSBhbiBpbnZpdGUgY29kZSBmb3IKKiBgY3JlYXRvcmAgLSBUaGUgaHVudCBjcmVhdG9yIChtdXN0IGF1dGhvcml6ZSB0aGUgY2FsbCkKKiBgaW52aXRlX2NvZGVgIC0gVGhlIHBsYWluLXRleHQgaW52aXRlIGNvZGUgdG8gaGFzaCBhbmQgc3RvcmUKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzCgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkb2VzIG5vdCBleGlzdAoqIGBVbmF1dGhvcml6ZWRgIC0gQ2FsbGVyIGlzIG5vdCB0aGUgaHVudCBjcmVhdG9yCiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IGluIERyYWZ0IHN0YXR1cwoqIGBJbnZhbGlkQW5zd2VyYCAtIEludml0ZSBjb2RlIGlzIGVtcHR5IG9yIGV4Y2VlZHMgMjU2IGJ5dGVzAAAAFGdlbmVyYXRlX2ludml0ZV9jb2RlAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAC2ludml0ZV9jb2RlAAAAABAAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAUZ2V0X2hlYWx0aF9kYXNoYm9hcmQAAAAAAAAAAQAAB9AAAAAOQ29udHJhY3RIZWFsdGgAAA==",
        "AAAAAAAAAXxSZXR1cm5zIHJhbmtlZCBwbGF5ZXJzIGZvciBhIGh1bnQgd2l0aCBwYWdpbmF0aW9uIHN1cHBvcnQgKHJlYWQtb25seSkuClNvcnRlZCBieSBzY29yZSBkZXNjZW5kaW5nLCB0aGVuIGJ5IGNvbXBsZXRpb24gdGltZSBhc2NlbmRpbmcgKGVhcmxpZXIgPSBiZXR0ZXIpLgpMaW1pdCBpcyBjYXBwZWQgYXQgMjAgdG8gY29udHJvbCBnYXMuIFJldHVybnMgZXJyb3IgaWYgaHVudCBkb2VzIG5vdCBleGlzdC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHF1ZXJ5CiogYGxpbWl0YCAtIE1heGltdW0gZW50cmllcyB0byByZXR1cm4gKGNhcHBlZCBhdCBgTUFYX0xFQURFUkJPQVJEX1NJWkVgKQAAABRnZXRfaHVudF9sZWFkZXJib2FyZAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+kAAAfQAAAAEUxlYWRlcmJvYXJkUmVzdWx0AAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAQ9SZXR1cm5zIGEgcGFnaW5hdGVkIHNsaWNlIG9mIGNsdWVzIGZvciBhIGh1bnQuIFVzZWZ1bCBmb3IgbGFyZ2UgaHVudHMgdG8gYm91bmQgZ2FzLgpQYWdlIGlzIDAtaW5kZXhlZC4gTWF4IHBhZ2Vfc2l6ZSBpcyBjYXBwZWQgYXQgTUFYX0JBVENIX1NJWkUgKDUwKS4KQSBgcGFnZV9zaXplYCBvZiBgMGAgZGVmYXVsdHMgdG8gYERFRkFVTFRfUEFHRV9TSVpFYC4KRXN0aW1hdGVkIGdhczogTyhwYWdlX3NpemUpIH41XzAwMCBnYXMgcGVyIGNsdWUgKyAxMF8wMDAgb3ZlcmhlYWQuAAAAABRsaXN0X2NsdWVzX3BhZ2luYXRlZAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAEcGFnZQAAAAQAAAAAAAAACXBhZ2Vfc2l6ZQAAAAAAAAQAAAABAAAD6gAAB9AAAAAIQ2x1ZUluZm8=",
        "AAAAAAAAAzJSZWdpc3RlcnMgYSBwbGF5ZXIgZm9yIGEgcHJpdmF0ZSBodW50IHVzaW5nIGEgdmFsaWQgaW52aXRlIGNvZGUuCgpUaGUgcHJvdmlkZWQgaW52aXRlIGNvZGUgaXMgaGFzaGVkICh3aXRoIGh1bnRfaWQgYXMgc2FsdCkgYW5kIGNvbXBhcmVkIGFnYWluc3QKdGhlIHN0b3JlZCBgaW52aXRlX2NvZGVfaGFzaGAuIElmIHRoZXkgbWF0Y2gsIHRoZSBwbGF5ZXIgaXMgcmVnaXN0ZXJlZC4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBwcml2YXRlIGh1bnQgdG8gcmVnaXN0ZXIgZm9yCiogYHBsYXllcmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgcGxheWVyIChtdXN0IGF1dGhvcml6ZSB0aGUgY2FsbCB2aWEgcmVxdWlyZV9hdXRoKQoqIGBpbnZpdGVfY29kZWAgLSBUaGUgcGxhaW4tdGV4dCBpbnZpdGUgY29kZSB0byB2YWxpZGF0ZQoKIyBSZXR1cm5zCmBPaygoKSlgIG9uIHN1Y2Nlc3MKCiMgRXJyb3JzCiogYEh1bnROb3RGb3VuZGAgLSBIdW50IGRvZXMgbm90IGV4aXN0CiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IGluIEFjdGl2ZSBzdGF0dXMsIGlzIG5vdCBwcml2YXRlICh1c2UKYHJlZ2lzdGVyX3BsYXllcmAgaW5zdGVhZCksIG9yIGhhcyBubyBpbnZpdGUgY29kZSBjb25maWd1cmVkCiogYEludmFsaWRBbnN3ZXJgIC0gVGhlIGludml0ZSBjb2RlIGlzIGVtcHR5LCBleGNlZWRzIDI1NiBieXRlcywgb3IgZG9lcyBub3QgbWF0Y2gKKiBgRHVwbGljYXRlUmVnaXN0cmF0aW9uYCAtIFBsYXllciBpcyBhbHJlYWR5IHJlZ2lzdGVyZWQgZm9yIHRoaXMgaHVudAAAAAAAFHJlZ2lzdGVyX3dpdGhfaW52aXRlAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAAC2ludml0ZV9jb2RlAAAAABAAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAKxCb290c3RyYXAgb3IgdHJhbnNmZXIgdGhlIHJhdGUtbGltaXQgYWRtaW4gcm9sZS4KClRoZSBmaXJzdCBjYWxsIHNldHMgdGhlIGFkbWluIHdpdGggbm8gcHJpb3ItYWRtaW4gY2hlY2suIFN1YnNlcXVlbnQKY2FsbHMgcmVxdWlyZSBgY2FsbGVyYCB0byBhbHJlYWR5IGJlIHRoZSBzdG9yZWQgYWRtaW4uAAAAFHNldF9yYXRlX2xpbWl0X2FkbWluAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAEFSZXR1cm5zIGh1bnRzIHdob3NlIGNhdGVnb3JpZXMgaW5jbHVkZSB0aGUgZXhhY3QgY2F0ZWdvcnkgc3RyaW5nLgAAAAAAABVnZXRfaHVudHNfYnlfY2F0ZWdvcnkAAAAAAAAEAAAAAAAAAAhjYXRlZ29yeQAAABAAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAAAAAApzY2FuX2xpbWl0AAAAAAAEAAAAAQAAA+oAAAfQAAAABEh1bnQ=",
        "AAAAAAAAAGhSZW1vdmVzIGEgY3JlYXRvciBmcm9tIHRoZSBibGFja2xpc3QsIHJlc3RvcmluZyB0aGVpciBhYmlsaXR5IHRvIGNyZWF0ZSBodW50cy4KQ2FsbGVyIG11c3QgYmUgdGhlIGFkbWluLgAAABVyZW1vdmVfZnJvbV9ibGFja2xpc3QAAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAVc2V0X3RpbWVfYm9udXNfY29uZmlnAAAAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAEXRpbWVfYm9udXNfY29uZmlnAAAAAAAD6AAAB9AAAAAPVGltZUJvbnVzQ29uZmlnAAAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAVdW5wYXVzZV9yZWdpc3RyYXRpb25zAAAAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAKdBZG1pbi1vbmx5OiBvdmVycmlkZSB0aGUgZGFpbHkgaHVudC1jcmVhdGlvbiBsaW1pdCBmb3IgYSBzcGVjaWZpYyBjcmVhdG9yLgoKUGFzcyBgbGltaXQgPSAwYCB0byByZW1vdmUgYW4gZXhpc3Rpbmcgb3ZlcnJpZGUsIGZhbGxpbmcgYmFjayB0byB0aGUKY29udHJhY3Qtd2lkZSBkZWZhdWx0LgAAAAAWc2V0X2NyZWF0b3JfaHVudF9saW1pdAAAAAAAAwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAJxTZWN1cmUgY2xvbmUgcGF0aCB0aGF0IHJlaGFzaGVzIGNsb25lZCBjbHVlcyBhZ2FpbnN0IHRoZSBuZXcgaHVudC9jbHVlIGNvbnRleHQuClRoZSBjcmVhdG9yIG11c3Qgc3VwcGx5IHRoZSBwbGFpbnRleHQgYW5zd2VycyBmb3IgZWFjaCBjbHVlIGluIHRoZSB0ZW1wbGF0ZS4AAAAXY2xvbmVfaHVudF93aXRoX2Fuc3dlcnMAAAAAAwAAAAAAAAAQdGVtcGxhdGVfaHVudF9pZAAAAAYAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAHYW5zd2VycwAAAAPqAAAAEAAAAAEAAAPpAAAABgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAXcmVtb3ZlX2dsb2JhbF92aWV3X29ubHkAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZ2aWV3ZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAAAAAAAXcmVtb3ZlX3ZpZXdfb25seV9hY2Nlc3MAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAABnZpZXdlcgAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAMdWYXJpYW50IG9mIGBzdWJtaXRfYW5zd2VyYCB0aGF0IGFjY2VwdHMgYSBwcmVjb21wdXRlZCBTSEEyNTYgYW5zd2VyIGhhc2guCgpTaGFyZXMgdGhlIGluY29ycmVjdC1hbnN3ZXIgc2VtYW50aWNzIG9mIGBzdWJtaXRfYW5zd2VyYDogYSB3cm9uZyBhbnN3ZXIKcmV0dXJucyBgT2soZmFsc2UpYCBhbmQgY29tbWl0cyB0aGUgZmFpbGVkIGF0dGVtcHQuAAAAABdzdWJtaXRfYW5zd2VyX3dpdGhfaGFzaAAAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAAthbnN3ZXJfaGFzaAAAAAPuAAAAIAAAAAAAAAAQc3VibWlzc2lvbl9ub25jZQAAAAYAAAAAAAAADHN1Ym1pdHRlZF9hdAAAAAYAAAABAAAD6QAAAAEAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAG1VcGRhdGVzIGEgaHVudCdzIGRlc2NyaXB0aW9uLiBPbmx5IHRoZSBodW50IGNyZWF0b3IgY2FuIHVwZGF0ZSBpdCwgYW5kIGl0IGNhbiBiZSB1cGRhdGVkIGZvciBhbnkgaHVudCBzdGF0dXMuAAAAAAAAF3VwZGF0ZV9odW50X2Rlc2NyaXB0aW9uAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAtkZXNjcmlwdGlvbgAAAAAQAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAAZZ2V0X2dsb2JhbF92aWV3X29ubHlfbGlzdAAAAAAAAAIAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+oAAAAT",
        "AAAAAAAAAItFbmFibGVzIG9yIGRpc2FibGVzIHBhcnRpYWwtc2NvcmUgY2xhaW1zIGZvciBhIGRyYWZ0IGh1bnQuCk9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gY2FsbCB0aGlzLCBhbmQgb25seSB3aGlsZSB0aGUgaHVudCBpcyBpbiBEcmFmdCBzdGF0dXMuAAAAABlzZXRfYWxsb3dfcGFydGlhbF9zY29yaW5nAAAAAAAAAwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAFWFsbG93X3BhcnRpYWxfc2NvcmluZwAAAAAAAAEAAAABAAAD6QAAAAIAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAJpVcGRhdGVzIHRoZSBtYXhpbXVtIG51bWJlciBvZiBhdHRlbXB0cyBhbGxvd2VkIHBlciBjbHVlIGFuZCBhdHRlbXB0IGNvb2xkb3duIGR1cmF0aW9uIGZvciBhIGRyYWZ0IGh1bnQuCk9ubHkgdGhlIGh1bnQgY3JlYXRvciBvciBjby1jcmVhdG9yIGNhbiB1cGRhdGUgaXQuAAAAAAAZc2V0X21heF9hdHRlbXB0c19wZXJfY2x1ZQAAAAAAAAQAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAABVtYXhfYXR0ZW1wdHNfcGVyX2NsdWUAAAAAAAAEAAAAAAAAABVhdHRlbXB0X2Nvb2xkb3duX3NlY3MAAAAAAAAEAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAKtTZXRzIHRoZSByZWdpc3RyYXRpb24gY3V0b2ZmIHRpbWVzdGFtcCBmb3IgYSBkcmFmdCBodW50LiBBIHZhbHVlIG9mIDAgZGlzYWJsZXMgdGhlIGN1dG9mZi4KT25seSB0aGUgaHVudCBjcmVhdG9yIGNhbiBjYWxsIHRoaXMsIGFuZCBvbmx5IHdoaWxlIHRoZSBodW50IGlzIGluIERyYWZ0IHN0YXR1cy4AAAAAGXNldF9yZWdpc3RyYXRpb25fZGVhZGxpbmUAAAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAVcmVnaXN0cmF0aW9uX2RlYWRsaW5lAAAAAAAABgAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAALpSZXBvcnRzIGhvdyBtdWNoIHN0b3JhZ2UgYSBodW50IGN1cnJlbnRseSBvY2N1cGllcywgd2l0aG91dCByZW1vdmluZwphbnl0aGluZy4gUmVhZC1vbmx5LCBzbyBpdCBuZWVkcyBubyBhdXRob3JpemF0aW9uIOKAlCBodW50IGV4aXN0ZW5jZSBhbmQKc2l6ZSBhcmUgYWxyZWFkeSBwdWJsaWMgdmlhIGBnZXRfaHVudF9pbmZvYC4AAAAAABpnZXRfaHVudF9zdG9yYWdlX2Zvb3RwcmludAAAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAB9AAAAAIR2NSZXBvcnQ=",
        "AAAAAAAAAilTY2FucyBhIGJvdW5kZWQgd2luZG93IG9mIHJlZ2lzdGVyZWQgcGxheWVycyBmb3IgYSBodW50IGFuZCByZXR1cm5zCnRoZWlyIGNvbXBhY3Qgcm93cy4gVGhpcyBtZXRob2QgZW5hYmxlcyBjbGllbnRzIHRvIHBhZ2UgdGhyb3VnaCBhbGwKcmVnaXN0ZXJlZCBwbGF5ZXJzIGluIG11bHRpcGxlIGNhbGxzIChib3VuZGVkIGJ5IGBNQVhfTEVBREVSQk9BUkRfU0NBTl9TSVpFYCkKYW5kIG1lcmdlIHJlc3VsdHMgb2ZmLWNoYWluIHRvIGJ1aWxkIGEgZnVsbCBsZWFkZXJib2FyZCB3aXRob3V0IGEgc2luZ2xlCmxhcmdlIG9uLWNoYWluIHNjYW4uIE9ubHkgdGhlIHJlcXVlc3RlZCByZWdpc3RyYXRpb24gc2xpY2UgaXMgcmVhZCwgc28KdGhlIGNvc3Qgb2YgYSBwYWdlIGRlcGVuZHMgb24gYHdpbmRvd19zaXplYCwgbm90IG9uIGhvdyBtYW55IHBsYXllcnMgdGhlCmh1bnQgaGFzLiBUaGlzIHJlYWQgcGF0aCBpcyBwdWJsaWM7IHRoZSBgX2NhbGxlcmAgYXJndW1lbnQgaXMKYWNjZXB0ZWQgZm9yIGZvcndhcmQgY29tcGF0aWJpbGl0eSBhbmQgaXMgY3VycmVudGx5IGlnbm9yZWQuAAAAAAAAG2dldF9odW50X2xlYWRlcmJvYXJkX3dpbmRvdwAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAC3N0YXJ0X2luZGV4AAAAAAQAAAAAAAAAC3dpbmRvd19zaXplAAAAAAQAAAAAAAAABmNhbGxlcgAAAAAD6AAAABMAAAABAAAD6QAAB9AAAAARTGVhZGVyYm9hcmRXaW5kb3cAAAAAAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAUNSZXR1cm5zIHdoZXRoZXIgdGhlIGh1bnQgaXMgZXhwaXJlZCBvciBjYW5jZWxsZWQuCgpBIGh1bnQgaXMgY29uc2lkZXJlZCBleHBpcmVkIHdoZW4gaXQgaGFzIGFuIGBlbmRfdGltZWAgc2V0IGFuZCB0aGUKY3VycmVudCBsZWRnZXIgdGltZXN0YW1wIGlzIGF0IG9yIHBhc3QgdGhhdCBlbmQgdGltZS4gQSBodW50IGlzCmNhbmNlbGxlZCB3aGVuIGl0cyBzdGF0dXMgaXMgYENhbmNlbGxlZGAuIFRoaXMgdmlldyBpcyBjb25zdW1lZCBieSB0aGUKcmV3YXJkIG1hbmFnZXIgdG8gZGVjaWRlIHdoZXRoZXIgYSBwb29sIG1heSBiZSBtaWdyYXRlZCB0byBhIG5ldyBodW50LgAAAAAcaXNfaHVudF9leHBpcmVkX29yX2NhbmNlbGxlZAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAAAQAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAvNTZXRzIG9yIGNsZWFycyBhIG1hbnVhbCBodW50IGRpZmZpY3VsdHkgb3ZlcnJpZGUuIFdpdGhvdXQgYW4gb3ZlcnJpZGUsCnRoZSByYXRpbmcgaXMgdGhlIGF2ZXJhZ2UgY2x1ZSBkaWZmaWN1bHR5LgoKT25seSB0aGUgaHVudCBjcmVhdG9yIG9yIGEgY28tY3JlYXRvciBjYW4gY2hhbmdlIHRoZSBvdmVycmlkZSwgYW5kIG9ubHkKd2hpbGUgdGhlIGh1bnQgaXMgaW4gRHJhZnQgc3RhdHVzLgoKIyBBcmd1bWVudHMKKiBgZW52YCAtIFRoZSBTb3JvYmFuIGVudmlyb25tZW50CiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgdG8gY29uZmlndXJlCiogYGNhbGxlcmAgLSBUaGUgY3JlYXRvciBvciBjby1jcmVhdG9yIG1ha2luZyB0aGUgY2hhbmdlCiogYGRpZmZpY3VsdHlfb3ZlcnJpZGVgIC0gYFNvbWUodmFsdWUpYCB0byBzZXQsIGBOb25lYCB0byBjbGVhcgoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGh1bnQgY3JlYXRvciBvciBhIGNvLWNyZWF0b3IKKiBgSW52YWxpZEh1bnRTdGF0dXNgIC0gSHVudCBpcyBub3QgaW4gRHJhZnQKKiBgSW52YWxpZERpZmZpY3VsdHlgIC0gT3ZlcnJpZGUgaXMgb3V0c2lkZSB0aGUgYWxsb3dlZCB0aWVyIHNjYWxlCgojIEV2ZW50cwoqIGBIdW50RGlmZmljdWx0eU92ZXJyaWRlU2V0YCAtIEVtaXR0ZWQgd2l0aCB0aGUgaHVudCBpZCwgY2FsbGVyLCBhbmQKdGhlIG5ldyBvdmVycmlkZSB2YWx1ZQAAAAAcc2V0X2h1bnRfZGlmZmljdWx0eV9vdmVycmlkZQAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAABNkaWZmaWN1bHR5X292ZXJyaWRlAAAAA+gAAAAEAAAAAQAAA+kAAAACAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAPVQYWdpbmF0ZWQgdmFyaWFudCBvZiBgZ2V0X2NvbXBsZXRlZF9jbHVlc2AgKHJlYWQtb25seSkuCmBvZmZzZXRgIGlzIDAtaW5kZXhlZDsgYGxpbWl0YCBpcyBjYXBwZWQgYXQgYE1BWF9CQVRDSF9TSVpFYCwgbWF0Y2hpbmcKYGxpc3RfY2x1ZXNgLiBSZXR1cm5zIGFuIGVtcHR5IHZlYyBpZiB0aGUgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkIG9yIHRoZQpvZmZzZXQgaXMgcGFzdCB0aGUgZW5kIG9mIHRoZSBjb21wbGV0ZWQgc2V0LgAAAAAAAB1nZXRfY29tcGxldGVkX2NsdWVzX3BhZ2luYXRlZAAAAAAAAAQAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAABA==",
        "AAAAAAAAAN1RdWVyeSB0aGUgY3VycmVudCBxdW90YSBzdGF0dXMgZm9yIGEgY3JlYXRvci4KClJldHVybnMgaG93IG1hbnkgaHVudHMgdGhlIGNyZWF0b3IgaGFzIGNyZWF0ZWQgdG9kYXksIHRoZWlyIGVmZmVjdGl2ZQpkYWlseSBsaW1pdCwgYW5kIHRoZSBjb29sZG93biBzZWNvbmRzIHVudGlsIHRoZSBuZXh0IGRheSBiZWdpbnMgKDAgd2hlbgp0aGUgbGltaXQgaGFzIG5vdCBiZWVuIHJlYWNoZWQpLgAAAAAAAB1nZXRfY3JlYXRvcl9yYXRlX2xpbWl0X3N0YXR1cwAAAAAAAAEAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAEAAAfQAAAAD1JhdGVMaW1pdFN0YXR1cwA=",
        "AAAAAAAAANZBZG1pbi1vbmx5OiB1cGRhdGUgdGhlIGNvbnRyYWN0LXdpZGUgZGVmYXVsdCBkYWlseSBodW50LWNyZWF0aW9uIGxpbWl0LgoKVGhpcyBpcyB0aGUgZmFsbGJhY2sgdXNlZCBmb3IgYW55IGNyZWF0b3IgdGhhdCBoYXMgbm8gcGVyLWNyZWF0b3IKb3ZlcnJpZGUuIFRoZSBpbml0aWFsIHZhbHVlIGlzIFtgcmF0ZV9saW1pdDo6REVGQVVMVF9IVU5UX0NSRUFUSU9OX0xJTUlUYF0uAAAAAAAfc2V0X2RlZmF1bHRfaHVudF9jcmVhdGlvbl9saW1pdAAAAAACAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPpAAAAAgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAQAAAAAAAAAAAAAABEh1bnQAAAAgAAAAAAAAAAxhY3RpdmF0ZWRfYXQAAAAGAAAARVdoZW4gdHJ1ZSwgcGxheWVycyBtYXkgY2xhaW0gdGhlaXIgcGFydGlhbCBzY29yZSBhZnRlciB0aGUgaHVudCBlbmRzLgAAAAAAABVhbGxvd19wYXJ0aWFsX3Njb3JpbmcAAAAAAAABAAAARU1pbmltdW0gc2Vjb25kcyBhIHBsYXllciBtdXN0IHdhaXQgYmV0d2VlbiBhdHRlbXB0cyBvbiB0aGUgc2FtZSBjbHVlLgAAAAAAABVhdHRlbXB0X2Nvb2xkb3duX3NlY3MAAAAAAAAEAAAAAAAAAApjYXRlZ29yaWVzAAAAAAPqAAAAEAAAAAAAAAAPY29tcGxldGVkX2NvdW50AAAAAAQAAAAAAAAACmNyZWF0ZWRfYXQAAAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAFREZWZhdWx0IHBvaW50IHZhbHVlIGFwcGxpZWQgdG8gY2x1ZXMgd2l0aCAwIHBvaW50cy4gQ2x1ZS1sZXZlbCBwb2ludHMgb3ZlcnJpZGUgdGhpcy4AAAAOZGVmYXVsdF9wb2ludHMAAAAAAAQAAAAAAAAAC2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAAE2RpZmZpY3VsdHlfb3ZlcnJpZGUAAAAD6AAAAAQAAAAAAAAAEWRpZmZpY3VsdHlfcmF0aW5nAAAAAAAABAAAAAAAAAAIZW5kX3RpbWUAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAABUU0hBMjU2IGhhc2ggb2YgdGhlIGludml0ZSBjb2RlIChzYWx0ZWQgd2l0aCBodW50X2lkKS4gTm9uZSBpZiBubyBpbnZpdGUgY29kZSBpcyBzZXQuAAAAEGludml0ZV9jb2RlX2hhc2gAAAPoAAAD7gAAACAAAAA+V2hlbiB0cnVlLCBvbmx5IHBsYXllcnMgd2l0aCBhIHZhbGlkIGludml0ZSBjb2RlIG1heSByZWdpc3Rlci4AAAAAAAppc19wcml2YXRlAAAAAAABAAAAQUNvbnRyb2xzIHdobyBjYW4gdmlldyB0aGUgaHVudCdzIGxlYWRlcmJvYXJkLiBEZWZhdWx0cyB0byBQdWJsaWMuAAAAAAAAFmxlYWRlcmJvYXJkX3Zpc2liaWxpdHkAAAAAB9AAAAAVTGVhZGVyYm9hcmRWaXNpYmlsaXR5AAAAAAAAAAAAABVtYXhfYXR0ZW1wdHNfcGVyX2NsdWUAAAAAAAAEAAAAPU1heGltdW0gbnVtYmVyIG9mIHBsYXllcnMgYWxsb3dlZCB0byByZWdpc3Rlci4gMCA9IHVubGltaXRlZC4AAAAAAAALbWF4X3BsYXllcnMAAAAABAAAAAAAAAAabWF4X3N1Ym1pc3Npb25zX3Blcl9taW51dGUAAAAAAAQAAABQUmVnaXN0cmF0aW9uIGN1dG9mZiB0aW1lc3RhbXAuIDAgPSBubyBkZWFkbGluZSAocmVnaXN0cmF0aW9uIG9wZW4gd2hpbGUgYWN0aXZlKS4AAAAVcmVnaXN0cmF0aW9uX2RlYWRsaW5lAAAAAAAABgAAAGZEeW5hbWljYWxseSByZWNhbGN1bGF0ZWQgb24gZXZlcnkgYGdldF9odW50YCByZWFkOyBub3QgbWVhbmluZ2Z1bCB3aGVuIHJlYWQgZnJvbSBhIHJhdyBzdHJ1Y3QgbGl0ZXJhbC4AAAAAAA9yZW1haW5pbmdfc2xvdHMAAAAABAAAAAAAAAAOcmVxdWlyZWRfY2x1ZXMAAAAAAAQAAAAAAAAADXJld2FyZF9jb25maWcAAAAAAAfQAAAADFJld2FyZENvbmZpZwAAAAAAAAAUc3RhcnRfbXVsdGlwbGllcl9icHMAAAAEAAAAAAAAAApzdGFydF90aW1lAAAAAAAGAAAAAAAAAAZzdGF0dXMAAAAAB9AAAAAKSHVudFN0YXR1cwAAAAAAOldoZW4gdHJ1ZSwgcGxheWVycyBtYXkgZm9ybSB0ZWFtcyBhbmQgc2hhcmUgY2x1ZSBwcm9ncmVzcy4AAAAAAAl0ZWFtX21vZGUAAAAAAAABAAAAAAAAABV0aW1lX2JvbnVzX2RlY2F5X3NlY3MAAAAAAAPoAAAABgAAAAAAAAASdGltZV9ib251c19taW5fYnBzAAAAAAPoAAAABAAAAAAAAAAUdGltZV9ib251c19zdGFydF9icHMAAAPoAAAABAAAAAAAAAAFdGl0bGUAAAAAAAAQAAAAAAAAAAt0b3RhbF9jbHVlcwAAAAAE",
        "AAAAAQAAAEBDbHVlIGluZm8gcmV0dXJuZWQgYnkgZ2V0X2NsdWUvbGlzdF9jbHVlcy4gRXhjbHVkZXMgYW5zd2VyIGhhc2guAAAAAAAAAAhDbHVlSW5mbwAAAAgAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAKZGlmZmljdWx0eQAAAAAABAAAAAAAAAAOaGludF9hdmFpbGFibGUAAAAAAAEAAAAAAAAAE2hpbnRfcGVuYWx0eV9wb2ludHMAAAAABAAAAAAAAAALaXNfcmVxdWlyZWQAAAAAAQAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAhxdWVzdGlvbgAAABAAAAAAAAAABndlaWdodAAAAAAABA==",
        "AAAAAQAAAS5SZXN1bHQgb2YgYSBgZ2NfaHVudGAgc3dlZXAgKGlzc3VlICM0NDYpLgoKQ291bnRzIGFyZSBzcGxpdCBieSBzdG9yYWdlIHRpZXIgYmVjYXVzZSB0aGUgdHdvIGFyZSBjaGFyZ2VkIGFuZCBleHBpcmUKZGlmZmVyZW50bHkgb24gU29yb2JhbjogaW5zdGFuY2UgZW50cmllcyBzaGFyZSB0aGUgY29udHJhY3QncyBvd24gVFRMLCB3aGlsZQpwZXJzaXN0ZW50IGVudHJpZXMgZWFjaCBjYXJyeSB0aGVpciBvd24uIEFuIG9wZXJhdG9yIHJlY2xhaW1pbmcgc3BhY2UgbmVlZHMKdG8gc2VlIHdoaWNoIHRpZXIgYWN0dWFsbHkgc2hyYW5rLgAAAAAAAAAAAAhHY1JlcG9ydAAAAAcAAAAoQ2x1ZXMgd2hvc2UgcGVyLWh1bnQgZW50cmllcyB3ZXJlIHN3ZXB0LgAAAAtjbHVlc19zd2VwdAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAmRW50cmllcyByZW1vdmVkIGZyb20gaW5zdGFuY2Ugc3RvcmFnZS4AAAAAABBpbnN0YW5jZV9yZW1vdmVkAAAABAAAAChFbnRyaWVzIHJlbW92ZWQgZnJvbSBwZXJzaXN0ZW50IHN0b3JhZ2UuAAAAEnBlcnNpc3RlbnRfcmVtb3ZlZAAAAAAABAAAACpQbGF5ZXJzIHdob3NlIHBlci1odW50IGVudHJpZXMgd2VyZSBzd2VwdC4AAAAAAA1wbGF5ZXJzX3N3ZXB0AAAAAAAABAAAAChUZWFtcyB3aG9zZSBwZXItaHVudCBlbnRyaWVzIHdlcmUgc3dlcHQuAAAAC3RlYW1zX3N3ZXB0AAAAAAQAAAAoYHBlcnNpc3RlbnRfcmVtb3ZlZCArIGluc3RhbmNlX3JlbW92ZWRgLgAAAA10b3RhbF9yZW1vdmVkAAAAAAAABA==",
        "AAAAAwAAAAAAAAAAAAAACkh1bnRTdGF0dXMAAAAAAAcAAAAlQSBodW50IHRoYXQgaGFzIG5ldmVyIGJlZW4gYWN0aXZhdGVkLgAAAAAAAAVEcmFmdAAAAAAAAAAAAAA1QSBodW50IGN1cnJlbnRseSBhY2NlcHRpbmcgcmVnaXN0cmF0aW9ucyBhbmQgYW5zd2Vycy4AAAAAAAAGQWN0aXZlAAAAAAABAAAAGkEgbm9ybWFsbHkgY29tcGxldGVkIGh1bnQuAAAAAAAJQ29tcGxldGVkAAAAAAAAAgAAACBBIGh1bnQgY2FuY2VsbGVkIGJ5IGl0cyBjcmVhdG9yLgAAAAlDYW5jZWxsZWQAAAAAAAADAAAAhEEgdGVtcG9yYXJpbHkgcGF1c2VkIGh1bnQuIFRoaXMgZXhwbGljaXQgdmFsdWUgcHJlc2VydmVzIHRoZSB3aXJlCmxheW91dCBhbHJlYWR5IGVtaXR0ZWQgYnkgdGhlIFBhdXNlZC1zdGF0ZSBpbXBsZW1lbnRhdGlvbiBvbiBtYWluLgAAAAZQYXVzZWQAAAAAAAQAAABNQSB0ZXJtaW5hbCBlbWVyZ2VuY3kgc3RhdGUgcmV0YWluZWQgZm9yIGNvbXBhdGliaWxpdHkgd2l0aCBvbGRlcgpkZXBsb3ltZW50cy4AAAAAAAAQRW1lcmdlbmN5U3RvcHBlZAAAAAUAAAA3QSB0ZXJtaW5hbCBodW50IHdob3NlIHN0b3JhZ2UgbWF5IGJlIGdhcmJhZ2UtY29sbGVjdGVkLgAAAAAIQXJjaGl2ZWQAAAAG",
        "AAAAAQAAAAAAAAAAAAAADFJld2FyZENvbmZpZwAAAAgAAAAAAAAADWNsYWltZWRfY291bnQAAAAAAAAEAAAAAAAAAAttYXhfd2lubmVycwAAAAAEAAAAAAAAAAxuZnRfY29udHJhY3QAAAPoAAAAEwAAAAAAAAALbmZ0X2VuYWJsZWQAAAAAAQAAAAAAAAANbmZ0X2ltYWdlX3VyaQAAAAAAA+gAAAAQAAAAAAAAAApuZnRfcmFyaXR5AAAAAAAEAAAAAAAAAAhuZnRfdGllcgAAAAQAAAAAAAAACHhsbV9wb29sAAAACw==",
        "AAAAAQAAAENJbnB1dCBwYXlsb2FkIGZvciBhZGRpbmcgbXVsdGlwbGUgY2x1ZXMgaW4gb25lIGNvbnRyYWN0IGludm9jYXRpb24uAAAAAAAAAAAOQmF0Y2hDbHVlSW5wdXQAAAAAAAUAAAAAAAAABmFuc3dlcgAAAAAAEAAAAH5EaWZmaWN1bHR5IHRpZXIgKDEtNSwgMSA9IGVhc2llc3QsIDUgPSBoYXJkZXN0KS4KRGlmZmljdWx0eSBtdWx0aXBsaWVzIHRoZSBjbHVlJ3MgcG9pbnRzOiBwb2ludHMgZWFybmVkID0gcG9pbnRzICogZGlmZmljdWx0eS4AAAAAAApkaWZmaWN1bHR5AAAAAAAEAAAAAAAAAAtpc19yZXF1aXJlZAAAAAABAAAAAAAAAAZwb2ludHMAAAAAAAQAAAAAAAAACHF1ZXN0aW9uAAAAEA==",
        "AAAAAQAAADlBZ2dyZWdhdGUgc3RhdGlzdGljcyBmb3IgYSBodW50IChyZWFkLW9ubHkgcXVlcnkgcmVzdWx0KS4AAAAAAAAAAAAADkh1bnRTdGF0aXN0aWNzAAAAAAAFAAAAAAAAAA1hdmVyYWdlX3Njb3JlAAAAAAAABAAAAAAAAAAPY29tcGxldGVkX2NvdW50AAAAAAQAAAAAAAAAF2NvbXBsZXRpb25fcmF0ZV9wZXJjZW50AAAAAAQAAAAAAAAADXRvdGFsX3BsYXllcnMAAAAAAAAEAAAAAAAAAA90b3RhbF9zY29yZV9zdW0AAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADkxlYWRlcmJvYXJkUm93AAAAAAAFAAAAAAAAAAxjb21wbGV0ZWRfYXQAAAAGAAAAAAAAAAVpbmRleAAAAAAAAAQAAAAAAAAADGlzX2NvbXBsZXRlZAAAAAEAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAFc2NvcmUAAAAAAAAE",
        "AAAAAQAAAFdQdWJsaWMgdmlldyBvZiBwbGF5ZXIgcHJvZ3Jlc3MsIHdpdGggYHBsYXllcmAgYW5kIGBodW50X2lkYCByZWNvbnN0cnVjdGVkIGZyb20gdGhlIGtleS4AAAAAAAAAAA5QbGF5ZXJQcm9ncmVzcwAAAAAADgAAAAAAAAASY2x1ZV9sYXN0X2F0dGVtcHRzAAAAAAPsAAAABAAAAAYAAAAAAAAADGNvbXBsZXRlZF9hdAAAAAYAAAAAAAAAFGNvbXBsZXRlZF9jbHVlX2luZGV4AAAD7AAAAAQAAAABAAAAAAAAAA9jb21wbGV0ZWRfY2x1ZXMAAAAD6gAAAAQAAACyVGhlIHBsYXllcidzIGZpbmlzaGluZyBwb3NpdGlvbiBhbW9uZyBhbGwgY29tcGxldGlvbnMgZm9yIHRoaXMgaHVudCwKZnJvemVuIGF0IHRoZSBtb21lbnQgYGlzX2NvbXBsZXRlZGAgd2FzIHNldCB0byBgdHJ1ZWAuICBaZXJvIG1lYW5zIHRoZQpwbGF5ZXIgaGFzIG5vdCB5ZXQgY29tcGxldGVkIHRoZSBodW50LgAAAAAAD2NvbXBsZXRpb25fcmFuawAAAAAEAAAAAAAAAAxoaW50ZWRfY2x1ZXMAAAPqAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAxpc19jb21wbGV0ZWQAAAABAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAAEnJlY2VudF9zdWJtaXNzaW9ucwAAAAAD6gAAAAYAAAAAAAAAGHJlcXVpcmVkX2NvbXBsZXRlZF9jb3VudAAAAAQAAAAAAAAADnJld2FyZF9jbGFpbWVkAAAAAAABAAAAAAAAAApzdGFydGVkX2F0AAAAAAAGAAAAAAAAAAt0b3RhbF9zY29yZQAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAD1JhdGVMaW1pdFN0YXR1cwAAAAADAAAAAAAAABBjb29sZG93bl9zZWNvbmRzAAAABgAAAAAAAAAPY3JlYXRpb25zX3RvZGF5AAAAAAQAAAAAAAAAC2RhaWx5X2xpbWl0AAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAD1RpbWVCb251c0NvbmZpZwAAAAADAAAAAAAAABNkZWNheV9kdXJhdGlvbl9zZWNzAAAAAAYAAAAAAAAAEm1pbl9tdWx0aXBsaWVyX2JwcwAAAAAABAAAAAAAAAAUc3RhcnRfbXVsdGlwbGllcl9icHMAAAAE",
        "AAAAAQAAAElMZWFkZXJib2FyZCBlbnRyeSBmb3IgYSBzaW5nbGUgcGxheWVyIGluIGEgaHVudCAocmVhZC1vbmx5IHF1ZXJ5IHJlc3VsdCkuAAAAAAAAAAAAABBMZWFkZXJib2FyZEVudHJ5AAAABQAAAAAAAAAMY29tcGxldGVkX2F0AAAABgAAAAAAAAAMaXNfY29tcGxldGVkAAAAAQAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAARyYW5rAAAABAAAAAAAAAAFc2NvcmUAAAAAAAAE",
        "AAAAAQAAAIxXcmFwcGVyIHJldHVybmVkIGJ5IGBnZXRfaHVudF9sZWFkZXJib2FyZGAgdGhhdCBpbmNsdWRlcyB0cnVuY2F0aW9uCmluZm9ybWF0aW9uIHNvIGNhbGxlcnMgY2FuIHRlbGwgd2hlbiB0aGUgdmlzaWJsZSBlbnRyaWVzIGFyZSBpbmNvbXBsZXRlLgAAAAAAAAARTGVhZGVyYm9hcmRSZXN1bHQAAAAAAAADAAAAAAAAAAdlbnRyaWVzAAAAA+oAAAfQAAAAEExlYWRlcmJvYXJkRW50cnkAAAAAAAAADXRvdGFsX3BsYXllcnMAAAAAAAAEAAAAAAAAAAl0cnVuY2F0ZWQAAAAAAAAB",
        "AAAAAQAAAAAAAAAAAAAAEUxlYWRlcmJvYXJkV2luZG93AAAAAAAABAAAAAAAAAAHZW50cmllcwAAAAPqAAAH0AAAAA5MZWFkZXJib2FyZFJvdwAAAAAAAAAAAAhmaW5pc2hlZAAAAAEAAAAAAAAACm5leHRfaW5kZXgAAAAAAAQAAAAAAAAACnF1ZXJpZWRfYXQAAAAAAAY=",
        "AAAAAgAAADFDb250cm9scyB3aG8gY2FuIHZpZXcgdGhlIGxlYWRlcmJvYXJkIGZvciBhIGh1bnQuAAAAAAAAAAAAABVMZWFkZXJib2FyZFZpc2liaWxpdHkAAAAAAAADAAAAAAAAACpBbnlvbmUgY2FuIHZpZXcgdGhlIGxlYWRlcmJvYXJkIChkZWZhdWx0KS4AAAAAAAZQdWJsaWMAAAAAAAAAAABHT25seSBwbGF5ZXJzIHdobyBoYXZlIHJlZ2lzdGVyZWQgZm9yIHRoZSBodW50IGNhbiB2aWV3IHRoZSBsZWFkZXJib2FyZC4AAAAADlJlZ2lzdGVyZWRPbmx5AAAAAAAAAAAAL09ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gdmlldyB0aGUgbGVhZGVyYm9hcmQuAAAAAAtDcmVhdG9yT25seQA=",
        "AAAABAAAAAAAAAAAAAAADUh1bnRFcnJvckNvZGUAAAAAAAA1AAAAAAAAAAxIdW50Tm90Rm91bmQAAAABAAAAAAAAAAxDbHVlTm90Rm91bmQAAAACAAAAAAAAABFJbnZhbGlkSHVudFN0YXR1cwAAAAAAAAMAAAAAAAAAE1BsYXllck5vdFJlZ2lzdGVyZWQAAAAABAAAAAAAAAAUQ2x1ZUFscmVhZHlDb21wbGV0ZWQAAAAFAAAAAAAAAA1JbnZhbGlkQW5zd2VyAAAAAAAABgAAAAAAAAANSHVudE5vdEFjdGl2ZQAAAAAAAAcAAAAAAAAADFVuYXV0aG9yaXplZAAAAAgAAAAAAAAAFkluc3VmZmljaWVudFJld2FyZFBvb2wAAAAAAAkAAAAAAAAAFUR1cGxpY2F0ZVJlZ2lzdHJhdGlvbgAAAAAAAAoAAAAAAAAADEludmFsaWRUaXRsZQAAAAsAAAAAAAAAEkludmFsaWREZXNjcmlwdGlvbgAAAAAADAAAAAAAAAAOSW52YWxpZEFkZHJlc3MAAAAAAA0AAAAAAAAADFRvb01hbnlDbHVlcwAAAA4AAAAAAAAAD0ludmFsaWRRdWVzdGlvbgAAAAAPAAAAAAAAAAxSZWZ1bmRGYWlsZWQAAAAQAAAAAAAAAAxOb0NsdWVzQWRkZWQAAAARAAAAAAAAABBIdW50Tm90Q29tcGxldGVkAAAAEgAAAAAAAAAUUmV3YXJkQWxyZWFkeUNsYWltZWQAAAATAAAAAAAAABhSZXdhcmREaXN0cmlidXRpb25GYWlsZWQAAAAUAAAAAAAAABNOb1Jld2FyZHNDb25maWd1cmVkAAAAABUAAAAAAAAAE0R1cGxpY2F0ZVN1Ym1pc3Npb24AAAAAFgAAAAAAAAARU3VibWlzc2lvbkV4cGlyZWQAAAAAAAAXAAAAAAAAAAxCYW5uZWRQbGF5ZXIAAAAYAAAAAAAAAA9Ob1JlcXVpcmVkQ2x1ZXMAAAAAGQAAAAAAAAARUmF0ZUxpbWl0RXhjZWVkZWQAAAAAAAAaAAAAAAAAAA1TY29yZU92ZXJmbG93AAAAAAAAGwAAAAAAAAATUmVnaXN0cmF0aW9uc1BhdXNlZAAAAAAcAAAAAAAAAA1BbnN3ZXJzUGF1c2VkAAAAAAAAHQAAAAAAAAANUmV3YXJkc1BhdXNlZAAAAAAAAB4AAAAAAAAAEUh1bnRFbmRUaW1lSW5QYXN0AAAAAAAAHwAAAAAAAAAOTm9QZW5kaW5nQWRtaW4AAAAAACAAAAAAAAAAFFBlbmRpbmdBZG1pbk1pc21hdGNoAAAAIQAAAAAAAAANSW52YWxpZFJhcml0eQAAAAAAACIAAAAAAAAAFkludmFsaWRUaW1lQm9udXNDb25maWcAAAAAACMAAAAAAAAAEkFkZHJlc3NCbGFja2xpc3RlZAAAAAAAJAAAAAAAAAAOQ29udHJhY3RQYXVzZWQAAAAAACUAAAAAAAAAEkludmFsaWRNYXhBdHRlbXB0cwAAAAAAJgAAAAAAAAANSW52YWxpZFdlaWdodAAAAAAAACcAAAAAAAAAEEhpbnROb3RBdmFpbGFibGUAAAAoAAAAAAAAABNIaW50QWxyZWFkeVVubG9ja2VkAAAAACkAAAAAAAAAEUluc3VmZmljaWVudFNjb3JlAAAAAAAAKgAAAAAAAAARVG9vTWFueUNhdGVnb3JpZXMAAAAAAAArAAAAAAAAAA9JbnZhbGlkQ2F0ZWdvcnkAAAAALAAAAAAAAAARSW52YWxpZERpZmZpY3VsdHkAAAAAAAAtAAAAAAAAABVDb3JydXB0UGxheWVyUHJvZ3Jlc3MAAAAAAAAuAAAAAAAAAA5IdW50Tm90U3RhcnRlZAAAAAAALwAAAAAAAAAUQWRtaW5BbHJlYWR5UHJvcG9zZWQAAAAwAAAAAAAAAA1JbnZhbGlkUG9pbnRzAAAAAAAAMQAAAAAAAAAISHVudEZ1bGwAAAAyAAAAAAAAACFMZWFkZXJib2FyZFZpc2liaWxpdHlVbmF1dGhvcml6ZWQAAAAAAAAzAAAAAAAAABJJbnZpdGVDb2RlUmVxdWlyZWQAAAAAADQAAAAAAAAADlRvb01hbnlBbGlhc2VzAAAAAAA1",
        "AAAAAQAAAAAAAAAAAAAAC0hlYWx0aEFsZXJ0AAAAAAMAAAAAAAAACmFsZXJ0X3R5cGUAAAAAABAAAAAAAAAABWNvdW50AAAAAAAABAAAAAAAAAALbGFzdF9sZWRnZXIAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADkNvbnRyYWN0SGVhbHRoAAAAAAAFAAAAAAAAAA1hY3RpdmVfYWxlcnRzAAAAAAAABAAAAAAAAAANYXZnX2dhc191bml0cwAAAAAAAAYAAAAAAAAAEmZhaWxlZF9pbnZvY2F0aW9ucwAAAAAABgAAAAAAAAAQZmFpbHVyZV9yYXRlX2JwcwAAAAQAAAAAAAAAEXRvdGFsX2ludm9jYXRpb25zAAAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVBdXRoRXJyb3IAAAAFAAAAAAAAAAxVbmF1dGhvcml6ZWQAAAABAAAAAAAAAApOb1Byb3Bvc2FsAAAAAAACAAAAAAAAAA9UaW1lbG9ja1BlbmRpbmcAAAAAAwAAAAAAAAAPVmVyc2lvbk1pc21hdGNoAAAAAAQAAAAAAAAAD0ludmFsaWRUaW1lbG9jawAAAAAF" ]),
      options
    )
  }
  public readonly fromJSON = {
    gc_hunt: this.txFromJSON<Result<GcReport>>,
        add_clue: this.txFromJSON<Result<u32>>,
        get_clue: this.txFromJSON<Result<ClueInfo>>,
        add_clues: this.txFromJSON<Result<Array<u32>>>,
        ban_player: this.txFromJSON<Result<void>>,
        clone_hunt: this.txFromJSON<Result<u64>>,
        close_hunt: this.txFromJSON<Result<void>>,
        list_clues: this.txFromJSON<Array<ClueInfo>>,
        list_hunts: this.txFromJSON<Array<Hunt>>,
        cancel_hunt: this.txFromJSON<Result<void>>,
        create_hunt: this.txFromJSON<Result<u64>>,
        accept_admin: this.txFromJSON<Result<void>>,
        archive_hunt: this.txFromJSON<Result<void>>,
        is_view_only: this.txFromJSON<boolean>,
        request_hint: this.txFromJSON<Result<string>>,
        search_hunts: this.txFromJSON<Array<Hunt>>,
        unban_player: this.txFromJSON<Result<void>>,
        activate_hunt: this.txFromJSON<Result<void>>,
        complete_hunt: this.txFromJSON<Result<void>>,
        get_hunt_info: this.txFromJSON<Result<Hunt>>,
        pause_answers: this.txFromJSON<Result<void>>,
        pause_rewards: this.txFromJSON<Result<void>>,
        run_migration: this.txFromJSON<Result<MigrationReport>>,
        set_clue_hint: this.txFromJSON<Result<void>>,
        set_team_mode: this.txFromJSON<Result<void>>,
        submit_answer: this.txFromJSON<Result<boolean>>,
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
        is_hunt_terminal: this.txFromJSON<Result<boolean>>,
        set_hunt_privacy: this.txFromJSON<Result<void>>,
        unpause_contract: this.txFromJSON<Result<void>>,
        blacklist_creator: this.txFromJSON<Result<void>>,
        get_active_alerts: this.txFromJSON<Array<HealthAlert>>,
        get_hunt_end_time: this.txFromJSON<Result<u64>>,
        initialize_schema: this.txFromJSON<null>,
        propose_new_admin: this.txFromJSON<Result<void>>,
        remove_co_creator: this.txFromJSON<Result<void>>,
        set_reward_config: this.txFromJSON<Result<void>>,
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
        set_rate_limit_admin: this.txFromJSON<Result<void>>,
        get_hunts_by_category: this.txFromJSON<Array<Hunt>>,
        remove_from_blacklist: this.txFromJSON<Result<void>>,
        set_time_bonus_config: this.txFromJSON<Result<void>>,
        unpause_registrations: this.txFromJSON<Result<void>>,
        set_creator_hunt_limit: this.txFromJSON<Result<void>>,
        clone_hunt_with_answers: this.txFromJSON<Result<u64>>,
        remove_global_view_only: this.txFromJSON<Result<void>>,
        remove_view_only_access: this.txFromJSON<Result<void>>,
        submit_answer_with_hash: this.txFromJSON<Result<boolean>>,
        update_hunt_description: this.txFromJSON<Result<void>>,
        get_global_view_only_list: this.txFromJSON<Array<string>>,
        set_allow_partial_scoring: this.txFromJSON<Result<void>>,
        set_max_attempts_per_clue: this.txFromJSON<Result<void>>,
        set_registration_deadline: this.txFromJSON<Result<void>>,
        get_hunt_storage_footprint: this.txFromJSON<GcReport>,
        get_hunt_leaderboard_window: this.txFromJSON<Result<LeaderboardWindow>>,
        is_hunt_expired_or_cancelled: this.txFromJSON<Result<boolean>>,
        set_hunt_difficulty_override: this.txFromJSON<Result<void>>,
        get_completed_clues_paginated: this.txFromJSON<Array<u32>>,
        get_creator_rate_limit_status: this.txFromJSON<RateLimitStatus>,
        set_default_hunt_creation_limit: this.txFromJSON<Result<void>>
  }
}
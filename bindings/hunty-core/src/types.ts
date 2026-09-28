import { Buffer } from "buffer";
import type {
  u32,
  u64,
  i64,
  i128,
  Option,
} from "@stellar/stellar-sdk/contract";

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


/**
 * Stored top-N leaderboard entry maintained incrementally on score changes.
 */
export interface LeaderboardIndexEntry {
  completed_at: u64;
  is_completed: boolean;
  player: string;
  score: u32;
}

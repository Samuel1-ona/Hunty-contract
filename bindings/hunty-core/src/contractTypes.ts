/**
 * contractTypes.ts — interfaces, union types, and error-code maps that mirror
 * the on-chain Soroban data structures for the HuntyCore contract.
 *
 * Pure declarations — no runtime logic.  Consumed by contractHelpers.ts and index.ts.
 */

import { Buffer } from "buffer";
import type {
  u32, i32, u64, i64, u128, i128, u256, i256,
  Option, Timepoint, Duration,
} from "@stellar/stellar-sdk/contract";

// ── Monitoring / health ────────────────────────────────────────────────────
export interface HealthAlert   { alert_type: string; count: u32; last_ledger: u64; }
export interface ContractHealth {
  active_alerts: u32; avg_gas_units: u64; failed_invocations: u64;
  failure_rate_bps: u32; total_invocations: u64;
}

// ── Core game types ────────────────────────────────────────────────────────
/** Stored clue with SHA256 answer hash. The hash is never exposed via get_clue/list_clues. */
export interface Clue {
  answer_hash: Buffer; clue_id: u32; is_required: boolean; points: u32; question: string;
}
export interface Hunt {
  activated_at: u64; created_at: u64; creator: string; description: string; end_time: u64;
  hunt_id: u64; invite_code_hash: Option<Buffer>; is_private: boolean; required_clues: u32;
  reward_config: HuntRewardConfig; status: HuntStatus; title: string; total_clues: u32;
}
/** Clue info returned by get_clue/list_clues. Excludes answer hash. */
export interface ClueInfo  { clue_id: u32; is_required: boolean; points: u32; question: string; }
export interface Location  { latitude: i64; longitude: i64; radius: u32; }
export type HuntStatus =
  | { tag: "Draft";     values: void }
  | { tag: "Active";    values: void }
  | { tag: "Completed"; values: void }
  | { tag: "Cancelled"; values: void };

/** On-chain reward configuration stored within a Hunt (tracks pool state). */
export interface HuntRewardConfig {
  claimed_count: u32; max_winners: u32; nft_contract: Option<string>;
  nft_enabled: boolean; xlm_pool: i128;
}
export interface BatchClueInput {
  question: string; answer: string; points: u32; is_required: boolean; difficulty: number;
}
/** Aggregate statistics for a hunt (read-only query result). */
export interface HuntStatistics {
  average_score: u32; completed_count: u32; completion_rate_percent: u32;
  total_players: u32; total_score_sum: u64;
}
export interface PlayerProgress {
  completed_at: u64; completed_clues: Array<u32>; hunt_id: u64; is_completed: boolean;
  player: string; reward_claimed: boolean; started_at: u64; total_score: u32;
}
/** Leaderboard entry for a single player in a hunt (read-only query result). */
export interface LeaderboardEntry {
  completed_at: u64; is_completed: boolean; player: string; rank: u32; score: u32;
}
export interface MigrationReport {
  dry_run: boolean; from_version: u32; message: string;
  steps_applied: u32; succeeded: boolean; to_version: u32;
}
/** Configuration for distributing rewards across the HuntyCore ↔ RewardManager boundary. */
export interface RewardConfig {
  nft_contract: Option<string>; nft_description: string; nft_hunt_title: string;
  nft_image_uri: string; nft_rarity: u32; nft_tier: u32; nft_title: string;
  xlm_amount: Option<i128>;
}

// ── Events ─────────────────────────────────────────────────────────────────
/** Emitted when a clue is added. Does not expose the answer hash. */
export interface ClueAddedEvent {
  clue_id: u32; creator: string; hunt_id: u64; is_required: boolean; points: u32; question: string;
}
export interface HuntCreatedEvent         { creator: string; hunt_id: u64; title: string; }
export interface ClueCompletedEvent       { clue_id: u32; hunt_id: u64; player: string; points_earned: u32; }
export interface HuntActivatedEvent       { activated_at: u64; hunt_id: u64; }
export interface HuntCancelledEvent       { hunt_id: u64; }
export interface HuntCompletedEvent       {
  completion_rank: u32; completion_time: u64; hunt_id: u64; player: string; total_score: u32;
}
export interface RewardClaimedEvent       { hunt_id: u64; nft_awarded: boolean; player: string; xlm_amount: i128; }
export interface AnswerIncorrectEvent     { clue_id: u32; hunt_id: u64; player: string; timestamp: u64; }
export interface HuntDeactivatedEvent     { hunt_id: u64; }
/** Emitted when a player registers for an active hunt. */
export interface PlayerRegisteredEvent    { hunt_id: u64; player: string; }
/** Emitted when a hunt creator generates or updates the invite code for a private hunt. */
export interface InviteCodeGeneratedEvent { creator: string; hunt_id: u64; }
/** Emitted when a hunt creator clears the invite code. */
export interface InviteCodeRevokedEvent   { creator: string; hunt_id: u64; }
/** Emitted when a player successfully registers using an invite code. */
export interface PlayerRegisteredWithInviteEvent { hunt_id: u64; player: string; }
export interface HuntStatusChangedEvent   { hunt_id: u64; new_status: HuntStatus; old_status: HuntStatus; }

// ── Error code maps ────────────────────────────────────────────────────────
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
  39: {message:"HuntIsPrivate"},
  40: {message:"InvalidInviteCode"},
  41: {message:"InviteNotConfigured"},
} as const;

export const RewardErrorCode = {
   1: {message:"NotInitialized"},
   2: {message:"InsufficientPool"},
   3: {message:"AlreadyDistributed"},
   4: {message:"TransferFailed"},
   5: {message:"InvalidAmount"},
   6: {message:"InvalidConfig"},
   7: {message:"NftMintFailed"},
   8: {message:"PoolAlreadyExists"},
   9: {message:"PoolNotFound"},
  10: {message:"Unauthorized"},
  11: {message:"BelowMinimumAmount"},
  12: {message:"AlreadyInitialized"},
  13: {message:"HuntNotFound"},
  /** A recursive distribution attempt was detected during an external XLM or NFT call. */
  14: {message:"ReentrancyDetected"},
  /** The tracked pool balance diverged from the actual XLM token balance. */
  15: {message:"PoolBalanceDivergence"},
} as const;

/**
 * contractHelpers.ts — the `Client` interface that describes every callable
 * method on the HuntyCore contract, plus the `FromJSON` helper type used by
 * the `Client` class implementation in index.ts.
 *
 * No Soroban SDK classes are instantiated here; this file only imports types.
 */

import type {
  u32, u64, i128, Option,
} from "@stellar/stellar-sdk/contract";
import {
  AssembledTransaction,
  MethodOptions,
  Result,
} from "@stellar/stellar-sdk/contract";
import type {
  ClueInfo, Hunt, HuntStatistics, LeaderboardEntry,
  MigrationReport, PlayerProgress, ContractHealth,
} from "./contractTypes";

// ── Client interface ───────────────────────────────────────────────────────

export interface Client {
  /**
   * Adds a clue to a hunt. Only the hunt creator can add clues.
   * Answers are hashed with SHA256 before storage; the hash is never exposed.
   *
   * # Errors: HuntNotFound · InvalidHuntStatus · Unauthorized · TooManyClues
   *           · InvalidQuestion · InvalidAnswer
   */
  add_clue: (
    args: { hunt_id: u64; question: string; answer: string; points: u32; is_required: boolean },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<u32>>>;

  /** Returns clue information for a hunt/clue. Does not expose the answer hash. */
  get_clue: (
    args: { hunt_id: u64; clue_id: u32 },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<ClueInfo>>>;

  /** Returns all clues for a hunt (question, points, required). Answer hashes are not exposed. */
  list_clues: (
    args: { hunt_id: u64 },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Array<ClueInfo>>>;

  cancel_hunt: (
    args: { hunt_id: u64; caller: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /**
   * Creates a new scavenger hunt with the provided metadata.
   *
   * # Errors: InvalidTitle · InvalidDescription · InvalidAddress
   */
  create_hunt: (
    args: {
      creator: string; title: string; description: string;
      _start_time: Option<u64>; end_time: Option<u64>;
      max_submissions_per_minute: u32; start_multiplier_bps: Option<u32>;
    },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<u64>>>;

  activate_hunt: (
    args: { hunt_id: u64; caller: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /**
   * Completes a hunt for a player and distributes rewards.
   *
   * # Errors: HuntNotFound · PlayerNotRegistered · HuntNotCompleted
   *           · RewardAlreadyClaimed · NoRewardsConfigured
   *           · InsufficientRewardPool · RewardDistributionFailed
   */
  complete_hunt: (
    args: { hunt_id: u64; player: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  get_hunt_info: (
    args: { hunt_id: u64 },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<Hunt>>>;

  /** Runs storage migrations up to `target_version`. Set `dry_run` to simulate without writes. */
  run_migration: (
    args: { admin: string; target_version: u32; dry_run: boolean },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<MigrationReport>>;

  /**
   * Verifies the submitted answer by hashing it and comparing with the stored answer hash.
   *
   * # Errors: HuntNotFound · HuntNotActive · PlayerNotRegistered · ClueNotFound
   *           · ClueAlreadyCompleted · InvalidAnswer
   */
  submit_answer: (
    args: {
      hunt_id: u64; clue_id: u32; player: string;
      answer: string; submission_nonce: u64; submitted_at: u64;
    },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  deactivate_hunt: (
    args: { hunt_id: u64; caller: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /**
   * Registers a player for an active hunt. The caller must authorize.
   *
   * # Errors: HuntNotFound · InvalidHuntStatus · HuntNotActive · DuplicateRegistration
   */
  register_player: (
    args: { hunt_id: u64; player: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /** Sets whether a hunt is private (invite-only). Only callable in Draft status. */
  set_hunt_privacy: (
    args: { hunt_id: u64; creator: string; is_private: boolean },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /** Generates or updates the invite code for a private hunt. */
  generate_invite_code: (
    args: { hunt_id: u64; creator: string; invite_code: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /** Clears the invite code for a private hunt. */
  revoke_invite_code: (
    args: { hunt_id: u64; creator: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /** Registers a player for a private hunt using a valid invite code. */
  register_with_invite: (
    args: { hunt_id: u64; player: string; invite_code: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<void>>>;

  /** Initializes schema version tracking on deploy or first admin call. */
  initialize_schema: (
    args: { admin: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<null>>;

  /** Returns the on-chain storage schema version (0 when uninitialized). */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>;

  /** Rolls back to the schema version captured before the last migration. */
  rollback_migration: (
    args: { admin: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Option<MigrationReport>>>;

  /** Sets the RewardManager contract address for cross-contract reward distribution. */
  set_reward_manager: (
    args: { reward_manager: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<null>>;

  /**
   * Returns the list of clue IDs the player has completed for a hunt (read-only).
   * Returns empty array if player is not registered.
   */
  get_completed_clues: (
    args: { hunt_id: u64; player: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Array<u32>>>;

  /**
   * Returns aggregate statistics for a hunt (read-only).
   * Returns error if hunt does not exist.
   */
  get_hunt_statistics: (
    args: { hunt_id: u64 },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<HuntStatistics>>>;

  /**
   * Returns player progress for a hunt (read-only).
   * Returns error if player is not registered.
   */
  get_player_progress: (
    args: { hunt_id: u64; player: string },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<PlayerProgress>>>;

  /** Returns contract health metrics for operator dashboards. */
  get_health_dashboard: (options?: MethodOptions) => Promise<AssembledTransaction<ContractHealth>>;

  /**
   * Returns the top N players by score for a hunt (read-only).
   * Limit is capped at 20. Returns error if hunt does not exist.
   */
  get_hunt_leaderboard: (
    args: { hunt_id: u64; limit: u32 },
    options?: MethodOptions
  ) => Promise<AssembledTransaction<Result<Array<LeaderboardEntry>>>>;
}

// ── fromJSON result-type map ───────────────────────────────────────────────
// These types drive the `fromJSON` map on the Client class.  Keeping them here
// keeps index.ts focused purely on the ContractSpec wiring.

export type FromJSONMap = {
  add_clue: Result<u32>;
  get_clue: Result<ClueInfo>;
  list_clues: Array<ClueInfo>;
  cancel_hunt: Result<void>;
  create_hunt: Result<u64>;
  activate_hunt: Result<void>;
  complete_hunt: Result<void>;
  get_hunt_info: Result<Hunt>;
  run_migration: MigrationReport;
  submit_answer: Result<void>;
  deactivate_hunt: Result<void>;
  register_player: Result<void>;
  initialize_schema: null;
  get_schema_version: u32;
  rollback_migration: Option<MigrationReport>;
  set_reward_manager: null;
  get_completed_clues: Array<u32>;
  get_hunt_statistics: Result<HuntStatistics>;
  get_player_progress: Result<PlayerProgress>;
  get_health_dashboard: ContractHealth;
  get_hunt_leaderboard: Result<Array<LeaderboardEntry>>;
};

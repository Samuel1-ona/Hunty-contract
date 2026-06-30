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
  answer_hash: Buffer;
  clue_id: u32;
  is_required: boolean;
  points: u32;
  question: string;
}


export interface Hunt {
  activated_at: u64;
  created_at: u64;
  creator: string;
  description: string;
  end_time: u64;
  hunt_id: u64;
  required_clues: u32;
  reward_config: HuntRewardConfig;
  status: HuntStatus;
  title: string;
  total_clues: u32;
}


/**
 * Clue info returned by get_clue/list_clues. Excludes answer hash.
 */
export interface ClueInfo {
  clue_id: u32;
  is_required: boolean;
  points: u32;
  question: string;
}


export interface Location {
  latitude: i64;
  longitude: i64;
  radius: u32;
}

export type HuntStatus = {tag: "Draft", values: void} | {tag: "Active", values: void} | {tag: "Completed", values: void} | {tag: "Cancelled", values: void};


/**
 * On-chain reward configuration stored within a Hunt (tracks pool state).
 */
export interface HuntRewardConfig {
  claimed_count: u32;
  max_winners: u32;
  nft_contract: Option<string>;
  nft_enabled: boolean;
  xlm_pool: i128;
}


/**
 * Emitted when a clue is added. Does not expose the answer hash.
 */
export interface ClueAddedEvent {
  clue_id: u32;
  creator: string;
  hunt_id: u64;
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


export interface PlayerProgress {
  completed_at: u64;
  completed_clues: Array<u32>;
  hunt_id: u64;
  is_completed: boolean;
  player: string;
  reward_claimed: boolean;
  started_at: u64;
  total_score: u32;
}


export interface HuntCreatedEvent {
  creator: string;
  hunt_id: u64;
  title: string;
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


export interface AnswerIncorrectEvent {
  clue_id: u32;
  hunt_id: u64;
  player: string;
  timestamp: u64;
}


export interface HuntDeactivatedEvent {
  hunt_id: u64;
}


/**
 * Emitted when a player registers for an active hunt.
 */
export interface PlayerRegisteredEvent {
  hunt_id: u64;
  player: string;
}


export interface HuntStatusChangedEvent {
  hunt_id: u64;
  new_status: HuntStatus;
  old_status: HuntStatus;
}

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
  23: {message:"SubmissionExpired"}
}


export interface MigrationReport {
  dry_run: boolean;
  from_version: u32;
  message: string;
  steps_applied: u32;
  succeeded: boolean;
  to_version: u32;
}


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
  /**
   * A recursive distribution attempt was detected during an external XLM or NFT call.
   */
  14: {message:"ReentrancyDetected"},
  /**
   * The tracked pool balance diverged from the actual XLM token balance.
   */
  15: {message:"PoolBalanceDivergence"}
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
  add_clue: ({hunt_id, question, answer, points, is_required}: {hunt_id: u64, question: string, answer: string, points: u32, is_required: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a get_clue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns clue information for a hunt/clue. Does not expose the answer hash.
   */
  get_clue: ({hunt_id, clue_id}: {hunt_id: u64, clue_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<ClueInfo>>>

  /**
   * Construct and simulate a list_clues transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns all clues for a hunt (question, points, required). Answer hashes are not exposed.
   */
  list_clues: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<ClueInfo>>>

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
   * * `start_time` - Optional start timestamp (0 means no start time restriction)
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
  create_hunt: ({creator, title, description, _start_time, end_time}: {creator: string, title: string, description: string, _start_time: Option<u64>, end_time: Option<u64>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

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
   * * `PlayerNotRegistered` - Player is not registered
   * * `HuntNotCompleted` - Player hasn't completed all required clues
   * * `RewardAlreadyClaimed` - Player already claimed their reward
   * * `NoRewardsConfigured` - No rewards set up for this hunt
   * * `InsufficientRewardPool` - All reward slots taken
   * * `RewardDistributionFailed` - Cross-contract call failed
   */
  complete_hunt: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunt_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_hunt_info: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Hunt>>>

  /**
   * Construct and simulate a run_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Runs storage migrations up to `target_version`. Set `dry_run` to simulate without writes.
   */
  run_migration: ({admin, target_version, dry_run}: {admin: string, target_version: u32, dry_run: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<MigrationReport>>

  /**
   * Construct and simulate a submit_answer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * This function verifies the submitted answer by hashing it and comparing
   * with the stored answer hash. If correct, updates player progress and emits
   * success events. If incorrect, emits an analytics event and returns an error.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `hunt_id` - The hunt ID
   * * `clue_id` - The clue ID to answer
   * * `player` - The address of the player submitting the answer
   * * `answer` - The plain-text answer submission
   * * `submission_nonce` - Caller-chosen unique nonce for this submission envelope
   * * `submitted_at` - Client timestamp captured when the submission was signed
   * 
   * # Returns
   * `Ok(())` on successful answer verification and progress update
   * 
   * # Errors
   * * `HuntNotFound` - Hunt does not exist
   * * `HuntNotActive` - Hunt is not currently active or has ended
   * * `PlayerNotRegistered` - Player has not registered for this hunt
   * * `ClueNotFound` - Clue does not exist in this hunt
   * * `ClueAlreadyCompleted` - Player has already completed this clue
   * * `InvalidAnswer` - Submitted answer does not match the stor
   */
  submit_answer: ({hunt_id, clue_id, player, answer, submission_nonce, submitted_at}: {hunt_id: u64, clue_id: u32, player: string, answer: string, submission_nonce: u64, submitted_at: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deactivate_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deactivate_hunt: ({hunt_id, caller}: {hunt_id: u64, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes schema version tracking on deploy or first admin call.
   */
  initialize_schema: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the on-chain storage schema version (0 when uninitialized).
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a rollback_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Rolls back to the schema version captured before the last migration.
   */
  rollback_migration: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<MigrationReport>>>

  /**
   * Construct and simulate a set_reward_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the RewardManager contract address for cross-contract reward distribution.
   */
  set_reward_manager: ({reward_manager}: {reward_manager: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

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
   * Construct and simulate a get_health_dashboard transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns contract health metrics for operator dashboards.
   */
  get_health_dashboard: (options?: MethodOptions) => Promise<AssembledTransaction<ContractHealth>>

  /**
   * Construct and simulate a get_hunt_leaderboard transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the top N players by score for a hunt (read-only).
   * Sorted by score descending, then by completion time ascending (earlier = better).
   * Limit is capped at 20 to control gas. Returns error if hunt does not exist.
   */
  get_hunt_leaderboard: ({hunt_id, limit}: {hunt_id: u64, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<LeaderboardEntry>>>>

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
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAAC0hlYWx0aEFsZXJ0AAAAAAMAAAAAAAAACmFsZXJ0X3R5cGUAAAAAABAAAAAAAAAABWNvdW50AAAAAAAABAAAAAAAAAALbGFzdF9sZWRnZXIAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADkNvbnRyYWN0SGVhbHRoAAAAAAAFAAAAAAAAAA1hY3RpdmVfYWxlcnRzAAAAAAAABAAAAAAAAAANYXZnX2dhc191bml0cwAAAAAAAAYAAAAAAAAAEmZhaWxlZF9pbnZvY2F0aW9ucwAAAAAABgAAAAAAAAAQZmFpbHVyZV9yYXRlX2JwcwAAAAQAAAAAAAAAEXRvdGFsX2ludm9jYXRpb25zAAAAAAAABg==",
        "AAAAAAAAAz5BZGRzIGEgY2x1ZSB0byBhIGh1bnQuIE9ubHkgdGhlIGh1bnQgY3JlYXRvciBjYW4gYWRkIGNsdWVzLgpBbnN3ZXJzIGFyZSBoYXNoZWQgd2l0aCBTSEEyNTYgYmVmb3JlIHN0b3JhZ2U7IHRoZSBoYXNoIGlzIG5ldmVyIGV4cG9zZWQuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBhZGQgdGhlIGNsdWUgdG8KKiBgcXVlc3Rpb25gIC0gVGhlIGNsdWUgcXVlc3Rpb24gdGV4dCAobWF4IDIwMDAgY2hhcnMsIG5vbi1lbXB0eSkKKiBgYW5zd2VyYCAtIFBsYWluLXRleHQgYW5zd2VyOyBub3JtYWxpemVkICh0cmltbWVkLCBsb3dlcmNhc2VkKSB0aGVuIGhhc2hlZAoqIGBwb2ludHNgIC0gUG9pbnRzIGF3YXJkZWQgZm9yIHNvbHZpbmcgdGhpcyBjbHVlCiogYGlzX3JlcXVpcmVkYCAtIFdoZXRoZXIgdGhpcyBjbHVlIG11c3QgYmUgc29sdmVkIHRvIGNvbXBsZXRlIHRoZSBodW50CgojIFJldHVybnMKVGhlIHNlcXVlbnRpYWwgY2x1ZSBJRCBhc3NpZ25lZCB3aXRoaW4gdGhlIGh1bnQKCiMgRXJyb3JzCiogYEh1bnROb3RGb3VuZGAgLSBIdW50IGRvZXMgbm90IGV4aXN0CiogYEludmFsaWRIdW50U3RhdHVzYCAtIEh1bnQgaXMgbm90IGluIERyYWZ0CiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90IHRoZSBodW50IGNyZWF0b3IKKiBgVG9vTWFueUNsdWVzYCAtIEh1bnQgYWxyZWFkeSBoYXMgbWF4IGNsdWVzCiogYEludmFsaWRRdWVzdGlvbmAgLSBRdWVzdGlvbiBlbXB0eSBvciB0b28gbG9uZwoqIGBJbnZhbGlkQW5zd2VyYCAtIEFuc3dlciBlbXB0eSBvciB0b28gbG9uZwAAAAAACGFkZF9jbHVlAAAABQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAhxdWVzdGlvbgAAABAAAAAAAAAABmFuc3dlcgAAAAAAEAAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAtpc19yZXF1aXJlZAAAAAABAAAAAQAAA+kAAAAEAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAEpSZXR1cm5zIGNsdWUgaW5mb3JtYXRpb24gZm9yIGEgaHVudC9jbHVlLiBEb2VzIG5vdCBleHBvc2UgdGhlIGFuc3dlciBoYXNoLgAAAAAACGdldF9jbHVlAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAdjbHVlX2lkAAAAAAQAAAABAAAD6QAAB9AAAAAIQ2x1ZUluZm8AAAfQAAAADUh1bnRFcnJvckNvZGUAAAA=",
        "AAAAAAAAAFlSZXR1cm5zIGFsbCBjbHVlcyBmb3IgYSBodW50IChxdWVzdGlvbiwgcG9pbnRzLCByZXF1aXJlZCkuIEFuc3dlciBoYXNoZXMgYXJlIG5vdCBleHBvc2VkLgAAAAAAAApsaXN0X2NsdWVzAAAAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAD6gAAB9AAAAAIQ2x1ZUluZm8=",
        "AAAAAAAAAAAAAAALY2FuY2VsX2h1bnQAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZjYWxsZXIAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAsdDcmVhdGVzIGEgbmV3IHNjYXZlbmdlciBodW50IHdpdGggdGhlIHByb3ZpZGVkIG1ldGFkYXRhLgoKIyBBcmd1bWVudHMKKiBgZW52YCAtIFRoZSBTb3JvYmFuIGVudmlyb25tZW50CiogYGNyZWF0b3JgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGh1bnQgY3JlYXRvciAodHlwaWNhbGx5IHVzZSBlbnYuaW52b2tlcigpIGZyb20gdGhlIGNhbGxlcikKKiBgdGl0bGVgIC0gVGhlIHRpdGxlIG9mIHRoZSBodW50IChtYXggMjAwIGNoYXJhY3RlcnMpCiogYGRlc2NyaXB0aW9uYCAtIFRoZSBkZXNjcmlwdGlvbiBvZiB0aGUgaHVudCAobWF4IDIwMDAgY2hhcmFjdGVycykKKiBgc3RhcnRfdGltZWAgLSBPcHRpb25hbCBzdGFydCB0aW1lc3RhbXAgKDAgbWVhbnMgbm8gc3RhcnQgdGltZSByZXN0cmljdGlvbikKKiBgZW5kX3RpbWVgIC0gT3B0aW9uYWwgZW5kIHRpbWVzdGFtcCAoMCBtZWFucyBubyBlbmQgdGltZSByZXN0cmljdGlvbikKCiMgUmV0dXJucwpUaGUgdW5pcXVlIGh1bnQgSUQgb2YgdGhlIG5ld2x5IGNyZWF0ZWQgaHVudAoKIyBFcnJvcnMKKiBgSW52YWxpZFRpdGxlYCAtIElmIHRpdGxlIGlzIGVtcHR5IG9yIGV4Y2VlZHMgbWF4aW11bSBsZW5ndGgKKiBgSW52YWxpZERlc2NyaXB0aW9uYCAtIElmIGRlc2NyaXB0aW9uIGV4Y2VlZHMgbWF4aW11bSBsZW5ndGgKKiBgSW52YWxpZEFkZHJlc3NgIC0gSWYgY3JlYXRvciBhZGRyZXNzIGlzIGludmFsaWQAAAAAC2NyZWF0ZV9odW50AAAAAAUAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAFdGl0bGUAAAAAAAAQAAAAAAAAAAtkZXNjcmlwdGlvbgAAAAAQAAAAAAAAAAtfc3RhcnRfdGltZQAAAAPoAAAABgAAAAAAAAAIZW5kX3RpbWUAAAPoAAAABgAAAAEAAAPpAAAABgAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAANYWN0aXZhdGVfaHVudAAAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAylDb21wbGV0ZXMgYSBodW50IGZvciBhIHBsYXllciBhbmQgZGlzdHJpYnV0ZXMgcmV3YXJkcy4KClRoaXMgZnVuY3Rpb24gdmVyaWZpZXMgdGhhdCB0aGUgcGxheWVyIGhhcyBjb21wbGV0ZWQgYWxsIHJlcXVpcmVkIGNsdWVzLAp0aGVuIGRpc3RyaWJ1dGVzIHJld2FyZHMgdmlhIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IChpZiBjb25maWd1cmVkKQphbmQgdXBkYXRlcyB0aGUgcGxheWVyJ3MgcmV3YXJkIHN0YXR1cy4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IElECiogYHBsYXllcmAgLSBUaGUgcGxheWVyIGNsYWltaW5nIGNvbXBsZXRpb24vcmV3YXJkcwoKIyBSZXR1cm5zCmBPaygoKSlgIG9uIHN1Y2Nlc3NmdWwgcmV3YXJkIGNsYWltCgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkb2VzIG5vdCBleGlzdAoqIGBQbGF5ZXJOb3RSZWdpc3RlcmVkYCAtIFBsYXllciBpcyBub3QgcmVnaXN0ZXJlZAoqIGBIdW50Tm90Q29tcGxldGVkYCAtIFBsYXllciBoYXNuJ3QgY29tcGxldGVkIGFsbCByZXF1aXJlZCBjbHVlcwoqIGBSZXdhcmRBbHJlYWR5Q2xhaW1lZGAgLSBQbGF5ZXIgYWxyZWFkeSBjbGFpbWVkIHRoZWlyIHJld2FyZAoqIGBOb1Jld2FyZHNDb25maWd1cmVkYCAtIE5vIHJld2FyZHMgc2V0IHVwIGZvciB0aGlzIGh1bnQKKiBgSW5zdWZmaWNpZW50UmV3YXJkUG9vbGAgLSBBbGwgcmV3YXJkIHNsb3RzIHRha2VuCiogYFJld2FyZERpc3RyaWJ1dGlvbkZhaWxlZGAgLSBDcm9zcy1jb250cmFjdCBjYWxsIGZhaWxlZAAAAAAAAA1jb21wbGV0ZV9odW50AAAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAAAAAAANZ2V0X2h1bnRfaW5mbwAAAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAH0AAAAARIdW50AAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAFlSdW5zIHN0b3JhZ2UgbWlncmF0aW9ucyB1cCB0byBgdGFyZ2V0X3ZlcnNpb25gLiBTZXQgYGRyeV9ydW5gIHRvIHNpbXVsYXRlIHdpdGhvdXQgd3JpdGVzLgAAAAAAAA1ydW5fbWlncmF0aW9uAAAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAA50YXJnZXRfdmVyc2lvbgAAAAAABAAAAAAAAAAHZHJ5X3J1bgAAAAABAAAAAQAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AA==",
        "AAAAAAAABABUaGlzIGZ1bmN0aW9uIHZlcmlmaWVzIHRoZSBzdWJtaXR0ZWQgYW5zd2VyIGJ5IGhhc2hpbmcgaXQgYW5kIGNvbXBhcmluZwp3aXRoIHRoZSBzdG9yZWQgYW5zd2VyIGhhc2guIElmIGNvcnJlY3QsIHVwZGF0ZXMgcGxheWVyIHByb2dyZXNzIGFuZCBlbWl0cwpzdWNjZXNzIGV2ZW50cy4gSWYgaW5jb3JyZWN0LCBlbWl0cyBhbiBhbmFseXRpY3MgZXZlbnQgYW5kIHJldHVybnMgYW4gZXJyb3IuCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgaHVudF9pZGAgLSBUaGUgaHVudCBJRAoqIGBjbHVlX2lkYCAtIFRoZSBjbHVlIElEIHRvIGFuc3dlcgoqIGBwbGF5ZXJgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIHBsYXllciBzdWJtaXR0aW5nIHRoZSBhbnN3ZXIKKiBgYW5zd2VyYCAtIFRoZSBwbGFpbi10ZXh0IGFuc3dlciBzdWJtaXNzaW9uCiogYHN1Ym1pc3Npb25fbm9uY2VgIC0gQ2FsbGVyLWNob3NlbiB1bmlxdWUgbm9uY2UgZm9yIHRoaXMgc3VibWlzc2lvbiBlbnZlbG9wZQoqIGBzdWJtaXR0ZWRfYXRgIC0gQ2xpZW50IHRpbWVzdGFtcCBjYXB0dXJlZCB3aGVuIHRoZSBzdWJtaXNzaW9uIHdhcyBzaWduZWQKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzZnVsIGFuc3dlciB2ZXJpZmljYXRpb24gYW5kIHByb2dyZXNzIHVwZGF0ZQoKIyBFcnJvcnMKKiBgSHVudE5vdEZvdW5kYCAtIEh1bnQgZG9lcyBub3QgZXhpc3QKKiBgSHVudE5vdEFjdGl2ZWAgLSBIdW50IGlzIG5vdCBjdXJyZW50bHkgYWN0aXZlIG9yIGhhcyBlbmRlZAoqIGBQbGF5ZXJOb3RSZWdpc3RlcmVkYCAtIFBsYXllciBoYXMgbm90IHJlZ2lzdGVyZWQgZm9yIHRoaXMgaHVudAoqIGBDbHVlTm90Rm91bmRgIC0gQ2x1ZSBkb2VzIG5vdCBleGlzdCBpbiB0aGlzIGh1bnQKKiBgQ2x1ZUFscmVhZHlDb21wbGV0ZWRgIC0gUGxheWVyIGhhcyBhbHJlYWR5IGNvbXBsZXRlZCB0aGlzIGNsdWUKKiBgSW52YWxpZEFuc3dlcmAgLSBTdWJtaXR0ZWQgYW5zd2VyIGRvZXMgbm90IG1hdGNoIHRoZSBzdG9yAAAADXN1Ym1pdF9hbnN3ZXIAAAAAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAAZhbnN3ZXIAAAAAABAAAAAAAAAAEHN1Ym1pc3Npb25fbm9uY2UAAAAGAAAAAAAAAAxzdWJtaXR0ZWRfYXQAAAAGAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAAAAAAAPZGVhY3RpdmF0ZV9odW50AAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAsFSZWdpc3RlcnMgYSBwbGF5ZXIgZm9yIGFuIGFjdGl2ZSBodW50LiBUaGUgY2FsbGVyIG11c3QgcGFzcyB0aGVpciBhZGRyZXNzIGFuZCBhdXRob3JpemU7Cm9ubHkgdGhhdCBpZGVudGl0eSBjYW4gcmVnaXN0ZXIgdGhlbXNlbHZlcy4gSW5pdGlhbGl6ZXMgcGxheWVyIHByb2dyZXNzIGFuZCBwcmV2ZW50cwpkdXBsaWNhdGUgcmVnaXN0cmF0aW9ucy4gUmVnaXN0cmF0aW9uIGlzIG9ubHkgYWxsb3dlZCB3aGlsZSB0aGUgaHVudCBpcyBhY3RpdmUgYW5kCihpZiBzZXQpIGJlZm9yZSBlbmRfdGltZS4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRvIHJlZ2lzdGVyIGZvcgoqIGBwbGF5ZXJgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIHBsYXllciAobXVzdCBhdXRob3JpemUgdGhlIGNhbGwgdmlhIHJlcXVpcmVfYXV0aCkKCiMgUmV0dXJucwpgT2soKCkpYCBvbiBzdWNjZXNzCgojIEVycm9ycwoqIGBIdW50Tm90Rm91bmRgIC0gSHVudCBkb2VzIG5vdCBleGlzdAoqIGBJbnZhbGlkSHVudFN0YXR1c2AgLSBIdW50IGlzIG5vdCBpbiBBY3RpdmUgc3RhdHVzCiogYEh1bnROb3RBY3RpdmVgIC0gSHVudCBoYXMgZW5kZWQgKHBhc3QgZW5kX3RpbWUpCiogYER1cGxpY2F0ZVJlZ2lzdHJhdGlvbmAgLSBQbGF5ZXIgaXMgYWxyZWFkeSByZWdpc3RlcmVkIGZvciB0aGlzIGh1bnQAAAAAAAAPcmVnaXN0ZXJfcGxheWVyAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAAEJJbml0aWFsaXplcyBzY2hlbWEgdmVyc2lvbiB0cmFja2luZyBvbiBkZXBsb3kgb3IgZmlyc3QgYWRtaW4gY2FsbC4AAAAAABFpbml0aWFsaXplX3NjaGVtYQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAENSZXR1cm5zIHRoZSBvbi1jaGFpbiBzdG9yYWdlIHNjaGVtYSB2ZXJzaW9uICgwIHdoZW4gdW5pbml0aWFsaXplZCkuAAAAABJnZXRfc2NoZW1hX3ZlcnNpb24AAAAAAAAAAAABAAAABA==",
        "AAAAAAAAAERSb2xscyBiYWNrIHRvIHRoZSBzY2hlbWEgdmVyc2lvbiBjYXB0dXJlZCBiZWZvcmUgdGhlIGxhc3QgbWlncmF0aW9uLgAAABJyb2xsYmFja19taWdyYXRpb24AAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPoAAAH0AAAAA9NaWdyYXRpb25SZXBvcnQA",
        "AAAAAAAAAE9TZXRzIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IGFkZHJlc3MgZm9yIGNyb3NzLWNvbnRyYWN0IHJld2FyZCBkaXN0cmlidXRpb24uAAAAABJzZXRfcmV3YXJkX21hbmFnZXIAAAAAAAEAAAAAAAAADnJld2FyZF9tYW5hZ2VyAAAAAAATAAAAAA==",
        "AAAAAAAAAKFSZXR1cm5zIHRoZSBsaXN0IG9mIGNsdWUgSURzIHRoYXQgdGhlIHBsYXllciBoYXMgY29tcGxldGVkIGZvciBhIGh1bnQgKHJlYWQtb25seSkuClVzZWZ1bCBmb3IgVUkgdG8gc2hvdyBwcm9ncmVzcy4gUmV0dXJucyBlbXB0eSB2ZWMgaWYgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkLgAAAAAAABNnZXRfY29tcGxldGVkX2NsdWVzAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAGcGxheWVyAAAAAAATAAAAAQAAA+oAAAAE",
        "AAAAAAAAAIlSZXR1cm5zIGFnZ3JlZ2F0ZSBzdGF0aXN0aWNzIGZvciBhIGh1bnQgKHJlYWQtb25seSk6IHRvdGFsIHBsYXllcnMsIGNvbXBsZXRpb24gcmF0ZSwgYXZlcmFnZSBzY29yZS4KUmV0dXJucyBlcnJvciBpZiBodW50IGRvZXMgbm90IGV4aXN0LgAAAAAAABNnZXRfaHVudF9zdGF0aXN0aWNzAAAAAAEAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAH0AAAAA5IdW50U3RhdGlzdGljcwAAAAAH0AAAAA1IdW50RXJyb3JDb2RlAAAA",
        "AAAAAAAAAJJSZXR1cm5zIHBsYXllciBwcm9ncmVzcyBmb3IgYSBodW50IChyZWFkLW9ubHkpLgpJbmNsdWRlcyBjb21wbGV0ZWQgY2x1ZXMsIHNjb3JlLCBhbmQgY29tcGxldGlvbiBzdGF0dXMuClJldHVybnMgZXJyb3IgaWYgcGxheWVyIGlzIG5vdCByZWdpc3RlcmVkLgAAAAAAE2dldF9wbGF5ZXJfcHJvZ3Jlc3MAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAD6QAAB9AAAAAOUGxheWVyUHJvZ3Jlc3MAAAAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAAAAADhSZXR1cm5zIGNvbnRyYWN0IGhlYWx0aCBtZXRyaWNzIGZvciBvcGVyYXRvciBkYXNoYm9hcmRzLgAAABRnZXRfaGVhbHRoX2Rhc2hib2FyZAAAAAAAAAABAAAH0AAAAA5Db250cmFjdEhlYWx0aAAA",
        "AAAAAAAAANhSZXR1cm5zIHRoZSB0b3AgTiBwbGF5ZXJzIGJ5IHNjb3JlIGZvciBhIGh1bnQgKHJlYWQtb25seSkuClNvcnRlZCBieSBzY29yZSBkZXNjZW5kaW5nLCB0aGVuIGJ5IGNvbXBsZXRpb24gdGltZSBhc2NlbmRpbmcgKGVhcmxpZXIgPSBiZXR0ZXIpLgpMaW1pdCBpcyBjYXBwZWQgYXQgMjAgdG8gY29udHJvbCBnYXMuIFJldHVybnMgZXJyb3IgaWYgaHVudCBkb2VzIG5vdCBleGlzdC4AAAAUZ2V0X2h1bnRfbGVhZGVyYm9hcmQAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPpAAAD6gAAB9AAAAAQTGVhZGVyYm9hcmRFbnRyeQAAB9AAAAANSHVudEVycm9yQ29kZQAAAA==",
        "AAAAAQAAAGFTdG9yZWQgY2x1ZSB3aXRoIFNIQTI1NiBhbnN3ZXIgaGFzaC4gVGhlIGhhc2ggaXMgbmV2ZXIgZXhwb3NlZCB2aWEgZ2V0X2NsdWUvbGlzdF9jbHVlcyBvciBldmVudHMuAAAAAAAAAAAAAARDbHVlAAAABQAAAAAAAAALYW5zd2VyX2hhc2gAAAAD7gAAACAAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAALaXNfcmVxdWlyZWQAAAAAAQAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAhxdWVzdGlvbgAAABA=",
        "AAAAAQAAAAAAAAAAAAAABEh1bnQAAAALAAAAAAAAAAxhY3RpdmF0ZWRfYXQAAAAGAAAAAAAAAApjcmVhdGVkX2F0AAAAAAAGAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAC2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAACGVuZF90aW1lAAAABgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAA5yZXF1aXJlZF9jbHVlcwAAAAAABAAAAAAAAAANcmV3YXJkX2NvbmZpZwAAAAAAB9AAAAAMUmV3YXJkQ29uZmlnAAAAAAAAAAZzdGF0dXMAAAAAB9AAAAAKSHVudFN0YXR1cwAAAAAAAAAAAAV0aXRsZQAAAAAAABAAAAAAAAAAC3RvdGFsX2NsdWVzAAAAAAQ=",
        "AAAAAQAAAEBDbHVlIGluZm8gcmV0dXJuZWQgYnkgZ2V0X2NsdWUvbGlzdF9jbHVlcy4gRXhjbHVkZXMgYW5zd2VyIGhhc2guAAAAAAAAAAhDbHVlSW5mbwAAAAQAAAAAAAAAB2NsdWVfaWQAAAAABAAAAAAAAAALaXNfcmVxdWlyZWQAAAAAAQAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAhxdWVzdGlvbgAAABA=",
        "AAAAAQAAAAAAAAAAAAAACExvY2F0aW9uAAAAAwAAAAAAAAAIbGF0aXR1ZGUAAAAHAAAAAAAAAAlsb25naXR1ZGUAAAAAAAAHAAAAAAAAAAZyYWRpdXMAAAAAAAQ=",
        "AAAAAgAAAAAAAAAAAAAACkh1bnRTdGF0dXMAAAAAAAQAAAAAAAAAAAAAAAVEcmFmdAAAAAAAAAAAAAAAAAAABkFjdGl2ZQAAAAAAAAAAAAAAAAAJQ29tcGxldGVkAAAAAAAAAAAAAAAAAAAJQ2FuY2VsbGVkAAAA",
        "AAAAAQAAAAAAAAAAAAAADFJld2FyZENvbmZpZwAAAAUAAAAAAAAADWNsYWltZWRfY291bnQAAAAAAAAEAAAAAAAAAAttYXhfd2lubmVycwAAAAAEAAAAAAAAAAxuZnRfY29udHJhY3QAAAPoAAAAEwAAAAAAAAALbmZ0X2VuYWJsZWQAAAAAAQAAAAAAAAAIeGxtX3Bvb2wAAAAL",
        "AAAAAQAAAD5FbWl0dGVkIHdoZW4gYSBjbHVlIGlzIGFkZGVkLiBEb2VzIG5vdCBleHBvc2UgdGhlIGFuc3dlciBoYXNoLgAAAAAAAAAAAA5DbHVlQWRkZWRFdmVudAAAAAAABgAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAALaXNfcmVxdWlyZWQAAAAAAQAAAAAAAAAGcG9pbnRzAAAAAAAEAAAAAAAAAAhxdWVzdGlvbgAAABA=",
        "AAAAAQAAADlBZ2dyZWdhdGUgc3RhdGlzdGljcyBmb3IgYSBodW50IChyZWFkLW9ubHkgcXVlcnkgcmVzdWx0KS4AAAAAAAAAAAAADkh1bnRTdGF0aXN0aWNzAAAAAAAFAAAAAAAAAA1hdmVyYWdlX3Njb3JlAAAAAAAABAAAAAAAAAAPY29tcGxldGVkX2NvdW50AAAAAAQAAAAAAAAAF2NvbXBsZXRpb25fcmF0ZV9wZXJjZW50AAAAAAQAAAAAAAAADXRvdGFsX3BsYXllcnMAAAAAAAAEAAAAAAAAAA90b3RhbF9zY29yZV9zdW0AAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADlBsYXllclByb2dyZXNzAAAAAAAIAAAAAAAAAAxjb21wbGV0ZWRfYXQAAAAGAAAAAAAAAA9jb21wbGV0ZWRfY2x1ZXMAAAAD6gAAAAQAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAMaXNfY29tcGxldGVkAAAAAQAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAA5yZXdhcmRfY2xhaW1lZAAAAAAAAQAAAAAAAAAKc3RhcnRlZF9hdAAAAAAABgAAAAAAAAALdG90YWxfc2NvcmUAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAEEh1bnRDcmVhdGVkRXZlbnQAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAFdGl0bGUAAAAAAAAQ",
        "AAAAAQAAAElMZWFkZXJib2FyZCBlbnRyeSBmb3IgYSBzaW5nbGUgcGxheWVyIGluIGEgaHVudCAocmVhZC1vbmx5IHF1ZXJ5IHJlc3VsdCkuAAAAAAAAAAAAABBMZWFkZXJib2FyZEVudHJ5AAAABQAAAAAAAAAMY29tcGxldGVkX2F0AAAABgAAAAAAAAAMaXNfY29tcGxldGVkAAAAAQAAAAAAAAAGcGxheWVyAAAAAAATAAAAAAAAAARyYW5rAAAABAAAAAAAAAAFc2NvcmUAAAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAEkNsdWVDb21wbGV0ZWRFdmVudAAAAAAABAAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAANcG9pbnRzX2Vhcm5lZAAAAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAEkh1bnRBY3RpdmF0ZWRFdmVudAAAAAAAAgAAAAAAAAAMYWN0aXZhdGVkX2F0AAAABgAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAEkh1bnRDYW5jZWxsZWRFdmVudAAAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAEkh1bnRDb21wbGV0ZWRFdmVudAAAAAAABQAAAAAAAAAPY29tcGxldGlvbl9yYW5rAAAAAAQAAAAAAAAAD2NvbXBsZXRpb25fdGltZQAAAAAGAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAALdG90YWxfc2NvcmUAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAElJld2FyZENsYWltZWRFdmVudAAAAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAtuZnRfYXdhcmRlZAAAAAABAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAACnhsbV9hbW91bnQAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAFEFuc3dlckluY29ycmVjdEV2ZW50AAAABAAAAAAAAAAHY2x1ZV9pZAAAAAAEAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAFEh1bnREZWFjdGl2YXRlZEV2ZW50AAAAAQAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAQAAADNFbWl0dGVkIHdoZW4gYSBwbGF5ZXIgcmVnaXN0ZXJzIGZvciBhbiBhY3RpdmUgaHVudC4AAAAAAAAAABVQbGF5ZXJSZWdpc3RlcmVkRXZlbnQAAAAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAAFkh1bnRTdGF0dXNDaGFuZ2VkRXZlbnQAAAAAAAMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAKbmV3X3N0YXR1cwAAAAAH0AAAAApIdW50U3RhdHVzAAAAAAAAAAAACm9sZF9zdGF0dXMAAAAAB9AAAAAKSHVudFN0YXR1cwAA",
        "AAAABAAAAAAAAAAAAAAADUh1bnRFcnJvckNvZGUAAAAAAAAXAAAAAAAAAAxIdW50Tm90Rm91bmQAAAABAAAAAAAAAAxDbHVlTm90Rm91bmQAAAACAAAAAAAAABFJbnZhbGlkSHVudFN0YXR1cwAAAAAAAAMAAAAAAAAAE1BsYXllck5vdFJlZ2lzdGVyZWQAAAAABAAAAAAAAAAUQ2x1ZUFscmVhZHlDb21wbGV0ZWQAAAAFAAAAAAAAAA1JbnZhbGlkQW5zd2VyAAAAAAAABgAAAAAAAAANSHVudE5vdEFjdGl2ZQAAAAAAAAcAAAAAAAAADFVuYXV0aG9yaXplZAAAAAgAAAAAAAAAFkluc3VmZmljaWVudFJld2FyZFBvb2wAAAAAAAkAAAAAAAAAFUR1cGxpY2F0ZVJlZ2lzdHJhdGlvbgAAAAAAAAoAAAAAAAAADEludmFsaWRUaXRsZQAAAAsAAAAAAAAAEkludmFsaWREZXNjcmlwdGlvbgAAAAAADAAAAAAAAAAOSW52YWxpZEFkZHJlc3MAAAAAAA0AAAAAAAAADFRvb01hbnlDbHVlcwAAAA4AAAAAAAAAD0ludmFsaWRRdWVzdGlvbgAAAAAPAAAAAAAAAAxSZWZ1bmRGYWlsZWQAAAAQAAAAAAAAAAxOb0NsdWVzQWRkZWQAAAARAAAAAAAAABBIdW50Tm90Q29tcGxldGVkAAAAEgAAAAAAAAAUUmV3YXJkQWxyZWFkeUNsYWltZWQAAAATAAAAAAAAABhSZXdhcmREaXN0cmlidXRpb25GYWlsZWQAAAAUAAAAAAAAABNOb1Jld2FyZHNDb25maWd1cmVkAAAAABUAAAAAAAAAE0R1cGxpY2F0ZVN1Ym1pc3Npb24AAAAAFgAAAAAAAAARU3VibWlzc2lvbkV4cGlyZWQAAAAAAAAX",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=",
        "AAAAAQAAAFdDb25maWd1cmF0aW9uIGZvciBkaXN0cmlidXRpbmcgcmV3YXJkcyBhY3Jvc3MgdGhlIEh1bnR5Q29yZSDihpQgUmV3YXJkTWFuYWdlciBib3VuZGFyeS4AAAAAAAAAAAxSZXdhcmRDb25maWcAAAAIAAAAAAAAAAxuZnRfY29udHJhY3QAAAPoAAAAEwAAAAAAAAAPbmZ0X2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAADm5mdF9odW50X3RpdGxlAAAAAAAQAAAAAAAAAA1uZnRfaW1hZ2VfdXJpAAAAAAAAEAAAAAAAAAAKbmZ0X3Jhcml0eQAAAAAABAAAAAAAAAAIbmZ0X3RpZXIAAAAEAAAAAAAAAAluZnRfdGl0bGUAAAAAAAAQAAAAAAAAAAp4bG1fYW1vdW50AAAAAAPoAAAACw==",
        "AAAABAAAAAAAAAAAAAAAD1Jld2FyZEVycm9yQ29kZQAAAAAPAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAAAQAAAAAAAAAQSW5zdWZmaWNpZW50UG9vbAAAAAIAAAAAAAAAEkFscmVhZHlEaXN0cmlidXRlZAAAAAAAAwAAAAAAAAAOVHJhbnNmZXJGYWlsZWQAAAAAAAQAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAAAFAAAAAAAAAA1JbnZhbGlkQ29uZmlnAAAAAAAABgAAAAAAAAANTmZ0TWludEZhaWxlZAAAAAAAAAcAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAAACAAAAAAAAAAMUG9vbE5vdEZvdW5kAAAACQAAAAAAAAAMVW5hdXRob3JpemVkAAAACgAAAAAAAAASQmVsb3dNaW5pbXVtQW1vdW50AAAAAAALAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAwAAAAAAAAADEh1bnROb3RGb3VuZAAAAA0AAABRQSByZWN1cnNpdmUgZGlzdHJpYnV0aW9uIGF0dGVtcHQgd2FzIGRldGVjdGVkIGR1cmluZyBhbiBleHRlcm5hbCBYTE0gb3IgTkZUIGNhbGwuAAAAAAAAElJlZW50cmFuY3lEZXRlY3RlZAAAAAAADgAAAERUaGUgdHJhY2tlZCBwb29sIGJhbGFuY2UgZGl2ZXJnZWQgZnJvbSB0aGUgYWN0dWFsIFhMTSB0b2tlbiBiYWxhbmNlLgAAABVQb29sQmFsYW5jZURpdmVyZ2VuY2UAAAAAAAAP" ]),
      options
    )
  }
  public readonly fromJSON = {
    add_clue: this.txFromJSON<Result<u32>>,
        get_clue: this.txFromJSON<Result<ClueInfo>>,
        list_clues: this.txFromJSON<Array<ClueInfo>>,
        cancel_hunt: this.txFromJSON<Result<void>>,
        create_hunt: this.txFromJSON<Result<u64>>,
        activate_hunt: this.txFromJSON<Result<void>>,
        complete_hunt: this.txFromJSON<Result<void>>,
        get_hunt_info: this.txFromJSON<Result<Hunt>>,
        run_migration: this.txFromJSON<MigrationReport>,
        submit_answer: this.txFromJSON<Result<void>>,
        deactivate_hunt: this.txFromJSON<Result<void>>,
        register_player: this.txFromJSON<Result<void>>,
        initialize_schema: this.txFromJSON<null>,
        get_schema_version: this.txFromJSON<u32>,
        rollback_migration: this.txFromJSON<Option<MigrationReport>>,
        set_reward_manager: this.txFromJSON<null>,
        get_completed_clues: this.txFromJSON<Array<u32>>,
        get_hunt_statistics: this.txFromJSON<Result<HuntStatistics>>,
        get_player_progress: this.txFromJSON<Result<PlayerProgress>>,
        get_health_dashboard: this.txFromJSON<ContractHealth>,
        get_hunt_leaderboard: this.txFromJSON<Result<Array<LeaderboardEntry>>>
  }
}
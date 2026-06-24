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
 * Event emitted when admin withdraws unclaimed rewards from a pool.
 */
export interface AdminWithdrawEvent {
  admin: string;
  amount: i128;
  hunt_id: u64;
}


/**
 * Event emitted when a reward pool is funded.
 */
export interface RewardPoolFundedEvent {
  amount: i128;
  funder: string;
  hunt_id: u64;
  new_balance: i128;
  total_deposited: i128;
}


/**
 * Event emitted when a reward pool is created for a hunt.
 */
export interface RewardPoolCreatedEvent {
  creator: string;
  hunt_id: u64;
  min_distribution_amount: i128;
}


/**
 * Event emitted when rewards are successfully distributed.
 */
export interface RewardsDistributedEvent {
  hunt_id: u64;
  nft_id: Option<u64>;
  player: string;
  xlm_amount: i128;
}


/**
 * Configuration for a reward pool, set at creation time.
 */
export interface RewardPoolConfig {
  /**
 * Address of the hunt creator who owns this pool.
 * Only the creator is authorized to fund it.
 */
creator: string;
  /**
 * Minimum XLM amount per distribution. 0 means no minimum enforced.
 */
min_distribution_amount: i128;
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
 * Internal record stored for each distribution.
 */
export interface DistributionRecord {
  nft_id: Option<u64>;
  xlm_amount: i128;
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
 * XLM amount distributed (0 if none).
 */
xlm_amount: i128;
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
  14: {message:"PoolBalanceDivergence"}
}

export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes the RewardManager with the XLM token contract address (SAC).
   * Must be called once before any reward distribution.
   */
  initialize: ({admin, xlm_token}: {admin: string, xlm_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refund_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Refunds the entire remaining pool balance for a hunt back to the pool creator.
   * Can only be called by the same creator that owns the pool.
   */
  refund_pool: ({creator, hunt_id}: {creator: string, hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a run_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  run_migration: ({admin, target_version, dry_run}: {admin: string, target_version: u32, dry_run: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<MigrationReport>>

  /**
   * Construct and simulate a validate_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Validates whether a pool can cover a given distribution amount.
   * 
   * Checks that:
   * - The pool exists (was created via create_reward_pool)
   * - The required_amount is positive
   * - The pool balance >= required_amount
   * - The required_amount meets the pool's minimum distribution threshold (if set)
   * 
   * Returns a `ValidationResult` with balance details regardless of validity,
   * so callers can diagnose shortfalls without a separate query.
   */
  validate_pool: ({hunt_id, required_amount}: {hunt_id: u64, required_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<ValidationResult>>

  /**
   * Construct and simulate a set_hunty_core transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the optional HuntyCore contract address used to validate hunt_id existence
   * in `create_reward_pool`. When set, pool creation will be rejected for unknown
   * hunt IDs. If not set, hunt_id is assumed caller-trusted.
   */
  set_hunty_core: ({admin, hunty_core}: {admin: string, hunty_core: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_reward_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the full status of a reward pool, including balance, totals, and configuration.
   * Returns None if no pool has been created for the given hunt_id.
   */
  get_reward_pool: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<RewardPoolStatus>>>

  /**
   * Construct and simulate a contract_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the contract version.
   */
  contract_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a fund_reward_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Funds the reward pool for a specific hunt.
   * 
   * The pool must have been created via `create_reward_pool` first.
   * Only the original pool creator is authorized to fund it.
   * Transfers XLM from the funder to this contract and records the balance.
   * 
   * # Arguments
   * * `funder` - The address funding the pool (must be the pool creator)
   * * `hunt_id` - The hunt to fund
   * * `amount` - XLM amount to add to the pool (must be > 0)
   * 
   * # Errors
   * * `PoolNotFound` - Pool has not been created yet
   * * `Unauthorized` - Funder is not the pool creator
   * * `InvalidAmount` - Amount is <= 0
   * * `NotInitialized` - XLM token address not set
   */
  fund_reward_pool: ({funder, hunt_id, amount}: {funder: string, hunt_id: u64, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_pool_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current reward pool balance for a hunt.
   */
  get_pool_balance: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_schema: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a create_reward_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a reward pool for a specific hunt.
   * 
   * Must be called before `fund_reward_pool`. Only the creator is authorized
   * to fund the pool after creation.
   * 
   * # Arguments
   * * `creator` - The hunt creator who will own and fund the pool
   * * `hunt_id` - The hunt this pool is for
   * * `min_distribution_amount` - Minimum XLM per distribution (0 = no minimum)
   * 
   * # Errors
   * * `PoolAlreadyExists` - A pool already exists for this hunt_id
   * * `InvalidAmount` - min_distribution_amount is negative
   * * `HuntNotFound` - hunt_id does not exist in HuntyCore (only when `set_hunty_core` has been called)
   */
  create_reward_pool: ({creator, hunt_id, min_distribution_amount}: {creator: string, hunt_id: u64, min_distribution_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Main entry point for reward distribution. Determines reward type from configuration,
   * routes to XLM and/or NFT handlers, and ensures atomic all-or-nothing execution.
   * 
   * # Arguments
   * * `hunt_id` - The hunt being rewarded
   * * `player_address` - The player receiving rewards
   * * `reward_config` - Configuration specifying XLM amount and/or NFT metadata
   * 
   * # Returns
   * `Ok(())` on success
   * 
   * # Errors
   * * `InvalidConfig` - No reward type configured or invalid values
   * * `NotInitialized` - XLM token not set (when XLM rewards requested)
   * * `AlreadyDistributed` - Rewards already distributed for this hunt/player
   * * `InsufficientPool` - Pool has insufficient XLM for requested amount
   * * `InvalidAmount` - XLM amount <= 0 (when XLM requested)
   * * `BelowMinimumAmount` - XLM amount is below the pool's minimum distribution threshold
   * * `NftMintFailed` - NFT minting failed (when NFT requested)
   */
  distribute_rewards: ({hunt_id, player_address, reward_config}: {hunt_id: u64, player_address: string, reward_config: RewardConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a rollback_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rollback_migration: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<MigrationReport>>>

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
   * Construct and simulate a get_health_dashboard transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_health_dashboard: (options?: MethodOptions) => Promise<AssembledTransaction<ContractHealth>>

  /**
   * Construct and simulate a is_reward_distributed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns whether a reward has been distributed to a player for a hunt.
   */
  is_reward_distributed: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_distribution_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the distribution status for a hunt/player pair.
   */
  get_distribution_status: ({hunt_id, player}: {hunt_id: u64, player: string}, options?: MethodOptions) => Promise<AssembledTransaction<DistributionStatus>>

  /**
   * Construct and simulate a set_nft_reward_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the default NftReward contract address used for NFT distributions
   * when a per-call NFT contract is not provided.
   */
  set_nft_reward_contract: ({admin, nft_contract}: {admin: string, nft_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a admin_withdraw_unclaimed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Allows the admin to withdraw any unclaimed (surplus) XLM remaining in a reward pool.
   * 
   * This is needed when a hunt concludes with fewer winners than anticipated,
   * leaving unspent XLM locked in the pool. Only the contract admin may call this.
   * 
   * # Arguments
   * * `admin` - The contract admin address (must match the stored admin)
   * * `hunt_id` - The hunt whose remaining pool balance to withdraw
   * * `recipient` - The address that will receive the withdrawn XLM
   * 
   * # Errors
   * * `NotInitialized` - Contract has not been initialized (no admin set)
   * * `Unauthorized` - Caller is not the contract admin
   * * `PoolNotFound` - No pool exists for this hunt_id
   * * `InvalidAmount` - Pool balance is zero (nothing to withdraw)
   */
  admin_withdraw_unclaimed: ({admin, hunt_id, recipient, amount}: {admin: string, hunt_id: u64, recipient: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_rewards_legacy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Legacy entry point for XLM-only distribution.
   * Kept for backward compatibility with HuntyCore. For NFT or full config support use distribute_rewards.
   * 
   * Note: `nft_enabled` is ignored — NFT distribution requires metadata and a contract address
   * that are not available on this path. Use `distribute_rewards` with `RewardConfig` instead.
   */
  distribute_rewards_legacy: ({player, hunt_id, xlm_amount, _nft_enabled}: {player: string, hunt_id: u64, xlm_amount: i128, _nft_enabled: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_total_xlm_distributed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total XLM distributed across all hunts (protocol-level metric).
   */
  get_total_xlm_distributed: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

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
        "AAAAAAAAAHxJbml0aWFsaXplcyB0aGUgUmV3YXJkTWFuYWdlciB3aXRoIHRoZSBYTE0gdG9rZW4gY29udHJhY3QgYWRkcmVzcyAoU0FDKS4KTXVzdCBiZSBjYWxsZWQgb25jZSBiZWZvcmUgYW55IHJld2FyZCBkaXN0cmlidXRpb24uAAAACmluaXRpYWxpemUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJeGxtX3Rva2VuAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAIlSZWZ1bmRzIHRoZSBlbnRpcmUgcmVtYWluaW5nIHBvb2wgYmFsYW5jZSBmb3IgYSBodW50IGJhY2sgdG8gdGhlIHBvb2wgY3JlYXRvci4KQ2FuIG9ubHkgYmUgY2FsbGVkIGJ5IHRoZSBzYW1lIGNyZWF0b3IgdGhhdCBvd25zIHRoZSBwb29sLgAAAAAAAAtyZWZ1bmRfcG9vbAAAAAACAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAAAAAAANcnVuX21pZ3JhdGlvbgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAAAAAAAB2RyeV9ydW4AAAAAAQAAAAEAAAfQAAAAD01pZ3JhdGlvblJlcG9ydAA=",
        "AAAAAAAAAaNWYWxpZGF0ZXMgd2hldGhlciBhIHBvb2wgY2FuIGNvdmVyIGEgZ2l2ZW4gZGlzdHJpYnV0aW9uIGFtb3VudC4KCkNoZWNrcyB0aGF0OgotIFRoZSBwb29sIGV4aXN0cyAod2FzIGNyZWF0ZWQgdmlhIGNyZWF0ZV9yZXdhcmRfcG9vbCkKLSBUaGUgcmVxdWlyZWRfYW1vdW50IGlzIHBvc2l0aXZlCi0gVGhlIHBvb2wgYmFsYW5jZSA+PSByZXF1aXJlZF9hbW91bnQKLSBUaGUgcmVxdWlyZWRfYW1vdW50IG1lZXRzIHRoZSBwb29sJ3MgbWluaW11bSBkaXN0cmlidXRpb24gdGhyZXNob2xkIChpZiBzZXQpCgpSZXR1cm5zIGEgYFZhbGlkYXRpb25SZXN1bHRgIHdpdGggYmFsYW5jZSBkZXRhaWxzIHJlZ2FyZGxlc3Mgb2YgdmFsaWRpdHksCnNvIGNhbGxlcnMgY2FuIGRpYWdub3NlIHNob3J0ZmFsbHMgd2l0aG91dCBhIHNlcGFyYXRlIHF1ZXJ5LgAAAAANdmFsaWRhdGVfcG9vbAAAAAAAAAIAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAPcmVxdWlyZWRfYW1vdW50AAAAAAsAAAABAAAH0AAAABBWYWxpZGF0aW9uUmVzdWx0",
        "AAAAAAAAANZTZXRzIHRoZSBvcHRpb25hbCBIdW50eUNvcmUgY29udHJhY3QgYWRkcmVzcyB1c2VkIHRvIHZhbGlkYXRlIGh1bnRfaWQgZXhpc3RlbmNlCmluIGBjcmVhdGVfcmV3YXJkX3Bvb2xgLiBXaGVuIHNldCwgcG9vbCBjcmVhdGlvbiB3aWxsIGJlIHJlamVjdGVkIGZvciB1bmtub3duCmh1bnQgSURzLiBJZiBub3Qgc2V0LCBodW50X2lkIGlzIGFzc3VtZWQgY2FsbGVyLXRydXN0ZWQuAAAAAAAOc2V0X2h1bnR5X2NvcmUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAKaHVudHlfY29yZQAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAJdSZXR1cm5zIHRoZSBmdWxsIHN0YXR1cyBvZiBhIHJld2FyZCBwb29sLCBpbmNsdWRpbmcgYmFsYW5jZSwgdG90YWxzLCBhbmQgY29uZmlndXJhdGlvbi4KUmV0dXJucyBOb25lIGlmIG5vIHBvb2wgaGFzIGJlZW4gY3JlYXRlZCBmb3IgdGhlIGdpdmVuIGh1bnRfaWQuAAAAAA9nZXRfcmV3YXJkX3Bvb2wAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAA+gAAAfQAAAAEFJld2FyZFBvb2xTdGF0dXM=",
        "AAAAAAAAAB1SZXR1cm5zIHRoZSBjb250cmFjdCB2ZXJzaW9uLgAAAAAAABBjb250cmFjdF92ZXJzaW9uAAAAAAAAAAEAAAAE",
        "AAAAAAAAAlVGdW5kcyB0aGUgcmV3YXJkIHBvb2wgZm9yIGEgc3BlY2lmaWMgaHVudC4KClRoZSBwb29sIG11c3QgaGF2ZSBiZWVuIGNyZWF0ZWQgdmlhIGBjcmVhdGVfcmV3YXJkX3Bvb2xgIGZpcnN0LgpPbmx5IHRoZSBvcmlnaW5hbCBwb29sIGNyZWF0b3IgaXMgYXV0aG9yaXplZCB0byBmdW5kIGl0LgpUcmFuc2ZlcnMgWExNIGZyb20gdGhlIGZ1bmRlciB0byB0aGlzIGNvbnRyYWN0IGFuZCByZWNvcmRzIHRoZSBiYWxhbmNlLgoKIyBBcmd1bWVudHMKKiBgZnVuZGVyYCAtIFRoZSBhZGRyZXNzIGZ1bmRpbmcgdGhlIHBvb2wgKG11c3QgYmUgdGhlIHBvb2wgY3JlYXRvcikKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0byBmdW5kCiogYGFtb3VudGAgLSBYTE0gYW1vdW50IHRvIGFkZCB0byB0aGUgcG9vbCAobXVzdCBiZSA+IDApCgojIEVycm9ycwoqIGBQb29sTm90Rm91bmRgIC0gUG9vbCBoYXMgbm90IGJlZW4gY3JlYXRlZCB5ZXQKKiBgVW5hdXRob3JpemVkYCAtIEZ1bmRlciBpcyBub3QgdGhlIHBvb2wgY3JlYXRvcgoqIGBJbnZhbGlkQW1vdW50YCAtIEFtb3VudCBpcyA8PSAwCiogYE5vdEluaXRpYWxpemVkYCAtIFhMTSB0b2tlbiBhZGRyZXNzIG5vdCBzZXQAAAAAAAAQZnVuZF9yZXdhcmRfcG9vbAAAAAMAAAAAAAAABmZ1bmRlcgAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAADNSZXR1cm5zIHRoZSBjdXJyZW50IHJld2FyZCBwb29sIGJhbGFuY2UgZm9yIGEgaHVudC4AAAAAEGdldF9wb29sX2JhbGFuY2UAAAABAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAACw==",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9zY2hlbWEAAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAjlDcmVhdGVzIGEgcmV3YXJkIHBvb2wgZm9yIGEgc3BlY2lmaWMgaHVudC4KCk11c3QgYmUgY2FsbGVkIGJlZm9yZSBgZnVuZF9yZXdhcmRfcG9vbGAuIE9ubHkgdGhlIGNyZWF0b3IgaXMgYXV0aG9yaXplZAp0byBmdW5kIHRoZSBwb29sIGFmdGVyIGNyZWF0aW9uLgoKIyBBcmd1bWVudHMKKiBgY3JlYXRvcmAgLSBUaGUgaHVudCBjcmVhdG9yIHdobyB3aWxsIG93biBhbmQgZnVuZCB0aGUgcG9vbAoqIGBodW50X2lkYCAtIFRoZSBodW50IHRoaXMgcG9vbCBpcyBmb3IKKiBgbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnRgIC0gTWluaW11bSBYTE0gcGVyIGRpc3RyaWJ1dGlvbiAoMCA9IG5vIG1pbmltdW0pCgojIEVycm9ycwoqIGBQb29sQWxyZWFkeUV4aXN0c2AgLSBBIHBvb2wgYWxyZWFkeSBleGlzdHMgZm9yIHRoaXMgaHVudF9pZAoqIGBJbnZhbGlkQW1vdW50YCAtIG1pbl9kaXN0cmlidXRpb25fYW1vdW50IGlzIG5lZ2F0aXZlCiogYEh1bnROb3RGb3VuZGAgLSBodW50X2lkIGRvZXMgbm90IGV4aXN0IGluIEh1bnR5Q29yZSAob25seSB3aGVuIGBzZXRfaHVudHlfY29yZWAgaGFzIGJlZW4gY2FsbGVkKQAAAAAAABJjcmVhdGVfcmV3YXJkX3Bvb2wAAAAAAAMAAAAAAAAAB2NyZWF0b3IAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAABdtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAPUmV3YXJkRXJyb3JDb2RlAA==",
        "AAAAAAAAA15NYWluIGVudHJ5IHBvaW50IGZvciByZXdhcmQgZGlzdHJpYnV0aW9uLiBEZXRlcm1pbmVzIHJld2FyZCB0eXBlIGZyb20gY29uZmlndXJhdGlvbiwKcm91dGVzIHRvIFhMTSBhbmQvb3IgTkZUIGhhbmRsZXJzLCBhbmQgZW5zdXJlcyBhdG9taWMgYWxsLW9yLW5vdGhpbmcgZXhlY3V0aW9uLgoKIyBBcmd1bWVudHMKKiBgaHVudF9pZGAgLSBUaGUgaHVudCBiZWluZyByZXdhcmRlZAoqIGBwbGF5ZXJfYWRkcmVzc2AgLSBUaGUgcGxheWVyIHJlY2VpdmluZyByZXdhcmRzCiogYHJld2FyZF9jb25maWdgIC0gQ29uZmlndXJhdGlvbiBzcGVjaWZ5aW5nIFhMTSBhbW91bnQgYW5kL29yIE5GVCBtZXRhZGF0YQoKIyBSZXR1cm5zCmBPaygoKSlgIG9uIHN1Y2Nlc3MKCiMgRXJyb3JzCiogYEludmFsaWRDb25maWdgIC0gTm8gcmV3YXJkIHR5cGUgY29uZmlndXJlZCBvciBpbnZhbGlkIHZhbHVlcwoqIGBOb3RJbml0aWFsaXplZGAgLSBYTE0gdG9rZW4gbm90IHNldCAod2hlbiBYTE0gcmV3YXJkcyByZXF1ZXN0ZWQpCiogYEFscmVhZHlEaXN0cmlidXRlZGAgLSBSZXdhcmRzIGFscmVhZHkgZGlzdHJpYnV0ZWQgZm9yIHRoaXMgaHVudC9wbGF5ZXIKKiBgSW5zdWZmaWNpZW50UG9vbGAgLSBQb29sIGhhcyBpbnN1ZmZpY2llbnQgWExNIGZvciByZXF1ZXN0ZWQgYW1vdW50CiogYEludmFsaWRBbW91bnRgIC0gWExNIGFtb3VudCA8PSAwICh3aGVuIFhMTSByZXF1ZXN0ZWQpCiogYEJlbG93TWluaW11bUFtb3VudGAgLSBYTE0gYW1vdW50IGlzIGJlbG93IHRoZSBwb29sJ3MgbWluaW11bSBkaXN0cmlidXRpb24gdGhyZXNob2xkCiogYE5mdE1pbnRGYWlsZWRgIC0gTkZUIG1pbnRpbmcgZmFpbGVkICh3aGVuIE5GVCByZXF1ZXN0ZWQpAAAAAAASZGlzdHJpYnV0ZV9yZXdhcmRzAAAAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAADnBsYXllcl9hZGRyZXNzAAAAAAATAAAAAAAAAA1yZXdhcmRfY29uZmlnAAAAAAAH0AAAAAxSZXdhcmRDb25maWcAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAAAAAAAAAAASZ2V0X3NjaGVtYV92ZXJzaW9uAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAScm9sbGJhY2tfbWlncmF0aW9uAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6AAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AA==",
        "AAAAAAAAAldVcGRhdGVzIHRoZSBgbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnRgIGZvciBhbiBleGlzdGluZyByZXdhcmQgcG9vbC4KCk9ubHkgdGhlIHBvb2wgY3JlYXRvciBpcyBhdXRob3JpemVkIHRvIGNhbGwgdGhpcy4gVXNlZnVsIHdoZW4gYSBjcmVhdG9yCmhhcyB1bmRlcmZ1bmRlZCB0aGUgcG9vbCBhbmQgbmVlZHMgdG8gbG93ZXIgdGhlIG1pbmltdW0gc28gZGlzdHJpYnV0aW9ucwpjYW4gcHJvY2VlZC4KCiMgQXJndW1lbnRzCiogYGNyZWF0b3JgIC0gVGhlIHBvb2wgY3JlYXRvciAobXVzdCBtYXRjaCB0aGUgc3RvcmVkIGNyZWF0b3IpCiogYGh1bnRfaWRgIC0gVGhlIGh1bnQgd2hvc2UgcG9vbCBjb25maWcgdG8gdXBkYXRlCiogYG1pbl9kaXN0cmlidXRpb25fYW1vdW50YCAtIE5ldyBtaW5pbXVtIFhMTSBwZXIgZGlzdHJpYnV0aW9uICgwID0gbm8gbWluaW11bSkKCiMgRXJyb3JzCiogYFBvb2xOb3RGb3VuZGAgLSBObyBwb29sIGV4aXN0cyBmb3IgdGhpcyBodW50X2lkCiogYFVuYXV0aG9yaXplZGAgLSBDYWxsZXIgaXMgbm90IHRoZSBwb29sIGNyZWF0b3IKKiBgSW52YWxpZEFtb3VudGAgLSBtaW5fZGlzdHJpYnV0aW9uX2Ftb3VudCBpcyBuZWdhdGl2ZQAAAAASdXBkYXRlX3Bvb2xfY29uZmlnAAAAAAADAAAAAAAAAAdjcmVhdG9yAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAXbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnQAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAAAAAAAAAAAUZ2V0X2hlYWx0aF9kYXNoYm9hcmQAAAAAAAAAAQAAB9AAAAAOQ29udHJhY3RIZWFsdGgAAA==",
        "AAAAAQAAAEFFdmVudCBlbWl0dGVkIHdoZW4gYWRtaW4gd2l0aGRyYXdzIHVuY2xhaW1lZCByZXdhcmRzIGZyb20gYSBwb29sLgAAAAAAAAAAAAASQWRtaW5XaXRoZHJhd0V2ZW50AAAAAAADAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAHaHVudF9pZAAAAAAG",
        "AAAAAAAAAEVSZXR1cm5zIHdoZXRoZXIgYSByZXdhcmQgaGFzIGJlZW4gZGlzdHJpYnV0ZWQgdG8gYSBwbGF5ZXIgZm9yIGEgaHVudC4AAAAAAAAVaXNfcmV3YXJkX2Rpc3RyaWJ1dGVkAAAAAAAAAgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAADdSZXR1cm5zIHRoZSBkaXN0cmlidXRpb24gc3RhdHVzIGZvciBhIGh1bnQvcGxheWVyIHBhaXIuAAAAABdnZXRfZGlzdHJpYnV0aW9uX3N0YXR1cwAAAAACAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABnBsYXllcgAAAAAAEwAAAAEAAAfQAAAAEkRpc3RyaWJ1dGlvblN0YXR1cwAA",
        "AAAAAAAAAHRTZXRzIHRoZSBkZWZhdWx0IE5mdFJld2FyZCBjb250cmFjdCBhZGRyZXNzIHVzZWQgZm9yIE5GVCBkaXN0cmlidXRpb25zCndoZW4gYSBwZXItY2FsbCBORlQgY29udHJhY3QgaXMgbm90IHByb3ZpZGVkLgAAABdzZXRfbmZ0X3Jld2FyZF9jb250cmFjdAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADG5mdF9jb250cmFjdAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA9SZXdhcmRFcnJvckNvZGUA",
        "AAAAAQAAACtFdmVudCBlbWl0dGVkIHdoZW4gYSByZXdhcmQgcG9vbCBpcyBmdW5kZWQuAAAAAAAAAAAVUmV3YXJkUG9vbEZ1bmRlZEV2ZW50AAAAAAAABQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAZmdW5kZXIAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAALbmV3X2JhbGFuY2UAAAAACwAAAAAAAAAPdG90YWxfZGVwb3NpdGVkAAAAAAs=",
        "AAAAAAAAArZBbGxvd3MgdGhlIGFkbWluIHRvIHdpdGhkcmF3IGFueSB1bmNsYWltZWQgKHN1cnBsdXMpIFhMTSByZW1haW5pbmcgaW4gYSByZXdhcmQgcG9vbC4KClRoaXMgaXMgbmVlZGVkIHdoZW4gYSBodW50IGNvbmNsdWRlcyB3aXRoIGZld2VyIHdpbm5lcnMgdGhhbiBhbnRpY2lwYXRlZCwKbGVhdmluZyB1bnNwZW50IFhMTSBsb2NrZWQgaW4gdGhlIHBvb2wuIE9ubHkgdGhlIGNvbnRyYWN0IGFkbWluIG1heSBjYWxsIHRoaXMuCgojIEFyZ3VtZW50cwoqIGBhZG1pbmAgLSBUaGUgY29udHJhY3QgYWRtaW4gYWRkcmVzcyAobXVzdCBtYXRjaCB0aGUgc3RvcmVkIGFkbWluKQoqIGBodW50X2lkYCAtIFRoZSBodW50IHdob3NlIHJlbWFpbmluZyBwb29sIGJhbGFuY2UgdG8gd2l0aGRyYXcKKiBgcmVjaXBpZW50YCAtIFRoZSBhZGRyZXNzIHRoYXQgd2lsbCByZWNlaXZlIHRoZSB3aXRoZHJhd24gWExNCgojIEVycm9ycwoqIGBOb3RJbml0aWFsaXplZGAgLSBDb250cmFjdCBoYXMgbm90IGJlZW4gaW5pdGlhbGl6ZWQgKG5vIGFkbWluIHNldCkKKiBgVW5hdXRob3JpemVkYCAtIENhbGxlciBpcyBub3QgdGhlIGNvbnRyYWN0IGFkbWluCiogYFBvb2xOb3RGb3VuZGAgLSBObyBwb29sIGV4aXN0cyBmb3IgdGhpcyBodW50X2lkCiogYEludmFsaWRBbW91bnRgIC0gUG9vbCBiYWxhbmNlIGlzIHplcm8gKG5vdGhpbmcgdG8gd2l0aGRyYXcpAAAAAAAYYWRtaW5fd2l0aGRyYXdfdW5jbGFpbWVkAAAABAAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAACXJlY2lwaWVudAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAD1Jld2FyZEVycm9yQ29kZQA=",
        "AAAAAQAAADdFdmVudCBlbWl0dGVkIHdoZW4gYSByZXdhcmQgcG9vbCBpcyBjcmVhdGVkIGZvciBhIGh1bnQuAAAAAAAAAAAWUmV3YXJkUG9vbENyZWF0ZWRFdmVudAAAAAAAAwAAAAAAAAAHY3JlYXRvcgAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAAF21pbl9kaXN0cmlidXRpb25fYW1vdW50AAAAAAs=",
        "AAAAAAAAAU1MZWdhY3kgZW50cnkgcG9pbnQgZm9yIFhMTS1vbmx5IGRpc3RyaWJ1dGlvbi4KS2VwdCBmb3IgYmFja3dhcmQgY29tcGF0aWJpbGl0eSB3aXRoIEh1bnR5Q29yZS4gRm9yIE5GVCBvciBmdWxsIGNvbmZpZyBzdXBwb3J0IHVzZSBkaXN0cmlidXRlX3Jld2FyZHMuCgpOb3RlOiBgbmZ0X2VuYWJsZWRgIGlzIGlnbm9yZWQg4oCUIE5GVCBkaXN0cmlidXRpb24gcmVxdWlyZXMgbWV0YWRhdGEgYW5kIGEgY29udHJhY3QgYWRkcmVzcwp0aGF0IGFyZSBub3QgYXZhaWxhYmxlIG9uIHRoaXMgcGF0aC4gVXNlIGBkaXN0cmlidXRlX3Jld2FyZHNgIHdpdGggYFJld2FyZENvbmZpZ2AgaW5zdGVhZC4AAAAAAAAZZGlzdHJpYnV0ZV9yZXdhcmRzX2xlZ2FjeQAAAAAAAAQAAAAAAAAABnBsYXllcgAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAp4bG1fYW1vdW50AAAAAAALAAAAAAAAAAxfbmZ0X2VuYWJsZWQAAAABAAAAAQAAAAE=",
        "AAAAAAAAAEtSZXR1cm5zIHRoZSB0b3RhbCBYTE0gZGlzdHJpYnV0ZWQgYWNyb3NzIGFsbCBodW50cyAocHJvdG9jb2wtbGV2ZWwgbWV0cmljKS4AAAAAGWdldF90b3RhbF94bG1fZGlzdHJpYnV0ZWQAAAAAAAAAAAAAAQAAAAs=",
        "AAAAAQAAADhFdmVudCBlbWl0dGVkIHdoZW4gcmV3YXJkcyBhcmUgc3VjY2Vzc2Z1bGx5IGRpc3RyaWJ1dGVkLgAAAAAAAAAXUmV3YXJkc0Rpc3RyaWJ1dGVkRXZlbnQAAAAABAAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZuZnRfaWQAAAAAA+gAAAAGAAAAAAAAAAZwbGF5ZXIAAAAAABMAAAAAAAAACnhsbV9hbW91bnQAAAAAAAs=",
        "AAAAAQAAADZDb25maWd1cmF0aW9uIGZvciBhIHJld2FyZCBwb29sLCBzZXQgYXQgY3JlYXRpb24gdGltZS4AAAAAAAAAAAAQUmV3YXJkUG9vbENvbmZpZwAAAAIAAABaQWRkcmVzcyBvZiB0aGUgaHVudCBjcmVhdG9yIHdobyBvd25zIHRoaXMgcG9vbC4KT25seSB0aGUgY3JlYXRvciBpcyBhdXRob3JpemVkIHRvIGZ1bmQgaXQuAAAAAAAHY3JlYXRvcgAAAAATAAAAQU1pbmltdW0gWExNIGFtb3VudCBwZXIgZGlzdHJpYnV0aW9uLiAwIG1lYW5zIG5vIG1pbmltdW0gZW5mb3JjZWQuAAAAAAAAF21pbl9kaXN0cmlidXRpb25fYW1vdW50AAAAAAs=",
        "AAAAAQAAADxGdWxsIHN0YXR1cyBvZiBhIHJld2FyZCBwb29sLCByZXR1cm5lZCBieSBnZXRfcmV3YXJkX3Bvb2woKS4AAAAAAAAAEFJld2FyZFBvb2xTdGF0dXMAAAAFAAAALEN1cnJlbnQgYXZhaWxhYmxlIGJhbGFuY2UgZm9yIGRpc3RyaWJ1dGlvbnMuAAAAB2JhbGFuY2UAAAAACwAAACZQb29sIGNyZWF0b3IgLyBvbmx5IGF1dGhvcml6ZWQgZnVuZGVyLgAAAAAAB2NyZWF0b3IAAAAAEwAAAC5NaW5pbXVtIFhMTSBwZXIgZGlzdHJpYnV0aW9uICgwID0gbm8gbWluaW11bSkuAAAAAAAXbWluX2Rpc3RyaWJ1dGlvbl9hbW91bnQAAAAACwAAAEBDdW11bGF0aXZlIHRvdGFsIGRlcG9zaXRlZCBpbnRvIHRoaXMgcG9vbCBhY3Jvc3MgYWxsIGZ1bmQgY2FsbHMuAAAAD3RvdGFsX2RlcG9zaXRlZAAAAAALAAAALEN1bXVsYXRpdmUgdG90YWwgZGlzdHJpYnV0ZWQgZnJvbSB0aGlzIHBvb2wuAAAAEXRvdGFsX2Rpc3RyaWJ1dGVkAAAAAAAACw==",
        "AAAAAQAAAD9SZXN1bHQgb2YgYSBwb29sIHZhbGlkYXRpb24gY2hlY2ssIHJldHVybmVkIGJ5IHZhbGlkYXRlX3Bvb2woKS4AAAAAAAAAABBWYWxpZGF0aW9uUmVzdWx0AAAAAwAAACZDdXJyZW50IHBvb2wgYmFsYW5jZSBhdCB0aW1lIG9mIGNoZWNrLgAAAAAAB2JhbGFuY2UAAAAACwAAAIFXaGV0aGVyIHRoZSBwb29sIGhhcyBzdWZmaWNpZW50IGZ1bmRzIGZvciB0aGUgcmVxdWlyZWQgYW1vdW50CmFuZCB0aGUgcmVxdWlyZWQgYW1vdW50IG1lZXRzIHRoZSBwb29sJ3MgbWluaW11bSBkaXN0cmlidXRpb24gc2l6ZS4AAAAAAAAIaXNfdmFsaWQAAAABAAAAKVJlcXVpcmVkIGFtb3VudCB0aGF0IHdhcyBjaGVja2VkIGFnYWluc3QuAAAAAAAACHJlcXVpcmVkAAAACw==",
        "AAAAAQAAAC1JbnRlcm5hbCByZWNvcmQgc3RvcmVkIGZvciBlYWNoIGRpc3RyaWJ1dGlvbi4AAAAAAAAAAAAAEkRpc3RyaWJ1dGlvblJlY29yZAAAAAAAAgAAAAAAAAAGbmZ0X2lkAAAAAAPoAAAABgAAAAAAAAAKeGxtX2Ftb3VudAAAAAAACw==",
        "AAAAAQAAAD9TdGF0dXMgb2YgYSByZXdhcmQgZGlzdHJpYnV0aW9uIGZvciBhIHNwZWNpZmljIGh1bnQgYW5kIHBsYXllci4AAAAAAAAAABJEaXN0cmlidXRpb25TdGF0dXMAAAAAAAMAAAAoV2hldGhlciBhbnkgcmV3YXJkIGhhcyBiZWVuIGRpc3RyaWJ1dGVkLgAAAAtkaXN0cmlidXRlZAAAAAABAAAAHE5GVCBJRCBpZiBhbiBORlQgd2FzIG1pbnRlZC4AAAAGbmZ0X2lkAAAAAAPoAAAABgAAACNYTE0gYW1vdW50IGRpc3RyaWJ1dGVkICgwIGlmIG5vbmUpLgAAAAAKeGxtX2Ftb3VudAAAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=",
        "AAAAAQAAAFdDb25maWd1cmF0aW9uIGZvciBkaXN0cmlidXRpbmcgcmV3YXJkcyBhY3Jvc3MgdGhlIEh1bnR5Q29yZSDihpQgUmV3YXJkTWFuYWdlciBib3VuZGFyeS4AAAAAAAAAAAxSZXdhcmRDb25maWcAAAAIAAAAAAAAAAxuZnRfY29udHJhY3QAAAPoAAAAEwAAAAAAAAAPbmZ0X2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAADm5mdF9odW50X3RpdGxlAAAAAAAQAAAAAAAAAA1uZnRfaW1hZ2VfdXJpAAAAAAAAEAAAAAAAAAAKbmZ0X3Jhcml0eQAAAAAABAAAAAAAAAAIbmZ0X3RpZXIAAAAEAAAAAAAAAAluZnRfdGl0bGUAAAAAAAAQAAAAAAAAAAp4bG1fYW1vdW50AAAAAAPoAAAACw==",
        "AAAABAAAAAAAAAAAAAAAD1Jld2FyZEVycm9yQ29kZQAAAAAOAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAAAQAAAAAAAAAQSW5zdWZmaWNpZW50UG9vbAAAAAIAAAAAAAAAEkFscmVhZHlEaXN0cmlidXRlZAAAAAAAAwAAAAAAAAAOVHJhbnNmZXJGYWlsZWQAAAAAAAQAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAAAFAAAAAAAAAA1JbnZhbGlkQ29uZmlnAAAAAAAABgAAAAAAAAANTmZ0TWludEZhaWxlZAAAAAAAAAcAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAAACAAAAAAAAAAMUG9vbE5vdEZvdW5kAAAACQAAAAAAAAAMVW5hdXRob3JpemVkAAAACgAAAAAAAAASQmVsb3dNaW5pbXVtQW1vdW50AAAAAAALAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAwAAAAAAAAADEh1bnROb3RGb3VuZAAAAA0AAAAAAAAAFVBvb2xCYWxhbmNlRGl2ZXJnZW5jZQAAAAAAAA4=" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<Result<void>>,
        refund_pool: this.txFromJSON<Result<void>>,
        run_migration: this.txFromJSON<MigrationReport>,
        validate_pool: this.txFromJSON<ValidationResult>,
        set_hunty_core: this.txFromJSON<Result<void>>,
        get_reward_pool: this.txFromJSON<Option<RewardPoolStatus>>,
        contract_version: this.txFromJSON<u32>,
        fund_reward_pool: this.txFromJSON<Result<void>>,
        get_pool_balance: this.txFromJSON<i128>,
        initialize_schema: this.txFromJSON<null>,
        create_reward_pool: this.txFromJSON<Result<void>>,
        distribute_rewards: this.txFromJSON<Result<void>>,
        get_schema_version: this.txFromJSON<u32>,
        rollback_migration: this.txFromJSON<Option<MigrationReport>>,
        update_pool_config: this.txFromJSON<Result<void>>,
        get_health_dashboard: this.txFromJSON<ContractHealth>,
        is_reward_distributed: this.txFromJSON<boolean>,
        get_distribution_status: this.txFromJSON<DistributionStatus>,
        set_nft_reward_contract: this.txFromJSON<Result<void>>,
        admin_withdraw_unclaimed: this.txFromJSON<Result<void>>,
        distribute_rewards_legacy: this.txFromJSON<boolean>,
        get_total_xlm_distributed: this.txFromJSON<i128>
  }
}
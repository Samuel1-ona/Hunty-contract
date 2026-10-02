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
 * NFT data structure stored on-chain.
 * NOTE: Do NOT add new fields here without a migration step — the Soroban
 * host rejects stored structs whose field count differs from the stored
 * ScVal map. Use per-NFT auxiliary keys for new metadata instead.
 */
export interface NftData {
  completion_player: string;
  hunt_id: u64;
  locked: boolean;
  metadata: NftMetadata;
  minted_at: u64;
  nft_id: u64;
  owner: string;
  transferable: boolean;
}


/**
 * Core display metadata for an NFT (title, description, image URI).
 * Supports off-chain storage references to keep gas costs low.
 */
export interface NftMetadata {
  /**
 * Original creator of the NFT (stamped at mint time for provenance/attribution).
 * Essential for secondary market royalty distribution and creator attribution.
 */
creator: Option<string>;
  description: string;
  /**
 * Arbitrary key-value metadata extensions beyond the core fields.
 * Max 10 extension fields per NFT.
 */
extensions: Map<string, string>;
  /**
 * Hunt title at time of mint (for context/display).
 */
hunt_title: string;
  image_uri: string;
  /**
 * Rarity tier: 0 = default, 1 = common, 2 = uncommon, 3 = rare, 4 = epic, 5 = legendary.
 */
rarity: u32;
  /**
 * Royalty in basis points (1 bp = 0.01%). For example, 250 = 2.5% royalty.
 * Used for secondary market sales to provide ongoing creator revenue.
 */
royalty_bps: Option<u32>;
  /**
 * Custom tier for special categories (0 = none).
 */
tier: u32;
  title: string;
}


/**
 * Collection-level metadata stored at initialization and exposed via a query.
 */
export interface CollectionMetadata {
  creator: Option<string>;
  description: string;
  name: string;
  total_supply: u64;
}


/**
 * Complete metadata returned by get_nft_metadata (includes NftData-derived fields).
 */
export interface NftMetadataResponse {
  completion_player: string;
  completion_timestamp: u64;
  creator: Option<string>;
  current_owner: string;
  description: string;
  /**
 * Arbitrary key-value metadata extensions.
 */
extensions: Map<string, string>;
  hunt_id: u64;
  hunt_title: string;
  image_uri: string;
  nft_id: u64;
  rarity: u32;
  royalty_bps: Option<u32>;
  /**
 * Schema version of the NFT metadata.
 */
schema_version: u32;
  tier: u32;
  title: string;
}

export const NftErrorCode = {
  1: {message:"NftNotFound"},
  2: {message:"Unauthorized"},
  3: {message:"NotOwner"},
  4: {message:"InvalidRecipient"},
  5: {message:"SoulboundNft"},
  6: {message:"InvalidRarity"},
  7: {message:"AlreadyInitialized"},
  8: {message:"MaxSupplyReached"},
  9: {message:"NotInitialized"},
  10: {message:"NotOperator"},
  11: {message:"NftNotTransferable"},
  12: {message:"NftLocked"},
  13: {message:"InvalidMetadata"},
  14: {message:"MetadataFrozen"},
  15: {message:"TooManyExtensions"},
  16: {message:"InvalidExtensionKey"},
  17: {message:"InvalidExtensionValue"},
  18: {message:"ExtensionNotFound"},
  19: {message:"InvalidMaxSupply"},
  20: {message:"InvalidRoyalty"},
  21: {message:"InvalidImageUri"}
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

export interface Client {
  /**
   * Construct and simulate a get_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Retrieves NFT data by ID.
   */
  get_nft: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<NftData>>>

  /**
   * Construct and simulate a burn_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Burns (permanently destroys) an NFT, removing it from storage and the owner's list.
   * 
   * # Authorization
   * The `owner` must authorize this call and be the current owner of the NFT.
   * 
   * # Locked vs. soulbound — deliberate, distinct policies
   * - **Locked** (`nft.locked`) blocks burning outright: this flag exists for
   * states like escrow, staking, or a dispute hold, where the NFT must
   * not be destroyed out from under whatever holds the lock.
   * - **Soulbound / non-transferable** (`!nft.transferable`) does *not*
   * block burning. `transferable` only gates `transfer_nft` — moving an
   * NFT to a different owner. Burning is destruction by its own owner,
   * not a transfer, so a soulbound NFT can still be burned by the owner
   * it's bound to. (If a given deployment wants soulbound NFTs to be
   * permanent even against their own owner, that is a separate policy
   * decision this function deliberately does not make — nothing here
   * currently checks `transferable`.)
   * 
   * # Errors
   * Returns `NftNotFound` if the NFT does not exist.
   * Returns `NotOwner` if the cal
   */
  burn_nft: ({nft_id, owner}: {nft_id: u64, owner: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a owner_of transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the owner of an NFT.
   */
  owner_of: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the configured admin address, if set.
   */
  get_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes the NFT reward contract with an admin, minter, and optional max supply cap.
   */
  initialize: ({admin, minter, max_supply, collection_metadata}: {admin: string, minter: string, max_supply: Option<u64>, collection_metadata: CollectionMetadata}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_operator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns true if `operator` is approved to manage all NFTs of `owner`.
   */
  is_operator: ({owner, operator}: {owner: string, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a has_hunt_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns `true` if `address` owns any NFT minted for `hunt_id`.
   * Performs an O(1) indexed lookup via the stored (owner, hunt_id) count mapping.
   */
  has_hunt_nft: ({address, hunt_id}: {address: string, hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a set_operator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Grants `operator` the ability to manage all NFTs owned by `owner`.
   * 
   * # Authorization
   * `owner` must authorize this call.
   */
  set_operator: ({owner, operator}: {owner: string, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a total_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the number of NFTs that currently exist — i.e. minted so far
   * minus burned. This decreases when an NFT is burned.
   * 
   * This is distinct from the `max_supply` cap (see `get_max_supply`),
   * which limits the *lifetime* mint count and is unaffected by burns:
   * a burned NFT's ID is never reused and never reopens room under the
   * cap for an additional mint.
   */
  total_supply: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a transfer_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Transfers an NFT to a new owner when the NFT is transferable.
   * Non-transferable (soulbound) NFTs remain bound to the minting recipient.
   */
  transfer_nft: ({nft_id, from_address, to_address, caller}: {nft_id: u64, from_address: string, to_address: string, caller: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a list_all_nfts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Lists all NFTs minted by the contract with pagination support.
   * 
   * Returns a vector of NftData structs, paginated by offset and limit.
   * The limit is bounded to MAX_SCAN_LIMIT (200) to prevent excessive gas consumption.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `offset` - The starting index for pagination (0-based)
   * * `limit` - The maximum number of NFTs to return (capped at MAX_SCAN_LIMIT)
   * 
   * # Returns
   * Vec<NftData> - A vector of NFT data structures, bounded by limit or remaining NFTs
   */
  list_all_nfts: ({offset, limit}: {offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<NftData>>>

  /**
   * Construct and simulate a run_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  run_migration: ({admin, target_version, dry_run}: {admin: string, target_version: u32, dry_run: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MigrationReport>>>

  /**
   * Construct and simulate a get_max_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the configured maximum total supply of NFTs.
   * 
   * - `None`  → no cap was set (unlimited minting)
   * - `Some(n)` → at most `n` NFTs may ever be minted, lifetime. This caps
   * the ever-minted count (see `total_supply` for the currently-live
   * count), so burning an NFT does not free up room under the cap.
   */
  get_max_supply: (options?: MethodOptions) => Promise<AssembledTransaction<Option<u64>>>

  /**
   * Construct and simulate a set_max_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates the maximum total supply cap. Admin only.
   * 
   * - Pass `None` to remove the cap (unlimited).
   * - Pass `Some(n)` where `n > 0` and `n >= current total_supply` to set a new cap.
   * Attempting to set a cap of 0 or lower than the already-minted count is
   * rejected with `InvalidMaxSupply` to prevent bricking the contract.
   * 
   * # Errors
   * * `NotInitialized` - Contract has not been initialized yet
   * * `Unauthorized`   - Caller is not the admin
   * * `InvalidMaxSupply` - Attempting to set cap to Some(0) or below already-minted supply
   */
  set_max_supply: ({admin, new_max}: {admin: string, new_max: Option<u64>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_player_nfts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns paginated NFT IDs owned by an address.
   * The limit is bounded to `MAX_SCAN_LIMIT` to prevent excessive gas consumption.
   */
  get_player_nfts: ({owner, offset, limit}: {owner: string, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u64>>>

  /**
   * Construct and simulate a mint_reward_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Mints a unique NFT as a reward for hunt completion.
   * 
   * `minter` must be an authorized minter (and must sign the transaction) when the
   * contract has been initialized. Before initialization the check is skipped so
   * that existing deployments remain functional.
   * 
   * Reward NFTs minted through this entrypoint are **soulbound** (non-transferable)
   * by default, matching `mint_reward_nft_from_map`'s default, so an authorized
   * minter gets the same behaviour from either path. Callers that want a
   * transferable reward or a completion rank should use `mint_reward_nft_from_map`
   * with the "transferable" / "completion_rank" keys set.
   * 
   * # Arguments
   * * `minter` - Address performing the mint (must be whitelisted after init)
   * * `hunt_id` - The hunt this NFT commemorates
   * * `player_address` - The address of the player completing the hunt (initial owner)
   * * `metadata` - NFT metadata (title, description, image URI, hunt_title, rarity, tier)
   * 
   * # Returns
   * The unique NFT ID of the minted NFT
   */
  mint_reward_nft: ({minter, hunt_id, player_address, metadata}: {minter: string, hunt_id: u64, player_address: string, metadata: NftMetadata}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a propose_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_upgrade: ({admin, target_version}: {admin: string, target_version: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<UpgradeProposal>>>

  /**
   * Construct and simulate a remove_operator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Revokes operator approval for `operator` over `owner`'s NFTs.
   * 
   * # Authorization
   * `owner` must authorize this call.
   */
  remove_operator: ({owner, operator}: {owner: string, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_nft_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns complete metadata for an NFT, including hunt info and completion details.
   */
  get_nft_metadata: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<NftMetadataResponse>>>

  /**
   * Construct and simulate a get_nfts_by_hunt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns paginated NFT IDs minted for a hunt.
   * The limit is bounded to `MAX_SCAN_LIMIT` to prevent excessive gas consumption.
   */
  get_nfts_by_hunt: ({hunt_id, offset, limit}: {hunt_id: u64, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u64>>>

  /**
   * Construct and simulate a verify_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Verifies whether `address` is the current owner of `nft_id`.
   * Returns `true` when the NFT exists and the stored owner equals `address`.
   */
  verify_ownership: ({address, nft_id}: {address: string, nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_nft_extension transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Gets the value of a specific extension field for an NFT.
   * 
   * # Arguments
   * * `nft_id` - The NFT to query
   * * `key` - The extension key to look up
   * 
   * # Returns
   * The extension value if found, None otherwise.
   */
  get_nft_extension: ({nft_id, key}: {nft_id: u64, key: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_schema: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_nft_extension transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets an extension field on an NFT. Only the NFT owner can call this.
   * Max 10 extension fields per NFT. If the key already exists, it is updated.
   * If the maximum is reached and the key is new, it returns an error.
   * 
   * # Arguments
   * * `nft_id` - The NFT to extend
   * * `owner` - The current owner (must authorize)
   * * `key` - The extension key (max 64 bytes)
   * * `value` - The extension value (max 512 bytes)
   */
  set_nft_extension: ({nft_id, owner, key, value}: {nft_id: u64, owner: string, key: string, value: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_hunt_nft_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total number of NFTs minted for a hunt.
   */
  get_hunt_nft_count: ({hunt_id}: {hunt_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_nft_extensions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Gets all extension fields for an NFT.
   * 
   * # Arguments
   * * `nft_id` - The NFT to query
   * 
   * # Returns
   * Map of all extension key-value pairs.
   */
  get_nft_extensions: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<Map<string, string>>>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a rollback_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rollback_migration: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MigrationReport>>>

  /**
   * Construct and simulate a set_reward_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the RewardManager contract address. Only the admin can call this.
   */
  set_reward_manager: ({admin, reward_manager}: {admin: string, reward_manager: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_upgrade_history transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_upgrade_history: ({offset, limit}: {offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<UpgradeHistoryEntry>>>

  /**
   * Construct and simulate a update_nft_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates mutable metadata fields (description, image_uri). Owner only.
   * Title, hunt info, and attributes remain immutable for collectibility.
   */
  update_nft_metadata: ({nft_id, updater, new_description, new_image_uri}: {nft_id: u64, updater: string, new_description: string, new_image_uri: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_remaining_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the number of NFTs that can still be minted.
   * 
   * - `None`  → unlimited (no cap configured)
   * - `Some(n)` → exactly `n` more NFTs may be minted before the cap is hit
   * 
   * Once the cap is reached this returns `Some(0)`, and any subsequent mint
   * will panic with `MaxSupplyReached`.
   */
  get_remaining_supply: (options?: MethodOptions) => Promise<AssembledTransaction<Option<u64>>>

  /**
   * Construct and simulate a get_upgrade_proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_upgrade_proposal: (options?: MethodOptions) => Promise<AssembledTransaction<Option<UpgradeProposal>>>

  /**
   * Construct and simulate a get_upgrade_timelock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_upgrade_timelock: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a remove_nft_extension transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes an extension field from an NFT. Only the NFT owner can call this.
   * 
   * # Arguments
   * * `nft_id` - The NFT to modify
   * * `owner` - The current owner (must authorize)
   * * `key` - The extension key to remove
   */
  remove_nft_extension: ({nft_id, owner, key}: {nft_id: u64, owner: string, key: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_upgrade_timelock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_upgrade_timelock: ({admin, delay_seconds}: {admin: string, delay_seconds: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_authorized_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds a contract to the authorized callers list. Only the admin can call this.
   */
  add_authorized_contract: ({admin, contract}: {admin: string, contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a admin_update_image_uris transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Batch-updates image URIs for all NFTs whose `image_uri` starts with `old_prefix`,
   * replacing it with `new_prefix`. Useful for migrating between IPFS gateways or CDNs.
   * 
   * Paginated like every other collection scan in this contract
   * (`list_all_nfts`, `get_player_nfts`, `get_nfts_by_hunt`): a single call
   * only ever touches up to `MAX_SCAN_LIMIT` NFTs starting at `offset`, so
   * it can't exceed the invocation resource budget regardless of
   * collection size. Drive a full migration by repeatedly calling this
   * with `offset` set to the previous call's `next_offset` until
   * `next_offset` stops advancing (or equals the collection size).
   * 
   * The operation is idempotent: re-running a batch over an
   * already-migrated range updates nothing (those URIs already start with
   * `new_prefix`, not `old_prefix`), so a retried or overlapping batch is
   * harmless.
   * 
   * # Authorization
   * Only the configured admin can call this function.
   * 
   * # Arguments
   * * `admin` - The admin address (must match the stored admin)
   * * `old_prefix` - The prefix to match (e.g. "ipfs://oldg
   */
  admin_update_image_uris: ({admin, old_prefix, new_prefix, offset, limit}: {admin: string, old_prefix: string, new_prefix: string, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<readonly [u32, u32]>>>

  /**
   * Construct and simulate a get_collection_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the collection-level metadata configured at initialization.
   */
  get_collection_metadata: (options?: MethodOptions) => Promise<AssembledTransaction<Option<CollectionMetadata>>>

  /**
   * Construct and simulate a search_nfts_by_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Searches NFTs by metadata fields with pagination support.
   * 
   * Allows filtering NFTs by various metadata fields. All filter parameters are optional -
   * only provided filters are applied. Returns matching NFTs with pagination.
   * 
   * # Arguments
   * * `env` - The Soroban environment
   * * `offset` - The starting index for pagination (0-based)
   * * `limit` - The maximum number of NFTs to return (capped at MAX_SCAN_LIMIT)
   * * `title_filter` - Optional filter for NFT title (exact match)
   * * `hunt_title_filter` - Optional filter for hunt title (exact match)
   * * `rarity_filter` - Optional filter for rarity tier (0-5)
   * * `tier_filter` - Optional filter for custom tier
   * * `creator_filter` - Optional filter for creator address
   * * `hunt_id_filter` - Optional filter for hunt ID
   * * `extension_key` - Optional extension key to search for
   * * `extension_value` - Optional extension value to match (requires extension_key)
   * 
   * # Returns
   * Vec<NftData> - A vector of matching NFT data structures, paginated by offset and limit
   */
  search_nfts_by_metadata: ({offset, limit, title_filter, hunt_title_filter, rarity_filter, tier_filter, creator_filter, hunt_id_filter, extension_key, extension_value}: {offset: u32, limit: u32, title_filter: Option<string>, hunt_title_filter: Option<string>, rarity_filter: Option<u32>, tier_filter: Option<u32>, creator_filter: Option<string>, hunt_id_filter: Option<u64>, extension_key: Option<string>, extension_value: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Array<NftData>>>

  /**
   * Construct and simulate a mint_reward_nft_from_map transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Mints a reward NFT from a generic metadata map. This is the entrypoint
   * used by cross-contract callers (e.g. RewardManager) that cannot depend
   * on this crate's `NftMetadata` type directly.
   * 
   * `minter` is the calling contract's address and must be whitelisted when the
   * contract has been initialized.
   * 
   * Expected keys in `metadata` (all optional, with sensible defaults):
   * - "title": String
   * - "description": String
   * - "image_uri": String
   * - "hunt_title": String (defaults to title when omitted/empty)
   * - "rarity": u32
   * - "tier": u32
   * - "creator": Address (defaults to player_address if omitted)
   * - "royalty_bps": u32 (optional, basis points for royalty percentage)
   * - "transferable": bool
   * - "extensions": Map<String, String> (optional, arbitrary key-value metadata)
   * 
   * # Errors
   * Returns `NftErrorCode::InvalidMetadata` when a key is **present** but holds
   * a value of the wrong type. An **absent** key silently takes its documented default.
   */
  mint_reward_nft_from_map: ({minter, hunt_id, player_address, metadata}: {minter: string, hunt_id: u64, player_address: string, metadata: Map<string, any>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a remove_authorized_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes a contract from the authorized callers list. Only the admin can call this.
   */
  remove_authorized_contract: ({admin, contract}: {admin: string, contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
      new ContractSpec([ "AAAAAQAAAPNORlQgZGF0YSBzdHJ1Y3R1cmUgc3RvcmVkIG9uLWNoYWluLgpOT1RFOiBEbyBOT1QgYWRkIG5ldyBmaWVsZHMgaGVyZSB3aXRob3V0IGEgbWlncmF0aW9uIHN0ZXAg4oCUIHRoZSBTb3JvYmFuCmhvc3QgcmVqZWN0cyBzdG9yZWQgc3RydWN0cyB3aG9zZSBmaWVsZCBjb3VudCBkaWZmZXJzIGZyb20gdGhlIHN0b3JlZApTY1ZhbCBtYXAuIFVzZSBwZXItTkZUIGF1eGlsaWFyeSBrZXlzIGZvciBuZXcgbWV0YWRhdGEgaW5zdGVhZC4AAAAAAAAAAAdOZnREYXRhAAAAAAgAAAAAAAAAEWNvbXBsZXRpb25fcGxheWVyAAAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAZsb2NrZWQAAAAAAAEAAAAAAAAACG1ldGFkYXRhAAAH0AAAAAtOZnRNZXRhZGF0YQAAAAAAAAAACW1pbnRlZF9hdAAAAAAAAAYAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAx0cmFuc2ZlcmFibGUAAAAB",
        "AAAAAAAAABlSZXRyaWV2ZXMgTkZUIGRhdGEgYnkgSUQuAAAAAAAAB2dldF9uZnQAAAAAAQAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAQAAA+gAAAfQAAAAB05mdERhdGEA",
        "AAAAAAAABABCdXJucyAocGVybWFuZW50bHkgZGVzdHJveXMpIGFuIE5GVCwgcmVtb3ZpbmcgaXQgZnJvbSBzdG9yYWdlIGFuZCB0aGUgb3duZXIncyBsaXN0LgoKIyBBdXRob3JpemF0aW9uClRoZSBgb3duZXJgIG11c3QgYXV0aG9yaXplIHRoaXMgY2FsbCBhbmQgYmUgdGhlIGN1cnJlbnQgb3duZXIgb2YgdGhlIE5GVC4KCiMgTG9ja2VkIHZzLiBzb3VsYm91bmQg4oCUIGRlbGliZXJhdGUsIGRpc3RpbmN0IHBvbGljaWVzCi0gKipMb2NrZWQqKiAoYG5mdC5sb2NrZWRgKSBibG9ja3MgYnVybmluZyBvdXRyaWdodDogdGhpcyBmbGFnIGV4aXN0cyBmb3IKc3RhdGVzIGxpa2UgZXNjcm93LCBzdGFraW5nLCBvciBhIGRpc3B1dGUgaG9sZCwgd2hlcmUgdGhlIE5GVCBtdXN0Cm5vdCBiZSBkZXN0cm95ZWQgb3V0IGZyb20gdW5kZXIgd2hhdGV2ZXIgaG9sZHMgdGhlIGxvY2suCi0gKipTb3VsYm91bmQgLyBub24tdHJhbnNmZXJhYmxlKiogKGAhbmZ0LnRyYW5zZmVyYWJsZWApIGRvZXMgKm5vdCoKYmxvY2sgYnVybmluZy4gYHRyYW5zZmVyYWJsZWAgb25seSBnYXRlcyBgdHJhbnNmZXJfbmZ0YCDigJQgbW92aW5nIGFuCk5GVCB0byBhIGRpZmZlcmVudCBvd25lci4gQnVybmluZyBpcyBkZXN0cnVjdGlvbiBieSBpdHMgb3duIG93bmVyLApub3QgYSB0cmFuc2Zlciwgc28gYSBzb3VsYm91bmQgTkZUIGNhbiBzdGlsbCBiZSBidXJuZWQgYnkgdGhlIG93bmVyCml0J3MgYm91bmQgdG8uIChJZiBhIGdpdmVuIGRlcGxveW1lbnQgd2FudHMgc291bGJvdW5kIE5GVHMgdG8gYmUKcGVybWFuZW50IGV2ZW4gYWdhaW5zdCB0aGVpciBvd24gb3duZXIsIHRoYXQgaXMgYSBzZXBhcmF0ZSBwb2xpY3kKZGVjaXNpb24gdGhpcyBmdW5jdGlvbiBkZWxpYmVyYXRlbHkgZG9lcyBub3QgbWFrZSDigJQgbm90aGluZyBoZXJlCmN1cnJlbnRseSBjaGVja3MgYHRyYW5zZmVyYWJsZWAuKQoKIyBFcnJvcnMKUmV0dXJucyBgTmZ0Tm90Rm91bmRgIGlmIHRoZSBORlQgZG9lcyBub3QgZXhpc3QuClJldHVybnMgYE5vdE93bmVyYCBpZiB0aGUgY2FsAAAACGJ1cm5fbmZ0AAAAAgAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAAAAAAVvd25lcgAAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAABxSZXR1cm5zIHRoZSBvd25lciBvZiBhbiBORlQuAAAACG93bmVyX29mAAAAAQAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAQAAA+gAAAAT",
        "AAAAAQAAAH5Db3JlIGRpc3BsYXkgbWV0YWRhdGEgZm9yIGFuIE5GVCAodGl0bGUsIGRlc2NyaXB0aW9uLCBpbWFnZSBVUkkpLgpTdXBwb3J0cyBvZmYtY2hhaW4gc3RvcmFnZSByZWZlcmVuY2VzIHRvIGtlZXAgZ2FzIGNvc3RzIGxvdy4AAAAAAAAAAAALTmZ0TWV0YWRhdGEAAAAACQAAAJtPcmlnaW5hbCBjcmVhdG9yIG9mIHRoZSBORlQgKHN0YW1wZWQgYXQgbWludCB0aW1lIGZvciBwcm92ZW5hbmNlL2F0dHJpYnV0aW9uKS4KRXNzZW50aWFsIGZvciBzZWNvbmRhcnkgbWFya2V0IHJveWFsdHkgZGlzdHJpYnV0aW9uIGFuZCBjcmVhdG9yIGF0dHJpYnV0aW9uLgAAAAAHY3JlYXRvcgAAAAPoAAAAEwAAAAAAAAALZGVzY3JpcHRpb24AAAAAEAAAAGBBcmJpdHJhcnkga2V5LXZhbHVlIG1ldGFkYXRhIGV4dGVuc2lvbnMgYmV5b25kIHRoZSBjb3JlIGZpZWxkcy4KTWF4IDEwIGV4dGVuc2lvbiBmaWVsZHMgcGVyIE5GVC4AAAAKZXh0ZW5zaW9ucwAAAAAD7AAAABAAAAAQAAAAMUh1bnQgdGl0bGUgYXQgdGltZSBvZiBtaW50IChmb3IgY29udGV4dC9kaXNwbGF5KS4AAAAAAAAKaHVudF90aXRsZQAAAAAAEAAAAAAAAAAJaW1hZ2VfdXJpAAAAAAAAEAAAAFZSYXJpdHkgdGllcjogMCA9IGRlZmF1bHQsIDEgPSBjb21tb24sIDIgPSB1bmNvbW1vbiwgMyA9IHJhcmUsIDQgPSBlcGljLCA1ID0gbGVnZW5kYXJ5LgAAAAAABnJhcml0eQAAAAAABAAAAIxSb3lhbHR5IGluIGJhc2lzIHBvaW50cyAoMSBicCA9IDAuMDElKS4gRm9yIGV4YW1wbGUsIDI1MCA9IDIuNSUgcm95YWx0eS4KVXNlZCBmb3Igc2Vjb25kYXJ5IG1hcmtldCBzYWxlcyB0byBwcm92aWRlIG9uZ29pbmcgY3JlYXRvciByZXZlbnVlLgAAAAtyb3lhbHR5X2JwcwAAAAPoAAAABAAAAC5DdXN0b20gdGllciBmb3Igc3BlY2lhbCBjYXRlZ29yaWVzICgwID0gbm9uZSkuAAAAAAAEdGllcgAAAAQAAAAAAAAABXRpdGxlAAAAAAAAEA==",
        "AAAAAAAAAC1SZXR1cm5zIHRoZSBjb25maWd1cmVkIGFkbWluIGFkZHJlc3MsIGlmIHNldC4AAAAAAAAJZ2V0X2FkbWluAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAFdJbml0aWFsaXplcyB0aGUgTkZUIHJld2FyZCBjb250cmFjdCB3aXRoIGFuIGFkbWluLCBtaW50ZXIsIGFuZCBvcHRpb25hbCBtYXggc3VwcGx5IGNhcC4AAAAACmluaXRpYWxpemUAAAAAAAQAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAGbWludGVyAAAAAAATAAAAAAAAAAptYXhfc3VwcGx5AAAAAAPoAAAABgAAAAAAAAATY29sbGVjdGlvbl9tZXRhZGF0YQAAAAfQAAAAEkNvbGxlY3Rpb25NZXRhZGF0YQAAAAAAAQAAA+kAAAACAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAAAAAAAEVSZXR1cm5zIHRydWUgaWYgYG9wZXJhdG9yYCBpcyBhcHByb3ZlZCB0byBtYW5hZ2UgYWxsIE5GVHMgb2YgYG93bmVyYC4AAAAAAAALaXNfb3BlcmF0b3IAAAAAAgAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAhvcGVyYXRvcgAAABMAAAABAAAAAQ==",
        "AAAAAAAAAI1SZXR1cm5zIGB0cnVlYCBpZiBgYWRkcmVzc2Agb3ducyBhbnkgTkZUIG1pbnRlZCBmb3IgYGh1bnRfaWRgLgpQZXJmb3JtcyBhbiBPKDEpIGluZGV4ZWQgbG9va3VwIHZpYSB0aGUgc3RvcmVkIChvd25lciwgaHVudF9pZCkgY291bnQgbWFwcGluZy4AAAAAAAAMaGFzX2h1bnRfbmZ0AAAAAgAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAABAAAAAQ==",
        "AAAAAAAAAHVHcmFudHMgYG9wZXJhdG9yYCB0aGUgYWJpbGl0eSB0byBtYW5hZ2UgYWxsIE5GVHMgb3duZWQgYnkgYG93bmVyYC4KCiMgQXV0aG9yaXphdGlvbgpgb3duZXJgIG11c3QgYXV0aG9yaXplIHRoaXMgY2FsbC4AAAAAAAAMc2V0X29wZXJhdG9yAAAAAgAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAhvcGVyYXRvcgAAABMAAAAA",
        "AAAAAAAAAWBSZXR1cm5zIHRoZSBudW1iZXIgb2YgTkZUcyB0aGF0IGN1cnJlbnRseSBleGlzdCDigJQgaS5lLiBtaW50ZWQgc28gZmFyCm1pbnVzIGJ1cm5lZC4gVGhpcyBkZWNyZWFzZXMgd2hlbiBhbiBORlQgaXMgYnVybmVkLgoKVGhpcyBpcyBkaXN0aW5jdCBmcm9tIHRoZSBgbWF4X3N1cHBseWAgY2FwIChzZWUgYGdldF9tYXhfc3VwcGx5YCksCndoaWNoIGxpbWl0cyB0aGUgKmxpZmV0aW1lKiBtaW50IGNvdW50IGFuZCBpcyB1bmFmZmVjdGVkIGJ5IGJ1cm5zOgphIGJ1cm5lZCBORlQncyBJRCBpcyBuZXZlciByZXVzZWQgYW5kIG5ldmVyIHJlb3BlbnMgcm9vbSB1bmRlciB0aGUKY2FwIGZvciBhbiBhZGRpdGlvbmFsIG1pbnQuAAAADHRvdGFsX3N1cHBseQAAAAAAAAABAAAABg==",
        "AAAAAAAAAIZUcmFuc2ZlcnMgYW4gTkZUIHRvIGEgbmV3IG93bmVyIHdoZW4gdGhlIE5GVCBpcyB0cmFuc2ZlcmFibGUuCk5vbi10cmFuc2ZlcmFibGUgKHNvdWxib3VuZCkgTkZUcyByZW1haW4gYm91bmQgdG8gdGhlIG1pbnRpbmcgcmVjaXBpZW50LgAAAAAADHRyYW5zZmVyX25mdAAAAAQAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAMZnJvbV9hZGRyZXNzAAAAEwAAAAAAAAAKdG9fYWRkcmVzcwAAAAAAEwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAAAAAAAehMaXN0cyBhbGwgTkZUcyBtaW50ZWQgYnkgdGhlIGNvbnRyYWN0IHdpdGggcGFnaW5hdGlvbiBzdXBwb3J0LgoKUmV0dXJucyBhIHZlY3RvciBvZiBOZnREYXRhIHN0cnVjdHMsIHBhZ2luYXRlZCBieSBvZmZzZXQgYW5kIGxpbWl0LgpUaGUgbGltaXQgaXMgYm91bmRlZCB0byBNQVhfU0NBTl9MSU1JVCAoMjAwKSB0byBwcmV2ZW50IGV4Y2Vzc2l2ZSBnYXMgY29uc3VtcHRpb24uCgojIEFyZ3VtZW50cwoqIGBlbnZgIC0gVGhlIFNvcm9iYW4gZW52aXJvbm1lbnQKKiBgb2Zmc2V0YCAtIFRoZSBzdGFydGluZyBpbmRleCBmb3IgcGFnaW5hdGlvbiAoMC1iYXNlZCkKKiBgbGltaXRgIC0gVGhlIG1heGltdW0gbnVtYmVyIG9mIE5GVHMgdG8gcmV0dXJuIChjYXBwZWQgYXQgTUFYX1NDQU5fTElNSVQpCgojIFJldHVybnMKVmVjPE5mdERhdGE+IC0gQSB2ZWN0b3Igb2YgTkZUIGRhdGEgc3RydWN0dXJlcywgYm91bmRlZCBieSBsaW1pdCBvciByZW1haW5pbmcgTkZUcwAAAA1saXN0X2FsbF9uZnRzAAAAAAAAAgAAAAAAAAAGb2Zmc2V0AAAAAAAEAAAAAAAAAAVsaW1pdAAAAAAAAAQAAAABAAAD6gAAB9AAAAAHTmZ0RGF0YQA=",
        "AAAAAAAAAAAAAAANcnVuX21pZ3JhdGlvbgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAAAAAAAB2RyeV9ydW4AAAAAAQAAAAEAAAPpAAAH0AAAAA9NaWdyYXRpb25SZXBvcnQAAAAH0AAAABBVcGdyYWRlQXV0aEVycm9y",
        "AAAAAAAAAS9SZXR1cm5zIHRoZSBjb25maWd1cmVkIG1heGltdW0gdG90YWwgc3VwcGx5IG9mIE5GVHMuCgotIGBOb25lYCAg4oaSIG5vIGNhcCB3YXMgc2V0ICh1bmxpbWl0ZWQgbWludGluZykKLSBgU29tZShuKWAg4oaSIGF0IG1vc3QgYG5gIE5GVHMgbWF5IGV2ZXIgYmUgbWludGVkLCBsaWZldGltZS4gVGhpcyBjYXBzCnRoZSBldmVyLW1pbnRlZCBjb3VudCAoc2VlIGB0b3RhbF9zdXBwbHlgIGZvciB0aGUgY3VycmVudGx5LWxpdmUKY291bnQpLCBzbyBidXJuaW5nIGFuIE5GVCBkb2VzIG5vdCBmcmVlIHVwIHJvb20gdW5kZXIgdGhlIGNhcC4AAAAADmdldF9tYXhfc3VwcGx5AAAAAAAAAAAAAQAAA+gAAAAG",
        "AAAAAAAAAgNVcGRhdGVzIHRoZSBtYXhpbXVtIHRvdGFsIHN1cHBseSBjYXAuIEFkbWluIG9ubHkuCgotIFBhc3MgYE5vbmVgIHRvIHJlbW92ZSB0aGUgY2FwICh1bmxpbWl0ZWQpLgotIFBhc3MgYFNvbWUobilgIHdoZXJlIGBuID4gMGAgYW5kIGBuID49IGN1cnJlbnQgdG90YWxfc3VwcGx5YCB0byBzZXQgYSBuZXcgY2FwLgpBdHRlbXB0aW5nIHRvIHNldCBhIGNhcCBvZiAwIG9yIGxvd2VyIHRoYW4gdGhlIGFscmVhZHktbWludGVkIGNvdW50IGlzCnJlamVjdGVkIHdpdGggYEludmFsaWRNYXhTdXBwbHlgIHRvIHByZXZlbnQgYnJpY2tpbmcgdGhlIGNvbnRyYWN0LgoKIyBFcnJvcnMKKiBgTm90SW5pdGlhbGl6ZWRgIC0gQ29udHJhY3QgaGFzIG5vdCBiZWVuIGluaXRpYWxpemVkIHlldAoqIGBVbmF1dGhvcml6ZWRgICAgLSBDYWxsZXIgaXMgbm90IHRoZSBhZG1pbgoqIGBJbnZhbGlkTWF4U3VwcGx5YCAtIEF0dGVtcHRpbmcgdG8gc2V0IGNhcCB0byBTb21lKDApIG9yIGJlbG93IGFscmVhZHktbWludGVkIHN1cHBseQAAAAAOc2V0X21heF9zdXBwbHkAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAHbmV3X21heAAAAAPoAAAABgAAAAEAAAPpAAAAAgAAB9AAAAAMTmZ0RXJyb3JDb2Rl",
        "AAAAAAAAAH1SZXR1cm5zIHBhZ2luYXRlZCBORlQgSURzIG93bmVkIGJ5IGFuIGFkZHJlc3MuClRoZSBsaW1pdCBpcyBib3VuZGVkIHRvIGBNQVhfU0NBTl9MSU1JVGAgdG8gcHJldmVudCBleGNlc3NpdmUgZ2FzIGNvbnN1bXB0aW9uLgAAAAAAAA9nZXRfcGxheWVyX25mdHMAAAAAAwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAABg==",
        "AAAAAAAAA8BNaW50cyBhIHVuaXF1ZSBORlQgYXMgYSByZXdhcmQgZm9yIGh1bnQgY29tcGxldGlvbi4KCmBtaW50ZXJgIG11c3QgYmUgYW4gYXV0aG9yaXplZCBtaW50ZXIgKGFuZCBtdXN0IHNpZ24gdGhlIHRyYW5zYWN0aW9uKSB3aGVuIHRoZQpjb250cmFjdCBoYXMgYmVlbiBpbml0aWFsaXplZC4gQmVmb3JlIGluaXRpYWxpemF0aW9uIHRoZSBjaGVjayBpcyBza2lwcGVkIHNvCnRoYXQgZXhpc3RpbmcgZGVwbG95bWVudHMgcmVtYWluIGZ1bmN0aW9uYWwuCgpSZXdhcmQgTkZUcyBtaW50ZWQgdGhyb3VnaCB0aGlzIGVudHJ5cG9pbnQgYXJlICoqc291bGJvdW5kKiogKG5vbi10cmFuc2ZlcmFibGUpCmJ5IGRlZmF1bHQsIG1hdGNoaW5nIGBtaW50X3Jld2FyZF9uZnRfZnJvbV9tYXBgJ3MgZGVmYXVsdCwgc28gYW4gYXV0aG9yaXplZAptaW50ZXIgZ2V0cyB0aGUgc2FtZSBiZWhhdmlvdXIgZnJvbSBlaXRoZXIgcGF0aC4gQ2FsbGVycyB0aGF0IHdhbnQgYQp0cmFuc2ZlcmFibGUgcmV3YXJkIG9yIGEgY29tcGxldGlvbiByYW5rIHNob3VsZCB1c2UgYG1pbnRfcmV3YXJkX25mdF9mcm9tX21hcGAKd2l0aCB0aGUgInRyYW5zZmVyYWJsZSIgLyAiY29tcGxldGlvbl9yYW5rIiBrZXlzIHNldC4KCiMgQXJndW1lbnRzCiogYG1pbnRlcmAgLSBBZGRyZXNzIHBlcmZvcm1pbmcgdGhlIG1pbnQgKG11c3QgYmUgd2hpdGVsaXN0ZWQgYWZ0ZXIgaW5pdCkKKiBgaHVudF9pZGAgLSBUaGUgaHVudCB0aGlzIE5GVCBjb21tZW1vcmF0ZXMKKiBgcGxheWVyX2FkZHJlc3NgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIHBsYXllciBjb21wbGV0aW5nIHRoZSBodW50IChpbml0aWFsIG93bmVyKQoqIGBtZXRhZGF0YWAgLSBORlQgbWV0YWRhdGEgKHRpdGxlLCBkZXNjcmlwdGlvbiwgaW1hZ2UgVVJJLCBodW50X3RpdGxlLCByYXJpdHksIHRpZXIpCgojIFJldHVybnMKVGhlIHVuaXF1ZSBORlQgSUQgb2YgdGhlIG1pbnRlZCBORlQAAAAPbWludF9yZXdhcmRfbmZ0AAAAAAQAAAAAAAAABm1pbnRlcgAAAAAAEwAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAA5wbGF5ZXJfYWRkcmVzcwAAAAAAEwAAAAAAAAAIbWV0YWRhdGEAAAfQAAAAC05mdE1ldGFkYXRhAAAAAAEAAAAG",
        "AAAAAAAAAAAAAAAPcHJvcG9zZV91cGdyYWRlAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAABAAAD6QAAB9AAAAAPVXBncmFkZVByb3Bvc2FsAAAAB9AAAAAQVXBncmFkZUF1dGhFcnJvcg==",
        "AAAAAAAAAHBSZXZva2VzIG9wZXJhdG9yIGFwcHJvdmFsIGZvciBgb3BlcmF0b3JgIG92ZXIgYG93bmVyYCdzIE5GVHMuCgojIEF1dGhvcml6YXRpb24KYG93bmVyYCBtdXN0IGF1dGhvcml6ZSB0aGlzIGNhbGwuAAAAD3JlbW92ZV9vcGVyYXRvcgAAAAACAAAAAAAAAAVvd25lcgAAAAAAABMAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAA=",
        "AAAAAQAAAEtDb2xsZWN0aW9uLWxldmVsIG1ldGFkYXRhIHN0b3JlZCBhdCBpbml0aWFsaXphdGlvbiBhbmQgZXhwb3NlZCB2aWEgYSBxdWVyeS4AAAAAAAAAABJDb2xsZWN0aW9uTWV0YWRhdGEAAAAAAAQAAAAAAAAAB2NyZWF0b3IAAAAD6AAAABMAAAAAAAAAC2Rlc2NyaXB0aW9uAAAAABAAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAx0b3RhbF9zdXBwbHkAAAAG",
        "AAAAAAAAAFFSZXR1cm5zIGNvbXBsZXRlIG1ldGFkYXRhIGZvciBhbiBORlQsIGluY2x1ZGluZyBodW50IGluZm8gYW5kIGNvbXBsZXRpb24gZGV0YWlscy4AAAAAAAAQZ2V0X25mdF9tZXRhZGF0YQAAAAEAAAAAAAAABm5mdF9pZAAAAAAABgAAAAEAAAPoAAAH0AAAABNOZnRNZXRhZGF0YVJlc3BvbnNlAA==",
        "AAAAAAAAAHtSZXR1cm5zIHBhZ2luYXRlZCBORlQgSURzIG1pbnRlZCBmb3IgYSBodW50LgpUaGUgbGltaXQgaXMgYm91bmRlZCB0byBgTUFYX1NDQU5fTElNSVRgIHRvIHByZXZlbnQgZXhjZXNzaXZlIGdhcyBjb25zdW1wdGlvbi4AAAAAEGdldF9uZnRzX2J5X2h1bnQAAAADAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAABm9mZnNldAAAAAAABAAAAAAAAAAFbGltaXQAAAAAAAAEAAAAAQAAA+oAAAAG",
        "AAAAAAAAAIZWZXJpZmllcyB3aGV0aGVyIGBhZGRyZXNzYCBpcyB0aGUgY3VycmVudCBvd25lciBvZiBgbmZ0X2lkYC4KUmV0dXJucyBgdHJ1ZWAgd2hlbiB0aGUgTkZUIGV4aXN0cyBhbmQgdGhlIHN0b3JlZCBvd25lciBlcXVhbHMgYGFkZHJlc3NgLgAAAAAAEHZlcmlmeV9vd25lcnNoaXAAAAACAAAAAAAAAAdhZGRyZXNzAAAAABMAAAAAAAAABm5mdF9pZAAAAAAABgAAAAEAAAAB",
        "AAAAAQAAAFFDb21wbGV0ZSBtZXRhZGF0YSByZXR1cm5lZCBieSBnZXRfbmZ0X21ldGFkYXRhIChpbmNsdWRlcyBOZnREYXRhLWRlcml2ZWQgZmllbGRzKS4AAAAAAAAAAAAAE05mdE1ldGFkYXRhUmVzcG9uc2UAAAAADwAAAAAAAAARY29tcGxldGlvbl9wbGF5ZXIAAAAAAAATAAAAAAAAABRjb21wbGV0aW9uX3RpbWVzdGFtcAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAD6AAAABMAAAAAAAAADWN1cnJlbnRfb3duZXIAAAAAAAATAAAAAAAAAAtkZXNjcmlwdGlvbgAAAAAQAAAAKEFyYml0cmFyeSBrZXktdmFsdWUgbWV0YWRhdGEgZXh0ZW5zaW9ucy4AAAAKZXh0ZW5zaW9ucwAAAAAD7AAAABAAAAAQAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAACmh1bnRfdGl0bGUAAAAAABAAAAAAAAAACWltYWdlX3VyaQAAAAAAABAAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAGcmFyaXR5AAAAAAAEAAAAAAAAAAtyb3lhbHR5X2JwcwAAAAPoAAAABAAAACNTY2hlbWEgdmVyc2lvbiBvZiB0aGUgTkZUIG1ldGFkYXRhLgAAAAAOc2NoZW1hX3ZlcnNpb24AAAAAAAQAAAAAAAAABHRpZXIAAAAEAAAAAAAAAAV0aXRsZQAAAAAAABA=",
        "AAAAAAAAAMNHZXRzIHRoZSB2YWx1ZSBvZiBhIHNwZWNpZmljIGV4dGVuc2lvbiBmaWVsZCBmb3IgYW4gTkZULgoKIyBBcmd1bWVudHMKKiBgbmZ0X2lkYCAtIFRoZSBORlQgdG8gcXVlcnkKKiBga2V5YCAtIFRoZSBleHRlbnNpb24ga2V5IHRvIGxvb2sgdXAKCiMgUmV0dXJucwpUaGUgZXh0ZW5zaW9uIHZhbHVlIGlmIGZvdW5kLCBOb25lIG90aGVyd2lzZS4AAAAAEWdldF9uZnRfZXh0ZW5zaW9uAAAAAAAAAgAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAAAAAANrZXkAAAAAEAAAAAEAAAPoAAAAEA==",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9zY2hlbWEAAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAYhTZXRzIGFuIGV4dGVuc2lvbiBmaWVsZCBvbiBhbiBORlQuIE9ubHkgdGhlIE5GVCBvd25lciBjYW4gY2FsbCB0aGlzLgpNYXggMTAgZXh0ZW5zaW9uIGZpZWxkcyBwZXIgTkZULiBJZiB0aGUga2V5IGFscmVhZHkgZXhpc3RzLCBpdCBpcyB1cGRhdGVkLgpJZiB0aGUgbWF4aW11bSBpcyByZWFjaGVkIGFuZCB0aGUga2V5IGlzIG5ldywgaXQgcmV0dXJucyBhbiBlcnJvci4KCiMgQXJndW1lbnRzCiogYG5mdF9pZGAgLSBUaGUgTkZUIHRvIGV4dGVuZAoqIGBvd25lcmAgLSBUaGUgY3VycmVudCBvd25lciAobXVzdCBhdXRob3JpemUpCiogYGtleWAgLSBUaGUgZXh0ZW5zaW9uIGtleSAobWF4IDY0IGJ5dGVzKQoqIGB2YWx1ZWAgLSBUaGUgZXh0ZW5zaW9uIHZhbHVlIChtYXggNTEyIGJ5dGVzKQAAABFzZXRfbmZ0X2V4dGVuc2lvbgAAAAAAAAQAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAANrZXkAAAAAEAAAAAAAAAAFdmFsdWUAAAAAAAAQAAAAAQAAA+kAAAACAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAAAAAAADNSZXR1cm5zIHRoZSB0b3RhbCBudW1iZXIgb2YgTkZUcyBtaW50ZWQgZm9yIGEgaHVudC4AAAAAEmdldF9odW50X25mdF9jb3VudAAAAAAAAQAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAQAAAAQ=",
        "AAAAAAAAAIFHZXRzIGFsbCBleHRlbnNpb24gZmllbGRzIGZvciBhbiBORlQuCgojIEFyZ3VtZW50cwoqIGBuZnRfaWRgIC0gVGhlIE5GVCB0byBxdWVyeQoKIyBSZXR1cm5zCk1hcCBvZiBhbGwgZXh0ZW5zaW9uIGtleS12YWx1ZSBwYWlycy4AAAAAAAASZ2V0X25mdF9leHRlbnNpb25zAAAAAAABAAAAAAAAAAZuZnRfaWQAAAAAAAYAAAABAAAD6AAAA+wAAAAQAAAAEA==",
        "AAAAAAAAAAAAAAASZ2V0X3NjaGVtYV92ZXJzaW9uAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAScm9sbGJhY2tfbWlncmF0aW9uAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AAAAB9AAAAAQVXBncmFkZUF1dGhFcnJvcg==",
        "AAAAAAAAAEZTZXRzIHRoZSBSZXdhcmRNYW5hZ2VyIGNvbnRyYWN0IGFkZHJlc3MuIE9ubHkgdGhlIGFkbWluIGNhbiBjYWxsIHRoaXMuAAAAAAASc2V0X3Jld2FyZF9tYW5hZ2VyAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADnJld2FyZF9tYW5hZ2VyAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAAAAAAAAAAAAATZ2V0X3VwZ3JhZGVfaGlzdG9yeQAAAAACAAAAAAAAAAZvZmZzZXQAAAAAAAQAAAAAAAAABWxpbWl0AAAAAAAABAAAAAEAAAPqAAAH0AAAABNVcGdyYWRlSGlzdG9yeUVudHJ5AA==",
        "AAAAAAAAAItVcGRhdGVzIG11dGFibGUgbWV0YWRhdGEgZmllbGRzIChkZXNjcmlwdGlvbiwgaW1hZ2VfdXJpKS4gT3duZXIgb25seS4KVGl0bGUsIGh1bnQgaW5mbywgYW5kIGF0dHJpYnV0ZXMgcmVtYWluIGltbXV0YWJsZSBmb3IgY29sbGVjdGliaWxpdHkuAAAAABN1cGRhdGVfbmZ0X21ldGFkYXRhAAAAAAQAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAHdXBkYXRlcgAAAAATAAAAAAAAAA9uZXdfZGVzY3JpcHRpb24AAAAAEAAAAAAAAAANbmV3X2ltYWdlX3VyaQAAAAAAABAAAAABAAAD6QAAAAIAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAARhSZXR1cm5zIHRoZSBudW1iZXIgb2YgTkZUcyB0aGF0IGNhbiBzdGlsbCBiZSBtaW50ZWQuCgotIGBOb25lYCAg4oaSIHVubGltaXRlZCAobm8gY2FwIGNvbmZpZ3VyZWQpCi0gYFNvbWUobilgIOKGkiBleGFjdGx5IGBuYCBtb3JlIE5GVHMgbWF5IGJlIG1pbnRlZCBiZWZvcmUgdGhlIGNhcCBpcyBoaXQKCk9uY2UgdGhlIGNhcCBpcyByZWFjaGVkIHRoaXMgcmV0dXJucyBgU29tZSgwKWAsIGFuZCBhbnkgc3Vic2VxdWVudCBtaW50CndpbGwgcGFuaWMgd2l0aCBgTWF4U3VwcGx5UmVhY2hlZGAuAAAAFGdldF9yZW1haW5pbmdfc3VwcGx5AAAAAAAAAAEAAAPoAAAABg==",
        "AAAAAAAAAAAAAAAUZ2V0X3VwZ3JhZGVfcHJvcG9zYWwAAAAAAAAAAQAAA+gAAAfQAAAAD1VwZ3JhZGVQcm9wb3NhbAA=",
        "AAAAAAAAAAAAAAAUZ2V0X3VwZ3JhZGVfdGltZWxvY2sAAAAAAAAAAQAAAAY=",
        "AAAAAAAAAMpSZW1vdmVzIGFuIGV4dGVuc2lvbiBmaWVsZCBmcm9tIGFuIE5GVC4gT25seSB0aGUgTkZUIG93bmVyIGNhbiBjYWxsIHRoaXMuCgojIEFyZ3VtZW50cwoqIGBuZnRfaWRgIC0gVGhlIE5GVCB0byBtb2RpZnkKKiBgb3duZXJgIC0gVGhlIGN1cnJlbnQgb3duZXIgKG11c3QgYXV0aG9yaXplKQoqIGBrZXlgIC0gVGhlIGV4dGVuc2lvbiBrZXkgdG8gcmVtb3ZlAAAAAAAUcmVtb3ZlX25mdF9leHRlbnNpb24AAAADAAAAAAAAAAZuZnRfaWQAAAAAAAYAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAADa2V5AAAAABAAAAABAAAD6QAAAAIAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAAAAAAAAUc2V0X3VwZ3JhZGVfdGltZWxvY2sAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADWRlbGF5X3NlY29uZHMAAAAAAAAGAAAAAQAAA+kAAAACAAAH0AAAABBVcGdyYWRlQXV0aEVycm9y",
        "AAAAAAAAAE1BZGRzIGEgY29udHJhY3QgdG8gdGhlIGF1dGhvcml6ZWQgY2FsbGVycyBsaXN0LiBPbmx5IHRoZSBhZG1pbiBjYW4gY2FsbCB0aGlzLgAAAAAAABdhZGRfYXV0aG9yaXplZF9jb250cmFjdAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAACGNvbnRyYWN0AAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAMTmZ0RXJyb3JDb2Rl",
        "AAAAAAAABABCYXRjaC11cGRhdGVzIGltYWdlIFVSSXMgZm9yIGFsbCBORlRzIHdob3NlIGBpbWFnZV91cmlgIHN0YXJ0cyB3aXRoIGBvbGRfcHJlZml4YCwKcmVwbGFjaW5nIGl0IHdpdGggYG5ld19wcmVmaXhgLiBVc2VmdWwgZm9yIG1pZ3JhdGluZyBiZXR3ZWVuIElQRlMgZ2F0ZXdheXMgb3IgQ0ROcy4KClBhZ2luYXRlZCBsaWtlIGV2ZXJ5IG90aGVyIGNvbGxlY3Rpb24gc2NhbiBpbiB0aGlzIGNvbnRyYWN0CihgbGlzdF9hbGxfbmZ0c2AsIGBnZXRfcGxheWVyX25mdHNgLCBgZ2V0X25mdHNfYnlfaHVudGApOiBhIHNpbmdsZSBjYWxsCm9ubHkgZXZlciB0b3VjaGVzIHVwIHRvIGBNQVhfU0NBTl9MSU1JVGAgTkZUcyBzdGFydGluZyBhdCBgb2Zmc2V0YCwgc28KaXQgY2FuJ3QgZXhjZWVkIHRoZSBpbnZvY2F0aW9uIHJlc291cmNlIGJ1ZGdldCByZWdhcmRsZXNzIG9mCmNvbGxlY3Rpb24gc2l6ZS4gRHJpdmUgYSBmdWxsIG1pZ3JhdGlvbiBieSByZXBlYXRlZGx5IGNhbGxpbmcgdGhpcwp3aXRoIGBvZmZzZXRgIHNldCB0byB0aGUgcHJldmlvdXMgY2FsbCdzIGBuZXh0X29mZnNldGAgdW50aWwKYG5leHRfb2Zmc2V0YCBzdG9wcyBhZHZhbmNpbmcgKG9yIGVxdWFscyB0aGUgY29sbGVjdGlvbiBzaXplKS4KClRoZSBvcGVyYXRpb24gaXMgaWRlbXBvdGVudDogcmUtcnVubmluZyBhIGJhdGNoIG92ZXIgYW4KYWxyZWFkeS1taWdyYXRlZCByYW5nZSB1cGRhdGVzIG5vdGhpbmcgKHRob3NlIFVSSXMgYWxyZWFkeSBzdGFydCB3aXRoCmBuZXdfcHJlZml4YCwgbm90IGBvbGRfcHJlZml4YCksIHNvIGEgcmV0cmllZCBvciBvdmVybGFwcGluZyBiYXRjaCBpcwpoYXJtbGVzcy4KCiMgQXV0aG9yaXphdGlvbgpPbmx5IHRoZSBjb25maWd1cmVkIGFkbWluIGNhbiBjYWxsIHRoaXMgZnVuY3Rpb24uCgojIEFyZ3VtZW50cwoqIGBhZG1pbmAgLSBUaGUgYWRtaW4gYWRkcmVzcyAobXVzdCBtYXRjaCB0aGUgc3RvcmVkIGFkbWluKQoqIGBvbGRfcHJlZml4YCAtIFRoZSBwcmVmaXggdG8gbWF0Y2ggKGUuZy4gImlwZnM6Ly9vbGRnAAAAF2FkbWluX3VwZGF0ZV9pbWFnZV91cmlzAAAAAAUAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAKb2xkX3ByZWZpeAAAAAAAEAAAAAAAAAAKbmV3X3ByZWZpeAAAAAAAEAAAAAAAAAAGb2Zmc2V0AAAAAAAEAAAAAAAAAAVsaW1pdAAAAAAAAAQAAAABAAAD6QAAA+0AAAACAAAABAAAAAQAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAAENSZXR1cm5zIHRoZSBjb2xsZWN0aW9uLWxldmVsIG1ldGFkYXRhIGNvbmZpZ3VyZWQgYXQgaW5pdGlhbGl6YXRpb24uAAAAABdnZXRfY29sbGVjdGlvbl9tZXRhZGF0YQAAAAAAAAAAAQAAA+gAAAfQAAAAEkNvbGxlY3Rpb25NZXRhZGF0YQAA",
        "AAAAAAAAA9VTZWFyY2hlcyBORlRzIGJ5IG1ldGFkYXRhIGZpZWxkcyB3aXRoIHBhZ2luYXRpb24gc3VwcG9ydC4KCkFsbG93cyBmaWx0ZXJpbmcgTkZUcyBieSB2YXJpb3VzIG1ldGFkYXRhIGZpZWxkcy4gQWxsIGZpbHRlciBwYXJhbWV0ZXJzIGFyZSBvcHRpb25hbCAtCm9ubHkgcHJvdmlkZWQgZmlsdGVycyBhcmUgYXBwbGllZC4gUmV0dXJucyBtYXRjaGluZyBORlRzIHdpdGggcGFnaW5hdGlvbi4KCiMgQXJndW1lbnRzCiogYGVudmAgLSBUaGUgU29yb2JhbiBlbnZpcm9ubWVudAoqIGBvZmZzZXRgIC0gVGhlIHN0YXJ0aW5nIGluZGV4IGZvciBwYWdpbmF0aW9uICgwLWJhc2VkKQoqIGBsaW1pdGAgLSBUaGUgbWF4aW11bSBudW1iZXIgb2YgTkZUcyB0byByZXR1cm4gKGNhcHBlZCBhdCBNQVhfU0NBTl9MSU1JVCkKKiBgdGl0bGVfZmlsdGVyYCAtIE9wdGlvbmFsIGZpbHRlciBmb3IgTkZUIHRpdGxlIChleGFjdCBtYXRjaCkKKiBgaHVudF90aXRsZV9maWx0ZXJgIC0gT3B0aW9uYWwgZmlsdGVyIGZvciBodW50IHRpdGxlIChleGFjdCBtYXRjaCkKKiBgcmFyaXR5X2ZpbHRlcmAgLSBPcHRpb25hbCBmaWx0ZXIgZm9yIHJhcml0eSB0aWVyICgwLTUpCiogYHRpZXJfZmlsdGVyYCAtIE9wdGlvbmFsIGZpbHRlciBmb3IgY3VzdG9tIHRpZXIKKiBgY3JlYXRvcl9maWx0ZXJgIC0gT3B0aW9uYWwgZmlsdGVyIGZvciBjcmVhdG9yIGFkZHJlc3MKKiBgaHVudF9pZF9maWx0ZXJgIC0gT3B0aW9uYWwgZmlsdGVyIGZvciBodW50IElECiogYGV4dGVuc2lvbl9rZXlgIC0gT3B0aW9uYWwgZXh0ZW5zaW9uIGtleSB0byBzZWFyY2ggZm9yCiogYGV4dGVuc2lvbl92YWx1ZWAgLSBPcHRpb25hbCBleHRlbnNpb24gdmFsdWUgdG8gbWF0Y2ggKHJlcXVpcmVzIGV4dGVuc2lvbl9rZXkpCgojIFJldHVybnMKVmVjPE5mdERhdGE+IC0gQSB2ZWN0b3Igb2YgbWF0Y2hpbmcgTkZUIGRhdGEgc3RydWN0dXJlcywgcGFnaW5hdGVkIGJ5IG9mZnNldCBhbmQgbGltaXQAAAAAAAAXc2VhcmNoX25mdHNfYnlfbWV0YWRhdGEAAAAACgAAAAAAAAAGb2Zmc2V0AAAAAAAEAAAAAAAAAAVsaW1pdAAAAAAAAAQAAAAAAAAADHRpdGxlX2ZpbHRlcgAAA+gAAAAQAAAAAAAAABFodW50X3RpdGxlX2ZpbHRlcgAAAAAAA+gAAAAQAAAAAAAAAA1yYXJpdHlfZmlsdGVyAAAAAAAD6AAAAAQAAAAAAAAAC3RpZXJfZmlsdGVyAAAAA+gAAAAEAAAAAAAAAA5jcmVhdG9yX2ZpbHRlcgAAAAAD6AAAABMAAAAAAAAADmh1bnRfaWRfZmlsdGVyAAAAAAPoAAAABgAAAAAAAAANZXh0ZW5zaW9uX2tleQAAAAAAA+gAAAAQAAAAAAAAAA9leHRlbnNpb25fdmFsdWUAAAAD6AAAABAAAAABAAAD6gAAB9AAAAAHTmZ0RGF0YQA=",
        "AAAAAAAAA5dNaW50cyBhIHJld2FyZCBORlQgZnJvbSBhIGdlbmVyaWMgbWV0YWRhdGEgbWFwLiBUaGlzIGlzIHRoZSBlbnRyeXBvaW50CnVzZWQgYnkgY3Jvc3MtY29udHJhY3QgY2FsbGVycyAoZS5nLiBSZXdhcmRNYW5hZ2VyKSB0aGF0IGNhbm5vdCBkZXBlbmQKb24gdGhpcyBjcmF0ZSdzIGBOZnRNZXRhZGF0YWAgdHlwZSBkaXJlY3RseS4KCmBtaW50ZXJgIGlzIHRoZSBjYWxsaW5nIGNvbnRyYWN0J3MgYWRkcmVzcyBhbmQgbXVzdCBiZSB3aGl0ZWxpc3RlZCB3aGVuIHRoZQpjb250cmFjdCBoYXMgYmVlbiBpbml0aWFsaXplZC4KCkV4cGVjdGVkIGtleXMgaW4gYG1ldGFkYXRhYCAoYWxsIG9wdGlvbmFsLCB3aXRoIHNlbnNpYmxlIGRlZmF1bHRzKToKLSAidGl0bGUiOiBTdHJpbmcKLSAiZGVzY3JpcHRpb24iOiBTdHJpbmcKLSAiaW1hZ2VfdXJpIjogU3RyaW5nCi0gImh1bnRfdGl0bGUiOiBTdHJpbmcgKGRlZmF1bHRzIHRvIHRpdGxlIHdoZW4gb21pdHRlZC9lbXB0eSkKLSAicmFyaXR5IjogdTMyCi0gInRpZXIiOiB1MzIKLSAiY3JlYXRvciI6IEFkZHJlc3MgKGRlZmF1bHRzIHRvIHBsYXllcl9hZGRyZXNzIGlmIG9taXR0ZWQpCi0gInJveWFsdHlfYnBzIjogdTMyIChvcHRpb25hbCwgYmFzaXMgcG9pbnRzIGZvciByb3lhbHR5IHBlcmNlbnRhZ2UpCi0gInRyYW5zZmVyYWJsZSI6IGJvb2wKLSAiZXh0ZW5zaW9ucyI6IE1hcDxTdHJpbmcsIFN0cmluZz4gKG9wdGlvbmFsLCBhcmJpdHJhcnkga2V5LXZhbHVlIG1ldGFkYXRhKQoKIyBFcnJvcnMKUmV0dXJucyBgTmZ0RXJyb3JDb2RlOjpJbnZhbGlkTWV0YWRhdGFgIHdoZW4gYSBrZXkgaXMgKipwcmVzZW50KiogYnV0IGhvbGRzCmEgdmFsdWUgb2YgdGhlIHdyb25nIHR5cGUuIEFuICoqYWJzZW50Kioga2V5IHNpbGVudGx5IHRha2VzIGl0cyBkb2N1bWVudGVkIGRlZmF1bHQuAAAAABhtaW50X3Jld2FyZF9uZnRfZnJvbV9tYXAAAAAEAAAAAAAAAAZtaW50ZXIAAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAOcGxheWVyX2FkZHJlc3MAAAAAABMAAAAAAAAACG1ldGFkYXRhAAAD7AAAABEAAAAAAAAAAQAAA+kAAAAGAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAAAAAAAFJSZW1vdmVzIGEgY29udHJhY3QgZnJvbSB0aGUgYXV0aG9yaXplZCBjYWxsZXJzIGxpc3QuIE9ubHkgdGhlIGFkbWluIGNhbiBjYWxsIHRoaXMuAAAAAAAacmVtb3ZlX2F1dGhvcml6ZWRfY29udHJhY3QAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIY29udHJhY3QAAAATAAAAAQAAA+kAAAACAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAABAAAAAAAAAAAAAAADE5mdEVycm9yQ29kZQAAABUAAAAAAAAAC05mdE5vdEZvdW5kAAAAAAEAAAAAAAAADFVuYXV0aG9yaXplZAAAAAIAAAAAAAAACE5vdE93bmVyAAAAAwAAAAAAAAAQSW52YWxpZFJlY2lwaWVudAAAAAQAAAAAAAAADFNvdWxib3VuZE5mdAAAAAUAAAAAAAAADUludmFsaWRSYXJpdHkAAAAAAAAGAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAcAAAAAAAAAEE1heFN1cHBseVJlYWNoZWQAAAAIAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAACQAAAAAAAAALTm90T3BlcmF0b3IAAAAACgAAAAAAAAASTmZ0Tm90VHJhbnNmZXJhYmxlAAAAAAALAAAAAAAAAAlOZnRMb2NrZWQAAAAAAAAMAAAAAAAAAA9JbnZhbGlkTWV0YWRhdGEAAAAADQAAAAAAAAAOTWV0YWRhdGFGcm96ZW4AAAAAAA4AAAAAAAAAEVRvb01hbnlFeHRlbnNpb25zAAAAAAAADwAAAAAAAAATSW52YWxpZEV4dGVuc2lvbktleQAAAAAQAAAAAAAAABVJbnZhbGlkRXh0ZW5zaW9uVmFsdWUAAAAAAAARAAAAAAAAABFFeHRlbnNpb25Ob3RGb3VuZAAAAAAAABIAAAAAAAAAEEludmFsaWRNYXhTdXBwbHkAAAATAAAAAAAAAA5JbnZhbGlkUm95YWx0eQAAAAAAFAAAAAAAAAAPSW52YWxpZEltYWdlVXJpAAAAABU=",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAD1VwZ3JhZGVQcm9wb3NhbAAAAAAEAAAAAAAAAAxlZmZlY3RpdmVfYXQAAAAGAAAAAAAAAAtwcm9wb3NlZF9hdAAAAAAGAAAAAAAAAAhwcm9wb3NlcgAAABMAAAAAAAAADnRhcmdldF92ZXJzaW9uAAAAAAAE",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVBdXRoRXJyb3IAAAAFAAAAAAAAAAxVbmF1dGhvcml6ZWQAAAABAAAAAAAAAApOb1Byb3Bvc2FsAAAAAAACAAAAAAAAAA9UaW1lbG9ja1BlbmRpbmcAAAAAAwAAAAAAAAAPVmVyc2lvbk1pc21hdGNoAAAAAAQAAAAAAAAAD0ludmFsaWRUaW1lbG9jawAAAAAF",
        "AAAAAQAAAAAAAAAAAAAAE1VwZ3JhZGVIaXN0b3J5RW50cnkAAAAABAAAAAAAAAALZXhlY3V0ZWRfYXQAAAAABgAAAAAAAAAIZXhlY3V0b3IAAAATAAAAAAAAAAxmcm9tX3ZlcnNpb24AAAAEAAAAAAAAAAp0b192ZXJzaW9uAAAAAAAE" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_nft: this.txFromJSON<Option<NftData>>,
        burn_nft: this.txFromJSON<Result<void>>,
        owner_of: this.txFromJSON<Option<string>>,
        get_admin: this.txFromJSON<Option<string>>,
        initialize: this.txFromJSON<Result<void>>,
        is_operator: this.txFromJSON<boolean>,
        has_hunt_nft: this.txFromJSON<boolean>,
        set_operator: this.txFromJSON<null>,
        total_supply: this.txFromJSON<u64>,
        transfer_nft: this.txFromJSON<Result<void>>,
        list_all_nfts: this.txFromJSON<Array<NftData>>,
        run_migration: this.txFromJSON<Result<MigrationReport>>,
        get_max_supply: this.txFromJSON<Option<u64>>,
        set_max_supply: this.txFromJSON<Result<void>>,
        get_player_nfts: this.txFromJSON<Array<u64>>,
        mint_reward_nft: this.txFromJSON<u64>,
        propose_upgrade: this.txFromJSON<Result<UpgradeProposal>>,
        remove_operator: this.txFromJSON<null>,
        get_nft_metadata: this.txFromJSON<Option<NftMetadataResponse>>,
        get_nfts_by_hunt: this.txFromJSON<Array<u64>>,
        verify_ownership: this.txFromJSON<boolean>,
        get_nft_extension: this.txFromJSON<Option<string>>,
        initialize_schema: this.txFromJSON<null>,
        set_nft_extension: this.txFromJSON<Result<void>>,
        get_hunt_nft_count: this.txFromJSON<u32>,
        get_nft_extensions: this.txFromJSON<Option<Map<string, string>>>,
        get_schema_version: this.txFromJSON<u32>,
        rollback_migration: this.txFromJSON<Result<MigrationReport>>,
        set_reward_manager: this.txFromJSON<Result<void>>,
        get_upgrade_history: this.txFromJSON<Array<UpgradeHistoryEntry>>,
        update_nft_metadata: this.txFromJSON<Result<void>>,
        get_remaining_supply: this.txFromJSON<Option<u64>>,
        get_upgrade_proposal: this.txFromJSON<Option<UpgradeProposal>>,
        get_upgrade_timelock: this.txFromJSON<u64>,
        remove_nft_extension: this.txFromJSON<Result<void>>,
        set_upgrade_timelock: this.txFromJSON<Result<void>>,
        add_authorized_contract: this.txFromJSON<Result<void>>,
        admin_update_image_uris: this.txFromJSON<Result<readonly [u32, u32]>>,
        get_collection_metadata: this.txFromJSON<Option<CollectionMetadata>>,
        search_nfts_by_metadata: this.txFromJSON<Array<NftData>>,
        mint_reward_nft_from_map: this.txFromJSON<Result<u64>>,
        remove_authorized_contract: this.txFromJSON<Result<void>>
  }
}
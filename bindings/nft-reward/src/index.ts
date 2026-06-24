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
 */
export interface NftData {
  hunt_id: u64;
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
 * Event emitted when an NFT is minted.
 */
export interface NftMintedEvent {
  hunt_id: u64;
  metadata: NftMetadata;
  minted_at: u64;
  nft_id: u64;
  owner: string;
  rarity: u32;
  tier: u32;
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
  hunt_id: u64;
  hunt_title: string;
  image_uri: string;
  nft_id: u64;
  rarity: u32;
  royalty_bps: Option<u32>;
  tier: u32;
  title: string;
}


/**
 * Event emitted when an NFT is transferred.
 */
export interface NftTransferredEvent {
  from: string;
  nft_id: u64;
  to: string;
}


/**
 * Event emitted when an NFT's mutable metadata is updated.
 */
export interface NftMetadataUpdatedEvent {
  nft_id: u64;
  updater: string;
}

export const NftErrorCode = {
  1: {message:"NftNotFound"},
  2: {message:"Unauthorized"},
  3: {message:"NotOwner"},
  4: {message:"InvalidRecipient"},
  5: {message:"SoulboundNft"},
  6: {message:"InvalidRarity"},
  7: {message:"AlreadyInitialized"},
  8: {message:"MaxSupplyReached"}
}


export interface MigrationReport {
  dry_run: boolean;
  from_version: u32;
  message: string;
  steps_applied: u32;
  succeeded: boolean;
  to_version: u32;
}

export interface Client {
  /**
   * Construct and simulate a burn transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Burns (permanently destroys) an NFT, removing it from storage and the owner's list.
   * 
   * # Authorization
   * The `owner` must authorize this call. The caller must also be the current owner.
   */
  burn: ({nft_id, owner}: {nft_id: u64, owner: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Retrieves NFT data by ID.
   */
  get_nft: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<NftData>>>

  /**
   * Construct and simulate a owner_of transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the owner of an NFT.
   */
  owner_of: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes the NFT reward contract with an optional max supply cap.
   * Call this once if you want to enforce a finite NFT supply.
   */
  initialize: ({max_supply}: {max_supply: Option<u64>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a total_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total number of NFTs minted so far.
   */
  total_supply: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a transfer_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Transfers an NFT from one address to another.
   * 
   * # Arguments
   * * `nft_id` - The NFT to transfer
   * * `from_address` - Current owner (must authorize the call)
   * * `to_address` - New owner
   * 
   * # Authorization
   * The `from_address` must authorize this call via `require_auth`.
   * For automatic transfers during reward distribution, the contract may be
   * the `from_address` when invoked by an authorized party.
   */
  transfer_nft: ({nft_id, from_address, to_address}: {nft_id: u64, from_address: string, to_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_nft_owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Alias for owner_of. Returns the owner of an NFT.
   */
  get_nft_owner: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a run_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  run_migration: ({admin, target_version, dry_run}: {admin: string, target_version: u32, dry_run: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<MigrationReport>>

  /**
   * Construct and simulate a get_player_nfts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns paginated NFT IDs owned by an address.
   */
  get_player_nfts: ({owner, offset, limit}: {owner: string, offset: u32, limit: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u64>>>

  /**
   * Construct and simulate a mint_reward_nft transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Mints a unique NFT as a reward for hunt completion.
   * 
   * `minter` must be an authorized minter (and must sign the transaction) when the
   * contract has been initialized.  Before initialization the check is skipped so
   * that existing deployments remain functional.
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
  mint_reward_nft: ({_minter, hunt_id, player_address, metadata}: {_minter: string, hunt_id: u64, player_address: string, metadata: NftMetadata}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a contract_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the contract version.
   */
  contract_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_nft_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns complete metadata for an NFT, including hunt info and completion details.
   */
  get_nft_metadata: ({nft_id}: {nft_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<NftMetadataResponse>>>

  /**
   * Construct and simulate a initialize_schema transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_schema: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_schema_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_schema_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a rollback_migration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rollback_migration: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<MigrationReport>>>

  /**
   * Construct and simulate a update_nft_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates mutable metadata fields (description, image_uri). Owner only.
   * Title, hunt info, and attributes remain immutable for collectibility.
   */
  update_nft_metadata: ({nft_id, updater, new_description, new_image_uri}: {nft_id: u64, updater: string, new_description: string, new_image_uri: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
   */
  mint_reward_nft_from_map: ({_minter, hunt_id, player_address, metadata}: {_minter: string, hunt_id: u64, player_address: string, metadata: Map<string, any>}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

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
      new ContractSpec([ "AAAAAAAAALVCdXJucyAocGVybWFuZW50bHkgZGVzdHJveXMpIGFuIE5GVCwgcmVtb3ZpbmcgaXQgZnJvbSBzdG9yYWdlIGFuZCB0aGUgb3duZXIncyBsaXN0LgoKIyBBdXRob3JpemF0aW9uClRoZSBgb3duZXJgIG11c3QgYXV0aG9yaXplIHRoaXMgY2FsbC4gVGhlIGNhbGxlciBtdXN0IGFsc28gYmUgdGhlIGN1cnJlbnQgb3duZXIuAAAAAAAABGJ1cm4AAAACAAAAAAAAAAZuZnRfaWQAAAAAAAYAAAAAAAAABW93bmVyAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAABlSZXRyaWV2ZXMgTkZUIGRhdGEgYnkgSUQuAAAAAAAAB2dldF9uZnQAAAAAAQAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAQAAA+gAAAfQAAAAB05mdERhdGEA",
        "AAAAAAAAABxSZXR1cm5zIHRoZSBvd25lciBvZiBhbiBORlQuAAAACG93bmVyX29mAAAAAQAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAQAAA+gAAAAT",
        "AAAAAQAAACNORlQgZGF0YSBzdHJ1Y3R1cmUgc3RvcmVkIG9uLWNoYWluLgAAAAAAAAAAB05mdERhdGEAAAAABgAAAAAAAAAHaHVudF9pZAAAAAAGAAAAAAAAAAhtZXRhZGF0YQAAB9AAAAALTmZ0TWV0YWRhdGEAAAAAAAAAAAltaW50ZWRfYXQAAAAAAAAGAAAAAAAAAAZuZnRfaWQAAAAAAAYAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAAMdHJhbnNmZXJhYmxlAAAAAQ==",
        "AAAAAAAAAH9Jbml0aWFsaXplcyB0aGUgTkZUIHJld2FyZCBjb250cmFjdCB3aXRoIGFuIG9wdGlvbmFsIG1heCBzdXBwbHkgY2FwLgpDYWxsIHRoaXMgb25jZSBpZiB5b3Ugd2FudCB0byBlbmZvcmNlIGEgZmluaXRlIE5GVCBzdXBwbHkuAAAAAAppbml0aWFsaXplAAAAAAABAAAAAAAAAAptYXhfc3VwcGx5AAAAAAPoAAAABgAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAAC9SZXR1cm5zIHRoZSB0b3RhbCBudW1iZXIgb2YgTkZUcyBtaW50ZWQgc28gZmFyLgAAAAAMdG90YWxfc3VwcGx5AAAAAAAAAAEAAAAG",
        "AAAAAAAAAYJUcmFuc2ZlcnMgYW4gTkZUIGZyb20gb25lIGFkZHJlc3MgdG8gYW5vdGhlci4KCiMgQXJndW1lbnRzCiogYG5mdF9pZGAgLSBUaGUgTkZUIHRvIHRyYW5zZmVyCiogYGZyb21fYWRkcmVzc2AgLSBDdXJyZW50IG93bmVyIChtdXN0IGF1dGhvcml6ZSB0aGUgY2FsbCkKKiBgdG9fYWRkcmVzc2AgLSBOZXcgb3duZXIKCiMgQXV0aG9yaXphdGlvbgpUaGUgYGZyb21fYWRkcmVzc2AgbXVzdCBhdXRob3JpemUgdGhpcyBjYWxsIHZpYSBgcmVxdWlyZV9hdXRoYC4KRm9yIGF1dG9tYXRpYyB0cmFuc2ZlcnMgZHVyaW5nIHJld2FyZCBkaXN0cmlidXRpb24sIHRoZSBjb250cmFjdCBtYXkgYmUKdGhlIGBmcm9tX2FkZHJlc3NgIHdoZW4gaW52b2tlZCBieSBhbiBhdXRob3JpemVkIHBhcnR5LgAAAAAADHRyYW5zZmVyX25mdAAAAAMAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAMZnJvbV9hZGRyZXNzAAAAEwAAAAAAAAAKdG9fYWRkcmVzcwAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADE5mdEVycm9yQ29kZQ==",
        "AAAAAAAAADBBbGlhcyBmb3Igb3duZXJfb2YuIFJldHVybnMgdGhlIG93bmVyIG9mIGFuIE5GVC4AAAANZ2V0X25mdF9vd25lcgAAAAAAAAEAAAAAAAAABm5mdF9pZAAAAAAABgAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAANcnVuX21pZ3JhdGlvbgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdGFyZ2V0X3ZlcnNpb24AAAAAAAQAAAAAAAAAB2RyeV9ydW4AAAAAAQAAAAEAAAfQAAAAD01pZ3JhdGlvblJlcG9ydAA=",
        "AAAAAQAAAH5Db3JlIGRpc3BsYXkgbWV0YWRhdGEgZm9yIGFuIE5GVCAodGl0bGUsIGRlc2NyaXB0aW9uLCBpbWFnZSBVUkkpLgpTdXBwb3J0cyBvZmYtY2hhaW4gc3RvcmFnZSByZWZlcmVuY2VzIHRvIGtlZXAgZ2FzIGNvc3RzIGxvdy4AAAAAAAAAAAALTmZ0TWV0YWRhdGEAAAAACAAAAJtPcmlnaW5hbCBjcmVhdG9yIG9mIHRoZSBORlQgKHN0YW1wZWQgYXQgbWludCB0aW1lIGZvciBwcm92ZW5hbmNlL2F0dHJpYnV0aW9uKS4KRXNzZW50aWFsIGZvciBzZWNvbmRhcnkgbWFya2V0IHJveWFsdHkgZGlzdHJpYnV0aW9uIGFuZCBjcmVhdG9yIGF0dHJpYnV0aW9uLgAAAAAHY3JlYXRvcgAAAAPoAAAAEwAAAAAAAAALZGVzY3JpcHRpb24AAAAAEAAAADFIdW50IHRpdGxlIGF0IHRpbWUgb2YgbWludCAoZm9yIGNvbnRleHQvZGlzcGxheSkuAAAAAAAACmh1bnRfdGl0bGUAAAAAABAAAAAAAAAACWltYWdlX3VyaQAAAAAAABAAAABWUmFyaXR5IHRpZXI6IDAgPSBkZWZhdWx0LCAxID0gY29tbW9uLCAyID0gdW5jb21tb24sIDMgPSByYXJlLCA0ID0gZXBpYywgNSA9IGxlZ2VuZGFyeS4AAAAAAAZyYXJpdHkAAAAAAAQAAACMUm95YWx0eSBpbiBiYXNpcyBwb2ludHMgKDEgYnAgPSAwLjAxJSkuIEZvciBleGFtcGxlLCAyNTAgPSAyLjUlIHJveWFsdHkuClVzZWQgZm9yIHNlY29uZGFyeSBtYXJrZXQgc2FsZXMgdG8gcHJvdmlkZSBvbmdvaW5nIGNyZWF0b3IgcmV2ZW51ZS4AAAALcm95YWx0eV9icHMAAAAD6AAAAAQAAAAuQ3VzdG9tIHRpZXIgZm9yIHNwZWNpYWwgY2F0ZWdvcmllcyAoMCA9IG5vbmUpLgAAAAAABHRpZXIAAAAEAAAAAAAAAAV0aXRsZQAAAAAAABA=",
        "AAAAAAAAAC5SZXR1cm5zIHBhZ2luYXRlZCBORlQgSURzIG93bmVkIGJ5IGFuIGFkZHJlc3MuAAAAAAAPZ2V0X3BsYXllcl9uZnRzAAAAAAMAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAAGb2Zmc2V0AAAAAAAEAAAAAAAAAAVsaW1pdAAAAAAAAAQAAAABAAAD6gAAAAY=",
        "AAAAAAAAAlpNaW50cyBhIHVuaXF1ZSBORlQgYXMgYSByZXdhcmQgZm9yIGh1bnQgY29tcGxldGlvbi4KCmBtaW50ZXJgIG11c3QgYmUgYW4gYXV0aG9yaXplZCBtaW50ZXIgKGFuZCBtdXN0IHNpZ24gdGhlIHRyYW5zYWN0aW9uKSB3aGVuIHRoZQpjb250cmFjdCBoYXMgYmVlbiBpbml0aWFsaXplZC4gIEJlZm9yZSBpbml0aWFsaXphdGlvbiB0aGUgY2hlY2sgaXMgc2tpcHBlZCBzbwp0aGF0IGV4aXN0aW5nIGRlcGxveW1lbnRzIHJlbWFpbiBmdW5jdGlvbmFsLgoKIyBBcmd1bWVudHMKKiBgbWludGVyYCAtIEFkZHJlc3MgcGVyZm9ybWluZyB0aGUgbWludCAobXVzdCBiZSB3aGl0ZWxpc3RlZCBhZnRlciBpbml0KQoqIGBodW50X2lkYCAtIFRoZSBodW50IHRoaXMgTkZUIGNvbW1lbW9yYXRlcwoqIGBwbGF5ZXJfYWRkcmVzc2AgLSBUaGUgYWRkcmVzcyBvZiB0aGUgcGxheWVyIGNvbXBsZXRpbmcgdGhlIGh1bnQgKGluaXRpYWwgb3duZXIpCiogYG1ldGFkYXRhYCAtIE5GVCBtZXRhZGF0YSAodGl0bGUsIGRlc2NyaXB0aW9uLCBpbWFnZSBVUkksIGh1bnRfdGl0bGUsIHJhcml0eSwgdGllcikKCiMgUmV0dXJucwpUaGUgdW5pcXVlIE5GVCBJRCBvZiB0aGUgbWludGVkIE5GVAAAAAAAD21pbnRfcmV3YXJkX25mdAAAAAAEAAAAAAAAAAdfbWludGVyAAAAABMAAAAAAAAAB2h1bnRfaWQAAAAABgAAAAAAAAAOcGxheWVyX2FkZHJlc3MAAAAAABMAAAAAAAAACG1ldGFkYXRhAAAH0AAAAAtOZnRNZXRhZGF0YQAAAAABAAAABg==",
        "AAAAAAAAAB1SZXR1cm5zIHRoZSBjb250cmFjdCB2ZXJzaW9uLgAAAAAAABBjb250cmFjdF92ZXJzaW9uAAAAAAAAAAEAAAAE",
        "AAAAAAAAAFFSZXR1cm5zIGNvbXBsZXRlIG1ldGFkYXRhIGZvciBhbiBORlQsIGluY2x1ZGluZyBodW50IGluZm8gYW5kIGNvbXBsZXRpb24gZGV0YWlscy4AAAAAAAAQZ2V0X25mdF9tZXRhZGF0YQAAAAEAAAAAAAAABm5mdF9pZAAAAAAABgAAAAEAAAPoAAAH0AAAABNOZnRNZXRhZGF0YVJlc3BvbnNlAA==",
        "AAAAAQAAACRFdmVudCBlbWl0dGVkIHdoZW4gYW4gTkZUIGlzIG1pbnRlZC4AAAAAAAAADk5mdE1pbnRlZEV2ZW50AAAAAAAHAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAACG1ldGFkYXRhAAAH0AAAAAtOZnRNZXRhZGF0YQAAAAAAAAAACW1pbnRlZF9hdAAAAAAAAAYAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAZyYXJpdHkAAAAAAAQAAAAAAAAABHRpZXIAAAAE",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9zY2hlbWEAAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAASZ2V0X3NjaGVtYV92ZXJzaW9uAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAScm9sbGJhY2tfbWlncmF0aW9uAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6AAAB9AAAAAPTWlncmF0aW9uUmVwb3J0AA==",
        "AAAAAAAAAItVcGRhdGVzIG11dGFibGUgbWV0YWRhdGEgZmllbGRzIChkZXNjcmlwdGlvbiwgaW1hZ2VfdXJpKS4gT3duZXIgb25seS4KVGl0bGUsIGh1bnQgaW5mbywgYW5kIGF0dHJpYnV0ZXMgcmVtYWluIGltbXV0YWJsZSBmb3IgY29sbGVjdGliaWxpdHkuAAAAABN1cGRhdGVfbmZ0X21ldGFkYXRhAAAAAAQAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAHdXBkYXRlcgAAAAATAAAAAAAAAA9uZXdfZGVzY3JpcHRpb24AAAAAEAAAAAAAAAANbmV3X2ltYWdlX3VyaQAAAAAAABAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAxOZnRFcnJvckNvZGU=",
        "AAAAAQAAAFFDb21wbGV0ZSBtZXRhZGF0YSByZXR1cm5lZCBieSBnZXRfbmZ0X21ldGFkYXRhIChpbmNsdWRlcyBOZnREYXRhLWRlcml2ZWQgZmllbGRzKS4AAAAAAAAAAAAAE05mdE1ldGFkYXRhUmVzcG9uc2UAAAAADQAAAAAAAAARY29tcGxldGlvbl9wbGF5ZXIAAAAAAAATAAAAAAAAABRjb21wbGV0aW9uX3RpbWVzdGFtcAAAAAYAAAAAAAAAB2NyZWF0b3IAAAAD6AAAABMAAAAAAAAADWN1cnJlbnRfb3duZXIAAAAAAAATAAAAAAAAAAtkZXNjcmlwdGlvbgAAAAAQAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAACmh1bnRfdGl0bGUAAAAAABAAAAAAAAAACWltYWdlX3VyaQAAAAAAABAAAAAAAAAABm5mdF9pZAAAAAAABgAAAAAAAAAGcmFyaXR5AAAAAAAEAAAAAAAAAAtyb3lhbHR5X2JwcwAAAAPoAAAABAAAAAAAAAAEdGllcgAAAAQAAAAAAAAABXRpdGxlAAAAAAAAEA==",
        "AAAAAQAAAClFdmVudCBlbWl0dGVkIHdoZW4gYW4gTkZUIGlzIHRyYW5zZmVycmVkLgAAAAAAAAAAAAATTmZ0VHJhbnNmZXJyZWRFdmVudAAAAAADAAAAAAAAAARmcm9tAAAAEwAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAAAAAAJ0bwAAAAAAEw==",
        "AAAAAAAAAqBNaW50cyBhIHJld2FyZCBORlQgZnJvbSBhIGdlbmVyaWMgbWV0YWRhdGEgbWFwLiBUaGlzIGlzIHRoZSBlbnRyeXBvaW50CnVzZWQgYnkgY3Jvc3MtY29udHJhY3QgY2FsbGVycyAoZS5nLiBSZXdhcmRNYW5hZ2VyKSB0aGF0IGNhbm5vdCBkZXBlbmQKb24gdGhpcyBjcmF0ZSdzIGBOZnRNZXRhZGF0YWAgdHlwZSBkaXJlY3RseS4KCmBtaW50ZXJgIGlzIHRoZSBjYWxsaW5nIGNvbnRyYWN0J3MgYWRkcmVzcyBhbmQgbXVzdCBiZSB3aGl0ZWxpc3RlZCB3aGVuIHRoZQpjb250cmFjdCBoYXMgYmVlbiBpbml0aWFsaXplZC4KCkV4cGVjdGVkIGtleXMgaW4gYG1ldGFkYXRhYCAoYWxsIG9wdGlvbmFsLCB3aXRoIHNlbnNpYmxlIGRlZmF1bHRzKToKLSAidGl0bGUiOiBTdHJpbmcKLSAiZGVzY3JpcHRpb24iOiBTdHJpbmcKLSAiaW1hZ2VfdXJpIjogU3RyaW5nCi0gImh1bnRfdGl0bGUiOiBTdHJpbmcgKGRlZmF1bHRzIHRvIHRpdGxlIHdoZW4gb21pdHRlZC9lbXB0eSkKLSAicmFyaXR5IjogdTMyCi0gInRpZXIiOiB1MzIKLSAiY3JlYXRvciI6IEFkZHJlc3MgKGRlZmF1bHRzIHRvIHBsYXllcl9hZGRyZXNzIGlmIG9taXR0ZWQpCi0gInJveWFsdHlfYnBzIjogdTMyIChvcHRpb25hbCwgYmFzaXMgcG9pbnRzIGZvciByb3lhbHR5IHBlcmNlbnRhZ2UpCi0gInRyYW5zZmVyYWJsZSI6IGJvb2wAAAAYbWludF9yZXdhcmRfbmZ0X2Zyb21fbWFwAAAABAAAAAAAAAAHX21pbnRlcgAAAAATAAAAAAAAAAdodW50X2lkAAAAAAYAAAAAAAAADnBsYXllcl9hZGRyZXNzAAAAAAATAAAAAAAAAAhtZXRhZGF0YQAAA+wAAAARAAAAAAAAAAEAAAAG",
        "AAAAAQAAADhFdmVudCBlbWl0dGVkIHdoZW4gYW4gTkZUJ3MgbXV0YWJsZSBtZXRhZGF0YSBpcyB1cGRhdGVkLgAAAAAAAAAXTmZ0TWV0YWRhdGFVcGRhdGVkRXZlbnQAAAAAAgAAAAAAAAAGbmZ0X2lkAAAAAAAGAAAAAAAAAAd1cGRhdGVyAAAAABM=",
        "AAAABAAAAAAAAAAAAAAADE5mdEVycm9yQ29kZQAAAAgAAAAAAAAAC05mdE5vdEZvdW5kAAAAAAEAAAAAAAAADFVuYXV0aG9yaXplZAAAAAIAAAAAAAAACE5vdE93bmVyAAAAAwAAAAAAAAAQSW52YWxpZFJlY2lwaWVudAAAAAQAAAAAAAAADFNvdWxib3VuZE5mdAAAAAUAAAAAAAAADUludmFsaWRSYXJpdHkAAAAAAAAGAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAcAAAAAAAAAEE1heFN1cHBseVJlYWNoZWQAAAAI",
        "AAAAAQAAAAAAAAAAAAAAD01pZ3JhdGlvblJlcG9ydAAAAAAGAAAAAAAAAAdkcnlfcnVuAAAAAAEAAAAAAAAADGZyb21fdmVyc2lvbgAAAAQAAAAAAAAAB21lc3NhZ2UAAAAAEAAAAAAAAAANc3RlcHNfYXBwbGllZAAAAAAAAAQAAAAAAAAACXN1Y2NlZWRlZAAAAAAAAAEAAAAAAAAACnRvX3ZlcnNpb24AAAAAAAQ=" ]),
      options
    )
  }
  public readonly fromJSON = {
    burn: this.txFromJSON<Result<void>>,
        get_nft: this.txFromJSON<Option<NftData>>,
        owner_of: this.txFromJSON<Option<string>>,
        initialize: this.txFromJSON<Result<void>>,
        total_supply: this.txFromJSON<u64>,
        transfer_nft: this.txFromJSON<Result<void>>,
        get_nft_owner: this.txFromJSON<Option<string>>,
        run_migration: this.txFromJSON<MigrationReport>,
        get_player_nfts: this.txFromJSON<Array<u64>>,
        mint_reward_nft: this.txFromJSON<u64>,
        contract_version: this.txFromJSON<u32>,
        get_nft_metadata: this.txFromJSON<Option<NftMetadataResponse>>,
        initialize_schema: this.txFromJSON<null>,
        get_schema_version: this.txFromJSON<u32>,
        rollback_migration: this.txFromJSON<Option<MigrationReport>>,
        update_nft_metadata: this.txFromJSON<Result<void>>,
        mint_reward_nft_from_map: this.txFromJSON<u64>
  }
}
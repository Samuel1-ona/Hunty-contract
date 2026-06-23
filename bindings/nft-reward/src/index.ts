import { ContractSpec } from "@stellar/stellar-sdk";
import type {
  AssembledTransaction,
  ContractClientOptions,
} from "@stellar/stellar-sdk/contract";
import { ContractClient } from "@stellar/stellar-sdk/contract";

export const networks = {
  testnet: { networkPassphrase: "Test SDF Network ; September 2015" },
  mainnet: { networkPassphrase: "Public Global Stellar Network ; September 2015" },
} as const;

export interface NftMetadata {
  title: string;
  description: string;
  image_uri: string;
  hunt_title: string;
  rarity: number;
  tier: number;
}

export interface NftMetadataResponse {
  nft_id: bigint;
  hunt_id: bigint;
  hunt_title: string;
  completion_timestamp: bigint;
  completion_player: string;
  current_owner: string;
  title: string;
  description: string;
  image_uri: string;
  rarity: number;
  tier: number;
}

export interface NftData {
  nft_id: bigint;
  hunt_id: bigint;
  owner: string;
  completion_player: string;
  metadata: NftMetadata;
  minted_at: bigint;
}

export class Client extends ContractClient {
  constructor(public readonly options: ContractClientOptions) {
    super(new ContractSpec([]), options);
  }

  async mint_reward_nft({
    hunt_id,
    player_address,
    metadata,
  }: {
    hunt_id: bigint;
    player_address: string;
    metadata: NftMetadata;
  }): Promise<AssembledTransaction<bigint>> {
    return this.call("mint_reward_nft", hunt_id, player_address, metadata);
  }

  async mint_reward_nft_from_map({
    hunt_id,
    player_address,
    metadata,
  }: {
    hunt_id: bigint;
    player_address: string;
    metadata: Map<string, any>;
  }): Promise<AssembledTransaction<bigint>> {
    return this.call("mint_reward_nft_from_map", hunt_id, player_address, metadata);
  }

  async get_nft({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<NftData | undefined>> {
    return this.call("get_nft", nft_id);
  }

  async get_nft_metadata({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<NftMetadataResponse | undefined>> {
    return this.call("get_nft_metadata", nft_id);
  }

  async update_nft_metadata({
    nft_id,
    updater,
    new_description,
    new_image_uri,
  }: {
    nft_id: bigint;
    updater: string;
    new_description: string;
    new_image_uri: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("update_nft_metadata", nft_id, updater, new_description, new_image_uri);
  }

  async total_supply(): Promise<AssembledTransaction<bigint>> {
    return this.call("total_supply");
  }

  async owner_of({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<string | undefined>> {
    return this.call("owner_of", nft_id);
  }

  async get_nft_owner({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<string | undefined>> {
    return this.call("get_nft_owner", nft_id);
  }

  async get_player_nfts({
    owner,
  }: {
    owner: string;
  }): Promise<AssembledTransaction<bigint[]>> {
    return this.call("get_player_nfts", owner);
  }

  async transfer_nft({
    nft_id,
    from_address,
    to_address,
  }: {
    nft_id: bigint;
    from_address: string;
    to_address: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("transfer_nft", nft_id, from_address, to_address);
  }
}

import { ContractSpec } from "@stellar/stellar-sdk";
import type {
  AssembledTransaction,
  ContractClientOptions,
} from "@stellar/stellar-sdk/contract";
import { ContractClient } from "@stellar/stellar-sdk/contract";

export * from "./types";

export const networks = {
  testnet: { networkPassphrase: "Test SDF Network ; September 2015" },
  mainnet: { networkPassphrase: "Public Global Stellar Network ; September 2015" },
} as const;

export interface RewardConfig {
  xlm_amount: bigint | undefined;
  nft_contract: string | undefined;
  nft_title: string;
  nft_description: string;
  nft_image_uri: string;
  nft_hunt_title: string;
  nft_rarity: number;
  nft_tier: number;
}

export interface RewardPoolConfig {
  creator: string;
  min_distribution_amount: bigint;
}

export interface RewardPoolStatus {
  balance: bigint;
  total_deposited: bigint;
  total_distributed: bigint;
  creator: string;
  min_distribution_amount: bigint;
}

export interface ValidationResult {
  is_valid: boolean;
  balance: bigint;
  required: bigint;
}

export interface DistributionRecord {
  xlm_amount: bigint;
  nft_id: bigint | undefined;
}

export interface DistributionStatus {
  distributed: boolean;
  xlm_amount: bigint;
  nft_id: bigint | undefined;
}

export class Client extends ContractClient {
  constructor(public readonly options: ContractClientOptions) {
    super(new ContractSpec([]), options);
  }

  async initialize({
    admin,
    xlm_token,
  }: {
    admin: string;
    xlm_token: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("initialize", admin, xlm_token);
  }

  async set_nft_reward_contract({
    admin,
    nft_contract,
  }: {
    admin: string;
    nft_contract: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("set_nft_reward_contract", admin, nft_contract);
  }

  async create_reward_pool({
    creator,
    hunt_id,
    min_distribution_amount,
  }: {
    creator: string;
    hunt_id: bigint;
    min_distribution_amount: bigint;
  }): Promise<AssembledTransaction<void>> {
    return this.call("create_reward_pool", creator, hunt_id, min_distribution_amount);
  }

  async fund_reward_pool({
    funder,
    hunt_id,
    amount,
  }: {
    funder: string;
    hunt_id: bigint;
    amount: bigint;
  }): Promise<AssembledTransaction<void>> {
    return this.call("fund_reward_pool", funder, hunt_id, amount);
  }

  async refund_pool({
    creator,
    hunt_id,
  }: {
    creator: string;
    hunt_id: bigint;
  }): Promise<AssembledTransaction<void>> {
    return this.call("refund_pool", creator, hunt_id);
  }

  async get_reward_pool({
    hunt_id,
  }: {
    hunt_id: bigint;
  }): Promise<AssembledTransaction<RewardPoolStatus | undefined>> {
    return this.call("get_reward_pool", hunt_id);
  }

  async validate_pool({
    hunt_id,
    required_amount,
  }: {
    hunt_id: bigint;
    required_amount: bigint;
  }): Promise<AssembledTransaction<ValidationResult>> {
    return this.call("validate_pool", hunt_id, required_amount);
  }

  async distribute_rewards({
    hunt_id,
    player_address,
    reward_config,
  }: {
    hunt_id: bigint;
    player_address: string;
    reward_config: RewardConfig;
  }): Promise<AssembledTransaction<void>> {
    return this.call("distribute_rewards", hunt_id, player_address, reward_config);
  }

  async get_distribution_status({
    hunt_id,
    player,
  }: {
    hunt_id: bigint;
    player: string;
  }): Promise<AssembledTransaction<DistributionStatus>> {
    return this.call("get_distribution_status", hunt_id, player);
  }

  async get_pool_balance({
    hunt_id,
  }: {
    hunt_id: bigint;
  }): Promise<AssembledTransaction<bigint>> {
    return this.call("get_pool_balance", hunt_id);
  }

  async is_reward_distributed({
    hunt_id,
    player,
  }: {
    hunt_id: bigint;
    player: string;
  }): Promise<AssembledTransaction<boolean>> {
    return this.call("is_reward_distributed", hunt_id, player);
  }

  async admin_withdraw_unclaimed({
    admin,
    hunt_id,
    recipient,
  }: {
    admin: string;
    hunt_id: bigint;
    recipient: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("admin_withdraw_unclaimed", admin, hunt_id, recipient);
  }
}

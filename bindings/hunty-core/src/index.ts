import { ContractSpec, Address } from "@stellar/stellar-sdk";
import { Buffer } from "buffer";
import type {
  AssembledTransaction,
  ContractClientOptions,
  XDR_BASE64,
} from "@stellar/stellar-sdk/contract";
import { ContractClient } from "@stellar/stellar-sdk/contract";

export * from "./types";

export const networks = {
  testnet: { networkPassphrase: "Test SDF Network ; September 2015" },
  mainnet: { networkPassphrase: "Public Global Stellar Network ; September 2015" },
} as const;

export const spec = new ContractSpec([
  // HuntStatus enum
  "AAAAAQAAAAAAAAAAAAAAAA9IdW50U3RhdHVzAAAAAAAABAAAAAAAAAAFRHJhZnQAAAAAAAABAAAAAAAAAAZBY3RpdmUAAAAAAAIAAAAAAAAACUNvbXBsZXRlZAAAAAAAAAMAAAAAAAAACUNhbmNlbGxlZAAAAAAAAAQ=",
]);

export interface Hunt {
  hunt_id: bigint;
  creator: string;
  title: string;
  description: string;
  status: HuntStatus;
  created_at: bigint;
  activated_at: bigint;
  end_time: bigint;
  reward_config: RewardConfig;
  total_clues: number;
  required_clues: number;
}

export type HuntStatus =
  | { tag: "Draft"; values: undefined }
  | { tag: "Active"; values: undefined }
  | { tag: "Completed"; values: undefined }
  | { tag: "Cancelled"; values: undefined };

export interface RewardConfig {
  xlm_pool: bigint;
  nft_enabled: boolean;
  nft_contract: string | undefined;
  max_winners: number;
  claimed_count: number;
  nft_rarity: number;
  nft_tier: number;
}

export interface ClueInfo {
  clue_id: number;
  question: string;
  points: number;
  is_required: boolean;
}

export interface PlayerProgress {
  player: string;
  hunt_id: bigint;
  completed_clues: number[];
  total_score: number;
  started_at: bigint;
  completed_at: bigint;
  is_completed: boolean;
  reward_claimed: boolean;
}

export interface LeaderboardEntry {
  rank: number;
  player: string;
  score: number;
  completed_at: bigint;
  is_completed: boolean;
  queried_at: bigint;
}

export interface HuntStatistics {
  total_players: number;
  completed_count: number;
  completion_rate_percent: number;
  total_score_sum: bigint;
  average_score: number;
}

export class Client extends ContractClient {
  constructor(public readonly options: ContractClientOptions) {
    super(new ContractSpec([]), options);
  }

  async create_hunt({
    creator,
    title,
    description,
    start_time,
    end_time,
  }: {
    creator: string;
    title: string;
    description: string;
    start_time: bigint | undefined;
    end_time: bigint | undefined;
  }): Promise<AssembledTransaction<bigint>> {
    return this.call("create_hunt", creator, title, description, start_time, end_time);
  }

  async add_clue({
    hunt_id,
    question,
    answer,
    points,
    is_required,
  }: {
    hunt_id: bigint;
    question: string;
    answer: string;
    points: number;
    is_required: boolean;
  }): Promise<AssembledTransaction<number>> {
    return this.call("add_clue", hunt_id, question, answer, points, is_required);
  }

  async get_clue({
    hunt_id,
    clue_id,
  }: {
    hunt_id: bigint;
    clue_id: number;
  }): Promise<AssembledTransaction<ClueInfo>> {
    return this.call("get_clue", hunt_id, clue_id);
  }

  async list_clues({ hunt_id }: { hunt_id: bigint }): Promise<AssembledTransaction<ClueInfo[]>> {
    return this.call("list_clues", hunt_id);
  }

  async activate_hunt({
    hunt_id,
    caller,
  }: {
    hunt_id: bigint;
    caller: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("activate_hunt", hunt_id, caller);
  }

  async deactivate_hunt({
    hunt_id,
    caller,
  }: {
    hunt_id: bigint;
    caller: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("deactivate_hunt", hunt_id, caller);
  }

  async cancel_hunt({
    hunt_id,
    caller,
  }: {
    hunt_id: bigint;
    caller: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("cancel_hunt", hunt_id, caller);
  }

  async get_hunt_info({ hunt_id }: { hunt_id: bigint }): Promise<AssembledTransaction<Hunt>> {
    return this.call("get_hunt_info", hunt_id);
  }

  async set_reward_manager({
    reward_manager,
  }: {
    reward_manager: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("set_reward_manager", reward_manager);
  }

  async register_player({
    hunt_id,
    player,
  }: {
    hunt_id: bigint;
    player: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("register_player", hunt_id, player);
  }

  async submit_answer({
    hunt_id,
    clue_id,
    player,
    answer,
  }: {
    hunt_id: bigint;
    clue_id: number;
    player: string;
    answer: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("submit_answer", hunt_id, clue_id, player, answer);
  }

  async complete_hunt({
    hunt_id,
    player,
  }: {
    hunt_id: bigint;
    player: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("complete_hunt", hunt_id, player);
  }

  async batch_complete_hunt({
    hunt_id,
    creator,
    players,
  }: {
    hunt_id: bigint;
    creator: string;
    players: string[];
  }): Promise<AssembledTransaction<void>> {
    return this.call("batch_complete_hunt", hunt_id, creator, players);
  }

  async get_player_progress({
    hunt_id,
    player,
  }: {
    hunt_id: bigint;
    player: string;
  }): Promise<AssembledTransaction<PlayerProgress>> {
    return this.call("get_player_progress", hunt_id, player);
  }

  async get_completed_clues({
    hunt_id,
    player,
  }: {
    hunt_id: bigint;
    player: string;
  }): Promise<AssembledTransaction<number[]>> {
    return this.call("get_completed_clues", hunt_id, player);
  }

  async get_hunt_leaderboard({
    hunt_id,
    limit,
  }: {
    hunt_id: bigint;
    limit: number;
  }): Promise<AssembledTransaction<LeaderboardEntry[]>> {
    return this.call("get_hunt_leaderboard", hunt_id, limit);
  }

  async get_hunt_statistics({
    hunt_id,
  }: {
    hunt_id: bigint;
  }): Promise<AssembledTransaction<HuntStatistics>> {
    return this.call("get_hunt_statistics", hunt_id);
  }
}

import { RateLimitConfig } from './types';

export type AppEnvironment = 'testnet' | 'staging' | 'mainnet';

export interface ContractConfig {
  huntyCoreId: string;
  rewardManagerId: string;
  nftRewardId: string;
}

export interface AppConfig {
  environment: AppEnvironment;
  port: number;
  adminSecret: string;
  rateLimit: RateLimitConfig;
  stellar: {
    network: string;
    rpcUrl: string;
    networkPassphrase: string;
  };
  contracts: ContractConfig;
}

const allowedEnvironments = new Set<AppEnvironment>(['testnet', 'staging', 'mainnet']);

function requireEnv(env: NodeJS.ProcessEnv, key: string): string {
  const value = env[key];
  if (!value || value.trim() === '') {
    throw new Error(`Missing required environment variable: ${key}`);
  }
  if (value.startsWith('replace-with-')) {
    throw new Error(`Environment variable ${key} still contains a placeholder value`);
  }
  return value;
}

function parsePositiveInteger(env: NodeJS.ProcessEnv, key: string): number {
  const rawValue = requireEnv(env, key);
  const value = Number(rawValue);
  if (!Number.isInteger(value) || value < 1) {
    throw new Error(`Environment variable ${key} must be a positive integer`);
  }
  return value;
}

export function loadConfig(env: NodeJS.ProcessEnv = process.env): AppConfig {
  const environment = requireEnv(env, 'APP_ENV');
  if (!allowedEnvironments.has(environment as AppEnvironment)) {
    throw new Error('APP_ENV must be one of: testnet, staging, mainnet');
  }

  return {
    environment: environment as AppEnvironment,
    port: parsePositiveInteger(env, 'PORT'),
    adminSecret: requireEnv(env, 'ADMIN_SECRET'),
    rateLimit: {
      maxMints: parsePositiveInteger(env, 'MAX_MINTS'),
      windowMs: parsePositiveInteger(env, 'MINT_WINDOW_MS'),
    },
    stellar: {
      network: requireEnv(env, 'STELLAR_NETWORK'),
      rpcUrl: requireEnv(env, 'SOROBAN_RPC_URL'),
      networkPassphrase: requireEnv(env, 'NETWORK_PASSPHRASE'),
    },
    contracts: {
      huntyCoreId: requireEnv(env, 'HUNTY_CORE_ID'),
      rewardManagerId: requireEnv(env, 'REWARD_MANAGER_ID'),
      nftRewardId: requireEnv(env, 'NFT_REWARD_ID'),
    },
  };
}

export function publicConfig(config: AppConfig) {
  return {
    environment: config.environment,
    stellar: config.stellar,
    contracts: config.contracts,
  };
}

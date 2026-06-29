import { describe, expect, it } from 'vitest';
import { loadConfig } from './config';

const validEnv = {
  APP_ENV: 'staging',
  PORT: '3000',
  ADMIN_SECRET: 'secret',
  MAX_MINTS: '3',
  MINT_WINDOW_MS: '60000',
  STELLAR_NETWORK: 'testnet',
  SOROBAN_RPC_URL: 'https://soroban-testnet.stellar.org',
  NETWORK_PASSPHRASE: 'Test SDF Network ; September 2015',
  HUNTY_CORE_ID: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
  REWARD_MANAGER_ID: 'CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB',
  NFT_REWARD_ID: 'CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC',
};

describe('loadConfig', () => {
  it('loads a complete environment config', () => {
    expect(loadConfig(validEnv)).toMatchObject({
      environment: 'staging',
      port: 3000,
      rateLimit: { maxMints: 3, windowMs: 60000 },
      contracts: {
        huntyCoreId: validEnv.HUNTY_CORE_ID,
        rewardManagerId: validEnv.REWARD_MANAGER_ID,
        nftRewardId: validEnv.NFT_REWARD_ID,
      },
    });
  });

  it('fails fast when a required value is missing', () => {
    const env = { ...validEnv, HUNTY_CORE_ID: '' };
    expect(() => loadConfig(env)).toThrow('Missing required environment variable: HUNTY_CORE_ID');
  });

  it('rejects placeholder values', () => {
    const env = { ...validEnv, ADMIN_SECRET: 'replace-with-secret' };
    expect(() => loadConfig(env)).toThrow('ADMIN_SECRET still contains a placeholder value');
  });

  it('rejects invalid environment names', () => {
    const env = { ...validEnv, APP_ENV: 'production' };
    expect(() => loadConfig(env)).toThrow('APP_ENV must be one of: testnet, staging, mainnet');
  });
});

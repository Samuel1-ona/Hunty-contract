import { RateLimitConfig, RateLimitConfigUpdate } from './types';
import { MintRateLimitError } from './errors';
import { MemoryMintStore, MintRecordStore } from './mintStore';

export class MintRateLimiter {
  private store: MintRecordStore;
  private config: RateLimitConfig;
  private adminSecret: string;

  constructor(
    config: RateLimitConfig,
    adminSecret: string,
    store: MintRecordStore = new MemoryMintStore(),
  ) {
    this.config = { ...config };
    this.adminSecret = adminSecret;
    this.store = store;
  }

  async check(address: string): Promise<void> {
    const now = Date.now();
    const cutoff = now - this.config.windowMs;
    const recentMints = await this.store.getRecent(address, cutoff);

    if (recentMints.length >= this.config.maxMints) {
      const oldestRecent = Math.min(...recentMints);
      const cooldownMs = oldestRecent + this.config.windowMs - now;
      throw new MintRateLimitError(cooldownMs);
    }
  }

  async recordMint(address: string): Promise<void> {
    const now = Date.now();
    const cutoff = now - this.config.windowMs;
    const result = await this.store.tryRecord(
      address,
      now,
      cutoff,
      this.config.maxMints,
    );
    if (!result.allowed) {
      const cooldownMs = result.oldestRecent + this.config.windowMs - now;
      throw new MintRateLimitError(cooldownMs);
    }
  }

  async mint(address: string): Promise<void> {
    const now = Date.now();
    const cutoff = now - this.config.windowMs;
    const result = await this.store.tryRecord(
      address,
      now,
      cutoff,
      this.config.maxMints,
    );
    if (!result.allowed) {
      const cooldownMs = result.oldestRecent + this.config.windowMs - now;
      throw new MintRateLimitError(cooldownMs);
    }
  }

  async getMintCount(address: string): Promise<number> {
    const now = Date.now();
    const cutoff = now - this.config.windowMs;
    const recent = await this.store.getRecent(address, cutoff);
    return recent.length;
  }

  getConfig(): RateLimitConfig {
    return { ...this.config };
  }

  updateConfig(update: RateLimitConfigUpdate, secret: string): void {
    if (secret !== this.adminSecret) {
      throw new Error('Unauthorized: invalid admin secret');
    }
    if (update.maxMints !== undefined) {
      if (update.maxMints < 1) {
        throw new Error('maxMints must be at least 1');
      }
      this.config.maxMints = update.maxMints;
    }
    if (update.windowMs !== undefined) {
      if (update.windowMs < 1000) {
        throw new Error('windowMs must be at least 1000');
      }
      this.config.windowMs = update.windowMs;
    }
  }
}

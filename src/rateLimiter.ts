import { RateLimitConfig, AddressMintRecord, RateLimitConfigUpdate } from './types';
import { MintRateLimitError } from './errors';

export class MintRateLimiter {
  private records: Map<string, AddressMintRecord> = new Map();
  private config: RateLimitConfig;
  private adminSecret: string;

  constructor(config: RateLimitConfig, adminSecret: string) {
    this.config = { ...config };
    this.adminSecret = adminSecret;
  }

  check(address: string): void {
    const now = Date.now();
    const record = this.records.get(address);
    const timestamps = record?.timestamps ?? [];

    const cutoff = now - this.config.windowMs;
    const recentMints = timestamps.filter(t => t > cutoff);

    if (recentMints.length >= this.config.maxMints) {
      const oldestRecent = Math.min(...recentMints);
      const cooldownMs = oldestRecent + this.config.windowMs - now;
      throw new MintRateLimitError(cooldownMs);
    }
  }

  recordMint(address: string): void {
    const now = Date.now();
    const record = this.records.get(address) ?? { timestamps: [] };
    const cutoff = now - this.config.windowMs;
    record.timestamps = [...record.timestamps.filter(t => t > cutoff), now];
    this.records.set(address, record);
  }

  mint(address: string): void {
    this.check(address);
    this.recordMint(address);
  }

  getMintCount(address: string): number {
    const now = Date.now();
    const cutoff = now - this.config.windowMs;
    const record = this.records.get(address);
    return record ? record.timestamps.filter(t => t > cutoff).length : 0;
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

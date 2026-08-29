export interface RateLimitConfig {
  maxMints: number;
  windowMs: number;
}

export interface MintResult {
  allowed: boolean;
  cooldownMs: number | null;
}

export interface AddressMintRecord {
  timestamps: number[];
}

export interface RateLimitConfigUpdate {
  maxMints?: number;
  windowMs?: number;
}

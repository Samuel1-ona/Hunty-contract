import { describe, it, expect, vi, beforeEach } from 'vitest';
import { MintRateLimiter } from './rateLimiter';
import { MintRateLimitError } from './errors';

describe('MintRateLimiter', () => {
  let limiter: MintRateLimiter;

  beforeEach(() => {
    vi.useFakeTimers();
    limiter = new MintRateLimiter({ maxMints: 3, windowMs: 60_000 }, 'secret');
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('mint', () => {
    it('allows minting within the limit', () => {
      limiter.mint('addr1');
      expect(limiter.getMintCount('addr1')).toBe(1);
    });

    it('blocks minting when limit is exceeded', () => {
      limiter.mint('addr1');
      limiter.mint('addr1');
      limiter.mint('addr1');
      expect(() => limiter.mint('addr1')).toThrow(MintRateLimitError);
    });

    it('returns cooldown time in the error', () => {
      limiter.mint('addr1');
      limiter.mint('addr1');
      limiter.mint('addr1');
      try {
        limiter.mint('addr1');
      } catch (err) {
        expect(err).toBeInstanceOf(MintRateLimitError);
        const typed = err as MintRateLimitError;
        expect(typed.cooldownMs).toBeGreaterThan(0);
        expect(typed.cooldownMs).toBeLessThanOrEqual(60_000);
        expect(typed.message).toContain('seconds');
      }
    });

    it('allows minting again after the window expires', () => {
      limiter.mint('addr1');
      limiter.mint('addr1');
      limiter.mint('addr1');
      expect(() => limiter.mint('addr1')).toThrow(MintRateLimitError);
      vi.advanceTimersByTime(60_001);
      expect(() => limiter.mint('addr1')).not.toThrow();
      expect(limiter.getMintCount('addr1')).toBe(1);
    });
  });

  describe('per-address tracking', () => {
    it('tracks mints independently per address', () => {
      limiter.mint('addr1');
      limiter.mint('addr1');
      limiter.mint('addr2');
      expect(limiter.getMintCount('addr1')).toBe(2);
      expect(limiter.getMintCount('addr2')).toBe(1);
    });

    it('allows different addresses to mint independently', () => {
      limiter.mint('addr1');
      limiter.mint('addr1');
      limiter.mint('addr1');
      expect(() => limiter.mint('addr2')).not.toThrow();
    });
  });

  describe('getMintCount', () => {
    it('returns 0 for addresses with no mints', () => {
      expect(limiter.getMintCount('unknown')).toBe(0);
    });

    it('only counts mints within the current window', () => {
      limiter.mint('addr1');
      vi.advanceTimersByTime(30_000);
      limiter.mint('addr1');
      expect(limiter.getMintCount('addr1')).toBe(2);
      vi.advanceTimersByTime(31_000);
      expect(limiter.getMintCount('addr1')).toBe(1);
    });
  });

  describe('admin config', () => {
    it('returns current config', () => {
      expect(limiter.getConfig()).toEqual({ maxMints: 3, windowMs: 60_000 });
    });

    it('updates maxMints', () => {
      limiter.updateConfig({ maxMints: 5 }, 'secret');
      expect(limiter.getConfig().maxMints).toBe(5);
    });

    it('updates windowMs', () => {
      limiter.updateConfig({ windowMs: 120_000 }, 'secret');
      expect(limiter.getConfig().windowMs).toBe(120_000);
    });

    it('rejects unauthorized updates', () => {
      expect(() => limiter.updateConfig({ maxMints: 5 }, 'wrong-secret')).toThrow('Unauthorized');
    });

    it('rejects invalid maxMints', () => {
      expect(() => limiter.updateConfig({ maxMints: 0 }, 'secret')).toThrow('maxMints must be at least 1');
    });

    it('rejects invalid windowMs', () => {
      expect(() => limiter.updateConfig({ windowMs: 500 }, 'secret')).toThrow('windowMs must be at least 1000');
    });

    it('applies updated limits immediately', () => {
      limiter.mint('addr1');
      limiter.mint('addr1');
      limiter.mint('addr1');
      expect(() => limiter.mint('addr1')).toThrow(MintRateLimitError);
      limiter.updateConfig({ maxMints: 5 }, 'secret');
      expect(() => limiter.mint('addr1')).not.toThrow();
    });
  });
});

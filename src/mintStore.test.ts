import { describe, it, expect } from 'vitest';
import { MemoryMintStore } from './mintStore';

describe('MemoryMintStore', () => {
  it('enforces the limit atomically inside tryRecord', async () => {
    const store = new MemoryMintStore();
    const now = 1_000_000;
    const cutoff = now - 60_000;

    expect(await store.tryRecord('a', now, cutoff, 2)).toEqual({ allowed: true, count: 1 });
    expect(await store.tryRecord('a', now + 1, cutoff, 2)).toEqual({ allowed: true, count: 2 });
    expect(await store.tryRecord('a', now + 2, cutoff, 2)).toEqual({
      allowed: false,
      oldestRecent: now,
    });
  });

  it('prunes timestamps at or before the cutoff', async () => {
    const store = new MemoryMintStore();
    await store.tryRecord('a', 1000, 0, 5);
    await store.tryRecord('a', 2000, 0, 5);

    expect(await store.getRecent('a', 1500)).toEqual([2000]);
  });
});

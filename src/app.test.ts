import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createServer, type Server } from 'http';
import type { AddressInfo } from 'net';
import type { Express } from 'express';
import { createApp } from './index';
import { MintRateLimiter } from './rateLimiter';
import { MemoryMintStore } from './mintStore';
import type { AppConfig } from './config';

const config: AppConfig = {
  environment: 'testnet',
  port: 0,
  adminSecret: 'test-secret',
  redisUrl: 'redis://127.0.0.1:6379',
  rateLimit: { maxMints: 3, windowMs: 60_000 },
  stellar: {
    network: 'testnet',
    rpcUrl: 'https://soroban-testnet.stellar.org',
    networkPassphrase: 'Test SDF Network ; September 2015',
  },
  contracts: {
    huntyCoreId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
    rewardManagerId: 'CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB',
    nftRewardId: 'CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC',
  },
};

// G + 55 base32 chars — matches the /mint address regex only.
const VALID_ADDRESS = 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF';

async function listen(app: Express): Promise<{ server: Server; baseUrl: string }> {
  const server = createServer(app);
  await new Promise<void>((resolve) => {
    server.listen(0, '127.0.0.1', resolve);
  });
  const { port } = server.address() as AddressInfo;
  return { server, baseUrl: `http://127.0.0.1:${port}` };
}

describe('createApp', () => {
  let limiter: MintRateLimiter;
  let server: Server | undefined;

  beforeEach(() => {
    limiter = new MintRateLimiter(config.rateLimit, config.adminSecret, new MemoryMintStore());
  });

  afterEach(async () => {
    if (server) {
      await new Promise<void>((resolve, reject) => {
        server!.close((err) => (err ? reject(err) : resolve()));
      });
      server = undefined;
    }
  });

  it('can be imported and exercised without Redis or auto-listen side effects', async () => {
    const app = createApp({ config, limiter });
    const listening = await listen(app);
    server = listening.server;

    const res = await fetch(`${listening.baseUrl}/health`);
    expect(res.status).toBe(200);
    const body = (await res.json()) as { status: string; environment: string };
    expect(body.status).toBe('ok');
    expect(body.environment).toBe('testnet');
  });

  it('serves /mint after the limiter is wired (no undefined.mint 500)', async () => {
    const app = createApp({ config, limiter });
    const listening = await listen(app);
    server = listening.server;

    const res = await fetch(`${listening.baseUrl}/mint`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ address: VALID_ADDRESS }),
    });
    expect(res.status).toBe(200);
    const body = (await res.json()) as { minted: boolean; mintsInWindow: number };
    expect(body.minted).toBe(true);
    expect(body.mintsInWindow).toBe(1);
  });

  it('keys the global limiter on the forwarded client IP when trustProxy is set', async () => {
    const app = createApp({ config: { ...config, trustProxy: 1 }, limiter });
    const listening = await listen(app);
    server = listening.server;

    const post = (ip: string) =>
      fetch(`${listening.baseUrl}/mint`, {
        method: 'POST',
        headers: { 'content-type': 'application/json', 'x-forwarded-for': ip },
        body: JSON.stringify({ address: 'invalid' }),
      });

    for (let i = 0; i < 100; i++) {
      expect((await post('203.0.113.1')).status).toBe(400);
    }
    expect((await post('203.0.113.1')).status).toBe(429);
    // A different client behind the same proxy is not affected.
    expect((await post('203.0.113.2')).status).toBe(400);
  });
});

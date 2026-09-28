import express from 'express';
import rateLimit from 'express-rate-limit';
import { MintRateLimiter } from './rateLimiter';
import { MintRateLimitError } from './errors';
import { AppConfig, loadConfig, publicConfig } from './config';
import { createMintStore } from './mintStore';

export interface AppDeps {
  config: AppConfig;
  limiter: MintRateLimiter;
}

/**
 * Build the Express app once dependencies (including the mint limiter) are ready.
 * Kept separate from listen() so tests can import and exercise routes without
 * starting a server or connecting to Redis as a side effect of import.
 */
export function createApp({ config, limiter }: AppDeps): express.Express {
  const app = express();
  // Must be set before any rate limiter so req.ip is the real client address
  // when running behind a reverse proxy / load balancer.
  if (config.trustProxy !== undefined) {
    app.set('trust proxy', config.trustProxy);
  }
  app.use(express.json({ limit: '10kb' }));

  const globalLimiter = rateLimit({
    windowMs: 15 * 60 * 1000,
    max: 100,
    standardHeaders: true,
    legacyHeaders: false,
  });

  app.use('/mint', globalLimiter);

  app.get('/health', (_req, res) => {
    res.json({ status: 'ok', ...publicConfig(config) });
  });

  app.get('/environment', (_req, res) => {
    if (config.environment === 'mainnet') {
      res.status(204).send();
      return;
    }

    res.type('html').send(`<!doctype html>
<html lang="en">
  <head><meta charset="utf-8"><title>Environment</title></head>
  <body style="margin:0;font-family:system-ui,sans-serif;background:#111827;color:#f9fafb;">
    <div style="display:inline-block;margin:16px;padding:8px 12px;border-radius:999px;background:#f59e0b;color:#111827;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;">
      ${config.environment}
    </div>
  </body>
</html>`);
  });

  app.post('/mint', async (req, res) => {
    const { address } = req.body;
    if (!address || typeof address !== 'string' || !/^G[A-Z2-7]{55}$/.test(address)) {
      res.status(400).json({ error: 'invalid address' });
      return;
    }
    try {
      await limiter.mint(address);
      res.json({ minted: true, mintsInWindow: await limiter.getMintCount(address) });
    } catch (err) {
      if (err instanceof MintRateLimitError) {
        res.status(429).json({
          error: err.message,
          cooldownMs: err.cooldownMs,
        });
        return;
      }
      throw err;
    }
  });

  app.get('/mint/count/:address', async (req, res) => {
    const count = await limiter.getMintCount(req.params.address);
    res.json({ address: req.params.address, mintsInWindow: count });
  });

  app.get('/admin/config', (req, res) => {
    const secret = req.headers['x-admin-secret'] as string;
    if (!secret || secret !== config.adminSecret) {
      res.status(403).json({ error: 'Unauthorized' });
      return;
    }
    res.json({ rateLimit: limiter.getConfig(), ...publicConfig(config) });
  });

  app.patch('/admin/config', (req, res) => {
    const secret = req.headers['x-admin-secret'] as string;
    if (!secret || secret !== config.adminSecret) {
      res.status(403).json({ error: 'Unauthorized' });
      return;
    }
    try {
      limiter.updateConfig(req.body, secret);
      res.json(limiter.getConfig());
    } catch (err) {
      res.status(400).json({ error: (err as Error).message });
    }
  });

  app.use((err: Error, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
    console.error(err);
    res.status(500).json({ error: 'Internal server error' });
  });

  return app;
}

export async function bootstrap(
  env: NodeJS.ProcessEnv = process.env,
): Promise<{ app: express.Express; config: AppConfig; limiter: MintRateLimiter }> {
  const config = loadConfig(env);
  const store = await createMintStore(config.redisUrl);
  const limiter = new MintRateLimiter(config.rateLimit, config.adminSecret, store);
  const app = createApp({ config, limiter });
  return { app, config, limiter };
}

export async function start(
  env: NodeJS.ProcessEnv = process.env,
): Promise<{ app: express.Express; config: AppConfig; limiter: MintRateLimiter }> {
  const { app, config, limiter } = await bootstrap(env);
  app.listen(config.port, () => {
    console.log(
      `Mint rate limiter API running on port ${config.port} (${config.environment}) with Redis-backed shared state`,
    );
  });
  return { app, config, limiter };
}

// Only listen when this file is the process entry point (npm start / node dist/index.js).
// Importing for tests must not connect to Redis or bind a port.
if (require.main === module) {
  void start().catch((err) => {
    console.error('Failed to start mint rate limiter API:', err);
    process.exit(1);
  });
}

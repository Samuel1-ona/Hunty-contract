import express from 'express';
import rateLimit from 'express-rate-limit';
import { MintRateLimiter } from './rateLimiter';
import { MintRateLimitError } from './errors';
import { loadConfig, publicConfig } from './config';
import { createMintStore } from './mintStore';

const app = express();
app.use(express.json({ limit: '10kb' }));

const globalLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 100,
  standardHeaders: true,
  legacyHeaders: false,

});

app.use('/mint', globalLimiter);

const config = loadConfig();

let limiter: MintRateLimiter;

async function bootstrap(): Promise<void> {
  const store = await createMintStore(config.redisUrl);
  limiter = new MintRateLimiter(config.rateLimit, config.adminSecret, store);

  app.listen(config.port, () => {
    console.log(
      `Mint rate limiter API running on port ${config.port} (${config.environment}) with Redis-backed shared state`,
    );
  });
}

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

void bootstrap().catch((err) => {
  console.error('Failed to start mint rate limiter API:', err);
  process.exit(1);
});

export { app, config, limiter };
export function getLimiter(): MintRateLimiter {
  return limiter;
}

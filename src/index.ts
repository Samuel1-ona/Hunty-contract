import express from 'express';
import { MintRateLimiter } from './rateLimiter';
import { MintRateLimitError } from './errors';
import { loadConfig, publicConfig } from './config';

const app = express();
app.use(express.json());

const config = loadConfig();
const limiter = new MintRateLimiter(config.rateLimit, config.adminSecret);

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

app.post('/mint', (req, res) => {
  const { address } = req.body;
  if (!address || typeof address !== 'string') {
    res.status(400).json({ error: 'address is required' });
    return;
  }
  try {
    limiter.mint(address);
    res.json({ minted: true, mintsInWindow: limiter.getMintCount(address) });
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

app.get('/mint/count/:address', (req, res) => {
  const count = limiter.getMintCount(req.params.address);
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
  if (!secret) {
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

app.listen(config.port, () => {
  console.log(`Mint rate limiter API running on port ${config.port} (${config.environment})`);
});

export { app, limiter, config };

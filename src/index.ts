import express from 'express';
import { MintRateLimiter } from './rateLimiter';
import { MintRateLimitError } from './errors';

const app = express();
app.use(express.json());

const limiter = new MintRateLimiter(
  { maxMints: 3, windowMs: 60_000 },
  process.env.ADMIN_SECRET ?? 'admin-secret'
);

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
  if (!secret || secret !== (process.env.ADMIN_SECRET ?? 'admin-secret')) {
    res.status(403).json({ error: 'Unauthorized' });
    return;
  }
  res.json(limiter.getConfig());
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

const port = process.env.PORT ?? 3000;
app.listen(port, () => {
  console.log(`Mint rate limiter API running on port ${port}`);
});

export { app, limiter };

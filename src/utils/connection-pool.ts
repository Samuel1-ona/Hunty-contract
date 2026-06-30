export class ConnectionPoolExhaustedError extends Error {
  constructor() {
    super('All connection endpoints are exhausted');
    this.name = 'ConnectionPoolExhaustedError';
  }
}

interface EndpointEntry {
  url: string;
  failedAt: number | null;
}

const RETRY_COOLDOWN_MS = 30_000;

export class ConnectionPool {
  private endpoints: EndpointEntry[] = [];
  private index = 0;

  addEndpoint(url: string): void {
    this.endpoints.push({ url, failedAt: null });
  }

  removeEndpoint(url: string): void {
    const idx = this.endpoints.findIndex(e => e.url === url);
    if (idx === -1) return;
    this.endpoints.splice(idx, 1);
    if (this.index >= this.endpoints.length) {
      this.index = 0;
    }
  }

  markFailed(url: string): void {
    const entry = this.endpoints.find(e => e.url === url);
    if (entry) {
      entry.failedAt = Date.now();
    }
  }

  getHealthyEndpoints(): string[] {
    const now = Date.now();
    return this.endpoints
      .filter(e => e.failedAt === null || (now - e.failedAt) >= RETRY_COOLDOWN_MS)
      .map(e => e.url);
  }

  getNextEndpoint(): string {
    const now = Date.now();
    const n = this.endpoints.length;

    if (n === 0) {
      throw new ConnectionPoolExhaustedError();
    }

    for (let i = 0; i < n; i++) {
      const idx = (this.index + i) % n;
      const entry = this.endpoints[idx];
      const healthy = entry.failedAt === null || (now - entry.failedAt) >= RETRY_COOLDOWN_MS;
      if (healthy) {
        this.index = (idx + 1) % n;
        return entry.url;
      }
    }

    throw new ConnectionPoolExhaustedError();
  }
}

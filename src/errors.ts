export class MintRateLimitError extends Error {
  public readonly cooldownMs: number;

  constructor(cooldownMs: number) {
    super(`Rate limit exceeded. Try again in ${Math.ceil(cooldownMs / 1000)} seconds.`);
    this.name = 'MintRateLimitError';
    this.cooldownMs = cooldownMs;
  }
}

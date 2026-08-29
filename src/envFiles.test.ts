/**
 * Regression tests for the environment-file hygiene fix.
 *
 * Previously `.env.mainnet`, `.env.staging` and `.env.testnet` were tracked in git
 * with `replace-with-...` placeholders. A developer filling one in locally to deploy
 * and running `git add -A` would commit a live mainnet admin secret.
 *
 * The fix:
 *   1. templates renamed to `.env.<environment>.example`
 *   2. `.gitignore` ignores `.env.*` and negates only `.env.*.example`
 *   3. `scripts/with-env.mjs` loads `.env.<environment>` and points at the template
 *      in its error message when the file is absent.
 */
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

const repoRoot = resolve(__dirname, '..');
const environments = ['testnet', 'staging', 'mainnet'] as const;
const withEnvScript = join(repoRoot, 'scripts', 'with-env.mjs');

/** `git check-ignore` exits 0 when the path IS ignored, 1 when it is not. */
function isIgnored(relativePath: string): boolean {
  const result = spawnSync('git', ['check-ignore', '-q', '--no-index', relativePath], {
    cwd: repoRoot,
  });
  return result.status === 0;
}

function trackedFiles(): string[] {
  return execFileSync('git', ['ls-files'], { cwd: repoRoot, encoding: 'utf8' })
    .split('\n')
    .filter(Boolean);
}

describe('environment file layout', () => {
  it('ships a tracked .example template for every environment', () => {
    for (const environment of environments) {
      expect(existsSync(join(repoRoot, `.env.${environment}.example`))).toBe(true);
    }
    expect(existsSync(join(repoRoot, '.env.example'))).toBe(true);
  });

  it('no longer tracks secret-bearing .env.<environment> files', () => {
    const tracked = trackedFiles();
    for (const environment of environments) {
      expect(tracked).not.toContain(`.env.${environment}`);
    }
    // Nothing tracked under `.env.*` unless it is an `.example` template.
    const trackedEnvFiles = tracked.filter(path => /(^|\/)\.env($|\.)/.test(path));
    for (const path of trackedEnvFiles) {
      expect(path.endsWith('.example')).toBe(true);
    }
  });

  it('keeps only placeholder values in tracked templates', () => {
    for (const environment of environments) {
      const contents = readFileSync(join(repoRoot, `.env.${environment}.example`), 'utf8');
      const adminSecret = contents.match(/^ADMIN_SECRET=(.*)$/m)?.[1];
      expect(adminSecret).toBeDefined();
      expect(adminSecret).toMatch(/^replace-with-/);
    }
  });
});

describe('.gitignore rules', () => {
  it('ignores real environment files that would hold live secrets', () => {
    expect(isIgnored('.env')).toBe(true);
    for (const environment of environments) {
      expect(isIgnored(`.env.${environment}`)).toBe(true);
    }
    // Arbitrary future environments and local overrides are covered too.
    expect(isIgnored('.env.local')).toBe(true);
    expect(isIgnored('.env.mainnet.local')).toBe(true);
  });

  it('still tracks the .example templates via the negation rule', () => {
    expect(isIgnored('.env.example')).toBe(false);
    for (const environment of environments) {
      expect(isIgnored(`.env.${environment}.example`)).toBe(false);
    }
  });
});

describe('scripts/with-env.mjs', () => {
  let sandbox: string;

  beforeAll(() => {
    sandbox = mkdtempSync(join(tmpdir(), 'hunty-with-env-'));
  });

  afterAll(() => {
    rmSync(sandbox, { recursive: true, force: true });
  });

  function runWithEnv(args: string[], cwd: string, env: NodeJS.ProcessEnv = {}) {
    return spawnSync(process.execPath, [withEnvScript, ...args], {
      cwd,
      encoding: 'utf8',
      env: { ...process.env, ...env },
    });
  }

  it('rejects unknown environments', () => {
    const result = runWithEnv(['production', process.execPath, '-e', ''], sandbox);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain('Usage: node scripts/with-env.mjs');
  });

  it('fails when .env.<environment> is missing and points at the template', () => {
    writeFileSync(join(sandbox, '.env.mainnet.example'), 'ADMIN_SECRET=replace-with-me\n');
    const result = runWithEnv(['mainnet', process.execPath, '-e', ''], sandbox);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain('Missing environment file: .env.mainnet');
    expect(result.stderr).toContain('cp .env.mainnet.example .env.mainnet');
  });

  it('does not silently fall back to the .example template', () => {
    // The template exists, but the real file does not: the run must fail rather
    // than boot with `replace-with-...` placeholders.
    const result = runWithEnv(
      ['mainnet', process.execPath, '-e', 'console.log(process.env.ADMIN_SECRET)'],
      sandbox
    );
    expect(result.status).toBe(1);
    expect(result.stdout).not.toContain('replace-with-me');
  });

  it('loads the real .env.<environment> file when present', () => {
    writeFileSync(join(sandbox, '.env.testnet'), 'ADMIN_SECRET=real-local-secret\nPORT=4123\n');
    const result = runWithEnv(
      ['testnet', process.execPath, '-e', 'console.log(process.env.ADMIN_SECRET + ":" + process.env.PORT)'],
      sandbox
    );
    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe('real-local-secret:4123');
  });

  it('allows an explicit opt-in to the template for dry runs', () => {
    const result = runWithEnv(
      ['mainnet', process.execPath, '-e', 'console.log(process.env.ADMIN_SECRET)'],
      sandbox,
      { WITH_ENV_ALLOW_EXAMPLE: '1' }
    );
    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe('replace-with-me');
    expect(result.stderr).toContain('WITH_ENV_ALLOW_EXAMPLE=1');
  });
});

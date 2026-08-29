#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { relative, resolve } from 'node:path';

const [, , environment, command, ...args] = process.argv;
const allowedEnvironments = new Set(['testnet', 'staging', 'mainnet']);

if (!allowedEnvironments.has(environment) || !command) {
  console.error('Usage: node scripts/with-env.mjs <testnet|staging|mainnet> <command> [...args]');
  process.exit(1);
}

const cwd = process.cwd();

// Real environment files (`.env.<environment>`) hold live secrets and are
// git-ignored. Only `.env.<environment>.example` templates — which contain
// `replace-with-...` placeholders — are tracked in version control.
const localFileName = `.env.${environment}`;
const exampleFileName = `${localFileName}.example`;
const localFile = resolve(cwd, localFileName);
const exampleFile = resolve(cwd, exampleFileName);

// Opt-in escape hatch for CI/dry runs that only need placeholder values.
// Never enabled by default so a missing secret fails loudly instead of
// silently booting with placeholders.
const allowExample = process.env.WITH_ENV_ALLOW_EXAMPLE === '1';

let envFile = null;
if (existsSync(localFile)) {
  envFile = localFile;
} else if (allowExample && existsSync(exampleFile)) {
  envFile = exampleFile;
  console.warn(
    `[with-env] ${localFileName} not found; using ${exampleFileName} because ` +
      'WITH_ENV_ALLOW_EXAMPLE=1. Placeholder values will fail config validation.'
  );
}

if (!envFile) {
  console.error(`Missing environment file: ${localFileName}`);
  if (existsSync(exampleFile)) {
    console.error(`Create it from the tracked template:\n  cp ${exampleFileName} ${localFileName}`);
    console.error(`Then replace every \`replace-with-...\` placeholder. ${localFileName} is git-ignored.`);
  } else {
    console.error(`Expected template ${exampleFileName} is also missing.`);
  }
  process.exit(1);
}

const parsedEnv = Object.fromEntries(
  readFileSync(envFile, 'utf8')
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(line => line && !line.startsWith('#'))
    .map(line => {
      const separatorIndex = line.indexOf('=');
      if (separatorIndex === -1) {
        return [line, ''];
      }
      const key = line.slice(0, separatorIndex).trim();
      const value = line.slice(separatorIndex + 1).trim().replace(/^['"]|['"]$/g, '');
      return [key, value];
    })
);

console.error(`[with-env] Loaded ${relative(cwd, envFile) || envFile} for ${environment}`);

const child = spawn(command, args, {
  stdio: 'inherit',
  shell: process.platform === 'win32',
  env: {
    ...process.env,
    ...parsedEnv,
  },
});

child.on('exit', code => {
  process.exit(code ?? 1);
});

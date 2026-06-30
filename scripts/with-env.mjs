#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

const [, , environment, command, ...args] = process.argv;
const allowedEnvironments = new Set(['testnet', 'staging', 'mainnet']);

if (!allowedEnvironments.has(environment) || !command) {
  console.error('Usage: node scripts/with-env.mjs <testnet|staging|mainnet> <command> [...args]');
  process.exit(1);
}

const envFile = resolve(process.cwd(), `.env.${environment}`);

if (!existsSync(envFile)) {
  console.error(`Missing environment file: .env.${environment}`);
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

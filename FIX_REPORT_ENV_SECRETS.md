# Fix Report — Tracked `.env.<environment>` files can leak live mainnet secrets

**Repository:** `madisonsc52-del/Hunty-contract`
**Audited commit:** `29f2dcf` (audit baseline) / working HEAD `955d5df`
**Status:** ✅ Resolved — 29/29 tests passing, build clean, no Rust/Cargo impact

---

## STEP 1–2 — Codebase understanding

Hunty is a **Stellar/Soroban treasure-hunt platform**. Two halves live in one repo:

| Half | Contents |
|---|---|
| **Rust / Soroban workspace** | `contracts/hunty-core`, `contracts/reward-manager`, `contracts/nft-reward` — hunts, XLM reward pools, NFT rewards. Built via `stellar contract build`, gated by `cargo fmt` / `clippy -D warnings` / `cargo test --workspace` / tarpaulin. |
| **Node/TypeScript API** | `src/index.ts` (Express), `src/rateLimiter.ts` (per-address mint rate limiter), `src/config.ts` (fail-fast env validation). Tested with Vitest. |

The API is deliberately **multi-environment**: `testnet`, `staging`, `mainnet` must never share
contract IDs, RPC URLs, or admin secrets. `scripts/with-env.mjs` is the loader — it reads
`.env.<environment>`, merges it into `process.env`, and spawns the real command. `package.json`
wires it into `dev:*`, `build:*`, and `start:*`.

`src/config.ts` already fails fast on any value still matching `replace-with-...`, so the
templates were *designed* to be inert placeholders.

---

## STEP 3 — The exact defect, located

At `HEAD` the repo tracked four env files:

```
.env.example            ← intended template
.env.mainnet            ← TRACKED, ADMIN_SECRET=replace-with-mainnet-admin-secret
.env.staging            ← TRACKED, ADMIN_SECRET=replace-with-staging-admin-secret
.env.testnet            ← TRACKED, ADMIN_SECRET=replace-with-testnet-admin-secret
```

And `.gitignore` (line 13) contained only:

```gitignore
.env
```

Three defects compounded into one exploitable path:

1. **`.env` does not match `.env.mainnet`.** Gitignore patterns are literal/glob — no implicit
   prefix matching. Verified: `git check-ignore .env.mainnet` returned non-zero pre-fix.
2. **The three env files were already tracked.** Once a path is in the index, `.gitignore` is
   irrelevant to it — git keeps diffing it forever.
3. **The tracked filename is the *same filename the deploy flow tells you to edit*.**
   `with-env.mjs` resolved `.env.${environment}` and errored `Missing environment file` if absent —
   so the only way to run `npm run start:mainnet` was to write real credentials into a tracked file.

**Failure chain (reproduced verbatim below):** engineer opens `.env.mainnet` → replaces
`replace-with-mainnet-admin-secret` with the live secret → deploys → `git add -A` → the live
mainnet admin secret is staged as a one-line diff against a placeholder, i.e. maximally
greppable in public history.

`DEVELOPMENT.md:707` even said *"Keep `.env.mainnet` out of version control (add it to
`.gitignore`)"* — the guidance existed; the mechanism did not.

---

## STEP 4 — The fix

### 1. Renamed the templates (git-history-preserving `git mv`)

```
.env.mainnet  → .env.mainnet.example
.env.staging  → .env.staging.example
.env.testnet  → .env.testnet.example
```

Filenames now differ from the file an operator edits, so there is no longer a tracked path that
deploy instructions push you to fill in. Each gained a header warning block:

```dotenv
# Template for the mainnet environment. Placeholder values only.
#
# Usage:
#   cp .env.mainnet.example .env.mainnet
# then replace every `replace-with-...` value with the real one.
#
# .env.mainnet is git-ignored; this .example file is the only tracked copy.
# NEVER put a real ADMIN_SECRET or key material in this file.
```

### 2. `.gitignore` — deny-by-default with an `.example` negation

```gitignore
# Environment files: never commit real secrets.
# Only `*.example` templates with placeholder values are tracked.
.env
.env.*
!.env.example
!.env.*.example
```

`.env.*` is a **catch-all**: it covers today's three environments *and* any future
`.env.preprod`, `.env.local`, `.env.mainnet.local`, etc. The two negations re-admit only
placeholder templates. `!.env.example` is required explicitly because `.env.*` matches it and it
does not end in `.example` preceded by a dot-segment.

> Note: the negation works here because no *parent directory* is excluded — git cannot re-include
> a file inside an ignored directory. Verified empirically, not assumed.

### 3. `scripts/with-env.mjs` — template-aware lookup

- Loads `.env.<environment>` as before (unchanged happy path).
- **Does not silently fall back to the `.example` template** — booting mainnet with
  `replace-with-...` values would be a worse failure than a hard stop, and `config.ts` would
  reject them anyway with a less obvious message.
- On a missing file, prints the exact remediation:

  ```
  Missing environment file: .env.mainnet
  Create it from the tracked template:
    cp .env.mainnet.example .env.mainnet
  Then replace every `replace-with-...` placeholder. .env.mainnet is git-ignored.
  ```

- Adds `WITH_ENV_ALLOW_EXAMPLE=1`, an explicit opt-in for CI dry-runs that need the template.
- Logs `[with-env] Loaded .env.mainnet for mainnet` to **stderr** (never stdout — stdout stays
  clean for the wrapped command's output).

### 4. Documentation corrected

- `README.md` — template list now names the `.example` files, adds the `cp` bootstrap block, and
  states that filled-in copies are git-ignored.
- `DEVELOPMENT.md:707` — the stale *"add it to `.gitignore`"* TODO replaced with the now-true
  statement plus a `git check-ignore -v .env.mainnet` verification command.
- `.env.example` — expanded header explaining the copy workflow.

### 5. Regression test — `src/envFiles.test.ts` (new, 10 tests)

Locks the fix so it cannot silently regress:

- Every environment ships a tracked `.example`; **no** `.env.<environment>` appears in `git ls-files`.
- Every tracked `ADMIN_SECRET` still starts with `replace-with-`.
- `git check-ignore` says `.env`, `.env.{testnet,staging,mainnet}`, `.env.local`,
  `.env.mainnet.local` are ignored — and `.env.example` + all `.env.*.example` are **not**.
- `with-env.mjs`: rejects unknown envs; errors with the `cp` hint when the file is missing;
  **does not** leak template values as a fallback; loads a real file correctly; honours
  `WITH_ENV_ALLOW_EXAMPLE=1`.

---

## STEP 5, 7, 8, 10 — Validation

| Check | Command | Result |
|---|---|---|
| Unit + regression tests | `npx vitest run` | ✅ **29/29 passed** (was 19; +10 new) |
| TypeScript build | `npm run build` (`tsc`) | ✅ exit 0 |
| Lint / typecheck | `npm run lint` (`tsc --noEmit`) | ✅ exit 0 |
| Env loader, real file | `npm run build:mainnet` | ✅ `[with-env] Loaded .env.mainnet` |
| Env loader, no file | `npm run build:mainnet` | ✅ exit 1 + `cp` remediation hint |
| CI file-size guard | `scripts/ci/check_file_size.sh` | ✅ no non-allowlisted oversize |
| CI storage-keys doc | `scripts/ci/check_storage_keys_doc.sh` | ✅ 88/88 keys documented |
| Rust workspace impact | `git diff --cached --name-only \| grep '\.(rs\|toml\|lock)$'` | ✅ **zero** matches → `cargo check --workspace`, clippy, fmt, tarpaulin untouched |

### The original failure mode, replayed against the patched tree

```console
$ cp .env.mainnet.example .env.mainnet
$ sed -i 's|^ADMIN_SECRET=.*|ADMIN_SECRET=SUPER_LIVE_MAINNET_SECRET_DO_NOT_LEAK|' .env.mainnet
$ git add -A                       # ← the exact command from the issue
$ git diff --cached --name-status
M   .env.example
R056 .env.mainnet -> .env.mainnet.example
R061 .env.staging -> .env.staging.example
R056 .env.testnet -> .env.testnet.example
M   .gitignore
M   DEVELOPMENT.md
M   README.md
M   scripts/with-env.mjs
A   src/envFiles.test.ts

$ git diff --cached | grep SUPER_LIVE_MAINNET_SECRET_DO_NOT_LEAK
SAFE: secret NOT staged

$ git check-ignore -v .env.mainnet
.gitignore:17:.env.*    .env.mainnet
```

The live secret is invisible to `git add -A`, while the loader still reads it correctly at runtime
(`node scripts/with-env.mjs mainnet ...` → `ADMIN_SECRET=SUPER_LIVE_MAINNET_SECRET_DO_NOT_LEAK`).

---

## STEP 6 — Confidence: **~99%**

The fix is deterministic and empirically verified rather than reasoned-about: gitignore semantics
were confirmed with `git check-ignore` in a clean scratch repo *and* in the real tree, and the
end-to-end leak scenario was replayed with a sentinel secret. `git ls-files` is now clean of
`.env.<environment>` paths, and no Rust/Cargo/CI surface was touched. The remaining ~1% is
operational, not technical — see below.

### Residual item outside this patch's scope

The three placeholder files exist in **prior git history** (commit `29f2dcf` and ancestors).
That is harmless — they only ever contained `replace-with-...` strings, never real secrets. No
history rewrite is needed. However, **if anyone has already pushed a filled-in `.env.mainnet`
before this fix lands, rotate `ADMIN_SECRET` immediately** — `.gitignore` cannot retract a
published secret. A `git log --all -p -- .env.mainnet | grep -v 'replace-with-'` sweep before
merge is a cheap confirmation.

Optional hardening (not included, to keep the diff scoped): a `gitleaks`/`detect-secrets` step in
`.github/workflows/security.yml`, and a `.githooks/pre-commit` clause rejecting staged
`.env.*` paths that don't end in `.example`.

---

## STEP 9 & 11 — Files changed

| File | Change |
|---|---|
| `.env.mainnet` → `.env.mainnet.example` | **Renamed** (`git mv`, R056) + warning header |
| `.env.staging` → `.env.staging.example` | **Renamed** (`git mv`, R061) + warning header |
| `.env.testnet` → `.env.testnet.example` | **Renamed** (`git mv`, R056) + warning header |
| `.gitignore` | **Modified** — `.env.*` catch-all + `!.env.example` / `!.env.*.example` negations |
| `scripts/with-env.mjs` | **Modified** — template-aware lookup, actionable error, `WITH_ENV_ALLOW_EXAMPLE` opt-in, stderr load log |
| `.env.example` | **Modified** — copy-workflow header |
| `README.md` | **Modified** — `.example` template list + `cp` bootstrap + git-ignore note |
| `DEVELOPMENT.md` | **Modified** — stale "add it to .gitignore" TODO resolved; testnet persist step annotated |
| `src/envFiles.test.ts` | **Added** — 10 regression tests |
| `FIX_REPORT_ENV_SECRETS.md` | **Added** — this report |

**Tracked env files after the fix** (`git ls-files | grep env`):

```
.env.example
.env.mainnet.example
.env.staging.example
.env.testnet.example
```

No non-`.example` environment file is tracked, and none can become tracked without an explicit
`git add -f`.

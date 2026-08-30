#!/usr/bin/env bash
# Fails if any `const *_KEY` under contracts/ is missing from docs/STORAGE_KEYS.md.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC="${ROOT}/docs/STORAGE_KEYS.md"
CONTRACTS="${ROOT}/contracts"

if [[ ! -f "$DOC" ]]; then
  echo "ERROR: docs/STORAGE_KEYS.md not found at $DOC"
  exit 1
fi

keys="$(
  # Match `const NAME_KEY` only (not NAME_KEY_BYTES / similar suffixes).
  grep -RhoE 'const[[:space:]]+[A-Z0-9_]+_KEY[[:space:]]*(:|=)' "$CONTRACTS" \
    | sed -E 's/const[[:space:]]+//; s/[[:space:]]*(:|=)$//' \
    | sort -u
)"

if [[ -z "$keys" ]]; then
  echo "ERROR: no *_KEY constants found under contracts/"
  exit 1
fi

missing=0
count=0
while IFS= read -r key; do
  [[ -z "$key" ]] && continue
  count=$((count + 1))
  if ! grep -qF "$key" "$DOC"; then
    echo "MISSING from docs/STORAGE_KEYS.md: $key"
    missing=1
  fi
done <<< "$keys"

if [[ "$missing" -ne 0 ]]; then
  echo "Storage key documentation is incomplete. Update docs/STORAGE_KEYS.md."
  exit 1
fi

echo "OK: all ${count} *_KEY constants are documented in docs/STORAGE_KEYS.md."

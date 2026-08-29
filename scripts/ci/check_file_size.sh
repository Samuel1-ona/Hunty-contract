#!/usr/bin/env bash
# Fails if any tracked file exceeds MAX_BYTES, unless listed in the allowlist.
# Prevents large binary blobs from entering git history via PR merges.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

# ~1 MiB default (issue #701)
MAX_BYTES="${MAX_FILE_BYTES:-1048576}"

# Paths relative to repo root. Existing historical blobs only.
ALLOWLIST_FILE="${FILE_SIZE_ALLOWLIST:-scripts/ci/file_size_allowlist.txt}"

is_allowlisted() {
  local path="$1"
  [[ -f "$ALLOWLIST_FILE" ]] || return 1
  # Exact path match; ignore blank lines and comments
  grep -vE '^\s*(#|$)' "$ALLOWLIST_FILE" | grep -Fxq -- "$path"
}

echo "Checking tracked file sizes (limit=${MAX_BYTES} bytes)..."
fail=0
oversized=0

while IFS= read -r -d '' path; do
  # Skip if missing from working tree (e.g. sparse checkout edge cases)
  [[ -f "$path" ]] || continue
  # Portable size: prefer stat -c (GNU), fall back to stat -f (BSD/macOS)
  if size=$(stat -c%s "$path" 2>/dev/null); then
    :
  elif size=$(stat -f%z "$path" 2>/dev/null); then
    :
  else
    echo "WARN: could not stat $path"
    continue
  fi

  if (( size > MAX_BYTES )); then
    if is_allowlisted "$path"; then
      echo "ALLOWLISTED oversized: $path ($size bytes)"
      continue
    fi
    echo "ERROR: $path is $size bytes (limit ${MAX_BYTES})"
    fail=1
    oversized=$((oversized + 1))
  fi
done < <(git ls-files -z)

if [[ "$fail" -ne 0 ]]; then
  echo "Found ${oversized} tracked file(s) over ${MAX_BYTES} bytes."
  echo "Remove the blob, shrink it, or add an explicit allowlist entry in ${ALLOWLIST_FILE}."
  exit 1
fi

echo "OK: no non-allowlisted tracked files exceed ${MAX_BYTES} bytes."

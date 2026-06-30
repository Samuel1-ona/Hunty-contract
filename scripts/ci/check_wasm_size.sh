#!/usr/bin/env bash
set -euo pipefail

# Usage: SOROBAN_MAX_WASM_BYTES can be set externally (defaults to 200000)
MAX_BYTES=${SOROBAN_MAX_WASM_BYTES:-200000}

echo "Using Soroban max wasm bytes = ${MAX_BYTES}"

fail=0
warn=0

shopt -s nullglob
wasm_files=(target/wasm32-unknown-unknown/release/*.wasm)
if [ ${#wasm_files[@]} -eq 0 ]; then
  echo "No wasm files found under target/wasm32-unknown-unknown/release/. Did the build succeed?"
  exit 2
fi

for f in "${wasm_files[@]}"; do
  size=$(stat -c%s "$f")
  human=$(du -h "$f" | cut -f1)
  pct=$(( size * 100 / MAX_BYTES ))
  echo "Found $f — $human ($size bytes) — $pct% of limit"

  if [ "$size" -gt "$MAX_BYTES" ]; then
    echo "ERROR: $f exceeds Soroban maximum ($size > $MAX_BYTES)"
    fail=1
  elif [ $(( size * 100 )) -ge $(( MAX_BYTES * 80 )) ]; then
    echo "WARNING: $f is over 80% of Soroban maximum ($pct%)"
    warn=1
  fi
done

if [ "$fail" -eq 1 ]; then
  echo "One or more wasm files exceed the Soroban maximum. Failing CI."
  exit 1
fi

if [ "$warn" -eq 1 ]; then
  echo "One or more wasm files are over 80% of the Soroban maximum."
fi

echo "WASM size check completed."

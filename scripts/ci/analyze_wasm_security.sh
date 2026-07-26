#!/usr/bin/env bash
# Analyze compiled Soroban/WASM binaries for security-relevant properties.
# Usage: analyze_wasm_security.sh [wasm_glob]
# Writes a markdown report to SECURITY_WASM_REPORT (default: security-wasm-report.md)
set -euo pipefail

WASM_GLOB="${1:-target/wasm32-unknown-unknown/release/*.wasm}"
REPORT_FILE="${SECURITY_WASM_REPORT:-security-wasm-report.md}"
# Soroban contracts typically only import from the host "v" / "x" env modules.
ALLOWED_IMPORT_MODULES_REGEX="${ALLOWED_IMPORT_MODULES_REGEX:-^(v|x|l|m|d|i|b|a)$}"

EXIT_CODE=0
FINDINGS=0

{
  echo "## WASM Binary Analysis"
  echo ""
} >"$REPORT_FILE"

shopt -s nullglob
# shellcheck disable=SC2206
wasm_files=($WASM_GLOB)

if [ ${#wasm_files[@]} -eq 0 ]; then
  {
    echo "❌ **No WASM files found** matching \`$WASM_GLOB\`."
    echo ""
    echo "Build contracts before running this analysis."
  } >>"$REPORT_FILE"
  echo "No WASM files found under $WASM_GLOB" >&2
  exit 2
fi

has_wasm_tools=0
if command -v wasm-tools >/dev/null 2>&1; then
  has_wasm_tools=1
elif command -v wasm-objdump >/dev/null 2>&1; then
  has_wasm_tools=0
else
  {
    echo "❌ **Neither \`wasm-tools\` nor \`wasm-objdump\` is available.**"
  } >>"$REPORT_FILE"
  echo "Install wasm-tools or wabt (wasm-objdump)" >&2
  exit 2
fi

list_imports() {
  local f="$1"
  if [ "$has_wasm_tools" -eq 1 ]; then
    # wasm-tools objdump lists imports; fall back to print if needed
    if wasm-tools objdump "$f" 2>/dev/null | grep -E 'import\[' >/dev/null 2>&1; then
      wasm-tools objdump "$f" 2>/dev/null | grep -E 'import\[' || true
    else
      wasm-tools print "$f" 2>/dev/null | grep -E '^\s*\(import' || true
    fi
  else
    wasm-objdump -x "$f" 2>/dev/null | grep -E '^\s* - func\[' | grep Import || \
      wasm-objdump -x "$f" 2>/dev/null | grep -i import || true
  fi
}

list_custom_sections() {
  local f="$1"
  if [ "$has_wasm_tools" -eq 1 ]; then
    wasm-tools dump "$f" 2>/dev/null | grep -i 'custom' || true
  else
    wasm-objdump -x "$f" 2>/dev/null | grep -i 'Custom' || true
  fi
}

has_name_section() {
  local f="$1"
  if [ "$has_wasm_tools" -eq 1 ]; then
    wasm-tools dump "$f" 2>/dev/null | grep -qi 'custom.*"name"\|name section' && return 0
    wasm-tools print "$f" 2>/dev/null | grep -qi '(; name' && return 0
    return 1
  else
    wasm-objdump -x "$f" 2>/dev/null | grep -qi 'name section' && return 0
    return 1
  fi
}

for f in "${wasm_files[@]}"; do
  # Skip deps / build scripts artifacts if any slip through
  base="$(basename "$f")"
  case "$base" in
    *.d | *.rlib) continue ;;
  esac

  size=$(stat -c%s "$f" 2>/dev/null || wc -c <"$f")
  human=$(du -h "$f" | cut -f1)

  {
    echo "### \`$base\`"
    echo ""
    echo "| Property | Value |"
    echo "|---|---|"
    echo "| Size | ${human} (${size} bytes) |"
  } >>"$REPORT_FILE"

  # --- Import surface ---
  imports_raw="$(list_imports "$f" || true)"
  import_count=0
  if [ -n "$imports_raw" ]; then
    import_count=$(printf '%s\n' "$imports_raw" | grep -c . || true)
  fi
  echo "| Imports | ${import_count} |" >>"$REPORT_FILE"
  echo "" >>"$REPORT_FILE"

  if [ "$import_count" -gt 0 ]; then
    {
      echo "<details><summary>Import surface</summary>"
      echo ""
      echo '```'
      printf '%s\n' "$imports_raw"
      echo '```'
      echo "</details>"
      echo ""
    } >>"$REPORT_FILE"
  fi

  # Flag non-Soroban host import modules when detectable from (import "mod" ...)
  suspicious=0
  while IFS= read -r line; do
    mod=""
    if [[ "$line" =~ \(import\ \"([^\"]+)\" ]]; then
      mod="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ import.*\"([^\"]+)\" ]]; then
      mod="${BASH_REMATCH[1]}"
    fi
    if [ -n "$mod" ] && ! [[ "$mod" =~ $ALLOWED_IMPORT_MODULES_REGEX ]]; then
      echo "⚠️ Unexpected import module \`$mod\` in \`$base\`" >>"$REPORT_FILE"
      suspicious=1
      FINDINGS=$((FINDINGS + 1))
      EXIT_CODE=1
    fi
  done <<<"$imports_raw"

  if [ "$suspicious" -eq 0 ]; then
    echo "✅ Import modules look consistent with Soroban host ABI." >>"$REPORT_FILE"
  fi
  echo "" >>"$REPORT_FILE"

  # --- Debug / name section ---
  if has_name_section "$f"; then
    echo "⚠️ **Name/debug section present** — prefer stripped release artifacts (\`strip = \"symbols\"\`)." >>"$REPORT_FILE"
    FINDINGS=$((FINDINGS + 1))
    # Reported for review; Soroban metadata tooling may retain names — do not hard-fail.
  else
    echo "✅ No name/debug section detected (symbols stripped)." >>"$REPORT_FILE"
  fi
  echo "" >>"$REPORT_FILE"

  # --- Custom sections ---
  custom="$(list_custom_sections "$f" || true)"
  {
    echo "<details><summary>Custom sections</summary>"
    echo ""
    echo '```'
    if [ -n "$custom" ]; then
      printf '%s\n' "$custom"
    else
      echo "(none reported)"
    fi
    echo '```'
    echo "</details>"
    echo ""
  } >>"$REPORT_FILE"

  # Flag unknown custom section names (beyond common/producers/target_features/contractspecv0)
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    lower="$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')"
    case "$lower" in
      *name*|*producers*|*target_features*|*contractspecv0*|*contractmetav0*|*contractenvmetav0*|*\"a\"*)
        ;;
      *)
        if echo "$lower" | grep -qi 'custom'; then
          echo "⚠️ Unexpected custom section metadata in \`$base\`: \`$line\`" >>"$REPORT_FILE"
          FINDINGS=$((FINDINGS + 1))
          # Informational for Soroban metadata variance — do not fail hard
        fi
        ;;
    esac
  done <<<"$custom"

  # Optional: cargo audit bin when available (best-effort for WASM)
  if command -v cargo >/dev/null 2>&1 && cargo audit -V >/dev/null 2>&1; then
    {
      echo "<details><summary>cargo audit bin (best-effort)</summary>"
      echo ""
      echo '```'
    } >>"$REPORT_FILE"
    if ! cargo audit bin "$f" >>"$REPORT_FILE" 2>&1; then
      echo "(cargo audit bin could not fully analyze this WASM artifact — lockfile audit remains authoritative)" >>"$REPORT_FILE"
      FINDINGS=$((FINDINGS + 1))
      # Do not fail the gate solely on incomplete binary metadata
    fi
    {
      echo '```'
      echo "</details>"
      echo ""
    } >>"$REPORT_FILE"
  fi
done

{
  echo "---"
  echo ""
  if [ "$FINDINGS" -eq 0 ]; then
    echo "**Summary:** No WASM security findings."
  else
    echo "**Summary:** ${FINDINGS} finding(s) reported."
  fi
} >>"$REPORT_FILE"

echo "WASM security analysis wrote $REPORT_FILE (findings=$FINDINGS)" >&2
exit "$EXIT_CODE"

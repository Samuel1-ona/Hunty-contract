#!/usr/bin/env bash
# Install the prebuilt Stellar CLI used by the contract build steps.
#
# soroban-sdk v28 requires contracts to be built with the Stellar CLI build
# system (`stellar contract build`), which performs spec shaking and writes the
# contract spec correctly. Installing the prebuilt release binary is much faster
# than `cargo install` on CI runners.
#
# Usage:
#   STELLAR_CLI_VERSION=28.1.0 scripts/ci/install_stellar_cli.sh
set -euo pipefail

VERSION="${STELLAR_CLI_VERSION:-28.1.0}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)   TRIPLE="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64)  TRIPLE="aarch64-unknown-linux-gnu" ;;
  Darwin-x86_64)  TRIPLE="x86_64-apple-darwin" ;;
  Darwin-arm64)   TRIPLE="aarch64-apple-darwin" ;;
  *)
    echo "Unsupported platform: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

URL="https://github.com/stellar/stellar-cli/releases/download/v${VERSION}/stellar-cli-${VERSION}-${TRIPLE}.tar.gz"

echo "Installing stellar-cli ${VERSION} (${TRIPLE})..."
tmp_dir="$(mktemp -d)"
curl -fsSL "${URL}" | tar xz -C "${tmp_dir}" stellar
if [ -w /usr/local/bin ]; then
  install -m 0755 "${tmp_dir}/stellar" /usr/local/bin/stellar
else
  sudo install -m 0755 "${tmp_dir}/stellar" /usr/local/bin/stellar
fi
rm -rf "${tmp_dir}"

stellar --version

#!/usr/bin/env bash
# Builds the web app, embeds it in the Rust binary and installs `trunade` into ~/.cargo/bin.
set -euo pipefail
cd "$(dirname "$0")"
pnpm --dir web install --frozen-lockfile
pnpm --dir web generate
cargo install --path server --locked
echo "Installed: $(command -v trunade || echo ~/.cargo/bin/trunade)"

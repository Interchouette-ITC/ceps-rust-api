#!/usr/bin/env bash
# Fetch demo tip CEP contract WASMs (same pack as ceps-rust-ts-client releases).
# See: https://github.com/Interchouette-ITC/ceps-rust-ts-client/blob/dev/docs/releases.md
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${CEPS_WASM_ROOT:-$ROOT/tests/wasm}"
TAG="${CEPS_CONTRACTS_TAG:-dev-preview}"
LABEL="${TAG#v}"
URL="https://github.com/Interchouette-ITC/ceps-rust-ts-client/releases/download/${TAG}/ceps-contracts-${LABEL}.tgz"

mkdir -p "$OUT"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "fetch-ceps-contracts: ${URL} -> ${OUT}"
curl -fsSL -o "$TMP/ceps-contracts.tgz" "$URL"
tar -xzf "$TMP/ceps-contracts.tgz" -C "$OUT"
test -f "$OUT/cep18/cep18.wasm"
echo "fetch-ceps-contracts: ok ($(find "$OUT" -name '*.wasm' | wc -l) wasm files)"

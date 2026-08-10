#!/usr/bin/env bash
# Ops helper only: print LOCAL_KEYS_JSON for the NCTL faucet (already funded).
# Not an e2e runner. Use from CI/local before `cargo test --test live_local`.
set -euo pipefail

NCTL_CONTAINER="${NCTL_CONTAINER:-casper-nctl-2-docker-dev}"
FAUCET_HEX="/app/casper-nctl/assets/net-1/faucet/public_key_hex"
FAUCET_PEM="/app/casper-nctl/assets/net-1/faucet/secret_key.pem"

if ! docker inspect -f '{{.State.Running}}' "$NCTL_CONTAINER" 2>/dev/null | grep -qx true; then
  echo "nctl container not running: $NCTL_CONTAINER" >&2
  exit 1
fi

pk="$(docker exec "$NCTL_CONTAINER" cat "$FAUCET_HEX" | tr -d '\r\n')"
pem="$(docker exec "$NCTL_CONTAINER" cat "$FAUCET_PEM")"
jq -nc --arg pk "$pk" --arg pem "$pem" '{($pk): $pem}'

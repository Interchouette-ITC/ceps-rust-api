#!/usr/bin/env bash
# Pre-test ops only: fund KMS (or other) public keys from the NCTL faucet.
# Not product code. Does not use ceps-client / rustSDK.
#
# NCTL's `nctl-transfer-native` only funds user-N after `source activate`.
# Arbitrary targets (KMS createKey pubkeys) use casper-client transfer,
# the same client NCTL wraps internally.
#
# For kms-secp256k1-api with BLOCKCHAIN_MODE=casper, pass createKey **address**
# (Casper PublicKey hex, 68 chars, starts with 0202/0203). Do not pass the
# bare SEC1 `public_key` field (66 chars).
#
# Default: docker exec into a running NCTL container (faucet assets already mounted).
# Fallback: host casper-client + NCTL_ASSETS on the host.
#
# Usage:
#   scripts/fund-kms-from-nctl.sh <public_key_hex> [more_keys...]
# Env:
#   NCTL_CONTAINER   default: casper-nctl-2-docker-dev
#   CEPS_RPC_URL     host RPC (host mode only); default http://127.0.0.1:11101
#   CEPS_CHAIN_NAME  default: casper-net-1
#   AMOUNT           motes; default 1000000000000000 (1_000_000 CSPR)
#   PAYMENT_AMOUNT   motes; default 100000000
#   NCTL_ASSETS      host assets root (host mode); default ../casper-nctl-2-docker/assets
#   CEPS_FAUCET_PEM  host faucet secret path override (host mode only)
#   FUND_MODE        docker | host | auto (default auto)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NCTL_CONTAINER="${NCTL_CONTAINER:-casper-nctl-2-docker-dev}"
CHAIN_NAME="${CEPS_CHAIN_NAME:-casper-net-1}"
AMOUNT="${AMOUNT:-1000000000000000}"
PAYMENT_AMOUNT="${PAYMENT_AMOUNT:-100000000}"
FUND_MODE="${FUND_MODE:-auto}"

IN_CONTAINER_CLIENT="/app/casper-nctl/assets/net-1/bin/casper-client"
IN_CONTAINER_FAUCET="/app/casper-nctl/assets/net-1/faucet/secret_key.pem"
IN_CONTAINER_RPC="http://127.0.0.1:11101"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <public_key_hex> [more_keys...]" >&2
  exit 2
fi

docker_ready() {
  command -v docker >/dev/null 2>&1 \
    && docker inspect -f '{{.State.Running}}' "$NCTL_CONTAINER" 2>/dev/null | grep -qx true
}

resolve_mode() {
  case "$FUND_MODE" in
    docker | host) echo "$FUND_MODE" ;;
    auto)
      if docker_ready; then
        echo docker
      else
        echo host
      fi
      ;;
    *)
      echo "FUND_MODE must be docker|host|auto" >&2
      exit 2
      ;;
  esac
}

fund_one_docker() {
  local pk="$1"
  local tid="$2"
  docker exec "$NCTL_CONTAINER" "$IN_CONTAINER_CLIENT" transfer \
    --node-address "$IN_CONTAINER_RPC" \
    --chain-name "$CHAIN_NAME" \
    --secret-key "$IN_CONTAINER_FAUCET" \
    --payment-amount "$PAYMENT_AMOUNT" \
    --amount "$AMOUNT" \
    --target-account "$pk" \
    --transfer-id "$tid"
}

fund_one_host() {
  local pk="$1"
  local tid="$2"
  local assets="${NCTL_ASSETS:-$ROOT/../casper-nctl-2-docker/assets}"
  local faucet="${CEPS_FAUCET_PEM:-$assets/faucet/secret_key.pem}"
  local rpc="${CEPS_RPC_URL:-http://127.0.0.1:11101}"
  local client="${CASPER_CLIENT:-casper-client}"

  if [[ ! -f "$faucet" ]]; then
    echo "faucet secret not found (set CEPS_FAUCET_PEM or NCTL_ASSETS)" >&2
    exit 1
  fi
  if ! command -v "$client" >/dev/null 2>&1; then
    echo "casper-client not on PATH (set CASPER_CLIENT)" >&2
    exit 1
  fi

  "$client" transfer \
    --node-address "$rpc" \
    --chain-name "$CHAIN_NAME" \
    --secret-key "$faucet" \
    --payment-amount "$PAYMENT_AMOUNT" \
    --amount "$AMOUNT" \
    --target-account "$pk" \
    --transfer-id "$tid"
}

MODE="$(resolve_mode)"
if [[ "$MODE" == docker ]]; then
  echo "fund mode=docker container=$NCTL_CONTAINER amount=$AMOUNT" >&2
else
  echo "fund mode=host amount=$AMOUNT" >&2
fi

i=1
for pk in "$@"; do
  echo "funding $pk ..." >&2
  if [[ "$MODE" == docker ]]; then
    fund_one_docker "$pk" "$i"
  else
    fund_one_host "$pk" "$i"
  fi
  i=$((i + 1))
done

echo "done ($((i - 1)) transfer(s) submitted; wait for block inclusion before CEP put)" >&2

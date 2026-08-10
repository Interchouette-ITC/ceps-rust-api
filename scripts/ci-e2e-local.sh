#!/usr/bin/env bash
# CI / local: NCTL up → LOCAL_KEYS_JSON from faucet → API hello + CEP-18 make-only.
# Real node, no wiremock. Requires: docker, curl, jq, running NCTL container.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NCTL_CONTAINER="${NCTL_CONTAINER:-casper-nctl-2-docker-dev}"
NCTL_IMAGE="${NCTL_IMAGE:-interchouette/casper-nctl-2-docker:dev}"
API_BIN="${API_BIN:-$ROOT/target/release/ceps-rust-api}"
RPC_URL="${CEPS_RPC_URL:-http://127.0.0.1:11101}"
API_PORT="${APP_PORT:-8080}"
FEATURES="${FEATURES:-ceps-all,swagger-ui,tx-return,sign-local,sign-kms,chain-put}"

ensure_nctl() {
  if docker inspect -f '{{.State.Running}}' "$NCTL_CONTAINER" 2>/dev/null | grep -qx true; then
    echo "nctl: using $NCTL_CONTAINER" >&2
    return
  fi
  echo "nctl: starting $NCTL_IMAGE as $NCTL_CONTAINER" >&2
  docker rm -f "$NCTL_CONTAINER" >/dev/null 2>&1 || true
  docker run -d --name "$NCTL_CONTAINER" \
    -p 11101-11105:11101-11105 \
    -p 18101-18105:18101-18105 \
    -p 28101-28105:28101-28105 \
    "$NCTL_IMAGE"
}

wait_rpc() {
  local i
  for i in $(seq 1 90); do
    if curl -sS -m 2 -X POST "$RPC_URL/rpc" \
      -H 'content-type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"info_get_status","params":[]}' \
      | grep -q api_version; then
      echo "rpc: up" >&2
      return 0
    fi
    sleep 5
  done
  echo "rpc: timeout waiting for $RPC_URL" >&2
  exit 1
}

build_local_keys_json() {
  local pk pem
  pk="$(docker exec "$NCTL_CONTAINER" cat /app/casper-nctl/assets/net-1/faucet/public_key_hex | tr -d '\r\n')"
  pem="$(docker exec "$NCTL_CONTAINER" cat /app/casper-nctl/assets/net-1/faucet/secret_key.pem)"
  jq -nc --arg pk "$pk" --arg pem "$pem" '{($pk): $pem}'
}

main() {
  ensure_nctl
  wait_rpc

  if [[ ! -x "$API_BIN" ]]; then
    echo "building release binary..." >&2
    (cd "$ROOT" && env -u CARGO_TARGET_DIR cargo build --release -p ceps-rust-api --features "$FEATURES")
  fi

  export LOCAL_KEYS_JSON
  LOCAL_KEYS_JSON="$(build_local_keys_json)"
  export SIGN_BACKEND=local
  export CEPS_RPC_URL="$RPC_URL"
  export CEPS_SSE_URL="${CEPS_SSE_URL:-http://127.0.0.1:18101/events}"
  export CEPS_CHAIN_NAME="${CEPS_CHAIN_NAME:-casper-net-1}"
  export APP_ADDR=127.0.0.1
  export APP_PORT="$API_PORT"

  "$API_BIN" &
  local api_pid=$!
  trap 'kill $api_pid 2>/dev/null || true' EXIT

  local i
  for i in $(seq 1 30); do
    if curl -sS -m 1 "http://127.0.0.1:${API_PORT}/health" | grep -q healthy; then
      break
    fi
    sleep 1
  done

  local hello
  hello="$(curl -sS "http://127.0.0.1:${API_PORT}/")"
  echo "$hello" | jq -e '.sign_backend == "local"' >/dev/null

  local pk
  pk="$(echo "$LOCAL_KEYS_JSON" | jq -r 'keys[0]')"
  curl -sS -X POST "http://127.0.0.1:${API_PORT}/v1/cep18/install" \
    -H 'content-type: application/json' \
    -d "$(jq -nc --arg pk "$pk" '{
      submit: "return",
      signer: {public_key: $pk},
      payment_amount: "500000000000",
      name: "CiTok",
      symbol: "CIT",
      decimals: 9,
      total_supply: "1000000000000",
      wasm: "cep18"
    }')" | jq -e '.transaction != null or .transaction_hash != null or .code == null' >/dev/null

  echo "e2e-local: ok" >&2
}

main "$@"

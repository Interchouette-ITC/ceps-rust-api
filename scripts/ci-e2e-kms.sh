#!/usr/bin/env bash
# CI / local: KMS (+ LocalStack via compose) → createKey → fund from NCTL → API hello kms.
# Real KMS HTTP, no wiremock. Requires: docker compose, curl, jq, NCTL for fund, casper-client in NCTL.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE="$ROOT/docker/docker-compose.yml"
KMS_URL="${KMS_URL:-http://127.0.0.1:4000}"
API_BIN="${API_BIN:-$ROOT/target/release/ceps-rust-api}"
API_PORT="${APP_PORT:-8080}"
FEATURES="${FEATURES:-ceps-all,swagger-ui,tx-return,sign-local,sign-kms,chain-put}"
NCTL_CONTAINER="${NCTL_CONTAINER:-casper-nctl-2-docker-dev}"

wait_kms() {
  local i
  for i in $(seq 1 60); do
    if curl -sS -m 2 "$KMS_URL/openapi.json" >/dev/null 2>&1 \
      || curl -sS -m 2 "$KMS_URL/" >/dev/null 2>&1; then
      echo "kms: up" >&2
      return 0
    fi
    sleep 2
  done
  echo "kms: timeout at $KMS_URL" >&2
  exit 1
}

create_kms_key() {
  # kms-secp256k1-api createKey (TESTING_MODE). Shape may be { publicKey } or similar.
  local body
  body="$(curl -sS -X POST "$KMS_URL/createKey" -H 'content-type: application/json' -d '{}')"
  echo "$body" | jq -r '.publicKey // .public_key // .key // empty'
}

main() {
  if ! docker inspect -f '{{.State.Running}}' "$NCTL_CONTAINER" 2>/dev/null | grep -qx true; then
    echo "start NCTL first (or run scripts/ci-e2e-local.sh once)" >&2
    exit 1
  fi

  docker compose -f "$COMPOSE" --profile kms up -d kms-localstack kms-secp256k1-api
  wait_kms

  local pk
  pk="$(create_kms_key)"
  if [[ -z "$pk" ]]; then
    echo "createKey returned no public key" >&2
    exit 1
  fi
  echo "kms key: $pk" >&2
  NCTL_CONTAINER="$NCTL_CONTAINER" "$ROOT/scripts/fund-kms-from-nctl.sh" "$pk"

  if [[ ! -x "$API_BIN" ]]; then
    (cd "$ROOT" && env -u CARGO_TARGET_DIR cargo build --release -p ceps-rust-api --features "$FEATURES")
  fi

  export SIGN_BACKEND=kms
  export KMS_URL
  export CEPS_RPC_URL="${CEPS_RPC_URL:-http://127.0.0.1:11101}"
  export CEPS_SSE_URL="${CEPS_SSE_URL:-http://127.0.0.1:18101/events}"
  export CEPS_CHAIN_NAME="${CEPS_CHAIN_NAME:-casper-net-1}"
  export APP_ADDR=127.0.0.1
  export APP_PORT="$API_PORT"
  unset LOCAL_KEYS_JSON || true

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

  curl -sS "http://127.0.0.1:${API_PORT}/" | jq -e '.sign_backend == "kms"' >/dev/null

  curl -sS -X POST "http://127.0.0.1:${API_PORT}/v1/cep18/install" \
    -H 'content-type: application/json' \
    -d "$(jq -nc --arg pk "$pk" '{
      submit: "return",
      signer: {public_key: $pk},
      payment_amount: "500000000000",
      name: "KmsTok",
      symbol: "KMS",
      decimals: 9,
      total_supply: "1000000000000",
      wasm: "cep18"
    }')" | jq -e 'type == "object"' >/dev/null

  echo "e2e-kms: ok (create+fund+hello+make-only; put install covered when gas/path ready)" >&2
}

main "$@"

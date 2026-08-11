# ceps-rust-api

HTTP API for Casper **CEP-18**, **CEP-78**, **CEP-85**, and **CEP-95**, built with [Actix Web](https://actix.rs/) and [utoipa](https://github.com/juhaku/utoipa).

It sits on [`ceps-rust-ts-client`](https://github.com/Interchouette-ITC/ceps-rust-ts-client) (`CEPClient`) for CEP args, transaction make, wait, and put of signed JSON. Optional put signing uses an in-process PEM keyring or the [kms-secp256k1-api](https://github.com/Interchouette-ITC/kms-secp256k1-api) HTTP peer. Request bodies never carry PEM material.

## Deployment model (read this first)

This API is built for **private-network microservice** use (VPC / cluster / closed LAN). It is **not** an internet-facing wallet API.

**Recommended production setup: private network + `SIGN_BACKEND=kms`.** Keep the listener off the public internet. Configure the [kms-secp256k1-api](https://github.com/Interchouette-ITC/kms-secp256k1-api) peer separately ([OVERVIEW](https://github.com/Interchouette-ITC/kms-secp256k1-api/blob/dev/docs/OVERVIEW.md), [Tutorial](https://github.com/Interchouette-ITC/kms-secp256k1-api/blob/dev/docs/Tutorial.md)); this API only needs `KMS_URL`.

| Exposure           | Role                                                                                                                                                                 |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Private + KMS**  | **Recommended.** Trusted microservice callers; this API asks KMS to sign; create/list keys on KMS.                                                                   |
| **Private + PEMs** | Supported fallback: `SIGN_BACKEND=local-production` + `LOCAL_KEYS_JSON_PRODUCTION` when you cannot run KMS. Same network isolation requirement.                      |
| **Public / demo**  | Only **`SIGN_BACKEND=none`** (default): `submit=return` builds unsigned Transaction JSON; sign elsewhere. Do **not** load PEMs or point at KMS on a public listener. |

**Shared warning for every signing backend** (`local`, `local-production`, `kms`): anyone who can reach this HTTP API and knows a **loaded** public key can ask the backend to sign a put. The public key is not a secret; **private network is mandatory** for signing modes.

## Quick start

```bash
cp .env.example .env
make build
make verify
make run
# Default: SIGN_BACKEND unset (none). No PEMs. Pass signer.public_key; submit=return only.
#
# Recommended production (private network + KMS):
#   SIGN_BACKEND=kms KMS_URL=http://127.0.0.1:4000 make run
#
# Private-network fallback without KMS:
#   SIGN_BACKEND=local-production LOCAL_KEYS_JSON_PRODUCTION='…' make run
#
# Lab put signing (NCTL users, private/dev only):
#   make export-local-keys
#   make run-local
```

`make run` defaults `RUST_LOG=info` so you see listen / version / docs lines on stdout.

| URL                                            | Purpose                                        |
| ---------------------------------------------- | ---------------------------------------------- |
| `http://127.0.0.1:8080/`                       | Hello (`name`, `version`, features, node URLs) |
| `http://127.0.0.1:8080/health`                 | Liveness                                       |
| `http://127.0.0.1:8080/docs/`                  | Swagger UI                                     |
| `http://127.0.0.1:8080/docs/ceps-openapi.json` | OpenAPI JSON                                   |

Default node settings match local NCTL: RPC `http://127.0.0.1:11101`, SSE `http://127.0.0.1:18101/events`, chain `casper-net-1`.

## MCP

Sidecar MCP for agents (Make/Docker lifecycle + HTTP wrappers for every API route). Tools use the `ceps_api_*` prefix. Details: [docs/mcp.md](docs/mcp.md), [mcp/README.md](mcp/README.md).

```bash
docker pull interchouette/ceps-rust-api-mcp:1.0.0
# or tip:
docker pull interchouette/ceps-rust-api-mcp:dev
# or org GHCR:
docker pull ghcr.io/interchouette-itc/ceps-rust-api-mcp:dev
make mcp-http   # Streamable HTTP on :4790 → http://127.0.0.1:4790/mcp
```

```bash
env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH make build
make verify-slices   # same feature slices as CI
```

## Stack

| Piece                               | Role                                                                |
| ----------------------------------- | ------------------------------------------------------------------- |
| This API                            | CEP HTTP routes, OpenAPI, put pipeline                              |
| `ceps-rust-ts-client` (`CEPClient`) | CEP entrypoints, make → JSON, `put_transaction`, `wait_transaction` |
| `kms-secp256k1-api`                 | Optional HTTP signer (create/list on KMS itself)                    |

**Separation:** **Recommended:** private network + KMS (KMS owns secrets, key create/list, and signatures; this API only calls `signTransaction`). Fallback without KMS: `local-production` holds PEMs in-process on the same private network. This API never exposes HTTP key create, list, or fund.

## Configuration

| Env                          | Default / notes                                                                                                                                      |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `APP_ADDR` / `APP_PORT`      | `0.0.0.0` / `8080`                                                                                                                                   |
| `CEPS_RPC_URL`               | `http://127.0.0.1:11101`                                                                                                                             |
| `CEPS_SSE_URL`               | `http://127.0.0.1:18101/events`                                                                                                                      |
| `CEPS_CHAIN_NAME`            | `casper-net-1`                                                                                                                                       |
| `SIGN_BACKEND`               | **Recommend `kms` on a private network.** Also: `none` (default) \| `local` \| `local-production`                                                    |
| `LOCAL_KEYS_JSON`            | **Tests / NCTL only.** Used when `SIGN_BACKEND=local`. JSON map or `{keys:[…]}`; PEM text **or** path.                                               |
| `LOCAL_KEYS_JSON_PRODUCTION` | **Private network fallback** when not using KMS. Same JSON as lab: PEM text **or** path. Prefer secret mounts (path), not inline PEMs in env.        |
| `KMS_URL`                    | KMS peer when `SIGN_BACKEND=kms`. Peer env/credentials: [kms docs](https://github.com/Interchouette-ITC/kms-secp256k1-api/blob/dev/docs/Tutorial.md) |
| `DOTENV_DISABLE`             | Set to `1` / `true` / `yes` to skip loading `.env` (CI / containers)                                                                                 |
| `CEPS_WASM_ROOT`             | Contract `.wasm` dir (default in-tree `tests/wasm/`)                                                                                                 |
| `RUST_LOG`                   | tracing filter (default `info`; include `actix_web=info` for request access logs)                                                                    |

See [`.env.example`](.env.example).

## Cargo features

Package **default** is a full build. Slim builds use `--no-default-features` plus the scopes you need.

| Feature                        | Effect                                                             |
| ------------------------------ | ------------------------------------------------------------------ |
| `cep18` … `cep95` / `ceps-all` | CEP route scopes                                                   |
| `all`                          | Alias for the package default                                      |
| `swagger-ui`                   | `/docs`                                                            |
| `tx-return`                    | Allow `submit=return` (Transaction JSON without put)               |
| `sign-local`                   | In-process keyring (`local` and `local-production`)                |
| `sign-kms`                     | Internal KMS `signTransaction` client for puts                     |
| `chain-put`                    | `POST /v1/chain/put-transaction` (already-signed Transaction JSON) |

Hello `/` reports `name`, `version`, node URLs, compiled CEP features, and docs path. It does **not** expose signing backend or keyring/KMS custody state.

## HTTP surface

Platform: health, hello. Optional: `POST /v1/chain/put-transaction` (`chain-put`).

CEP mutates share an envelope: `submit` (`put` \| `return`), `wait` (`accepted` \| `processed`), `signer.public_key`, `payment_amount`.

`signer.public_key` is always the **account / initiator** (Casper public-key hex). You pass the public keys your services operate.

| Area   | Examples                                                                           |
| ------ | ---------------------------------------------------------------------------------- |
| CEP-18 | `/v1/cep18/install`, `transfer`, `mint`, `…/{hash}/balance-of/{owner}`             |
| CEP-78 | `/v1/cep78/install`, `mint`, `transfer`, `…/owner-of/{token}`                      |
| CEP-85 | `/v1/cep85/install`, `mint`, `transfer`, `…/balance-of/{owner}/{id}`               |
| CEP-95 | `/v1/cep95/install`, `transfer-from`, `bind-odra-install`, `…/owner-of/{token_id}` |

OpenAPI lists the paths compiled into this binary. Full route lists are easiest in `/docs/`.

### Signing backends

**Prefer private network + `kms` in production.** Other modes are lab, demo, or fallback.

| Mode                 | Network                               | Secrets in this process                         | Put?                                            |
| -------------------- | ------------------------------------- | ----------------------------------------------- | ----------------------------------------------- |
| **kms**              | **Private only (recommended)**        | None here; KMS holds keys                       | Yes                                             |
| **local-production** | **Private only** (fallback if no KMS) | `LOCAL_KEYS_JSON_PRODUCTION` (PEM text or path) | Yes                                             |
| **local**            | Lab / tests                           | `LOCAL_KEYS_JSON` (PEM text or path)            | Yes                                             |
| **none** (default)   | Public demo OK                        | None. `signer.public_key` = initiator only.     | No (use `return` + external sign + `chain-put`) |

Keyring JSON shapes (same for lab and production env vars):

```json
{ "01ab…": "-----BEGIN PRIVATE KEY-----\n…\n-----END PRIVATE KEY-----" }
```

```json
{ "01ab…": "/run/secrets/operator.pem" }
```

```json
{
  "keys": [
    { "public_key": "01ab…", "secret_key_pem": "-----BEGIN…" },
    { "public_key": "02cd…", "secret_key_path": "/run/secrets/other.pem" }
  ]
}
```

Inline PEM vs path: if the value contains `-----BEGIN` … `PRIVATE KEY`, it is treated as PEM text; otherwise it must be an existing file path.

#### How signing modes are tested

| Mode                 | Keys                                                          | Fund                                                        | Then                            |
| -------------------- | ------------------------------------------------------------- | ----------------------------------------------------------- | ------------------------------- |
| **none**             | Caller’s public-key hex as `signer`                           | External                                                    | `submit=return` unit paths      |
| **local**            | NCTL users → `LOCAL_KEYS_JSON` (`make export-local-keys`)     | Already funded by NCTL                                      | `make run-local` / `live_local` |
| **local-production** | Same shape as lab; use production secrets / mounted PEM paths | Operator                                                    | Manual / private deploy         |
| **kms**              | Create on KMS HTTP                                            | Fund from NCTL **faucet** (`scripts/fund-kms-from-nctl.sh`) | `live_kms` / LocalStack stack   |

CI covers KMS **sign** via wiremock. Live KMS create+fund is a **pre-test** step (restarted empty KMS ⇒ recreate and refund).

#### `chain-put` (external sign)

1. CEP `submit=return` → Transaction JSON
2. Sign elsewhere
3. `POST /v1/chain/put-transaction` → `CEPClient::put_transaction`

### Keys and funding (not product HTTP)

| Context                         | Keys                                                         | Funding                       |
| ------------------------------- | ------------------------------------------------------------ | ----------------------------- |
| **Private + KMS (recommended)** | Create on KMS API                                            | Lab: fund from NCTL faucet    |
| Private, no KMS (fallback)      | `LOCAL_KEYS_JSON_PRODUCTION`                                 | Operator                      |
| none / public demo              | Public keys in requests only                                 | External signer → `chain-put` |
| NCTL lab                        | `LOCAL_KEYS_JSON` from NCTL users (`make export-local-keys`) | Already funded by NCTL        |
| This API                        | Never create/list/fund over HTTP                             | Never                         |

## Docker

API image: `interchouette/ceps-rust-api` (`:dev`, `:latest`, version tags). Build locally with `make docker-build` / `make docker-build-dev` (context = parent dir; needs sibling `ceps-rust-ts-client` + `rustSDK`). Publish via `make docker-push-dev` or CI. Details: [`docker/README.md`](docker/README.md).

```bash
# API only (point CEPS_RPC_URL at a reachable node)
make docker-run

# Recommended: API + KMS peer (private network; signing only; create keys on KMS)
SIGN_BACKEND=kms KMS_URL=http://kms-secp256k1-api:4000 make docker-run-kms
```

Compose file: [`docker/docker-compose.yml`](docker/docker-compose.yml).

## Examples

### none → CEP make (unsigned JSON)

```bash
make run

curl -sS -X POST http://127.0.0.1:8080/v1/cep18/install \
  -H 'content-type: application/json' \
  -d '{"submit":"return","signer":{"public_key":"<your-public-key>"},"payment_amount":"500000000000","name":"Demo","symbol":"DMO","decimals":9,"total_supply":"1000000000000","wasm":"cep18"}'
```

### NCTL lab local keys → put

```bash
make run-local

curl -sS -X POST http://127.0.0.1:8080/v1/cep18/install \
  -H 'content-type: application/json' \
  -d '{"submit":"put","signer":{"public_key":"<nctl-user>"},"payment_amount":"500000000000","name":"Demo","symbol":"DMO","decimals":9,"total_supply":"1000000000000","wasm":"cep18"}'
```

### KMS put signing (private network)

Pre-test / ops (outside this API): create on KMS (`BLOCKCHAIN_MODE=casper`), fund createKey **`address`** (Casper PublicKey hex; not the SEC1 `public_key` field), then:

```bash
scripts/fund-kms-from-nctl.sh <createKey-address>

SIGN_BACKEND=kms KMS_URL=http://127.0.0.1:4000 make run
# CEP submit=put with signer.public_key = that same address
```

After a CEP-95 Odra install, call `POST /v1/cep95/bind-odra-install` with `installer_public_key` and `package_hash_key_name` (returns contract/package hashes).

### Live harness (Rust integration)

Ops scripts prepare env; assertions are Rust tests (not bash e2e).

```bash
export LOCAL_KEYS_JSON="$(NCTL_USERS='1 2 3' NCTL_CONTAINER=casper-nctl-2-docker-dev scripts/export-nctl-local-keys.sh)"
SIGN_BACKEND=local cargo test -p ceps-rust-api --test live_local -- --nocapture

# CEPS_KMS_PUBLIC_KEY=… SIGN_BACKEND=kms KMS_URL=… cargo test -p ceps-rust-api --test live_kms -- --nocapture
```

Ops only: `scripts/fund-kms-from-nctl.sh`, `scripts/export-nctl-local-keys.sh`. Application code under `crates/` does not load faucet or NCTL concepts.

## Make targets

| Target                                   | Purpose                                                       |
| ---------------------------------------- | ------------------------------------------------------------- |
| `make build` / `run`                     | Debug binary                                                  |
| `make verify`                            | fmt, clippy, pem-ban, tests                                   |
| `make verify-slices`                     | CI feature-slice builds                                       |
| `make docker-build` / `docker-build-dev` | Image (parent context; sibling client + rustSDK)              |
| `make docker-push-dev`                   | Push API `:dev` to Hub + GHCR                                 |
| `make docker-run` / `docker-run-kms`     | Compose up                                                    |
| `make wasm-from-ceps`                    | Stage tip CEP WASMs from sibling checkouts into `tests/wasm/` |
| `make export-local-keys`                 | Print `LOCAL_KEYS_JSON` (NCTL users 1 2 3 by default)         |
| `make run-local`                         | Lab run with `SIGN_BACKEND=local` + exported keys             |
| `scripts/export-nctl-local-keys.sh`      | Same export (ops script)                                      |
| `scripts/fund-kms-from-nctl.sh`          | Ops: fund a KMS public key from NCTL faucet                   |
| `make version-show`                      | Crate / image tag                                             |

## License

See [`LICENSE`](LICENSE).

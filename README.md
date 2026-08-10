# ceps-rust-api

HTTP API for Casper **CEP-18**, **CEP-78**, **CEP-85**, and **CEP-95**, built with [Actix Web](https://actix.rs/) and [utoipa](https://github.com/juhaku/utoipa).

It sits on [`ceps-rust-ts-client`](https://github.com/Interchouette-ITC/ceps-rust-ts-client) for CEP args, transaction make, and wait, and on [`casper-rust-wasm-sdk`](https://github.com/casper-ecosystem/casper-rust-wasm-sdk) for sign and put. Optional signing uses a local PEM keyring or the [kms-secp256k1-api](https://github.com/Interchouette-ITC/kms-secp256k1-api) HTTP peer. Request bodies never carry PEM material.

## Quick start

```bash
cp .env.example .env
make build
make verify
make run
```

| URL | Purpose |
| --- | --- |
| `http://127.0.0.1:8080/` | Hello (features + `sign_backend`) |
| `http://127.0.0.1:8080/health` | Liveness |
| `http://127.0.0.1:8080/docs/` | Swagger UI |
| `http://127.0.0.1:8080/docs/ceps-openapi.json` | OpenAPI JSON |

Default node settings match local NCTL: RPC `http://127.0.0.1:11101`, SSE `http://127.0.0.1:18101/events`, chain `casper-net-1`.

```bash
env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH make build
make verify-slices   # same feature slices as CI
```

## Stack

| Piece | Role |
| --- | --- |
| This API | Actix CEP routes, OpenAPI, feature-gated add-ons |
| `ceps-rust-ts-client` | CEP entrypoints, make → Transaction JSON, wait |
| `casper-rust-wasm-sdk` | Sign helpers, put transaction |
| `kms-secp256k1-api` | HTTP key create + sign (not a Rust path-dep) |

**Separation:** KMS owns secrets and signatures. This API owns CEP recipes and chain transactions (including native fund). This API proxies KMS over HTTP; it does not embed KMS.

## Configuration

| Env | Default / notes |
| --- | --- |
| `APP_ADDR` / `APP_PORT` | `0.0.0.0` / `8080` |
| `CEPS_RPC_URL` | `http://127.0.0.1:11101` |
| `CEPS_SSE_URL` | `http://127.0.0.1:18101/events` |
| `CEPS_CHAIN_NAME` | `casper-net-1` |
| `SIGN_BACKEND` | unset, empty, or `none` → no put signer; `local` or `kms` |
| `LOCAL_KEYS_JSON` | Static local keyring when `SIGN_BACKEND=local` (e.g. NCTL user PEMs) |
| `KMS_URL` | KMS peer when using `sign-kms` routes / `SIGN_BACKEND=kms` |
| `CEPS_WASM_ROOT` | Directory of contract `.wasm` files |
| `RUST_LOG` | tracing filter |

See [`.env.example`](.env.example).

## Cargo features

Package **default** is a full build. Slim builds use `--no-default-features` plus the scopes you need.

| Feature | Effect |
| --- | --- |
| `cep18` … `cep95` / `ceps-all` | CEP route scopes |
| `all` | Alias for the package default |
| `swagger-ui` | `/docs` |
| `tx-return` | Allow `submit=return` (Transaction JSON without put) |
| `sign-local` | Static in-process keyring from `LOCAL_KEYS_JSON`; `GET /v1/keys` |
| `sign-kms` | KMS HTTP client, `POST /v1/kms/create-key`, `GET /v1/kms/list-keys`, `POST /v1/chain/fund` |
| `chain-put` | `POST /v1/chain/put-transaction` |

`Makefile` `FEATURES` defaults to the same set as package default. Docker accepts `--build-arg FEATURES=…`.

Hello `/` reports compiled features, `sign_backend`, and whether `KMS_URL` is set.

## HTTP surface

Platform routes: health, hello, instances registry, wasm list, chain balance / account / transaction queries.

CEP mutates share an envelope: `submit` (`put` \| `return`), `wait` (`accepted` \| `processed`), `signer.public_key`, `payment_amount`.

| Area | Examples |
| --- | --- |
| CEP-18 | `/v1/cep18/install`, `transfer`, `mint`, `…/{hash}/balance-of/{owner}` |
| CEP-78 | `/v1/cep78/install`, `mint`, `transfer`, `…/owner-of/{token}` |
| CEP-85 | `/v1/cep85/install`, `mint`, `transfer`, `…/balance-of/{owner}/{id}` |
| CEP-95 | `/v1/cep95/install`, `transfer-from`, `bind-odra-install`, `…/owner-of/{token_id}` |

OpenAPI lists the paths compiled into this binary. Full route lists are easiest in `/docs/`.

### Signing

Put signing is optional. Pick a backend with `SIGN_BACKEND` (unset / empty / `none` = no put signer).

| Mode | How |
| --- | --- |
| none | Queries and `submit=return` (with `tx-return`) work; `submit=put` → `no_signer` |
| local | Static PEMs from `LOCAL_KEYS_JSON` (typically NCTL faucet / users). No HTTP key create. No PEM in request bodies. |
| kms | Keys stay in KMS (`KMS_URL`). Create with `POST /v1/kms/create-key`. This API never holds PEM. |

There is no local `keys/create`. Dynamic key create is KMS-only.

### Fund (`sign-kms`)

`POST /v1/chain/fund` is a native CSPR transfer compiled with `sign-kms`. The **funder** signs with the active `SIGN_BACKEND` (`local` or `kms`). The **recipient** is only a public key (often a KMS key).

Bootstrap from NCTL faucet to a first KMS key:

1. Put the NCTL faucet (or a funded user) into `LOCAL_KEYS_JSON`, `SIGN_BACKEND=local`.
2. `POST /v1/kms/create-key` → recipient public key.
3. `POST /v1/chain/fund` with faucet as `signer` and the KMS public key as `target`.
4. Later puts can use `SIGN_BACKEND=kms` for that key.

KMS does not move CSPR; this API does, because fund is a chain transaction.

External sign path: `submit=return` → sign elsewhere → `POST /v1/chain/put-transaction` (`chain-put`).

## Docker

Image: `interchouette/ceps-rust-api` (build locally with `make docker-build`). Details: [`docker/README.md`](docker/README.md).

```bash
# API only (point CEPS_RPC_URL at a reachable node)
make docker-run

# API + KMS (+ LocalStack under profile kms)
SIGN_BACKEND=kms KMS_URL=http://kms-secp256k1-api:4000 make docker-run-kms
```

Compose file: [`docker/docker-compose.yml`](docker/docker-compose.yml).

## Examples

### NCTL local keys → CEP make

Load NCTL user PEMs into `LOCAL_KEYS_JSON`, then run with `SIGN_BACKEND=local`. No key create on this path.

```bash
SIGN_BACKEND=local CEPS_RPC_URL=http://127.0.0.1:11101 make run

curl -sS -X POST http://127.0.0.1:8080/v1/cep18/install \
  -H 'content-type: application/json' \
  -d '{"submit":"return","signer":{"public_key":"<nctl-user>"},"payment_amount":"500000000000","name":"Demo","symbol":"DMO","decimals":9,"total_supply":"1000000000000","wasm":"cep18"}'
```

### KMS create → fund from NCTL faucet → install

```bash
# Faucet (or funded user) in LOCAL_KEYS_JSON; KMS_URL set; features include sign-kms
SIGN_BACKEND=local KMS_URL=http://127.0.0.1:4000 make run

curl -sS -X POST http://127.0.0.1:8080/v1/kms/create-key

curl -sS -X POST http://127.0.0.1:8080/v1/chain/fund \
  -H 'content-type: application/json' \
  -d '{"submit":"put","wait":"accepted","signer":{"public_key":"<faucet>"},"payment_amount":"1000000000","target":"<kms-pk>","amount":"2500000000"}'
```

After a CEP-95 Odra install, call `POST /v1/cep95/bind-odra-install` with `installer_public_key` and `package_hash_key_name` (optional `label` registers an instance).

### Live harness (tests only)

```bash
export CEPS_FAUCET_PEM_PATH=/path/to/faucet/private.pem
# Fund an existing NCTL user, or create via KMS:
# export CEPS_FUND_TARGET=01…
# export KMS_URL=http://127.0.0.1:4000
cargo test -p ceps-api --test live_bootstrap -- --nocapture
```

Helpers live under `crates/ceps-api/tests/integration/`. Application code under `crates/` does not load faucet or NCTL concepts.

## Make targets

| Target | Purpose |
| --- | --- |
| `make build` / `run` | Debug binary |
| `make verify` | fmt, clippy, pem-ban, tests |
| `make verify-slices` | CI feature-slice builds |
| `make docker-build` | Release image (`FEATURES` → `--build-arg`) |
| `make docker-run` / `docker-run-kms` | Compose up |
| `make version-show` | Crate / image tag |

## License

See [`LICENSE`](LICENSE).

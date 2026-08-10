# ceps-rust-api

HTTP API for Casper **CEP-18**, **CEP-78**, **CEP-85**, and **CEP-95**, built with [Actix Web](https://actix.rs/) and [utoipa](https://github.com/juhaku/utoipa).

It sits on [`ceps-rust-ts-client`](https://github.com/Interchouette-ITC/ceps-rust-ts-client) for CEP args, transaction make, and wait, and on [`casper-rust-wasm-sdk`](https://github.com/casper-ecosystem/casper-rust-wasm-sdk) for sign and put. Optional put signing uses a static local PEM keyring or the [kms-secp256k1-api](https://github.com/Interchouette-ITC/kms-secp256k1-api) HTTP peer. Request bodies never carry PEM material.

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
| This API | CEP routes, OpenAPI, put pipeline |
| `ceps-rust-ts-client` | CEP entrypoints, make → Transaction JSON, wait |
| `casper-rust-wasm-sdk` | Sign helpers, put transaction |
| `kms-secp256k1-api` | Optional HTTP signer (create/list keys on KMS itself) |

**Separation:** KMS owns secrets, key create/list, and signatures. This API owns CEP recipes and chain puts/queries. Call KMS directly to create keys; this API only calls KMS `signTransaction` when `SIGN_BACKEND=kms`.

## Configuration

| Env | Default / notes |
| --- | --- |
| `APP_ADDR` / `APP_PORT` | `0.0.0.0` / `8080` |
| `CEPS_RPC_URL` | `http://127.0.0.1:11101` |
| `CEPS_SSE_URL` | `http://127.0.0.1:18101/events` |
| `CEPS_CHAIN_NAME` | `casper-net-1` |
| `SIGN_BACKEND` | unset, empty, or `none` → no put signer; `local` or `kms` |
| `LOCAL_KEYS_JSON` | Static local keyring when `SIGN_BACKEND=local` (e.g. NCTL user PEMs) |
| `KMS_URL` | KMS peer when `SIGN_BACKEND=kms` |
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
| `sign-local` | Static in-process keyring from `LOCAL_KEYS_JSON` |
| `sign-kms` | Internal KMS sign client for `SIGN_BACKEND=kms` |
| `chain-put` | `POST /v1/chain/put-transaction` |

There is no HTTP key create, key list, or fund route on this API.

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
| local | Static PEMs from `LOCAL_KEYS_JSON` (typically NCTL faucet / users). No HTTP key create. |
| kms | Keys stay in KMS (`KMS_URL`). Create keys on the KMS API. This process never holds PEM. |

External sign path: `submit=return` → sign elsewhere → `POST /v1/chain/put-transaction` (`chain-put`).

### Funding (not an HTTP product feature)

This API does **not** expose fund. Who moves CSPR:

| Context | How |
| --- | --- |
| **Local / NCTL lab** | Use already-funded NCTL users in `LOCAL_KEYS_JSON`. Or transfer with NCTL / `casper-client` outside this API. |
| **Tests** | Harness under `crates/ceps-api/tests/integration/` may load a faucet PEM and call SDK transfer (and optionally KMS `createKey` directly). Never product routes. |
| **Production** | Accounts are funded out of band (treasury, exchange, ops). Day-to-day: create keys on KMS if needed, set `SIGN_BACKEND=kms`, call CEP routes with `signer.public_key`. |

## Docker

Image: `interchouette/ceps-rust-api` (build locally with `make docker-build`). Details: [`docker/README.md`](docker/README.md).

```bash
# API only (point CEPS_RPC_URL at a reachable node)
make docker-run

# API + KMS peer (signing only; create keys on KMS)
SIGN_BACKEND=kms KMS_URL=http://kms-secp256k1-api:4000 make docker-run-kms
```

Compose file: [`docker/docker-compose.yml`](docker/docker-compose.yml).

## Examples

### NCTL local keys → CEP make

```bash
SIGN_BACKEND=local CEPS_RPC_URL=http://127.0.0.1:11101 make run

curl -sS -X POST http://127.0.0.1:8080/v1/cep18/install \
  -H 'content-type: application/json' \
  -d '{"submit":"return","signer":{"public_key":"<nctl-user>"},"payment_amount":"500000000000","name":"Demo","symbol":"DMO","decimals":9,"total_supply":"1000000000000","wasm":"cep18"}'
```

### KMS put signing

Create the key on KMS (its HTTP API). Fund that public key out of band. Then:

```bash
SIGN_BACKEND=kms KMS_URL=http://127.0.0.1:4000 make run
# CEP submit=put with signer.public_key = the KMS public key
```

After a CEP-95 Odra install, call `POST /v1/cep95/bind-odra-install` with `installer_public_key` and `package_hash_key_name` (optional `label` registers an instance).

### Live harness (tests only)

```bash
export CEPS_FAUCET_PEM_PATH=/path/to/faucet/private.pem
# Fund an existing NCTL user, or create via KMS then fund in harness:
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

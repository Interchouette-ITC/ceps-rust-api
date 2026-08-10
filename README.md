# ceps-rust-api

HTTP API for Casper **CEP-18**, **CEP-78**, **CEP-85**, and **CEP-95**, built with [Actix Web](https://actix.rs/) and [utoipa](https://github.com/juhaku/utoipa).

It sits on [`ceps-rust-ts-client`](https://github.com/Interchouette-ITC/ceps-rust-ts-client) for CEP args, transaction make, and wait, and on [`casper-rust-wasm-sdk`](https://github.com/casper-ecosystem/casper-rust-wasm-sdk) for sign and put. Optional signing uses a local PEM keyring or the [kms-secp256k1-api](https://github.com/Interchouette-ITC/kms-secp256k1-api) HTTP peer. Request bodies never carry PEM material.

Canonical repo: [Interchouette-ITC/ceps-rust-api](https://github.com/Interchouette-ITC/ceps-rust-api).

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
| This API | Actix routes, OpenAPI, feature-gated add-ons |
| `ceps-rust-ts-client` | CEP entrypoints, make → Transaction JSON, wait |
| `casper-rust-wasm-sdk` | Sign helpers, put transaction |
| `kms-secp256k1-api` | Optional Docker HTTP signer (not a Rust path-dep) |

## Configuration

| Env | Default / notes |
| --- | --- |
| `APP_ADDR` / `APP_PORT` | `0.0.0.0` / `8080` |
| `CEPS_RPC_URL` | `http://127.0.0.1:11101` |
| `CEPS_SSE_URL` | `http://127.0.0.1:18101/events` |
| `CEPS_CHAIN_NAME` | `casper-net-1` |
| `SIGN_BACKEND` | unset, empty, or `none` → no put signer; `local` or `kms` |
| `LOCAL_KEYS_JSON` | Local keyring when `SIGN_BACKEND=local` |
| `KMS_URL` | KMS peer when `SIGN_BACKEND=kms` |
| `CEPS_WASM_ROOT` | Directory of contract `.wasm` files |
| `RUST_LOG` | tracing filter |

See [`.env.example`](.env.example).

## Cargo features

Package **default** is a full demo build. Slim builds use `--no-default-features` plus the scopes you need.

| Feature | Effect |
| --- | --- |
| `cep18` … `cep95` / `ceps-all` | CEP route scopes |
| `all` | Alias for the package default (CEPs + add-ons below) |
| `swagger-ui` | `/docs` |
| `tx-return` | Allow `submit=return` (Transaction JSON without put) |
| `sign-local` | In-process keyring for `SIGN_BACKEND=local` (`LOCAL_KEYS_JSON` and/or keys from custody create) |
| `sign-kms` | KMS HTTP client + `/v1/kms/*` (including `POST /v1/kms/create-key`) |
| `chain-put` | `POST /v1/chain/put-transaction` |
| `custody` | Demo helpers: `POST /v1/keys/create` (local keyring only) and `POST /v1/chain/fund` (native CSPR transfer; put signs with local or KMS) |

`custody` pulls in `sign-local` because `/v1/keys/create` writes the process keyring. KMS key creation is not under `custody`; it is `/v1/kms/create-key` (`sign-kms`).

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
| local | Demo / lab: PEMs stay in process memory (`LOCAL_KEYS_JSON` bootstrap, or generate with `POST /v1/keys/create`). No PEM in HTTP bodies. |
| kms | Keys stay in KMS (`KMS_URL`). Create with `POST /v1/kms/create-key`. API does not hold PEM. |

`POST /v1/chain/fund` funds an account with a native transfer. The funder signs via the active backend (local keyring or KMS). Create the recipient with local custody or with KMS, then fund.

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

### Local custody → fund → CEP-18 make

Needs a reachable RPC and CSPR on a funder key the API can sign (bootstrap out of band, or the live harness below). Local create is for demos without KMS: the new secret stays in the process keyring.

```bash
SIGN_BACKEND=local CEPS_RPC_URL=http://127.0.0.1:11101 make run

curl -sS -X POST http://127.0.0.1:8080/v1/keys/create \
  -H 'content-type: application/json' \
  -d '{"algo":"ed25519"}'

curl -sS -X POST http://127.0.0.1:8080/v1/chain/fund \
  -H 'content-type: application/json' \
  -d '{"submit":"put","wait":"accepted","signer":{"public_key":"<funder>"},"payment_amount":"1000000000","target":"<new>","amount":"2500000000"}'

curl -sS -X POST http://127.0.0.1:8080/v1/cep18/install \
  -H 'content-type: application/json' \
  -d '{"submit":"return","signer":{"public_key":"<new>"},"payment_amount":"500000000000","name":"Demo","symbol":"DMO","decimals":9,"total_supply":"1000000000000","wasm":"cep18"}'
```

With KMS (`SIGN_BACKEND=kms`), create the recipient via `POST /v1/kms/create-key`, then use the same `fund` / install envelopes (funder must be a KMS key the peer can sign).

After a CEP-95 Odra install, call `POST /v1/cep95/bind-odra-install` with `installer_public_key` and `package_hash_key_name` (optional `label` registers an instance).

### Live harness (tests only)

```bash
export CEPS_FAUCET_PEM_PATH=/path/to/faucet/private.pem
# optional: CEPS_FAUCET_PUBLIC_KEY=01…
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

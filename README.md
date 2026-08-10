# ceps-rust-api

HTTP API for Casper **CEP-18**, **CEP-78**, **CEP-85**, and **CEP-95**, built with [Actix Web](https://actix.rs/) and [utoipa](https://github.com/juhaku/utoipa).

It sits on [`ceps-rust-ts-client`](https://github.com/Interchouette-ITC/ceps-rust-ts-client) (`CEPClient`) for CEP args, transaction make, wait, and put of signed JSON. Optional put signing uses a lab local PEM keyring (`LOCAL_KEYS_JSON`) or the [kms-secp256k1-api](https://github.com/Interchouette-ITC/kms-secp256k1-api) HTTP peer. Request bodies never carry PEM material.

## Quick start

```bash
cp .env.example .env
make build
make verify
make run
```

| URL                                            | Purpose                           |
| ---------------------------------------------- | --------------------------------- |
| `http://127.0.0.1:8080/`                       | Hello (features + `sign_backend`) |
| `http://127.0.0.1:8080/health`                 | Liveness                          |
| `http://127.0.0.1:8080/docs/`                  | Swagger UI                        |
| `http://127.0.0.1:8080/docs/ceps-openapi.json` | OpenAPI JSON                      |

Default node settings match local NCTL: RPC `http://127.0.0.1:11101`, SSE `http://127.0.0.1:18101/events`, chain `casper-net-1`.

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

**Separation:** KMS owns secrets, key create/list, and signatures. This API owns CEP recipes and put pipeline. Call KMS directly to create keys; this API only calls KMS `signTransaction` when `SIGN_BACKEND=kms`. No account/balance/transaction query routes (use node RPC or CEP `wait` / `CallResult`).

## Configuration

| Env                     | Default / notes                                                                                                                                                    |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `APP_ADDR` / `APP_PORT` | `0.0.0.0` / `8080`                                                                                                                                                 |
| `CEPS_RPC_URL`          | `http://127.0.0.1:11101`                                                                                                                                           |
| `CEPS_SSE_URL`          | `http://127.0.0.1:18101/events`                                                                                                                                    |
| `CEPS_CHAIN_NAME`       | `casper-net-1`                                                                                                                                                     |
| `SIGN_BACKEND`          | unset, empty, or `none` → no put signer; `local` or `kms`                                                                                                          |
| `LOCAL_KEYS_JSON`       | **Lab / NCTL / tests only.** Env JSON blob (not a file path): `public_key` → PEM. Load NCTL faucet/user PEMs for `SIGN_BACKEND=local`. Not a production key store. |
| `KMS_URL`               | KMS peer when `SIGN_BACKEND=kms`                                                                                                                                   |
| `CEPS_WASM_ROOT`        | Directory of contract `.wasm` files                                                                                                                                |
| `RUST_LOG`              | tracing filter                                                                                                                                                     |

See [`.env.example`](.env.example).

## Cargo features

Package **default** is a full build. Slim builds use `--no-default-features` plus the scopes you need.

| Feature                        | Effect                                                                      |
| ------------------------------ | --------------------------------------------------------------------------- |
| `cep18` … `cep95` / `ceps-all` | CEP route scopes                                                            |
| `all`                          | Alias for the package default                                               |
| `swagger-ui`                   | `/docs`                                                                     |
| `tx-return`                    | Allow `submit=return` (Transaction JSON without put)                        |
| `sign-local`                   | Lab keyring from `LOCAL_KEYS_JSON` (NCTL PEMs). Not for production custody. |
| `sign-kms`                     | Internal KMS `signTransaction` client for puts                              |
| `chain-put`                    | `POST /v1/chain/put-transaction` (already-signed Transaction JSON)          |

There is no HTTP key create, key list, or fund route on this API.

`Makefile` `FEATURES` defaults to the same set as package default. Docker accepts `--build-arg FEATURES=…`.

Hello `/` reports compiled features, `sign_backend`, and whether `KMS_URL` is set.

## HTTP surface

Platform: health, hello, instances registry, wasm list. Optional: `POST /v1/chain/put-transaction` (`chain-put`).

CEP mutates share an envelope: `submit` (`put` \| `return`), `wait` (`accepted` \| `processed`), `signer.public_key`, `payment_amount`.

| Area   | Examples                                                                           |
| ------ | ---------------------------------------------------------------------------------- |
| CEP-18 | `/v1/cep18/install`, `transfer`, `mint`, `…/{hash}/balance-of/{owner}`             |
| CEP-78 | `/v1/cep78/install`, `mint`, `transfer`, `…/owner-of/{token}`                      |
| CEP-85 | `/v1/cep85/install`, `mint`, `transfer`, `…/balance-of/{owner}/{id}`               |
| CEP-95 | `/v1/cep95/install`, `transfer-from`, `bind-odra-install`, `…/owner-of/{token_id}` |

OpenAPI lists the paths compiled into this binary. Full route lists are easiest in `/docs/`.

### Signing

| Mode  | How                                                                                                  |
| ----- | ---------------------------------------------------------------------------------------------------- |
| none  | `submit=return` (with `tx-return`) works; `submit=put` → `no_signer`                                 |
| local | NCTL lab: PEMs in `LOCAL_KEYS_JSON`. Already funded on NCTL. No create/fund in this API.             |
| kms   | Production-shaped: keys in KMS. Create on KMS; fund **before** use (see below). API only signs puts. |

#### How signing modes are tested (same shape)

| Mode      | Keys                                  | Fund                                                                                                           | Then                          |
| --------- | ------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ----------------------------- |
| **local** | NCTL faucet/users → `LOCAL_KEYS_JSON` | Already funded by NCTL                                                                                         | `SIGN_BACKEND=local`, CEP put |
| **kms**   | Create on KMS HTTP                    | Fund those public keys from NCTL faucet **before** tests (`scripts/fund-kms-from-nctl.sh`; not this API) | `SIGN_BACKEND=kms`, CEP put   |


CI always covers KMS **sign** via wiremock. Live KMS create+fund is a **pre-test** step (restarted empty KMS ⇒ recreate and refund).

`wait_transaction` on `CEPClient`: use when you put with wait off, or already have a hash; normal CEP put uses `TransactionParams::wait`. This API’s CEP routes rely on that; we do not re-expose get_transaction over HTTP.

#### Why account/tx query routes were removed

They were early “platform demo” helpers (check CSPR balance after fund, fetch tx by hash). That is node-explorer work, not CEP. Contract queries stay on CEP routes; put outcome stays `wait` / `CallResult`.

#### `chain-put` (external sign)

1. CEP `submit=return` → Transaction JSON
2. Sign elsewhere
3. `POST /v1/chain/put-transaction` → `CEPClient::put_transaction`

### Keys and funding (not product HTTP)

| Context                | Keys                                    | Funding                                                   |
| ---------------------- | --------------------------------------- | --------------------------------------------------------- |
| NCTL lab / local tests | `LOCAL_KEYS_JSON` (NCTL faucet + users) | Already funded by NCTL                                    |
| KMS live / prod        | Create on KMS API                       | Out of band / pre-test from NCTL faucet to KMS public key |
| This API               | Never create/list/fund over HTTP        | Never                                                     |

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

Pre-test / ops (outside this API): create on KMS, fund the public key, then:

```bash
# After KMS createKey → public_key hex:
scripts/fund-kms-from-nctl.sh <kms-public-key-hex>
# Uses docker exec into NCTL (default container casper-nctl-2-docker-dev)
# or host casper-client + NCTL assets. nctl-transfer-native only funds user-N.

SIGN_BACKEND=kms KMS_URL=http://127.0.0.1:4000 make run
# CEP submit=put with signer.public_key = the funded KMS public key
```

After a CEP-95 Odra install, call `POST /v1/cep95/bind-odra-install` with `installer_public_key` and `package_hash_key_name` (optional `label` registers an instance).

### Live harness (tests only)

```bash
# Put funded NCTL faucet/user PEMs into LOCAL_KEYS_JSON, then:
cargo test -p ceps-api --test live_bootstrap -- --nocapture
```

Helpers live under `crates/ceps-api/tests/integration/`. Application code under `crates/` does not load faucet or NCTL concepts.

## Make targets

| Target                               | Purpose                                    |
| ------------------------------------ | ------------------------------------------ |
| `make build` / `run`                 | Debug binary                               |
| `make verify`                        | fmt, clippy, pem-ban, tests                |
| `make verify-slices`                 | CI feature-slice builds                    |
| `make docker-build`                  | Release image (`FEATURES` → `--build-arg`) |
| `make docker-run` / `docker-run-kms` | Compose up                                 |
| `make version-show`                  | Crate / image tag                          |

## License

See [`LICENSE`](LICENSE).

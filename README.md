# ceps-rust-api

Actix HTTP API for Casper **CEP-18 / CEP-78 / CEP-85 / CEP-95**. Socle works with no signer. Signing is optional via `SIGN_BACKEND` (`local` or `kms`). Unset or empty means **none**. No PEM in HTTP bodies. **Transactions only** (no deploy vocabulary).

| Piece | Stack |
| ----- | ----- |
| This API | Actix Web + utoipa |
| CEP logic | `ceps-client` (make → JSON + wait) |
| Sign / naked put | SDK helpers + optional KMS HTTP peer |

## Feature matrix

| Cargo feature | ON | OFF |
| ------------- | -- | --- |
| `cep18`…`cep95` / `ceps-all` | CEP route scopes | 404 |
| `tx-return` | `submit=return` allowed (Transaction JSON) | `return` → 400 `feature_disabled` |
| `sign-local` | `LOCAL_KEYS_JSON` keyring | no local signing |
| `sign-kms` | KMS HTTP + `/v1/kms/*` | no KMS |
| `chain-put` | `POST /v1/chain/put-transaction` | 404 |
| `custody` | `POST /v1/keys/create`, `POST /v1/chain/fund` (implies `sign-local`) | absent |

Runtime: `SIGN_BACKEND` unset/empty/`none` → put fails with `no_signer`; return works if `tx-return`. Hello `/` returns `features`, `sign_backend`, `kms_url_configured`.

## Develop

```bash
cp .env.example .env
make build
make verify
make run
# curl http://127.0.0.1:8080/
# curl http://127.0.0.1:8080/docs/ceps-openapi.json
```

```bash
env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH make build
```

## External-sign flow

1. `POST /v1/cep18/...` with `submit=return` → Transaction JSON  
2. Sign outside (or KMS)  
3. `POST /v1/chain/put-transaction` with signed JSON (`chain-put`)

## Status

Socle: health, hello, instances, wasm list, chain queries. Add-ons: sign backends, tx-return, chain-put, custody, CEP routes behind features. KMS is Docker HTTP only.

## Docker compose

```bash
# API only (point CEPS_RPC_URL at a reachable node)
make docker-run

# API + KMS peer
make docker-run-kms
# then set SIGN_BACKEND=kms and KMS_URL=http://kms-secp256k1-api:4000 on the API service
```

## Live harness (tests only)

Integration helpers under `crates/ceps-api/tests/integration/` load a faucet PEM from env (never from HTTP):

```bash
export CEPS_FAUCET_PEM_PATH=/path/to/faucet/secret_key.pem
export CEPS_FAUCET_PUBLIC_KEY=01…   # optional; NCTL default known
cargo test -p ceps-api --test live_bootstrap -- --nocapture
```

Product code under `crates/` never imports faucet or NCTL concepts.

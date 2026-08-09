# ceps-rust-api

Actix HTTP API for Casper **CEP-18 / CEP-78 / CEP-85 / CEP-95**. Socle is queries and CEP routes without a signer. Signing is optional via `SIGN_BACKEND` (`local` or `kms`). Unset or empty means **none** (you do not need to set `SIGN_BACKEND=none`). No PEM keys in HTTP bodies.

| Piece    | Stack                                                    |
| -------- | -------------------------------------------------------- |
| This API | Actix Web + utoipa                                       |
| KMS      | Optional Docker peer (`interchouette/kms-secp256k1-api`) |

## Hop and run (outline)

1. Point this API at `CEPS_RPC_URL` / `CEPS_SSE_URL` and run (`make run`). Leave `SIGN_BACKEND` unset for a no-signer socle.
2. For KMS signing: run KMS (+ LocalStack when using local AWS KMS), set `KMS_URL` and `SIGN_BACKEND=kms`.
3. For local PEM keyring signing: set `SIGN_BACKEND=local` and `LOCAL_KEYS_JSON` (when that feature is wired).
4. Open `/docs/` for Swagger UI; OpenAPI JSON at `/docs/ceps-openapi.json`.

## Develop

```bash
cp .env.example .env
make build
make verify
cargo run -p ceps-api
# curl http://127.0.0.1:8080/
# curl http://127.0.0.1:8080/health
# curl http://127.0.0.1:8080/docs/ceps-openapi.json
```

Unset Cursor sandbox target dir when building:

```bash
env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH make build
```

## Features

Cargo features: `cep18`, `cep78`, `cep85`, `cep95`, `all` (default), `swagger-ui` (default).

## Status

Skeleton: health, hello (`sign_backend`, `kms_url_configured`), OpenAPI, Actix `/docs/` (absolute redirect). CEP routes, sign backends, and KMS HTTP client land next.

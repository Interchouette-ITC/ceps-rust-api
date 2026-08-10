# Docker image (ceps-rust-api)

Multi-stage build → `gcr.io/distroless/cc-debian13:nonroot` (Debian 13 / trixie family).

| Item | Value |
| --- | --- |
| Binary | `ceps-rust-api` |
| Builder | `rust:slim-trixie` |
| Port | `APP_PORT` (default `8080`) |
| Compose | [`docker-compose.yml`](docker-compose.yml) |

## Build features

Default `FEATURES` matches the package default (CEPs + swagger / tx-return / sign-local / sign-kms / chain-put). Override at build time:

```bash
# From this repo (context = parent so path-deps resolve):
make docker-build
# Or:
docker build -f docker/Dockerfile --build-arg FEATURES=ceps-all,swagger-ui -t interchouette/ceps-rust-api:slim ..
```

## Where to pull images

| Registry | Image |
| --- | --- |
| Docker Hub | `interchouette/ceps-rust-api` |

```bash
docker pull interchouette/ceps-rust-api:latest
```

## Tags

| Tag | Meaning |
| --- | --- |
| `:dev` / `:latest` | Rolling / published tag from `make docker-build` (`TAG`, `APP_VERSION`) |
| `:X.Y.Z` | Version from crate `Cargo.toml` when built with `make docker-build` |

```bash
make version-show
make docker-build
```

## Compose

[`docker-compose.yml`](docker-compose.yml) runs the API and, under profiles, optional KMS (+ LocalStack).

Point `CEPS_RPC_URL` / `CEPS_SSE_URL` at a reachable Casper node (for example NCTL on the host via `host.docker.internal`).

| Profile | Services |
| --- | --- |
| (default) | `ceps-rust-api` only |
| `kms` | API + `kms-secp256k1-api` (+ LocalStack) |
| `localstack` | LocalStack only (also included under `kms`) |

```bash
# API only
make docker-run

# API + KMS peer
SIGN_BACKEND=kms KMS_URL=http://kms-secp256k1-api:4000 make docker-run-kms

make docker-stop
```

Unset or empty `SIGN_BACKEND` means no put signer (`none`). KMS images default to published Hub tags (`interchouette/kms-secp256k1-api`, `interchouette/kms-localstack`).

## Env (API container)

Typical compose / runtime vars: `APP_ADDR`, `APP_PORT`, `CEPS_RPC_URL`, `CEPS_SSE_URL`, `CEPS_CHAIN_NAME`, `SIGN_BACKEND`, `KMS_URL`, `LOCAL_KEYS_JSON`, `CEPS_WASM_ROOT`, `RUST_LOG`. See [`.env.example`](../.env.example) and the root [README](../README.md).

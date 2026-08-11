# MCP for agents

Rust binary `ceps-rust-api-mcp` **v1.0.0** (mcpkit), dual transport (stdio / Streamable HTTP).

**Separate package:** lives in [`mcp/`](../mcp/) - **no dependency** on the `ceps-rust-api` library crate. Lifecycle uses Make/Docker; product tools call the HTTP API on `:8080`.

Tool names use the `ceps_api_*` prefix so they do not collide with `ceps-client-mcp` (`ceps18_*`, …).

Published image: [`interchouette/ceps-rust-api-mcp`](https://hub.docker.com/r/interchouette/ceps-rust-api-mcp) (`:1.0.0`, `:latest`, `:dev`).

## Run without compiling

```bash
# MCP HTTP sidecar (needs Docker socket + this repo mounted as workspace)
docker pull interchouette/ceps-rust-api-mcp:1.0.0
docker run --rm -d --name ceps-rust-api-mcp \
  -p 4790:4790 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$PWD":/workspace \
  --add-host=host.docker.internal:host-gateway \
  -e CEPS_API_ROOT=/workspace \
  -e CEPS_HOST_ROOT="$PWD" \
  -e CEPS_API_URL=http://host.docker.internal:8080 \
  interchouette/ceps-rust-api-mcp:1.0.0
```

`CEPS_HOST_ROOT` is **required** in the MCP container: absolute host clone path (never `/workspace`, never `/`). Lifecycle **start** tools refuse without it.

From a clone, `make mcp-http` pulls the Hub image (builds locally only if pull fails) and passes `CEPS_HOST_ROOT=$(CURDIR)`.

| Mode | How | Endpoint |
| --- | --- | --- |
| **HTTP** (Docker / Hub) | `make mcp-http` | `http://127.0.0.1:4790/mcp` |
| **HTTP** (host) | `make run-mcp-http` | same URL |
| **stdio** (host) | `make run-mcp` | process on stdin/stdout |

```bash
ceps-rust-api-mcp                         # stdio
ceps-rust-api-mcp --http                  # 127.0.0.1:4790
ceps-rust-api-mcp --http --listen 0.0.0.0:4790
```

Example Cursor config: [`mcp/mcp.json.example`](../mcp/mcp.json.example). Product Cursor wiring: `.cursor/mcp.json` → `ceps-api-mcp.sh` (image `:dev`).

## Make matrix (MCP itself)

| Command | Effect |
| --- | --- |
| `make mcp-build` | Host release-build MCP binary |
| `make mcp-docker-build` | Build `ceps-rust-api-mcp:1.0.0` (+ `:latest`) |
| `make mcp-docker-build-dev` | Build `:dev` (Hub + GHCR tags) |
| `make mcp-http` | Pull Hub image (or build) + start sidecar on **4790** |
| `make mcp-http-stop` | Stop MCP sidecar |
| `make mcp-docker-push-dev` / `mcp-docker-push-release` | Push Hub + GHCR |
| `make run-mcp` | Host **stdio** MCP |
| `make run-mcp-http` | Host HTTP on `127.0.0.1:4790` |

Compose: [`docker/docker-compose.mcp.yml`](../docker/docker-compose.mcp.yml). Image Dockerfile: [`mcp/Dockerfile`](../mcp/Dockerfile) (includes `docker`/`make`/`curl` so lifecycle tools work via the mounted repo + docker.sock).

### Host binds / docker.sock

| Claim | Verdict |
| --- | --- |
| MCP uses sock + in-container `/workspace` | **True** - setup only |
| Lifecycle start tools need a safe host bind root | **True** - refuse when `CEPS_API_ROOT=/workspace` and `CEPS_HOST_ROOT` is missing/unsafe |

Always pass `-e CEPS_HOST_ROOT=<absolute host clone>`. Mounting `docker.sock` is still host-root equivalent for anything Docker can do; the refuse guard only blocks the known `/workspace` bind-root class.

### Published MCP images

| Registry | Image |
| --- | --- |
| Docker Hub | `interchouette/ceps-rust-api-mcp` |
| Personal GHCR | `ghcr.io/groussac/ceps-rust-api-mcp` (Make/CI only) |
| Worker GHCR | `ghcr.io/interchouette/ceps-rust-api-mcp` (Make/CI only) |
| Org GHCR | `ghcr.io/interchouette-itc/ceps-rust-api-mcp` |

Tags: `:dev`, `:X.Y.Z`, `:latest`.

### Tests

```bash
cd mcp
cargo test
cargo clippy --all-targets -- -D warnings
```

## Tool catalog

### Lifecycle (Make parity)

MCP drives **Docker/Make** from `CEPS_API_ROOT`. Not exposed: `mcp-docker-push-*` / version-bump.

| Tool | Make / behavior |
| --- | --- |
| `ceps_api_help` | `make help` + tool map |
| `ceps_api_build` / `ceps_api_build_release` / `ceps_api_check` | matching targets; optional `features` |
| `ceps_api_lint` / `ceps_api_test` / `ceps_api_verify` | lint + test suite |
| `ceps_api_docker_build` | API image |
| `ceps_api_docker_run` | compose up `:8080` (refuses unsafe host bind root) |
| `ceps_api_docker_run_kms` | compose with `kms` profile |
| `ceps_api_docker_stop` | compose down |
| `ceps_api_version_show` | `make version-show` |
| `ceps_api_start` / `ceps_api_stop` | host `cargo run` API (pid under `mcp/.run/`) |
| `ceps_api_status` | containers + `GET /health` |

### HTTP API

Base URL: `CEPS_API_URL` (default `http://127.0.0.1:8080`). POST tools take a JSON `body` string.

| Group | Tools (prefix `ceps_api_`) |
| --- | --- |
| Platform | `hello`, `health`, `openapi` |
| Chain | `put_transaction` → `POST /v1/chain/put-transaction` |
| CEP-18 mutate | `cep18_install`, `upgrade`, `transfer`, `transfer_from`, `approve`, `increase_allowance`, `decrease_allowance`, `mint`, `burn`, `change_events_mode`, `change_security` |
| CEP-18 query | `cep18_name`, `symbol`, `decimals`, `total_supply`, `events_mode`, `is_mint_and_burn_enabled`, `balance_of`, `allowances` |
| CEP-78 mutate | `cep78_install`, `upgrade`, `mint`, `transfer`, `burn`, `register_owner`, `approve`, `revoke`, `set_approval_for_all`, `set_token_metadata`, `set_variables`, `mint_session`, `transfer_session`, `updated_receipts` |
| CEP-78 query | `cep78_collection_name`, `collection_symbol`, `total_token_supply`, `number_of_minted_tokens`, `events_mode`, `owner_of`, `balance_of`, `is_approved_for_all`, `get_approved`, `metadata` |
| CEP-85 mutate | `cep85_install`, `upgrade`, `mint`, `batch_mint`, `transfer`, `batch_transfer`, `burn`, `batch_burn`, `set_approval_for_all`, `set_uri`, `set_total_supply_of`, `set_total_supply_of_batch`, `change_security`, `set_modalities` |
| CEP-85 query | `cep85_collection_name`, `collection_uri`, `balance_of`, `supply_of`, `total_supply_of`, `uri`, `is_non_fungible`, `is_approved_for_all` |
| CEP-95 mutate | `cep95_install`, `transfer_from`, `safe_transfer_from`, `approve`, `revoke_approval`, `approve_for_all`, `revoke_approval_for_all`, `mint`, `burn`, `bind_odra_install` |
| CEP-95 query | `cep95_name`, `symbol`, `total_supply`, `owner_of`, `balance_of`, `get_approved`, `is_approved_for_all`, `token_metadata` |

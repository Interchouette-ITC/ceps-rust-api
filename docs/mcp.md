# MCP (ceps-rust-api-mcp)

Sidecar MCP for this repository: Make/Docker lifecycle plus HTTP wrappers for the Actix API on `:8080`.

Tool names use the `ceps_api_*` prefix so they do not collide with `ceps-client-mcp` (`ceps18_*`, …).

## Transports

| Mode | Command | URL |
| --- | --- | --- |
| stdio | `make run-mcp` or Cursor Docker image | (Cursor) |
| HTTP host | `make run-mcp-http` | `http://127.0.0.1:4790/mcp` |
| HTTP Docker | `make mcp-http` | `http://127.0.0.1:4790/mcp` |

## Images

| Registry | Image |
| --- | --- |
| Docker Hub | `interchouette/ceps-rust-api-mcp` |
| Org GHCR | `ghcr.io/interchouette-itc/ceps-rust-api-mcp` |

Tags: `:dev`, `:X.Y.Z`, `:latest`.

## Env

| Variable | Default | Role |
| --- | --- | --- |
| `CEPS_API_MCP_ADDR` | `127.0.0.1:4790` | MCP HTTP listen |
| `CEPS_API_URL` | `http://127.0.0.1:8080` | Target API |
| `CEPS_API_ROOT` | cwd / `/workspace` | Repo root inside process |
| `CEPS_HOST_ROOT` | (required in Docker) | Absolute host clone for binds |

## Tool groups

| Group | Examples |
| --- | --- |
| Lifecycle | `ceps_api_help`, `ceps_api_build`, `ceps_api_lint`, `ceps_api_test`, `ceps_api_verify`, `ceps_api_docker_*`, `ceps_api_start` / `stop`, `ceps_api_status`, `ceps_api_version_show` |
| Platform | `ceps_api_hello`, `ceps_api_health`, `ceps_api_openapi` |
| Chain | `ceps_api_put_transaction` |
| CEP-18/78/85/95 | `ceps_api_cep18_install`, `ceps_api_cep78_mint`, … (one tool per HTTP route; POST tools take a JSON `body` string) |

Push and version-bump targets are not exposed as MCP tools (use Make/CI).

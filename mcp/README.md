# ceps-rust-api-mcp

MCP sidecar for [ceps-rust-api](../README.md): Make/Docker lifecycle tools plus HTTP wrappers for every public API route.

| Transport | How | Endpoint |
| --- | --- | --- |
| **stdio** (default) | `make run-mcp` or Cursor Docker image | Cursor MCP |
| **HTTP (host)** | `make run-mcp-http` | Streamable HTTP on **4790** |
| **HTTP (Docker)** | `make mcp-http` | Hub/local image on **4790** → `http://127.0.0.1:4790/mcp` |

## Images

| Registry | Image |
| --- | --- |
| Docker Hub | `interchouette/ceps-rust-api-mcp` |
| Org GHCR | `ghcr.io/interchouette-itc/ceps-rust-api-mcp` |

Tags: `:dev` (tip), `:X.Y.Z` + `:latest` (release).

```bash
docker pull interchouette/ceps-rust-api-mcp:dev
```

## Env

| Variable | Default | Role |
| --- | --- | --- |
| `CEPS_API_MCP_ADDR` | `127.0.0.1:4790` | HTTP listen address |
| `CEPS_API_URL` | `http://127.0.0.1:8080` | Target API base URL |
| `CEPS_API_ROOT` | (cwd / `/workspace`) | In-process repo root |
| `CEPS_HOST_ROOT` | required in Docker | Absolute host clone path for binds |

Tool names use the `ceps_api_*` prefix (distinct from `ceps-client-mcp` `ceps18_*` tools). See [docs/mcp.md](../docs/mcp.md).

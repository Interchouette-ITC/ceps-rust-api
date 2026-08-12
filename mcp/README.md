# ceps-rust-api-mcp

Rust **rmcp** server (`ceps-rust-api-mcp` **v1.0.0**) to drive Make/Docker lifecycle and call the HTTP API from Cursor.

This is a **separate Cargo package** under `mcp/` - it does **not** link the API library. Lifecycle shells out to `make`/`docker`; API tools use `reqwest`.

Tool names use the `ceps_api_*` prefix (distinct from `ceps-client-mcp` `ceps18_*` tools).

## Transports

| Mode              | How                 | Use                                                       |
| ----------------- | ------------------- | --------------------------------------------------------- |
| **stdio (host)**  | `make run-mcp`      | MCP over stdin/stdout                                     |
| **HTTP (host)**   | `make run-mcp-http` | Streamable HTTP on **4790**                               |
| **HTTP (Docker)** | `make mcp-http`     | Hub/local image on **4790** → `http://127.0.0.1:4790/mcp` |

```bash
docker pull interchouette/ceps-rust-api-mcp:1.0.0
make mcp-http           # pull-first sidecar (sets CEPS_HOST_ROOT)
make mcp-http-stop
make mcp-docker-build   # local image if needed
```

See [docs/mcp.md](../docs/mcp.md) for the host-bind / docker.sock notes and full tool catalog.

## Env

| Var                 | Default                              | Role                                                                                          |
| ------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------- |
| `CEPS_API_ROOT`     | parent of `mcp/` / cwd with Makefile | In-container or host repo root for Make/Docker                                                |
| `CEPS_HOST_ROOT`    | falls back to `CEPS_API_ROOT`        | **Host** clone path for any Docker `-v` sources; **required** when `CEPS_API_ROOT=/workspace` |
| `CEPS_API_URL`      | `http://127.0.0.1:8080`              | HTTP tools base URL                                                                           |
| `MCP_HTTP`          | unset                                | Force HTTP transport                                                                          |
| `CEPS_API_MCP_ADDR` | `127.0.0.1:4790`                     | HTTP listen address                                                                           |

Cursor example: [`mcp.json.example`](mcp.json.example).

## Tests

```bash
cd mcp
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

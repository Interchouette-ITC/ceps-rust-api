# Demo tip CEP contract WASMs

On-chain contract bytecode for install and live tests. Same layout as the client
`tests/wasm/` tree: committed in git, default `CEPS_WASM_ROOT`.

| Dir | Contents |
| --- | --- |
| `cep18/` | CEP-18 fungible |
| `cep78/` | CEP-78 NFT + session helpers |
| `cep85/` | CEP-85 multi-token |
| `cep95/` | CEP-95 |

Refresh from the sibling **ceps-rust-ts-client** checkout (client owns tip staging):

```bash
make wasm-from-ceps
# copies $(CEPS_CLIENT_PRODUCT)/tests/wasm/{cep18,cep78,cep85,cep95}
```

Or unpack a client release `ceps-contracts-*.tgz` into the client `tests/wasm/`, then run `make wasm-from-ceps` here.

Short names like `cep18` resolve to `cep18/cep18.wasm` via `ceps_client::wasm`.

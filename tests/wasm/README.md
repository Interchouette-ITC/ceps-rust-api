# Demo tip CEP contract WASMs

On-chain contract bytecode for install and live tests. Same layout as the client
`tests/wasm/` tree: committed in git, default `CEPS_WASM_ROOT`.

| Dir      | Contents                     |
| -------- | ---------------------------- |
| `cep18/` | CEP-18 fungible              |
| `cep78/` | CEP-78 NFT + session helpers |
| `cep85/` | CEP-85 multi-token           |
| `cep95/` | CEP-95 (when staged)         |

Refresh from sibling tip CEP checkouts:

```bash
make wasm-from-ceps
```

Short names like `cep18` resolve to `cep18/cep18.wasm`.

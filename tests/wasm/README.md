# Demo tip CEP contract WASMs

Same role as [`ceps-rust-ts-client/tests/wasm`](https://github.com/Interchouette-ITC/ceps-rust-ts-client/tree/dev/tests/wasm): on-chain contract bytecode for install/live tests.

Committed in-tree for CI and local runs. Refresh from the client tip release pack:

```bash
make fetch-wasm
# or: CEPS_CONTRACTS_TAG=dev-preview ./scripts/fetch-ceps-contracts.sh
```

Default `CEPS_WASM_ROOT` is this directory. Short names like `cep18` resolve to `cep18/cep18.wasm`.

//! Shared helpers for CEP route modules.

use crate::error::ApiError;
use crate::state::AppState;
use ceps_client::CepCore;
use std::path::PathBuf;

pub fn cep_core(state: &AppState) -> Result<CepCore, ApiError> {
    CepCore::new(
        state.config.rpc_url.clone(),
        Some(state.config.sse_url.clone()),
        Some(state.config.chain_name.clone()),
        None,
    )
    .map_err(ApiError::from_cep)
}

pub fn resolve_wasm(state: &AppState, wasm_name: &str) -> Result<Vec<u8>, ApiError> {
    let path = if wasm_name.contains('/') || wasm_name.contains('\\') {
        PathBuf::from(wasm_name)
    } else {
        state.config.wasm_root.join(wasm_name)
    };
    std::fs::read(&path).map_err(|e| ApiError::NotFound(format!("wasm {}: {e}", path.display())))
}

pub fn bind_contract(
    core: &mut CepCore,
    contract_hash: &str,
    package_hash: Option<&str>,
) -> Result<(), ApiError> {
    core.set_contract_hash(contract_hash, package_hash)
        .map_err(ApiError::from_cep)
}

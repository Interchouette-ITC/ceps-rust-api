//! Shared helpers for CEP route modules.

use crate::error::ApiError;
use crate::state::AppState;
use ceps_client::CEPClient;
#[cfg(test)]
use std::path::{Path, PathBuf};

pub fn cep_core(state: &AppState) -> Result<CEPClient, ApiError> {
    CEPClient::new(
        state.config.rpc_url.clone(),
        Some(state.config.sse_url.clone()),
        Some(state.config.chain_name.clone()),
        None,
    )
    .map_err(ApiError::from_cep)
}

/// Resolve a wasm name or path under [`Config::wasm_root`](crate::config::Config::wasm_root).
///
/// Accepts the same aliases as [`ceps_client::wasm::resolve_path`]: absolute/relative
/// filesystem path, file under the root, or short alias (`cep18` → `cep18/cep18.wasm`).
pub fn resolve_wasm(state: &AppState, wasm_name: &str) -> Result<Vec<u8>, ApiError> {
    ceps_client::wasm::load_from(&state.config.wasm_root, wasm_name).map_err(map_wasm_err)
}

#[cfg(test)]
fn resolve_wasm_path(root: &Path, wasm_name: &str) -> Result<PathBuf, ApiError> {
    ceps_client::wasm::resolve_path(root, wasm_name).map_err(map_wasm_err)
}

fn map_wasm_err(err: ceps_client::CEPError) -> ApiError {
    use ceps_client::CEPError;
    match err {
        CEPError::InvalidArgument(m) => ApiError::BadRequest(m),
        CEPError::WasmNotFound { name, tried } => ApiError::NotFound(format!(
            "wasm {name} not found under roots (tried {})",
            tried.join(", ")
        )),
        CEPError::Io(e) => ApiError::NotFound(format!("wasm I/O: {e}")),
        other => ApiError::from_cep(other),
    }
}

pub fn bind_contract(
    core: &mut CEPClient,
    contract_hash: &str,
    package_hash: Option<&str>,
) -> Result<(), ApiError> {
    core.set_contract_hash(contract_hash, package_hash)
        .map_err(ApiError::from_cep)
}

/// Decode optional hex (`0x` optional) to bytes. Empty / missing → `None`.
pub fn optional_hex_bytes(hex: Option<&str>) -> Result<Option<Vec<u8>>, ApiError> {
    let Some(raw) = hex.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let s = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    if !s.len().is_multiple_of(2) {
        return Err(ApiError::BadRequest(
            "hex data must have even length".into(),
        ));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(Some(out))
}

fn hex_nibble(b: u8) -> Result<u8, ApiError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(ApiError::BadRequest(format!(
            "invalid hex digit {}",
            b as char
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn resolve_alias_cep18_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("cep18");
        fs::create_dir_all(&nested).unwrap();
        let wasm = nested.join("cep18.wasm");
        let mut f = fs::File::create(&wasm).unwrap();
        writeln!(f, "fake").unwrap();
        drop(f);

        let path = resolve_wasm_path(dir.path(), "cep18").unwrap();
        assert_eq!(path, wasm);
    }

    #[test]
    fn resolve_relative_under_root() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("cep78");
        fs::create_dir_all(&nested).unwrap();
        let wasm = nested.join("mint_session.wasm");
        fs::write(&wasm, b"x").unwrap();

        let path = resolve_wasm_path(dir.path(), "cep78/mint_session.wasm").unwrap();
        assert_eq!(path, wasm);
    }
}

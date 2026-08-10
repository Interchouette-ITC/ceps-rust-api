//! Shared helpers for CEP route modules.

use crate::error::ApiError;
use crate::state::AppState;
use ceps_client::CEPClient;
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
/// Accepts:
/// - absolute/relative filesystem path (contains `/` or `\`)
/// - file under `wasm_root` (`cep18.wasm` or `cep18/cep18.wasm`)
/// - short alias (`cep18` → `wasm_root/cep18/cep18.wasm`)
pub fn resolve_wasm(state: &AppState, wasm_name: &str) -> Result<Vec<u8>, ApiError> {
    let path = resolve_wasm_path(&state.config.wasm_root, wasm_name)?;
    std::fs::read(&path).map_err(|e| ApiError::NotFound(format!("wasm {}: {e}", path.display())))
}

fn resolve_wasm_path(root: &Path, wasm_name: &str) -> Result<PathBuf, ApiError> {
    let name = wasm_name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("wasm name is empty".into()));
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if name.contains('/') || name.contains('\\') {
        let as_path = PathBuf::from(name);
        candidates.push(as_path.clone());
        if as_path.is_relative() {
            candidates.push(root.join(&as_path));
        }
    } else {
        candidates.push(root.join(name));
        if !name.ends_with(".wasm") {
            candidates.push(root.join(format!("{name}.wasm")));
            candidates.push(root.join(name).join(format!("{name}.wasm")));
        }
    }

    for path in &candidates {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    Err(ApiError::NotFound(format!(
        "wasm {name} not found under {} (tried {})",
        root.display(),
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

pub fn bind_contract(
    core: &mut CEPClient,
    contract_hash: &str,
    package_hash: Option<&str>,
) -> Result<(), ApiError> {
    core.set_contract_hash(contract_hash, package_hash)
        .map_err(ApiError::from_cep)
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

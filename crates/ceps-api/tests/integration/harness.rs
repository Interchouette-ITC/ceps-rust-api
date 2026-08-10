//! CI / local integration helpers.
//!
//! Bootstrap CSPR only: load a faucet PEM from env (never from HTTP).
//! Product code under `crates/` must not import this module.

use ceps_api::config::{Config, SignBackend};
use ceps_api::sign::LocalKeyring;
use ceps_api::state::AppState;
use std::path::PathBuf;

/// Faucet public key used when PEM path alone is set (NCTL default faucet).
pub const DEFAULT_FAUCET_PUBLIC_KEY: &str =
    "0109360ea3ef3fa2d90ffc33d500d7080730bb488718efc76c62645248d9ec8379";

#[must_use]
pub async fn rpc_reachable(rpc_url: &str) -> bool {
    let url = if rpc_url.ends_with("/rpc") {
        rpc_url.to_string()
    } else {
        format!("{}/rpc", rpc_url.trim_end_matches('/'))
    };
    reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "info_get_status",
            "params": []
        }))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// `(public_key, pem)` from `CEPS_FAUCET_PEM` or `CEPS_FAUCET_PEM_PATH` (+ optional `CEPS_FAUCET_PUBLIC_KEY`).
pub fn load_faucet_pem() -> Option<(String, String)> {
    if let Ok(pem) = std::env::var("CEPS_FAUCET_PEM") {
        if !pem.trim().is_empty() {
            let pk = std::env::var("CEPS_FAUCET_PUBLIC_KEY")
                .unwrap_or_else(|_| DEFAULT_FAUCET_PUBLIC_KEY.to_string());
            return Some((pk, pem));
        }
    }
    let path = std::env::var("CEPS_FAUCET_PEM_PATH").ok()?;
    let pem = std::fs::read_to_string(PathBuf::from(path)).ok()?;
    if pem.trim().is_empty() {
        return None;
    }
    let pk = std::env::var("CEPS_FAUCET_PUBLIC_KEY")
        .unwrap_or_else(|_| DEFAULT_FAUCET_PUBLIC_KEY.to_string());
    Some((pk, pem))
}

/// App state with local signing and faucet key in the process keyring.
pub fn state_with_faucet(public_key: &str, pem: &str) -> AppState {
    let mut cfg = Config::default();
    cfg.sign_backend = SignBackend::Local;
    cfg.rpc_url = std::env::var("CEPS_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:11101".into());
    cfg.sse_url =
        std::env::var("CEPS_SSE_URL").unwrap_or_else(|_| "http://127.0.0.1:18101/events".into());
    cfg.chain_name = std::env::var("CEPS_CHAIN_NAME").unwrap_or_else(|_| "casper-net-1".into());
    let ring = LocalKeyring::new();
    ring.insert(public_key.to_string(), pem.to_string());
    cfg.local_keys = ring;
    AppState::new(cfg)
}

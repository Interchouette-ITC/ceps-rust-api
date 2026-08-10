//! Integration helpers for live tests against a reachable node.
//!
//! Prefer pre-funded NCTL **user** keys via `LOCAL_KEYS_JSON` (never faucet).
//! Faucet is for KMS funding only (`scripts/fund-kms-from-nctl.sh`).
//! Product code under `crates/` must not import this module.

use ceps_rust_api::config::{Config, SignBackend};
use ceps_rust_api::sign::LocalKeyring;
use ceps_rust_api::state::AppState;

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

/// App state from env (`CEPS_RPC_URL`, `LOCAL_KEYS_JSON`, optional `SIGN_BACKEND` / `KMS_URL`).
pub fn state_from_env() -> AppState {
    let mut cfg = Config::from_env().unwrap_or_else(|e| {
        panic!("invalid config for live test: {e}");
    });
    if cfg.rpc_url.is_empty() {
        cfg.rpc_url = "http://127.0.0.1:11101".into();
    }
    // Lab convenience: LOCAL_KEYS_JSON alone ⇒ local backend for live tests.
    if cfg.sign_backend == SignBackend::None {
        if let Ok(raw) = std::env::var("LOCAL_KEYS_JSON") {
            if let Ok(ring) = LocalKeyring::from_json(&raw, "LOCAL_KEYS_JSON") {
                if !ring.is_empty() {
                    cfg.local_keys = ring;
                    cfg.sign_backend = SignBackend::Local;
                }
            }
        }
    }
    AppState::new(cfg)
}

//! Integration helpers for live tests against a reachable node.
//!
//! Prefer pre-funded NCTL keys via product `LOCAL_KEYS_JSON` (faucet + users).
//! No faucet env, no native-transfer funding in this repo.
//! Product code under `crates/` must not import this module.

use ceps_api::config::{Config, SignBackend};
use ceps_api::sign::LocalKeyring;
use ceps_api::state::AppState;

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
    let mut cfg = Config::from_env();
    if cfg.rpc_url.is_empty() {
        cfg.rpc_url = "http://127.0.0.1:11101".into();
    }
    if cfg.sign_backend == SignBackend::None && !cfg.local_keys.is_empty() {
        cfg.sign_backend = SignBackend::Local;
    }
    AppState::new(cfg)
}

/// App state with an explicit local keyring (tests that inject one key).
pub fn state_with_local_keys(ring: LocalKeyring) -> AppState {
    let mut cfg = Config::from_env();
    cfg.sign_backend = SignBackend::Local;
    cfg.local_keys = ring;
    AppState::new(cfg)
}

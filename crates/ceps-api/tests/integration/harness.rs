//! CI / local integration helpers.
//!
//! Bootstrap CSPR only (tests).
//!
//! Typical order: create a recipient public key on KMS (direct HTTP), then fund it
//! FROM the faucet PEM (usually NCTL faucet / user-1) via SDK transfer.
//! Product HTTP routes never expose fund or key create.

use casper_rust_wasm_sdk::types::transaction_params::transaction_str_params::TransactionStrParams;
use ceps_api::config::{Config, SignBackend};
use ceps_api::sign::LocalKeyring;
use ceps_api::state::AppState;
use ceps_client::CepCore;
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
pub fn state_with_faucet(public_key: &str, pem: &str, kms_url: Option<String>) -> AppState {
    let mut cfg = Config::default();
    cfg.sign_backend = SignBackend::Local;
    cfg.rpc_url = std::env::var("CEPS_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:11101".into());
    cfg.sse_url =
        std::env::var("CEPS_SSE_URL").unwrap_or_else(|_| "http://127.0.0.1:18101/events".into());
    cfg.chain_name = std::env::var("CEPS_CHAIN_NAME").unwrap_or_else(|_| "casper-net-1".into());
    if let Some(url) = kms_url {
        cfg.kms_url = url;
    }
    let ring = LocalKeyring::new();
    ring.insert(public_key.to_string(), pem.to_string());
    cfg.local_keys = ring;
    AppState::new(cfg)
}

/// Call KMS `POST /createKey` directly (not via ceps-rust-api).
pub async fn kms_create_key(kms_url: &str) -> Result<String, String> {
    let base = kms_url.trim_end_matches('/');
    let resp = reqwest::Client::new()
        .post(format!("{base}/createKey"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("createKey status {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    body.get("public_key")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| "createKey missing public_key".into())
}

/// Native transfer from faucet PEM to `target` (harness only; not an HTTP route).
pub async fn fund_from_faucet(
    rpc_url: &str,
    chain_name: &str,
    faucet_pem: &str,
    target: &str,
    amount: &str,
    payment_amount: &str,
) -> Result<String, String> {
    let core = CepCore::new(
        rpc_url.to_string(),
        None,
        Some(chain_name.to_string()),
        None,
    )
    .map_err(|e| e.to_string())?;
    let str_params = TransactionStrParams::default();
    str_params.set_chain_name(chain_name);
    str_params.set_payment_amount(payment_amount);
    str_params.set_secret_key(faucet_pem);
    let put = core
        .sdk()
        .transfer_transaction(
            None,
            target,
            amount,
            str_params,
            None,
            Some(core.verbosity()),
            Some(core.rpc_url().to_string()),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(
        casper_rust_wasm_sdk::types::hash::transaction_hash::TransactionHash::from(
            put.result.transaction_hash,
        )
        .to_string(),
    )
}
